"""Drive a running WordCraft window over its control channel for memory measurements: wait for its
key, size the window, optionally zoom, page down through the document, hold, print the window
process's memory by region (macOS `vmmap --summary`), then quit the app.

Usage: python3 -I drive_app.py <port> <profile dir> [--pages N] [--zoom PERCENT] [--hold SECONDS]
                               [--vmmap 1]

Start the app first with `WORDCRAFT_PROFILE_DIR=<profile dir>` (so the run leaves the user's own
preferences, recent files and recovery copies alone), `--control <port>` and the document, for
example under measure_macos.py:

  python3 -I measure_macos.py "app text-100p" WORDCRAFT_PROFILE_DIR=<dir> -- <wordcraft> --control 7987 <doc> &
  python3 -I drive_app.py 7987 <dir> --pages 95 --vmmap 1
"""
import json, os, socket, subprocess, sys, time

port, profile = int(sys.argv[1]), sys.argv[2]
opts = dict(zip(sys.argv[3::2], sys.argv[4::2]))
pages, hold = int(opts.get("--pages", 0)), float(opts.get("--hold", 5))

def key_file():
    for root, _, files in os.walk(profile):
        if f"control-key.local-{port}" in files:
            return os.path.join(root, f"control-key.local-{port}")
    return None

deadline = time.monotonic() + 60
while (path := key_file()) is None:
    if time.monotonic() > deadline:
        sys.exit("drive_app: the app wrote no control key within 60 s")
    time.sleep(0.2)
time.sleep(0.5)
key = open(path).read().strip()
conn = socket.create_connection(("127.0.0.1", port)).makefile("rw")
ids = 0

def call(method, params=None):
    global ids
    ids += 1
    conn.write(json.dumps({"id": ids, "key": key, "method": method, "params": params or {}}) + "\n")
    conn.flush()
    reply = json.loads(conn.readline() or "{}")
    if not reply.get("ok"):
        print(f"drive_app: {method} failed: {reply.get('error')}", file=sys.stderr)
    return reply.get("result")

def execute(command, params=None):
    return call("engine.execute", {"command": command, "params": params or {}})

# The headless measurements' window: 1440 x 900 points.
call("ui.resize", {"width": 1440, "height": 900})
time.sleep(2)
if "--zoom" in opts:
    execute("view.zoom", {"value": int(opts["--zoom"])})
    time.sleep(1)
for _ in range(pages):
    execute("caret.pageDown")
    # Time for the window to draw the pages now in view.
    time.sleep(0.15)
time.sleep(hold)
if opts.get("--vmmap") == "1":
    pids = subprocess.run(["lsof", "-ti", f"tcp:{port}", "-sTCP:LISTEN"], capture_output=True, text=True).stdout.split()
    if pids:
        out = subprocess.run(["vmmap", "--summary", pids[0]], capture_output=True, text=True).stdout
        keep = ("Physical footprint", "REGION TYPE", "IOAccelerator", "IOSurface", "Malloc Large", "Malloc Small", "owned unmapped", "TOTAL")
        print("\n".join(l for l in out.splitlines() if l.startswith(keep)))
        # vmmap rounds the total to 0.1 GB; footprint gives it in MB.
        total = subprocess.run(["footprint", pids[0]], capture_output=True, text=True).stdout
        print("\n".join(l.strip() for l in total.splitlines() if "Footprint:" in l))
call("app.quit")
