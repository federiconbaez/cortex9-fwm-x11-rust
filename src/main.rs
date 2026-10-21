//! fwm: el gestor de ventanas.
use fwm::{affinity, ipc, layout};
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::time::Duration;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

struct Ventana {
    id: Window,
    clase: String,
    titulo: String,
    vec: [f32; affinity::DIM],
    grupo: usize,
    rect: layout::Rect,
}

struct Wm {
    conn: RustConnection,
    root: Window,
    ancho: u32,
    alto: u32,
    umbral: f32,
    ventanas: Vec<Ventana>,
    colores: [u32; 6],
}

fn texto_prop(conn: &RustConnection, w: Window, prop: Atom) -> String {
    conn.get_property(false, w, prop, AtomEnum::ANY, 0, 1024)
        .ok()
        .and_then(|c| c.reply().ok())
        .map(|r| String::from_utf8_lossy(&r.value).replace('\0', " ").trim().to_string())
        .unwrap_or_default()
}

impl Wm {
    fn atom(&self, nombre: &str) -> Atom {
        self.conn.intern_atom(false, nombre.as_bytes()).unwrap().reply().unwrap().atom
    }

    fn gestionar(&mut self, w: Window) {
        if self.ventanas.iter().any(|v| v.id == w) {
            return;
        }
        let net_name = self.atom("_NET_WM_NAME");
        let mut titulo = texto_prop(&self.conn, w, net_name);
        if titulo.is_empty() {
            titulo = texto_prop(&self.conn, w, AtomEnum::WM_NAME.into());
        }
        let clase = texto_prop(&self.conn, w, AtomEnum::WM_CLASS.into());
        let _ = self.conn.change_window_attributes(
            w,
            &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE | EventMask::STRUCTURE_NOTIFY),
        );
        let _ = self.conn.map_window(w);
        let vec = affinity::vector(&clase, &titulo);
        eprintln!("fwm: gestiono 0x{w:x} clase={clase:?} titulo={titulo:?}");
        self.ventanas.push(Ventana { id: w, clase, titulo, vec, grupo: 0, rect: layout::Rect { x: 0, y: 0, w: 1, h: 1 } });
        self.reordenar();
    }

    fn soltar(&mut self, w: Window) {
        let antes = self.ventanas.len();
        self.ventanas.retain(|v| v.id != w);
        if self.ventanas.len() != antes {
            self.reordenar();
        }
    }

    fn reordenar(&mut self) {
        let vs: Vec<_> = self.ventanas.iter().map(|v| v.vec).collect();
        let grupos = affinity::agrupar(&vs, self.umbral);
        let rects = layout::disponer(&grupos, self.ancho, self.alto);
        for ((v, g), r) in self.ventanas.iter_mut().zip(grupos).zip(rects) {
            v.grupo = g;
            v.rect = r;
            let borde = 3;
            let _ = self.conn.configure_window(
                v.id,
                &ConfigureWindowAux::new()
                    .x(r.x)
                    .y(r.y)
                    .width(r.w.saturating_sub(2 * borde).max(1))
                    .height(r.h.saturating_sub(2 * borde).max(1))
                    .border_width(borde),
            );
            let _ = self.conn.change_window_attributes(
                v.id,
                &ChangeWindowAttributesAux::new().border_pixel(self.colores[g % self.colores.len()]),
            );
        }
        let _ = self.conn.flush();
    }

    fn atender(&mut self, linea: &str) -> serde_json::Value {
        match ipc::parsear(linea) {
            Err(e) => json!({"ok": false, "error": e}),
            Ok(ipc::Peticion::List) => json!({"ok": true, "umbral": self.umbral, "ventanas":
                self.ventanas.iter().map(|v| json!({"id": v.id, "clase": v.clase, "titulo": v.titulo, "grupo": v.grupo, "rect": v.rect})).collect::<Vec<_>>()}),
            Ok(ipc::Peticion::Focus { id }) => {
                if !self.ventanas.iter().any(|v| v.id == id) {
                    return json!({"ok": false, "error": "ventana no gestionada"});
                }
                let _ = self.conn.set_input_focus(InputFocus::PARENT, id, x11rb::CURRENT_TIME);
                let _ = self.conn.configure_window(id, &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE));
                let _ = self.conn.flush();
                json!({"ok": true})
            }
            Ok(ipc::Peticion::Affinity { a, b }) => {
                let va = self.ventanas.iter().find(|v| v.id == a);
                let vb = self.ventanas.iter().find(|v| v.id == b);
                match (va, vb) {
                    (Some(x), Some(y)) => json!({"ok": true, "afinidad": affinity::coseno(&x.vec, &y.vec)}),
                    _ => json!({"ok": false, "error": "ventana no gestionada"}),
                }
            }
            Ok(ipc::Peticion::SetThreshold { value }) => {
                if !(0.0..=1.0).contains(&value) {
                    return json!({"ok": false, "error": "umbral fuera de [0,1]"});
                }
                self.umbral = value;
                self.reordenar();
                json!({"ok": true})
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (conn, pantalla) = x11rb::connect(None)?;
    let s = &conn.setup().roots[pantalla];
    let (root, ancho, alto) = (s.root, s.width_in_pixels as u32, s.height_in_pixels as u32);
    let pedido = conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::new().event_mask(EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY),
    )?;
    if pedido.check().is_err() {
        return Err("ya hay otro gestor de ventanas en este display".into());
    }
    let mut wm = Wm {
        conn,
        root,
        ancho,
        alto,
        umbral: affinity::UMBRAL,
        ventanas: vec![],
        colores: [0x00e5ff, 0xff4fa3, 0xffc94f, 0x7dff6b, 0xb08cff, 0xff7b3d],
    };
    // Adoptar ventanas que ya estaban mapeadas.
    let hijos = wm.conn.query_tree(wm.root)?.reply()?.children;
    for w in hijos {
        let attrs = wm.conn.get_window_attributes(w)?.reply();
        if let Ok(a) = attrs {
            if a.map_state == MapState::VIEWABLE && !a.override_redirect {
                wm.gestionar(w);
            }
        }
    }
    let ruta = ipc::ruta_socket();
    let _ = std::fs::remove_file(&ruta);
    let oyente = UnixListener::bind(&ruta)?;
    oyente.set_nonblocking(true)?;
    eprintln!("fwm: {ancho}x{alto}, bus en {ruta}");

    loop {
        let mut hubo = false;
        while let Some(ev) = wm.conn.poll_for_event()? {
            hubo = true;
            match ev {
                Event::MapRequest(e) => wm.gestionar(e.window),
                Event::DestroyNotify(e) => wm.soltar(e.window),
                Event::UnmapNotify(e) if e.event != wm.root => wm.soltar(e.window),
                Event::ConfigureRequest(e) => {
                    if !wm.ventanas.iter().any(|v| v.id == e.window) {
                        let aux = ConfigureWindowAux::from_configure_request(&e);
                        let _ = wm.conn.configure_window(e.window, &aux);
                        let _ = wm.conn.flush();
                    }
                }
                Event::PropertyNotify(e) => {
                    if let Some(i) = wm.ventanas.iter().position(|v| v.id == e.window) {
                        let net_name = wm.atom("_NET_WM_NAME");
                        if e.atom == net_name || e.atom == u32::from(AtomEnum::WM_NAME) {
                            let mut t = texto_prop(&wm.conn, e.window, net_name);
                            if t.is_empty() {
                                t = texto_prop(&wm.conn, e.window, AtomEnum::WM_NAME.into());
                            }
                            let v = &mut wm.ventanas[i];
                            v.titulo = t;
                            v.vec = affinity::vector(&v.clase, &v.titulo);
                            wm.reordenar();
                        }
                    }
                }
                _ => {}
            }
        }
        match oyente.accept() {
            Ok((flujo, _)) => {
                hubo = true;
                atender_cliente(&mut wm, flujo);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => eprintln!("fwm: bus: {e}"),
        }
        if !hubo {
            std::thread::sleep(Duration::from_millis(8));
        }
    }
}

fn atender_cliente(wm: &mut Wm, flujo: UnixStream) {
    let _ = flujo.set_nonblocking(false);
    let _ = flujo.set_read_timeout(Some(Duration::from_millis(500)));
    let mut escritor = match flujo.try_clone() {
        Ok(f) => f,
        Err(_) => return,
    };
    for linea in BufReader::new(flujo).lines() {
        let Ok(linea) = linea else { break };
        if linea.trim().is_empty() {
            continue;
        }
        let r = wm.atender(&linea);
        if writeln!(escritor, "{r}").is_err() {
            break;
        }
    }
}
