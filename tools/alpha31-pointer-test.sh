#!/usr/bin/env bash
set -euo pipefail
runner=${1:-./target/release/wordcraft-gnome}
cmd=("$runner"); [[ "$runner" != *.sh ]] || cmd=(bash "$runner")
export LEAF29_ACK_DIR
LEAF29_ACK_DIR=${LEAF29_ACK_DIR:-$(mktemp -d)}
rm -f "$LEAF29_ACK_DIR/alpha31-navigation"
log=alpha31-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
"${cmd[@]}" --alpha31-ui-test >"$log" 2>&1 &
pid=$!
trap 'scrot "${log%.log}-failure.png" 2>/dev/null || true; kill "$pid" 2>/dev/null || true' EXIT
for attempt in $(seq 1 600); do
  grep -q '^ALPHA31_NAV_READY ' "$log" && break
  kill -0 "$pid" || { cat "$log"; exit 1; }
  sleep 0.05
done
read -r _ x y < <(grep '^ALPHA31_NAV_READY ' "$log")
window=$(xdotool search --onlyvisible --name '^Leaf Alpha31 Regression$' | head -n1)
xdotool windowfocus --sync "$window"
xdotool mousemove --window "$window" "$x" "$y" click 1
xdotool key End Return
sleep 0.15
scrot "${log%.log}-structure.png"
xdotool key space Home Return
sleep 0.15
scrot "${log%.log}-pages.png"
touch "$LEAF29_ACK_DIR/alpha31-navigation"
for attempt in $(seq 1 200); do
  grep -q '^ALPHA31_FORMAT_READY' "$log" && break
  kill -0 "$pid" || { cat "$log"; exit 1; }
  sleep 0.05
done
sleep 0.2
scrot "${log%.log}-format.png"
wait "$pid" || { cat "$log"; exit 1; }
grep -q ALPHA31_UI_OK "$log"
trap - EXIT
