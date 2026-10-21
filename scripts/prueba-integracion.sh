#!/usr/bin/env bash
# Prueba de punta a punta en un display virtual: arranca Xvfb, fwm y 5 ventanas,
# y verifica por el bus IPC que se agrupan por tema y no se solapan.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release -q
B=target/release
source scripts/display-libre.sh; arrancar_xvfb
trap 'rc=$?; set +e; kill $(jobs -p) 2>/dev/null; wait 2>/dev/null; rm -rf "${XDG_RUNTIME_DIR:-}"; exit $rc' EXIT
export DISPLAY=$D
arrancar_fwm /tmp/fwm-prueba-${D#:}.log $B/fwm
$B/fwm-demo-ventana terminal "cargo build fwm" 1b2a3a &
$B/fwm-demo-ventana browser "x.com Cortex9Sur borrador" 2a1b33 &
$B/fwm-demo-ventana terminal "cargo test fwm" 1b3a2a &
$B/fwm-demo-ventana blender "Federica World tile T_0_0" 33301b &
$B/fwm-demo-ventana browser "Cortex9Sur publicado" 3a1b1b &
sleep 1
python3 scripts/verificar.py
$B/fwm-captura docs/captura-fwm.png >/dev/null
echo "captura: docs/captura-fwm.png"
