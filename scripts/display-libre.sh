# Arranca un Xvfb propio en un display libre (Xvfb elige el número con -displayfd),
# así dos corridas al mismo tiempo nunca comparten pantalla ni bus.
arrancar_xvfb() {
  local f; f=$(mktemp)
  Xvfb -displayfd 3 -screen 0 1280x800x24 -nolisten tcp 3>"$f" >/dev/null 2>&1 &
  for _ in $(seq 1 50); do [ -s "$f" ] && break; sleep 0.1; done
  [ -s "$f" ] || { echo "no pude arrancar Xvfb" >&2; return 1; }
  D=":$(tr -d '[:space:]' < "$f")"; rm -f "$f"
}

# Arranca fwm con su bus en un directorio propio (mktemp -d) y no sigue hasta que
# el socket acepta conexiones: nada de sleeps fijos ni sockets viejos de otra corrida.
arrancar_fwm() {
  local log=$1; shift
  export XDG_RUNTIME_DIR; XDG_RUNTIME_DIR=$(mktemp -d)
  "$@" 2>"$log" &
  local s="$XDG_RUNTIME_DIR/fwm-${D//[^[:alnum:]]/}.sock"
  for _ in $(seq 1 100); do
    python3 -c "import socket,sys;s=socket.socket(socket.AF_UNIX);s.connect(sys.argv[1])" "$s" 2>/dev/null && return 0
    sleep 0.1
  done
  echo "fwm no abrió su bus en $s" >&2; cat "$log" >&2; return 1
}
