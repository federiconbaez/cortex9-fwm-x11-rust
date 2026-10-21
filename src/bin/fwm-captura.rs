//! Captura la ventana raíz del display actual a PNG. Uso: fwm-captura salida.png
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let salida = std::env::args().nth(1).unwrap_or("captura.png".into());
    let (conn, pantalla) = x11rb::connect(None)?;
    let s = &conn.setup().roots[pantalla];
    let (w, h) = (s.width_in_pixels, s.height_in_pixels);
    let img = conn.get_image(ImageFormat::Z_PIXMAP, s.root, 0, 0, w, h, !0)?.reply()?;
    if img.depth != 24 && img.depth != 32 {
        return Err(format!("profundidad {} no soportada", img.depth).into());
    }
    let mut rgb = Vec::with_capacity(w as usize * h as usize * 3);
    for px in img.data.chunks_exact(4) {
        rgb.extend_from_slice(&[px[2], px[1], px[0]]);
    }
    let f = std::io::BufWriter::new(std::fs::File::create(&salida)?);
    let mut enc = png::Encoder::new(f, w as u32, h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()?.write_image_data(&rgb)?;
    println!("{salida}");
    Ok(())
}
