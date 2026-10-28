#!/bin/bash
# usage: run.sh <script.py> [app args]
# Runs the debug binary under Xvfb, drives it with the script (see
# drive.py for click/drag/key/shot) and reports the app's exit status.
# Screenshots go to $TRACEDRAW_SHOTS (default /tmp/tracedraw-visual).
# The splash screen is skipped unless TRACEDRAW_NO_SPLASH=0.
set -e
cd "$(dirname "$0")/../.."
export DISPLAY=:99
export TRACEDRAW_SHOTS="${TRACEDRAW_SHOTS:-/tmp/tracedraw-visual}"
mkdir -p "$TRACEDRAW_SHOTS/config"
LOG="$TRACEDRAW_SHOTS/app.log"
# Interface size 100 %: on Linux one interface point per pixel on this
# unscaled screen (the size the scripts' coordinates are taken at). Rui saves its preferences
# as they change, so each run starts from the defaults (the layout the
# coordinates are taken at) unless TRACEDRAW_KEEP_SETTINGS=1 keeps what
# the last run left.
python3 - "$TRACEDRAW_SHOTS/config/settings.json" "${TRACEDRAW_KEEP_SETTINGS:-0}" <<'PY'
import json, sys, os
s = {}
if sys.argv[2] == "1":
    try:
        s = json.load(open(sys.argv[1]))
    except (OSError, ValueError):
        s = {}
s["ui_size"] = 1.0
s["language"] = os.environ.get("RUI_TEST_LANGUAGE", "en")
json.dump(s, open(sys.argv[1], "w"), indent=2)
PY
Xvfb :99 -screen 0 1600x1000x24 >/dev/null 2>&1 & XPID=$!
sleep 1
SCRIPT=$1; shift
BIN=${RUI_BINARY:-./target/debug/rui}
(WINIT_UNIX_BACKEND=x11 TRACEDRAW_NO_SPLASH="${TRACEDRAW_NO_SPLASH:-1}" TRACEDRAW_CONFIG_DIR="$TRACEDRAW_SHOTS/config" LIBGL_ALWAYS_SOFTWARE=1 "$BIN" "$@" >"$LOG" 2>&1; echo "APP EXIT $?" >> "$LOG") &
sleep 6
STATUS=0
python3 "$(dirname "$0")/drive.py" "$SCRIPT" || STATUS=$?
sleep 1
if grep "APP EXIT" "$LOG"; then STATUS=1; else echo "app still running"; fi
if grep -i "panic" "$LOG"; then echo "PANIC in log"; STATUS=1; fi
pkill -x rui 2>/dev/null || true; sleep 0.5; kill $XPID 2>/dev/null || true
exit "$STATUS"
