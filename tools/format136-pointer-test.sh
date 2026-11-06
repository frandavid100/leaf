#!/usr/bin/env bash
set -euo pipefail
runner=${1:-./target/release/wordcraft-gnome}
cmd=("$runner"); [[ "$runner" != *.sh ]] || cmd=(bash "$runner")
export LEAF29_ACK_DIR
LEAF29_ACK_DIR=$(mktemp -d "${LEAF29_ACK_PARENT:-/tmp}/leaf136-ack.XXXXXX")
log=alpha31-format136-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
"${cmd[@]}" --format136-ui-test > "$log" 2>&1 &
pid=$!
trap 'scrot "${log%.log}-failure.png" 2>/dev/null || true; kill "$pid" 2>/dev/null || true; rm -rf "$LEAF29_ACK_DIR"' EXIT
stage=0
while kill -0 "$pid" 2>/dev/null; do
  if ! grep -q "^FORMAT136_READY $stage " "$log"; then sleep 0.05; continue; fi
  read -r _ _ role x y < <(grep "^FORMAT136_READY $stage " "$log" | tail -n1)
  window=$(xdotool search --onlyvisible --name '^Leaf Format136 Regression$' | head -n1)
  xdotool windowfocus --sync "$window"
  # The diagnostic can map a new family while the previous popover is still
  # relinquishing its native popup grab. Wait for that transition before the
  # single pointer activation; keep the coordinates and assertions unchanged.
  sleep 0.35
  # Let GTK/X11 consume popup entry and the following in-surface motion before
  # pressing, as in the existing text-pointer regression.
  xdotool mousemove --sync "$x" "$y"
  sleep 0.08
  xdotool mousemove --sync "$((x+1))" "$((y+1))"
  sleep 0.08
  # Require actual GTK entry into this exact control on its popup surface.
  # Repeat only motion while mapping settles; activate it exactly once.
  for attempt in $(seq 1 60); do
    grep -q "^POINTER_TARGET_ENTERED FORMAT136_READY $stage " "$log" && break
    kill -0 "$pid" || { cat "$log"; exit 1; }
    xdotool mousemove --sync "$x" "$y"
    sleep 0.04
    xdotool mousemove --sync "$((x+1))" "$((y+1))"
    sleep 0.04
  done
  grep -q "^POINTER_TARGET_ENTERED FORMAT136_READY $stage " "$log" || { cat "$log"; exit 1; }
  scrot "${log%.log}-$stage-before.png"
  xdotool mousedown 1
  sleep 0.12
  xdotool mouseup 1
  if [[ "$role" == hex ]]; then
    xdotool key ctrl+a
    xdotool type --clearmodifiers --delay 20 '#234567'
    xdotool key Return
  fi
  sleep 0.35
  scrot "${log%.log}-$stage-$role.png"
  touch "$LEAF29_ACK_DIR/format136-$stage"
  stage=$((stage+1))
done
wait "$pid" || { cat "$log"; exit 1; }
cat "$log"
grep -q FORMAT136_UI_OK "$log"
rm -rf "$LEAF29_ACK_DIR"
trap - EXIT
