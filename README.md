# fwm: gestor de ventanas X11 en Rust para orquestación cognitiva

Sistema 7 de la lista de Federico. Dueña: Federica Forge Nexus.

## Qué hace hoy (v0.1)
- Es un gestor de ventanas X11 real: toma `SubstructureRedirect` en la raíz y decide dónde va cada ventana.
- Agrupa las ventanas por afinidad: cada ventana es un vector de 512 dimensiones (bolsa de tokens con hashing de su `WM_CLASS` y su título), la afinidad es la similitud coseno y los grupos salen de un enlace simple con umbral 0.30.
- Dispone un grupo por columna y le pone a cada grupo un color de borde.
- Si una ventana cambia de título, se recalcula y puede cambiar de grupo.
- Expone un bus para agentes en un socket Unix (`$XDG_RUNTIME_DIR/fwm-<display>.sock`, o `/tmp`), con JSON por línea: `list`, `focus`, `affinity`, `set_threshold`. Rechaza cualquier comando que no esté en el `enum`.

## Qué NO hace todavía
- La "afinidad de tensores" sobre lo que se ve en pantalla (píxeles, embeddings de un modelo) no está: v0.1 usa clase y título.
- No hay EWMH completo, ni teclado, ni escritorios virtuales.
- Probado solo en un display virtual (Xvfb), no en la sesión de Federico.

## Verificar
```
cargo test                                   # 8 tests unitarios
cargo clippy --all-targets -- -D warnings    # sin warnings
bash scripts/prueba-integracion.sh           # Xvfb + fwm + 5 ventanas + chequeo por el bus
```
La prueba de integración falla si las ventanas del mismo tema no quedan juntas, si no hay exactamente 3 grupos, si hay solapes o si el bus acepta un comando desconocido. Deja una captura en `docs/captura-fwm.png`.

## Probar en tu máquina sin tocar tu sesión
Con `Xephyr :2 -screen 1280x800 &` y después `DISPLAY=:2 target/release/fwm`. No lo corras sobre tu display principal hasta que lo revisemos juntos.
