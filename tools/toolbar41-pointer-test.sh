#!/usr/bin/env bash
# Exercise the actual native drag handle before the preference/resize assertions.
set -euo pipefail
log=${LEAF_TOOLBAR_LOG:-toolbar41-pointer.log}
leaf_runner=${1:-./target/release/wordcraft-gnome}
leaf_cmd=("$leaf_runner")
if [[ "$leaf_runner" = *.sh ]]; then leaf_cmd=(bash "$leaf_runner"); fi
LEAF_TOOLBAR_DND=1 "${leaf_cmd[@]}" --toolbar41-ui-test >"$log" 2>&1 &
leaf_pid=$!
trap 'kill "$leaf_pid" 2>/dev/null || true' EXIT
for attempt in $(seq 1 500); do
  if grep -q '^TOOLBAR41_DRAG ' "$log"; then break; fi
  kill -0 "$leaf_pid" || { cat "$log"; exit 1; }
  sleep 0.1
done
grep -q '^TOOLBAR41_DRAG ' "$log" || { cat "$log"; exit 1; }
read -r _ sx sy tx ty < <(grep '^TOOLBAR41_DRAG ' "$log" | tail -n1)
# Widget bounds are logical units; XTest pointer coordinates are device pixels.
scale=${GDK_SCALE:-1}
sx=$((sx*scale)); sy=$((sy*scale)); tx=$((tx*scale)); ty=$((ty*scale))
leaf_window=$(xdotool search --onlyvisible --name '^Leaf Toolbar41$' | head -n1)
xdotool windowfocus --sync "$leaf_window"
xdotool mousemove --window "$leaf_window" "$sx" "$sy"
sleep 0.1
xdotool mousedown 1
for third in 1 2 3; do
  xdotool mousemove --window "$leaf_window" "$((sx+(tx-sx)*third/3))" "$((sy+(ty-sy)*third/3))"
  sleep 0.12
done
sleep 0.3
xdotool mouseup 1
wait "$leaf_pid" || { cat "$log"; exit 1; }
grep -q TOOLBAR41_DRAG_OK "$log"
grep -q TOOLBAR41_UI_OK "$log"
cat "$log"
trap - EXIT
