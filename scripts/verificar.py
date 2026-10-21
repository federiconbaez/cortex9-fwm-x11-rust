import json, os, socket, sys
ruta = f"{os.environ.get('XDG_RUNTIME_DIR','/tmp')}/fwm-{''.join(c for c in os.environ['DISPLAY'] if c.isalnum())}.sock"
def pedir(*msgs):
    s = socket.socket(socket.AF_UNIX); s.connect(ruta)
    f = s.makefile("rw")
    out = []
    for m in msgs:
        f.write(json.dumps(m) + "\n"); f.flush(); out.append(json.loads(f.readline()))
    s.close(); return out
(lista,) = pedir({"cmd": "list"})
vs = lista["ventanas"]
for v in vs: print(f"  grupo {v['grupo']}  {v['clase']:9} {v['titulo']!r:32} {v['rect']}")
por = {v["titulo"]: v["grupo"] for v in vs}
fallas = []
if len(vs) != 5: fallas.append(f"esperaba 5 ventanas, hay {len(vs)}")
if por.get("cargo build fwm") != por.get("cargo test fwm"): fallas.append("las terminales no quedaron juntas")
if por.get("x.com Cortex9Sur borrador") != por.get("Cortex9Sur publicado"): fallas.append("los navegadores no quedaron juntos")
if len(set(por.values())) != 3: fallas.append(f"esperaba 3 grupos, hay {len(set(por.values()))}")
R = [v["rect"] for v in vs]
for i, a in enumerate(R):
    for b in R[i+1:]:
        if a["x"] < b["x"]+b["w"] and b["x"] < a["x"]+a["w"] and a["y"] < b["y"]+b["h"] and b["y"] < a["y"]+a["h"]:
            fallas.append(f"solape {a} {b}")
foco, malo, inyeccion = pedir({"cmd": "focus", "id": vs[0]["id"]}, {"cmd": "focus", "id": 1}, {"cmd": "exec", "sh": "rm -rf /"})
if not foco["ok"]: fallas.append("focus falló")
if malo["ok"]: fallas.append("focus aceptó una ventana ajena")
if inyeccion["ok"]: fallas.append("el bus aceptó un comando desconocido")
ids = {v["titulo"]: v["id"] for v in vs}
(mismo, distinto) = pedir({"cmd": "affinity", "a": ids["cargo build fwm"], "b": ids["cargo test fwm"]},
                          {"cmd": "affinity", "a": ids["cargo build fwm"], "b": ids["Cortex9Sur publicado"]})
print("  afinidad terminal vs terminal:", round(mismo["afinidad"], 3), "| terminal vs navegador:", round(distinto["afinidad"], 3))
if not mismo["afinidad"] > distinto["afinidad"]: fallas.append("la afinidad no distingue temas")
print("FALLAS:" if fallas else "OK: 5 ventanas, 3 grupos por tema, sin solapes, bus rechaza lo desconocido")
for x in fallas: print("  -", x)
sys.exit(1 if fallas else 0)
