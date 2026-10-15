"""Scripts for the headless `ui_shot` example (crates/ui-egui/examples/ui_shot.rs) used in the memory
measurements. Usage: python -I make_ui_scripts.py <dir with the test documents>.

Every script renders a picture after each step ({"shot": ...}). egui_kittest hands work to the GPU
only when it renders, so a script that only runs {"steps": n} piles up every texture upload and
release and shows gigabytes the real app never uses.
"""
import json, os, sys

d = sys.argv[1].replace("\\", "/")
throwaway = f"{d}/throwaway.png"

def ex(cmd, params=None):
    return {"method": "engine.execute", "params": {"command": cmd, "params": params or {}}}

def write(name, lines):
    with open(os.path.join(d, name), "w", encoding="utf-8", newline="\n") as f:
        for line in lines:
            f.write(json.dumps(line) + "\n")
    print("wrote", name)

def opened(doc):
    return [{"empty": True}, ex("file.open", {"path": f"{d}/{doc}.docx"}), {"steps": 10}, {"shot": throwaway}]

def pages(n):
    out = []
    for _ in range(n):
        out += [ex("caret.pageDown"), {"steps": 3}, {"shot": throwaway}]
    return out

hold = [{"wait": 4000}, {"shot": throwaway}]
write("ui-empty.jsonl", [{"empty": True}, {"steps": 10}, {"shot": throwaway}] + hold)
write("ui-sample.jsonl", [{"steps": 10}, {"shot": throwaway}] + hold)
write("ui-text-open.jsonl", opened("text-100p") + hold)
write("ui-text-100p.jsonl", opened("text-100p") + pages(95) + hold)
write("ui-pics-100p.jsonl", opened("pics-100p") + pages(95) + hold)
write("ui-zoom200.jsonl", opened("text-100p") + [ex("view.zoom", {"value": 200}), {"steps": 6}, {"shot": throwaway}] + pages(30) + hold)
for doc in ("latin", "cjk", "emoji", "cjk-emoji"):
    write(f"ui-font-{doc}.jsonl", opened(doc) + hold)
