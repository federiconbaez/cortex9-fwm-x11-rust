import json, os, random, socket, statistics, subprocess, sys, time, unicodedata
ruta = f"{os.environ.get('XDG_RUNTIME_DIR','/tmp')}/fwm-{''.join(c for c in os.environ['DISPLAY'] if c.isalnum())}.sock"
def pedir(m):
    s = socket.socket(socket.AF_UNIX); s.connect(ruta); f = s.makefile("rw")
    f.write(json.dumps(m) + "\n"); f.flush(); r = json.loads(f.readline()); s.close(); return r
sh = lambda *a: subprocess.run(a, capture_output=True, text=True).stdout.strip()
norm = lambda t: unicodedata.normalize("NFKD", t).encode("ascii", "ignore").decode().lower()
pid_tema = dict(l.split("\t") for l in open(os.environ["VERDAD"]).read().split("\n") if l)
N = len(pid_tema)
# esperar a que estén las N ventanas y el OCR haya leído todas
t0 = time.time(); vs = []
while time.time() - t0 < float(os.environ.get("ESPERA", "300")):
    try: vs = pedir({"cmd": "list"})["ventanas"]
    except OSError: time.sleep(0.5); continue
    if len(vs) == N and all(len(v["pantalla"]) > 20 for v in vs): break
    time.sleep(1)
listo = time.time() - t0
vs = pedir({"cmd": "list"})["ventanas"]
verdad = {}
for v in vs:
    pid = sh("xprop", "-id", str(v["id"]), "_NET_WM_PID").split("=")[-1].strip()
    if pid in pid_tema: verdad[pid_tema[pid]] = v["id"]
print(f"  {len(vs)} ventanas, OCR listo en {listo:.1f}s, verdad resuelta para {len(verdad)}/{N}")
for v in vs: print(f"  0x{v['id']:x} {v['titulo']!r} grupo {v['grupo']} pantalla: {v['pantalla'][:60]!r}")
def por_bus(q):
    vs = pedir({"cmd": "list"})["ventanas"]
    mejor = max(vs, key=lambda v: norm(v["pantalla"]).count(q))
    if q not in norm(mejor["pantalla"]): return None
    pedir({"cmd": "focus", "id": mejor["id"]}); return mejor["id"]
def por_xdotool(q):
    ids = sh("xdotool", "search", "--name", "Documento").split()
    if not ids: return None
    sh("xdotool", "windowactivate", "--sync", ids[0]); return int(ids[0])
random.seed(7); consultas = [t for t in verdad for _ in range(5)]; random.shuffle(consultas)
res = {}
for nombre, f in (("bus fwm", por_bus), ("xdotool", por_xdotool)):
    ts, ok = [], 0
    for q in consultas:
        a = time.perf_counter(); elegido = f(q)
        for _ in range(50):
            if sh("xdotool", "getactivewindow") == str(elegido or -1): break
            time.sleep(0.01)
        ts.append((time.perf_counter() - a) * 1000)
        ok += elegido == verdad[q] and sh("xdotool", "getactivewindow") == str(elegido)
    res[nombre] = (ok, ts)
    print(f"  {nombre:8}: acierto {ok}/{len(consultas)} ({100*ok/len(consultas):.0f}%), "
          f"tiempo mediana {statistics.median(ts):.1f} ms, p90 {sorted(ts)[int(.9*len(ts))-1]:.1f} ms")
json.dump({k: {"aciertos": o, "n": len(consultas), "ms": t} for k, (o, t) in res.items()} | {"ocr_listo_s": listo, "N": N},
          open("docs/valor-resultado.json", "w"), indent=1)
fallas = []
if len(verdad) != N: fallas.append("no se pudo resolver la verdad de todas las ventanas")
if res["bus fwm"][0] <= res["xdotool"][0]: fallas.append("el bus no superó a la línea de base")
print("FALLAS:" if fallas else "OK: el agente encuentra la ventana del tema por el bus mejor que por título")
for x in fallas: print("  -", x)
sys.exit(1 if fallas else 0)
