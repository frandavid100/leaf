#!/usr/bin/env bash
set -euo pipefail
runner=${1:-./target/release/wordcraft-gnome}
cmd=("$runner"); [[ "$runner" != *.sh ]] || cmd=(bash "$runner")
export LEAF29_ACK_DIR
LEAF29_ACK_DIR=${LEAF29_ACK_DIR:-$(mktemp -d)}
log=alpha31-captions-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
rm -f "$LEAF29_ACK_DIR"/caption-*
"${cmd[@]}" --captions-ui-test >"$log" 2>&1 &
pid=$!
trap 'scrot "${log%.log}-failure.png" 2>/dev/null || true; kill "$pid" 2>/dev/null || true' EXIT
for stage in $(seq 0 6); do
  for attempt in $(seq 1 600); do
    grep -q "^CAPTION_INPUT $stage " "$log" && break
    kill -0 "$pid" || { cat "$log"; exit 1; }
    sleep 0.05
  done
  read -r _ _ role x y < <(grep "^CAPTION_INPUT $stage " "$log")
  window=$(xdotool search --onlyvisible --name '^Leaf Caption Regression$' | head -n1)
  xdotool windowfocus --sync "$window"
  sleep 0.35
  xdotool mousemove --sync "$x" "$y"
  sleep 0.08
  xdotool mousemove --sync "$((x+1))" "$((y+1))"
  sleep 0.08
  if [[ "$role" != hover ]]; then
    for attempt in $(seq 1 60); do
      grep -q "^POINTER_TARGET_ENTERED CAPTION_INPUT $stage " "$log" && break
      kill -0 "$pid" || { cat "$log"; exit 1; }
      xdotool mousemove --sync "$x" "$y"
      sleep 0.04
      xdotool mousemove --sync "$((x+1))" "$((y+1))"
      sleep 0.04
    done
    grep -q "^POINTER_TARGET_ENTERED CAPTION_INPUT $stage " "$log" || { cat "$log"; exit 1; }
    scrot "${log%.log}-$stage-before.png"
    xdotool mousedown 1
    sleep 0.12
    xdotool mouseup 1
  fi
  if [[ "$role" == type ]]; then
    sleep 0.4
    xdotool type --clearmodifiers --delay 10 'Authored caption'
  fi
  sleep 0.4
  scrot "${log%.log}-$stage.png"
  touch "$LEAF29_ACK_DIR/caption-$stage"
done
wait "$pid" || { cat "$log"; exit 1; }
grep -q CAPTION_UI_OK "$log"
trap - EXIT
