#!/usr/bin/env bash
set -euo pipefail
runner=${1:-./target/release/wordcraft-gnome}
cmd=("$runner"); [[ "$runner" != *.sh ]] || cmd=(bash "$runner")
export LEAF20_ACK_DIR
LEAF20_ACK_DIR=$(mktemp -d "${LEAF20_ACK_PARENT:-/tmp}/leaf20-ack.XXXXXX")
log=sweep20-pointer-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
"${cmd[@]}" --sweep20-pointer-test >"$log" 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true; rm -rf "$LEAF20_ACK_DIR"' EXIT
last=0
while kill -0 "$pid" 2>/dev/null; do
  count=$(grep -c '^SWEEP20_READY ' "$log" || true)
  if (( count <= last )); then sleep 0.05; continue; fi
  line=$(grep '^SWEEP20_READY ' "$log" | sed -n "$((last+1))p")
  read -r _ stage role ax ay bx by <<< "$line"
  window=$(xdotool search --onlyvisible --name '^Leaf Sweep20 Regression$' | head -n1)
  xdotool windowfocus --sync "$window"
  case "$role" in
    click) xdotool mousemove --window "$window" "$ax" "$ay" click 1 ;;
    second) xdotool key Home Down Return ;;
    cancel) xdotool key Escape ;;
    hover) xdotool mousemove --window "$window" "$ax" "$ay" ;;
    ruler-keyboard) xdotool key ctrl+alt+r ;;
    drag) xdotool mousemove --window "$window" "$ax" "$ay" mousedown 1 mousemove --sync --window "$window" "$bx" "$by"; sleep 0.3; xdotool mouseup 1 ;;
    cancel-drag) xdotool mousemove --window "$window" "$ax" "$ay" mousedown 1 mousemove --sync --window "$window" "$bx" "$by"; sleep 0.2; xdotool key Escape; xdotool mouseup 1 ;;
    right) xdotool key Right ;;
    undo) xdotool key ctrl+z ;;
    *) exit 1 ;;
  esac
  sleep 0.55
  scrot "${log%.log}-${stage}.png"
  touch "$LEAF20_ACK_DIR/$stage"
  last=$((last+1))
done
status=0
wait "$pid" || status=$?
cat "$log"
(( status == 0 ))
grep -q SWEEP20_POINTER_OK "$log"
