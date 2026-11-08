#!/usr/bin/env bash
set -euo pipefail
runner=${1:-./target/release/wordcraft-gnome}
command=("$runner"); [[ "$runner" != *.sh ]] || command=(bash "$runner")
export LEAF29_ACK_DIR
LEAF29_ACK_DIR=$(mktemp -d "${LEAF29_ACK_PARENT:-/tmp}/leaf38-ack.XXXXXX")
log=workflows38-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
"${command[@]}" --workflows38-ui-test > "$log" 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true; rm -rf "$LEAF29_ACK_DIR"' EXIT
for _ in $(seq 1 3000); do
  grep -q '^WORKFLOWS38_INPUT_READY' "$log" && break
  kill -0 "$pid" 2>/dev/null || { cat "$log";exit 1; }
  sleep .02
done
grep -q '^WORKFLOWS38_INPUT_READY' "$log" || { cat "$log";exit 1; }
window=$(xdotool search --onlyvisible --name '^Leaf Workflows38$' | head -n1)
xdotool windowfocus --sync "$window"
xdotool type --clearmodifiers --delay 2 'Animated typing'
printf ' Paste ñ' | xclip -selection clipboard
xdotool key ctrl+v
touch "$LEAF29_ACK_DIR/workflows38-input"
wait "$pid" || { cat "$log";exit 1; }
grep -q WORKFLOWS38_UI_OK "$log"
cat "$log"

"${command[@]}" --workflows38-prepare-startup > workflows38-prepare.log 2>&1 || { cat workflows38-prepare.log;exit 1; }
startup_log="workflows38-startup-${LANGUAGE:-en}.log"
"${command[@]}" --workflows38-startup-test > "$startup_log" 2>&1 &
pid=$!
for _ in $(seq 1 1500); do
  grep -q WORKFLOWS38_STARTUP_VISIBLE "$startup_log" && break
  kill -0 "$pid" 2>/dev/null || { cat "$startup_log";exit 1; }
  sleep .02
done
grep -q WORKFLOWS38_STARTUP_VISIBLE "$startup_log" || { cat "$startup_log";exit 1; }
scrot "workflows38-startup-${LANGUAGE:-en}-${GDK_SCALE:-1}.png"
touch "$LEAF29_ACK_DIR/workflows38-startup"
wait "$pid" || { cat "$startup_log";exit 1; }
grep -q WORKFLOWS38_STARTUP_OK "$startup_log"
