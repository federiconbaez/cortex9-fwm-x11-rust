//! Afinidad entre ventanas.
//!
//! v0.1 define la "afinidad" de forma medible: cada ventana se convierte en un vector
//! de 512 dimensiones (bolsa de tokens con hashing) a partir de su clase (WM_CLASS) y
//! su título, y la afinidad es la similitud coseno. Los grupos salen de un
//! agrupamiento por enlace simple con umbral. No se analizan píxeles todavía.

pub const DIM: usize = 512;
pub const UMBRAL: f32 = 0.30;

fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Palabras que aparecen en casi cualquier ventana y no dicen de qué trata.
/// Las encontró la prueba con Chrome real: su WM_CLASS en modo app trae "text_html_title".
const VACIAS: &[&str] = &["text", "html", "title", "data", "http", "https", "www", "com", "body", "style", "google", "chrome", "chromium", "the", "and", "untitled"];

pub fn tokens(texto: &str) -> Vec<String> {
    texto
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 3 && !VACIAS.contains(t))
        .map(str::to_string)
        .collect()
}

/// Vector normalizado. El título pesa el doble que la clase: dos ventanas de la misma
/// aplicación con temas distintos tienen que poder separarse.
pub fn vector(clase: &str, titulo: &str) -> [f32; DIM] {
    vector_completo(clase, titulo, "")
}

/// Igual que `vector`, sumando el texto que la ventana muestra en pantalla (OCR) con peso 1.
pub fn vector_completo(clase: &str, titulo: &str, contenido: &str) -> [f32; DIM] {
    let mut v = [0f32; DIM];
    for t in tokens(contenido) {
        v[(fnv1a(&t) % DIM as u64) as usize] += 1.0;
    }
    for t in tokens(clase) {
        v[(fnv1a(&t) % DIM as u64) as usize] += 1.0;
    }
    for t in tokens(titulo) {
        v[(fnv1a(&t) % DIM as u64) as usize] += 2.0;
    }
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if n > 0.0 {
        v.iter_mut().for_each(|x| *x /= n);
    }
    v
}

pub fn coseno(a: &[f32; DIM], b: &[f32; DIM]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Baja el peso de lo que comparten casi todas las ventanas abiertas (al menos 3/4 de ellas,
/// con 3 o más ventanas): una palabra que muestran todas, como "Documento" o el nombre
/// del navegador, no dice nada sobre el tema. Lo demás conserva su peso.
pub fn ponderar_idf(vs: &[[f32; DIM]]) -> Vec<[f32; DIM]> {
    let n = vs.len() as f32;
    let mut df = [0f32; DIM];
    for v in vs {
        for (d, x) in df.iter_mut().zip(v) {
            if *x > 0.0 {
                *d += 1.0;
            }
        }
    }
    vs.iter()
        .map(|v| {
            let mut w = [0f32; DIM];
            for i in 0..DIM {
                let comun = n >= 3.0 && df[i] * 4.0 >= n * 3.0;
                w[i] = v[i] * if comun { 0.15 } else { 1.0 };
            }
            let norma = w.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norma > 0.0 {
                w.iter_mut().for_each(|x| *x /= norma);
            }
            w
        })
        .collect()
}

/// Devuelve el número de grupo de cada vector. Determinista: los grupos se numeran
/// por orden de aparición.
pub fn agrupar(vs: &[[f32; DIM]], umbral: f32) -> Vec<usize> {
    let vs = &ponderar_idf(vs);
    let n = vs.len();
    let mut padre: Vec<usize> = (0..n).collect();
    fn raiz(p: &mut [usize], mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    for i in 0..n {
        for j in i + 1..n {
            if coseno(&vs[i], &vs[j]) >= umbral {
                let (a, b) = (raiz(&mut padre, i), raiz(&mut padre, j));
                if a != b {
                    padre[a.max(b)] = a.min(b);
                }
            }
        }
    }
    let mut etiqueta = vec![usize::MAX; n];
    let mut siguiente = 0;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let r = raiz(&mut padre, i);
        if etiqueta[r] == usize::MAX {
            etiqueta[r] = siguiente;
            siguiente += 1;
        }
        out.push(etiqueta[r]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identicas_tienen_afinidad_uno() {
        let a = vector("Firefox", "Cortex9Sur en X");
        assert!((coseno(&a, &a) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn agrupa_por_tema() {
        let vs = [
            vector("terminal", "cargo build fwm"),
            vector("browser", "x.com Cortex9Sur"),
            vector("terminal", "cargo test fwm"),
            vector("browser", "Cortex9Sur borrador"),
        ];
        let g = agrupar(&vs, UMBRAL);
        assert_eq!(g[0], g[2]);
        assert_eq!(g[1], g[3]);
        assert_ne!(g[0], g[1]);
    }

    #[test]
    fn vacio_no_rompe() {
        let v = vector("", "");
        assert_eq!(coseno(&v, &v), 0.0);
        assert_eq!(agrupar(&[], UMBRAL), Vec::<usize>::new());
    }

    #[test]
    fn misma_app_temas_distintos_se_separan() {
        let c = "text_html,_title_x google-chrome";
        let vs = [vector(c, "cargo test fwm"), vector(c, "Cortex9Sur borrador"), vector(c, "cargo build fwm"), vector(c, "Cortex9Sur publicado")];
        assert_eq!(agrupar(&vs, UMBRAL), vec![0, 1, 0, 1]);
    }

    #[test]
    fn titulos_genericos_se_separan_por_contenido() {
        let c = "google-chrome";
        let vs = [
            vector_completo(c, "Documento", "cargo build compila el crate fwm sin errores"),
            vector_completo(c, "Documento", "borrador del post de Cortex9Sur sobre soberania de datos"),
            vector_completo(c, "Documento", "cargo test corre los tests del crate fwm"),
            vector_completo(c, "Documento", "Cortex9Sur publico el post sobre la nube soberana"),
        ];
        // Sin contenido serían todos iguales; con contenido quedan dos grupos.
        assert_eq!(agrupar(&vs, UMBRAL), vec![0, 1, 0, 1]);
    }

    #[test]
    fn caso_de_integracion_tres_grupos() {
        let vs = [
            vector("terminal", "cargo build fwm"),
            vector("browser", "x.com Cortex9Sur borrador"),
            vector("terminal", "cargo test fwm"),
            vector("blender", "Federica World tile T_0_0"),
            vector("browser", "Cortex9Sur publicado"),
        ];
        let g = agrupar(&vs, UMBRAL);
        assert_eq!(g, vec![0, 1, 0, 2, 1]);
    }
}
