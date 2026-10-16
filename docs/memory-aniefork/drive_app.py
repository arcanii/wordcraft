"""Drive a running WordCraft window over its control channel for memory measurements: wait for its
key, size the window, optionally zoom, page down through the document, optionally save it and
type into it, hold, print the window process's memory by region (macOS `vmmap --summary`), the
size of the profile's recovery folder and any recovery or AutoSave lines in its log, then close
the document (discarding what was typed) and quit the app.

Usage: python3 -I drive_app.py <port> <profile dir> [--pages N] [--zoom PERCENT] [--save 1]
                               [--type SECONDS] [--hold SECONDS] [--vmmap 1]

--save 1 saves the document first, which is what lets AutoSave run; --type types a letter every
quarter of a second, so recovery copies (every 10 s while there are unsaved changes) and AutoSave
run as they would for someone writing.

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
if opts.get("--save") == "1":
    execute("file.save")
    time.sleep(1)
typing = float(opts.get("--type", 0))
end = time.monotonic() + typing
while time.monotonic() < end:
    execute("text.insert", {"text": "a"})
    time.sleep(0.25)
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
if typing:
    size = sum(os.path.getsize(os.path.join(r, f)) for r, _, fs in os.walk(os.path.join(profile, "recovery")) for f in fs)
    print(f"recovery folder: {size / 2**20:.0f} MB")
    for root, _, files in os.walk(os.path.join(profile, "logs")):
        for f in files:
            lines = [l.strip() for l in open(os.path.join(root, f), errors="replace") if "recovery" in l.lower() or "autosave" in l.lower()]
            for l in lines[-6:]:
                print("log:", l[:200])
    # Typed changes are discarded, so quitting doesn't stop at a save prompt.
    execute("file.close", {"force": True})
call("app.quit")
