#!/usr/bin/env bash
set -euo pipefail
runner=${1:-./target/release/wordcraft-gnome}
cmd=("$runner"); [[ "$runner" != *.sh ]] || cmd=(bash "$runner")
export LEAF29_ACK_DIR
LEAF29_ACK_DIR=$(mktemp -d "${LEAF29_ACK_PARENT:-/tmp}/leaf136-ack.XXXXXX")
log=alpha31-format136-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
"${cmd[@]}" --format136-ui-test > "$log" 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true; rm -rf "$LEAF29_ACK_DIR"' EXIT
stage=0
while kill -0 "$pid" 2>/dev/null; do
  if ! grep -q "^FORMAT136_READY $stage " "$log"; then sleep 0.05; continue; fi
  read -r _ _ role x y < <(grep "^FORMAT136_READY $stage " "$log" | tail -n1)
  window=$(xdotool search --onlyvisible --name '^Leaf Format136 Regression$' | head -n1)
  xdotool windowfocus --sync "$window"
  xdotool mousemove --sync "$x" "$y" click 1
  if [[ "$role" == hex ]]; then
    xdotool key ctrl+a
    xdotool type --clearmodifiers --delay 20 '#234567'
    xdotool key Return
  fi
  sleep 0.35
  if [[ "$role" == hex || $((stage % 4)) == 0 ]]; then
    scrot "${log%.log}-$stage-$role.png"
  fi
  touch "$LEAF29_ACK_DIR/format136-$stage"
  stage=$((stage+1))
done
wait "$pid" || { cat "$log"; exit 1; }
cat "$log"
grep -q FORMAT136_UI_OK "$log"
