import json, os, socket, sys, time
ruta = f"/tmp/fwm-{''.join(c for c in os.environ['DISPLAY'] if c.isalnum())}.sock"
def lista():
    s = socket.socket(socket.AF_UNIX); s.connect(ruta); f = s.makefile("rw")
    f.write('{"cmd":"list"}\n'); f.flush(); r = json.loads(f.readline()); s.close(); return r["ventanas"]
t0 = time.time(); vs = []
while time.time() - t0 < float(os.environ.get("ESPERA", "40")):
    try: vs = lista()
    except OSError: time.sleep(0.5); continue
    if len(vs) == 4 and all(len(v["pantalla"]) > 20 for v in vs): break
    time.sleep(0.5)
time.sleep(3); vs = lista()
print(f"  OCR listo en {time.time()-t0:.1f}s")
for v in vs: print(f"  grupo {v['grupo']}  {v['titulo']!r}  pantalla: {v['pantalla'][:70]!r}")
tema = lambda v: "rust" if "cargo" in v["pantalla"].lower() else ("cortex" if "cortex" in v["pantalla"].lower() else "?")
fallas = []
if len(vs) != 4: fallas.append(f"esperaba 4 ventanas, hay {len(vs)}")
if any(tema(v) == "?" for v in vs): fallas.append("el OCR no leyó el tema de alguna ventana")
g = {}
for v in vs: g.setdefault(tema(v), set()).add(v["grupo"])
if not fallas:
    if len(g["rust"]) != 1 or len(g["cortex"]) != 1: fallas.append("un tema quedó partido en varios grupos")
    elif g["rust"] == g["cortex"]: fallas.append("los dos temas quedaron en el mismo grupo")
print("FALLAS:" if fallas else "OK: mismo título en las 4, separadas en 2 grupos por lo que muestran en pantalla")
for x in fallas: print("  -", x)
sys.exit(1 if fallas else 0)
