#!/usr/bin/env bash
# Prueba de lo que pidió Federico: agrupar por lo que las ventanas MUESTRAN.
# 4 ventanas de Chrome con el mismo título ("Documento") y contenido de dos temas.
# Sin visión quedan todas juntas; con OCR se tienen que separar por tema.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release -q
D=:81; B=$PWD/target/release; P=$(mktemp -d)
Xvfb $D -screen 0 1280x800x24 -nolisten tcp >/dev/null 2>&1 &
trap 'kill $(jobs -p) 2>/dev/null; pkill -f "user-data-dir=$P" 2>/dev/null; rm -rf "$P"' EXIT
sleep 1; export DISPLAY=$D
FWM_VISION=${FWM_VISION:-1} $B/fwm 2>/tmp/fwm-vision.log & sleep 0.5
abrir() { google-chrome --user-data-dir="$P/$1" --no-first-run --no-default-browser-check --disable-gpu \
  --app="data:text/html,<title>Documento</title><body style='background:white;color:black;font:34px sans-serif;margin:24px'>$2" >/dev/null 2>&1 & }
abrir a "Compilar el crate con cargo build y revisar los warnings de clippy en Rust"
abrir b "Borrador del post de Cortex9Sur sobre la nube soberana argentina"
abrir c "Correr cargo test para el crate en Rust y medir la cobertura"
abrir d "Cortex9Sur publico el post sobre la nube soberana y los datos"
python3 scripts/verificar_vision.py
$B/fwm-captura docs/captura-vision.png >/dev/null && echo "captura: docs/captura-vision.png"
