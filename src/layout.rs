//! Disposición: una columna por grupo, ventanas apiladas dentro de cada columna.

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

pub const HUECO: u32 = 6;

/// `grupos[i]` es el grupo de la ventana i. Devuelve un rectángulo por ventana.
pub fn disponer(grupos: &[usize], ancho: u32, alto: u32) -> Vec<Rect> {
    let ngrupos = grupos.iter().copied().max().map_or(0, |m| m + 1);
    if ngrupos == 0 {
        return vec![];
    }
    let col_w = ancho.saturating_sub(HUECO) / ngrupos as u32;
    let mut cuenta = vec![0u32; ngrupos];
    for &g in grupos {
        cuenta[g] += 1;
    }
    let mut usados = vec![0u32; ngrupos];
    grupos
        .iter()
        .map(|&g| {
            let fila_h = alto.saturating_sub(HUECO) / cuenta[g];
            let r = Rect {
                x: (HUECO + g as u32 * col_w) as i32,
                y: (HUECO + usados[g] * fila_h) as i32,
                w: col_w.saturating_sub(HUECO).max(1),
                h: fila_h.saturating_sub(HUECO).max(1),
            };
            usados[g] += 1;
            r
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn se_solapan(a: &Rect, b: &Rect) -> bool {
        a.x < b.x + b.w as i32 && b.x < a.x + a.w as i32 && a.y < b.y + b.h as i32 && b.y < a.y + a.h as i32
    }

    #[test]
    fn sin_solapes_y_dentro_de_pantalla() {
        let g = [0, 1, 0, 2, 1, 0];
        let rs = disponer(&g, 1280, 800);
        for (i, a) in rs.iter().enumerate() {
            assert!(a.x >= 0 && a.y >= 0);
            assert!(a.x as u32 + a.w <= 1280 && a.y as u32 + a.h <= 800);
            for b in &rs[i + 1..] {
                assert!(!se_solapan(a, b), "{a:?} {b:?}");
            }
        }
    }

    #[test]
    fn misma_columna_por_grupo() {
        let rs = disponer(&[0, 1, 0], 1000, 600);
        assert_eq!(rs[0].x, rs[2].x);
        assert_ne!(rs[0].x, rs[1].x);
    }
}
