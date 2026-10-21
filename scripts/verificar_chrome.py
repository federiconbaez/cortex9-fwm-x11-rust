import json, os, socket, subprocess, sys, time
ruta = f"/tmp/fwm-{''.join(c for c in os.environ['DISPLAY'] if c.isalnum())}.sock"
def pedir(m):
    s = socket.socket(socket.AF_UNIX); s.connect(ruta); f = s.makefile("rw")
    f.write(json.dumps(m) + "\n"); f.flush(); r = json.loads(f.readline()); s.close(); return r
vs = pedir({"cmd": "list"})["ventanas"]
for v in vs: print(f"  grupo {v['grupo']}  {v['clase'][:20]:20} {v['titulo']!r}")
fallas = []
por = {}
for v in vs:
    for t in ("cargo test fwm", "cargo build fwm", "Cortex9Sur borrador", "Cortex9Sur publicado"):
        if t in v["titulo"]: por[t] = v
if len(por) != 4: fallas.append(f"esperaba 4 ventanas de Chrome con título, encontré {len(por)}")
else:
    if por["cargo test fwm"]["grupo"] != por["cargo build fwm"]["grupo"]: fallas.append("las de cargo no quedaron juntas")
    if por["Cortex9Sur borrador"]["grupo"] != por["Cortex9Sur publicado"]["grupo"]: fallas.append("las de Cortex9Sur no quedaron juntas")
    if por["cargo test fwm"]["grupo"] == por["Cortex9Sur borrador"]["grupo"]: fallas.append("mezcló los dos temas")
    lista = subprocess.run(["xprop", "-root", "_NET_CLIENT_LIST"], capture_output=True, text=True).stdout
    if not all(hex(v["id"]) in lista for v in por.values()): fallas.append("_NET_CLIENT_LIST incompleto")
    objetivo = por["Cortex9Sur publicado"]["id"]
    subprocess.run(["xdotool", "windowactivate", str(objetivo)], capture_output=True); time.sleep(0.5)
    act = subprocess.run(["xdotool", "getactivewindow"], capture_output=True, text=True).stdout.strip()
    print("  xdotool activó:", act, "esperado:", objetivo)
    if act != str(objetivo): fallas.append("xdotool windowactivate no quedó como ventana activa")
print("FALLAS:" if fallas else "OK: Chrome real, 2 temas en 2 grupos, EWMH visible y xdotool activa ventanas")
for x in fallas: print("  -", x)
sys.exit(1 if fallas else 0)
