//! Visión: lee lo que cada ventana muestra en pantalla y lo convierte en texto con OCR
//! (Tesseract, español e inglés). Corre en un hilo aparte para no frenar al WM.
//! El texto se suma al vector de afinidad, así que dos ventanas con títulos genéricos
//! se agrupan por su contenido visible.

use std::sync::mpsc::{Receiver, Sender, channel};

pub struct Pedido {
    pub id: u32,
    pub ancho: u32,
    pub alto: u32,
    /// Píxeles RGB, 3 bytes por píxel.
    pub rgb: Vec<u8>,
}

pub struct Lectura {
    pub id: u32,
    pub texto: String,
}

/// Huella barata del contenido: si no cambia, no se vuelve a leer la ventana.
pub fn huella(bgrx: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bgrx.iter().step_by(97) {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h ^ bgrx.len() as u64
}

pub fn bgrx_a_rgb(bgrx: &[u8]) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(bgrx.len() / 4 * 3);
    for px in bgrx.chunks_exact(4) {
        rgb.extend_from_slice(&[px[2], px[1], px[0]]);
    }
    rgb
}

pub fn disponible() -> bool {
    std::process::Command::new("tesseract").arg("--version").output().is_ok_and(|o| o.status.success())
}

fn leer(p: &Pedido) -> Result<String, Box<dyn std::error::Error>> {
    let ruta = std::env::temp_dir().join(format!("fwm-ocr-{}-{:x}.png", std::process::id(), p.id));
    {
        let f = std::io::BufWriter::new(std::fs::File::create(&ruta)?);
        let mut enc = png::Encoder::new(f, p.ancho, p.alto);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header()?.write_image_data(&p.rgb)?;
    }
    let out = std::process::Command::new("tesseract")
        .arg(&ruta)
        .arg("stdout")
        .args(["-l", "spa+eng", "--psm", "3"])
        .output()?;
    let _ = std::fs::remove_file(&ruta);
    Ok(String::from_utf8_lossy(&out.stdout).split_whitespace().collect::<Vec<_>>().join(" "))
}

/// Arranca el lector. Devuelve el canal para pedir lecturas y el de resultados.
pub fn arrancar() -> (Sender<Pedido>, Receiver<Lectura>) {
    let (tx_p, rx_p) = channel::<Pedido>();
    let (tx_l, rx_l) = channel::<Lectura>();
    std::thread::spawn(move || {
        for p in rx_p {
            match leer(&p) {
                Ok(texto) => {
                    if tx_l.send(Lectura { id: p.id, texto }).is_err() {
                        break;
                    }
                }
                Err(e) => eprintln!("fwm: visión: {e}"),
            }
        }
    });
    (tx_p, rx_l)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn huella_cambia_con_el_contenido() {
        let a = vec![0u8; 4000];
        let mut b = a.clone();
        b[97 * 3] = 255;
        assert_ne!(huella(&a), huella(&b));
        assert_eq!(huella(&a), huella(&a.clone()));
    }

    #[test]
    fn convierte_bgrx() {
        assert_eq!(bgrx_a_rgb(&[1, 2, 3, 0, 4, 5, 6, 0]), vec![3, 2, 1, 6, 5, 4]);
    }
}
