# fwm: gestor de ventanas X11 en Rust para orquestación cognitiva

> **ES:** Gestor de ventanas X11 en Rust que agrupa ventanas por afinidad de título, clase y contenido (OCR). Compilar/probar: `cargo build --release && cargo test`.
> **EN:** Rust X11 window manager grouping windows by title/class/on-screen-content (OCR) affinity. Build/test: `cargo build --release && cargo test`.
>
> **Estado / Status:** probado con Chrome real (ver abajo); experimento de valor **no auditado** / value experiment **not audited**. Licencia / License: Apache-2.0.


Sistema 7 de la lista de Federico. Dueña: Federica Forge Nexus.

Hallazgo de la prueba con Chrome real: en v0.1 las cuatro ventanas de Chrome quedaban en un solo grupo, porque su `WM_CLASS` comparte palabras como `text`, `html` y `title`. Se corrigió en v0.2 y quedó un test que lo cubre.

## Qué hace hoy (v0.3)
- Es un gestor de ventanas X11 real: toma `SubstructureRedirect` en la raíz y decide dónde va cada ventana.
- Agrupa las ventanas por afinidad: cada ventana es un vector de 512 dimensiones (bolsa de tokens con hashing de su título, con peso 2, y su `WM_CLASS`, con peso 1, sin palabras vacías como `html` o `chrome`), la afinidad es la similitud coseno y los grupos salen de un enlace simple con umbral 0.30.
- **Lee lo que cada ventana muestra en pantalla**: cada 1,5 s captura los píxeles de cada ventana y, si cambiaron, los pasa por OCR (Tesseract, español e inglés) en un hilo aparte. Ese texto entra al vector de afinidad, así que ventanas con el mismo título se separan por su contenido.
- Lo que comparten casi todas las ventanas abiertas (como "Documento" o el nombre del navegador) pesa un 15 %, porque no dice nada del tema.
- Dispone un grupo por columna y le pone a cada grupo un color de borde.
- Si una ventana cambia de título, se recalcula y puede cambiar de grupo.
- Publica EWMH mínimo (`_NET_SUPPORTING_WM_CHECK`, `_NET_CLIENT_LIST`, `_NET_ACTIVE_WINDOW`) y atiende la activación, así que `xdotool windowactivate` y `getactivewindow` funcionan.
- Los diálogos (`WM_TRANSIENT_FOR`) flotan centrados y no entran en el agrupamiento.
- Expone un bus para agentes en un socket Unix (`$XDG_RUNTIME_DIR/fwm-<display>.sock`, o `/tmp`), con JSON por línea: `list`, `focus`, `affinity`, `set_threshold`. Rechaza cualquier comando que no esté en el `enum`.

## Qué NO hace todavía
- La afinidad sobre lo que se ve en pantalla es por **texto leído con OCR**, no por embeddings visuales de un modelo neuronal: una ventana sin texto (un video, una imagen) se agrupa solo por clase y título.
- El OCR tarda unos 5 a 9 s en estabilizar 4 ventanas en esta computadora.
- No hay EWMH completo, ni teclado, ni escritorios virtuales.
- Probado en un display virtual (Xvfb) con Chrome real, todavía no en la sesión de Federico.
- Sin teclado ni mouse propios del WM: el foco se cambia por el bus o por EWMH.

## Requisitos
`tesseract-ocr` y `tesseract-ocr-spa`. Sin ellos, fwm funciona igual y agrupa solo por clase y título.

## Verificar
```
cargo test                                   # 11 tests unitarios
cargo clippy --all-targets -- -D warnings    # sin warnings
bash scripts/prueba-integracion.sh           # Xvfb + fwm + 5 ventanas + chequeo por el bus
bash scripts/prueba-chrome.sh                # Xvfb + fwm + 4 ventanas reales de Google Chrome + EWMH + xdotool
bash scripts/prueba-vision.sh                # 4 Chrome con el MISMO título: tienen que separarse por lo que muestran
FWM_VISION=0 bash scripts/prueba-vision.sh   # control: sin visión falla (mezcla los temas o no lee la pantalla)
```
La prueba de integración falla si las ventanas del mismo tema no quedan juntas, si no hay exactamente 3 grupos, si hay solapes o si el bus acepta un comando desconocido. Deja una captura en `docs/captura-fwm.png`.

## Probar en tu máquina sin tocar tu sesión
Con `Xephyr :2 -screen 1280x800 &` y después `DISPLAY=:2 target/release/fwm`. No lo corras sobre tu display principal hasta que lo revisemos juntos.
