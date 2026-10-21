# Arranca un Xvfb propio en un display libre (Xvfb elige el número con -displayfd),
# así dos corridas al mismo tiempo nunca comparten pantalla ni bus.
arrancar_xvfb() {
  local f; f=$(mktemp)
  Xvfb -displayfd 3 -screen 0 1280x800x24 -nolisten tcp 3>"$f" >/dev/null 2>&1 &
  for _ in $(seq 1 50); do [ -s "$f" ] && break; sleep 0.1; done
  [ -s "$f" ] || { echo "no pude arrancar Xvfb" >&2; return 1; }
  D=":$(tr -d '[:space:]' < "$f")"; rm -f "$f"
}
