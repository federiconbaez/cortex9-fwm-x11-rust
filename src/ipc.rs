//! Bus IPC para agentes: socket Unix, una petición JSON por línea, una respuesta JSON por línea.
//!
//! Peticiones:
//!   {"cmd":"list"}               -> ventanas con id, clase, título, grupo y rectángulo
//!   {"cmd":"focus","id":N}       -> enfoca y sube la ventana N
//!   {"cmd":"affinity","a":N,"b":M} -> similitud coseno entre dos ventanas
//!   {"cmd":"set_threshold","value":F} -> cambia el umbral de agrupamiento y reordena

use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Peticion {
    List,
    Focus { id: u32 },
    Affinity { a: u32, b: u32 },
    SetThreshold { value: f32 },
}

pub fn parsear(linea: &str) -> Result<Peticion, String> {
    serde_json::from_str(linea.trim()).map_err(|e| e.to_string())
}

pub fn ruta_socket() -> String {
    let disp = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".into());
    let disp: String = disp.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    format!("{dir}/fwm-{disp}.sock")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsea_comandos() {
        assert_eq!(parsear(r#"{"cmd":"list"}"#).unwrap(), Peticion::List);
        assert_eq!(parsear(r#"{"cmd":"focus","id":7}"#).unwrap(), Peticion::Focus { id: 7 });
        assert!(parsear(r#"{"cmd":"rm -rf"}"#).is_err());
        assert!(parsear("basura").is_err());
    }
}
