#!/usr/bin/env bash
set -euo pipefail
export LEAF29_ACK_DIR
LEAF29_ACK_DIR=$(mktemp -d "${LEAF29_ACK_PARENT:-/tmp}/leaf29-ack.XXXXXX")
log=alpha29-pointer-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
runner=${1:-./target/release/wordcraft-gnome}
cmd=("$runner"); [[ "$runner" != *.sh ]] || cmd=(bash "$runner")
review_log=alpha29-review-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
"${cmd[@]}" --review29-ui-test > "$review_log" 2>&1 || { cat "$review_log"; exit 1; }
grep -q REVIEW29_UI_OK "$review_log"
bash tools/alpha30-pointer-test.sh "$runner"
bash tools/alpha31-pointer-test.sh "$runner"
"${cmd[@]}" --alpha29-ui-test > "$log" 2>&1 &
pid=$!
trap 'scrot "${log%.log}-final.png" 2>/dev/null || true; kill "$pid" 2>/dev/null || true; rm -rf "$LEAF29_ACK_DIR"' EXIT
for stage in $(seq 0 22); do
  for attempt in $(seq 1 600); do
    grep -q "^ALPHA29_POINTER $stage " "$log" && break
    kill -0 "$pid" 2>/dev/null || { cat "$log"; exit 1; }
    sleep 0.1
  done
  read -r _ _ role ax ay bx by < <(grep "^ALPHA29_POINTER $stage " "$log")
  window=$(xdotool search --onlyvisible --name '^Leaf Alpha29 Regression$' | head -n1)
  xdotool windowfocus --sync "$window"
  if [[ "$role" == click ]]; then
    xdotool mousemove --window "$window" "$ax" "$ay" click 1
  else
  xdotool mousemove --window "$window" "$ax" "$ay" mousedown 1 mousemove --sync --window "$window" "$bx" "$by" 
  fi
  sleep 0.2
  scrot "${log%.log}-$stage-preview.png"
  if [[ "$role" == cancel ]]; then xdotool key Escape; fi
  xdotool mouseup 1
  sleep 0.2
  touch "$LEAF29_ACK_DIR/$stage"
done
wait "$pid" || { cat "$log"; exit 1; }
cat "$log"
grep -q ALPHA29_CONTROLS_OK "$log"
grep -q ALPHA29_POINTER_OK "$log"
