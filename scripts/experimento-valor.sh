#!/usr/bin/env bash
# Experimento de valor del sistema 7 (métrica aprobada por Research):
# un agente tiene que encontrar la ventana de un tema entre N ventanas de Chrome
# que se llaman todas "Documento". Se compara:
#   A) bus IPC de fwm: pide la lista, elige por lo que la ventana muestra (OCR) y la enfoca
#   B) línea de base: xdotool search --name Documento, primera coincidencia, windowactivate
# Se mide tiempo por búsqueda y tasa de acierto contra la verdad (PID de cada Chrome).
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release -q
B=$PWD/target/release; P=$(mktemp -d)
source scripts/display-libre.sh; arrancar_xvfb
trap 'rc=$?; set +e; kill $(jobs -p) 2>/dev/null; pkill -f "user-data-dir=$P" 2>/dev/null; wait 2>/dev/null; rm -rf "$P" "${XDG_RUNTIME_DIR:-}"; exit $rc' EXIT
export DISPLAY=$D
arrancar_fwm /tmp/fwm-valor-${D#:}.log env FWM_VISION=${FWM_VISION:-1} $B/fwm
: > "$P/verdad.tsv"
abrir() { google-chrome --user-data-dir="$P/$1" --no-first-run --no-default-browser-check --disable-gpu \
  --app="data:text/html,<title>Documento</title><body style='background:white;color:black;font:34px sans-serif;margin:24px'>$3" >/dev/null 2>&1 &
  printf '%s\t%s\n' "$!" "$2" >> "$P/verdad.tsv"; }
abrir a clippy     "Compilar el crate con cargo y revisar los warnings de clippy"
abrir b cortex9sur "Borrador del post de Cortex9Sur sobre la nube soberana"
abrir c edenor     "Factura de luz de Edenor con vencimiento el viernes"
abrir d empanadas  "Receta de empanadas tucumanas al horno de barro"
abrir e river      "Entradas para ver a River en el Monumental el domingo"
abrir f tesis      "Capitulo dos de la tesis sobre permeabilidad cognitiva"
abrir g turno      "Turno con el clinico en el Hospital Italiano a las nueve"
abrir h vuelo      "Reserva del vuelo a Cordoba por Aerolineas Argentinas"
VERDAD=$P/verdad.tsv python3 scripts/verificar_valor.py
