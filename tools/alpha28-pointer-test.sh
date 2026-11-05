#!/usr/bin/env bash
set -euo pipefail
export LEAF28_ACK_DIR
LEAF28_ACK_DIR=$(mktemp -d)
log=alpha28-pointer-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
./target/release/wordcraft-gnome --alpha28-ui-test > "$log" 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true; rm -rf "$LEAF28_ACK_DIR"' EXIT
for phase in 0 1; do
  for attempt in $(seq 1 600); do
    grep -q "^ALPHA28_POINTER $phase " "$log" && break
    kill -0 "$pid" 2>/dev/null || { cat "$log"; exit 1; }
    sleep 0.1
  done
  read -r _ _ ax ay bx by < <(grep "^ALPHA28_POINTER $phase " "$log")
  window=$(xdotool search --onlyvisible --class Leaf | head -n1)
  xdotool windowfocus --sync "$window"
  xdotool mousemove --window "$window" "$ax" "$ay" mousedown 1 mousemove --sync --window "$window" "$bx" "$by"
  sleep 0.3
  scrot "${log%.log}-$phase-preview.png"
  xdotool mouseup 1
  sleep 0.3
  touch "$LEAF28_ACK_DIR/$phase"
done
wait "$pid" || { cat "$log"; exit 1; }
cat "$log"
grep -q ALPHA28_UI_OK "$log"
grep -q ALPHA28_POINTER_OK "$log"
