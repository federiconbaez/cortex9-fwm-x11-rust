//! Abre una ventana de prueba con clase, título y color dados, y espera.
//! Uso: fwm-demo-ventana <clase> <titulo> <color_hex>
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::wrapper::ConnectionExt as _;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let (clase, titulo) = (a.get(1).cloned().unwrap_or("demo".into()), a.get(2).cloned().unwrap_or("ventana".into()));
    let color = u32::from_str_radix(a.get(3).map(String::as_str).unwrap_or("303040"), 16)?;
    let (conn, pantalla) = x11rb::connect(None)?;
    let root = conn.setup().roots[pantalla].root;
    let w = conn.generate_id()?;
    conn.create_window(0, w, root, 0, 0, 300, 200, 0, WindowClass::INPUT_OUTPUT, 0,
        &CreateWindowAux::new().background_pixel(color))?;
    let wm_class = format!("{clase}\0{clase}\0");
    conn.change_property8(PropMode::REPLACE, w, AtomEnum::WM_CLASS, AtomEnum::STRING, wm_class.as_bytes())?;
    conn.change_property8(PropMode::REPLACE, w, AtomEnum::WM_NAME, AtomEnum::STRING, titulo.as_bytes())?;
    conn.map_window(w)?;
    conn.flush()?;
    loop {
        conn.wait_for_event()?;
    }
}
