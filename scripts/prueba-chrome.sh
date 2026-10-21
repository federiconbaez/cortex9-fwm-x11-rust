#!/usr/bin/env bash
# Prueba con una aplicación real: 4 ventanas de Google Chrome con títulos de dos temas.
# Verifica agrupamiento, EWMH (_NET_CLIENT_LIST, _NET_ACTIVE_WINDOW) y activación con xdotool.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release -q
B=$PWD/target/release; P=$(mktemp -d)
source scripts/display-libre.sh; arrancar_xvfb
trap 'kill $(jobs -p) 2>/dev/null; pkill -f "user-data-dir=$P" 2>/dev/null; rm -rf "$P"' EXIT
export DISPLAY=$D
$B/fwm 2>/tmp/fwm-chrome-${D#:}.log & sleep 0.5
abrir() { google-chrome --user-data-dir="$P/$1" --no-first-run --no-default-browser-check --disable-gpu \
  --app="data:text/html,<title>$2</title><body style='background:$3;color:white;font:40px sans-serif'><h1>$2</h1>" >/dev/null 2>&1 & }
abrir a "cargo test fwm" "%23123" ; abrir b "Cortex9Sur borrador" "%23412"
abrir c "cargo build fwm" "%23132"; abrir d "Cortex9Sur publicado" "%23421"
for i in $(seq 1 40); do n=$(xprop -root _NET_CLIENT_LIST 2>/dev/null | tr -cd ',' | wc -c); [ "$n" -ge 3 ] && break; sleep 0.5; done
sleep 2
python3 scripts/verificar_chrome.py
$B/fwm-captura docs/captura-chrome.png >/dev/null && echo "captura: docs/captura-chrome.png"
