#!/usr/bin/env bash
set -euo pipefail
runner=${1:-./target/release/wordcraft-gnome}
cmd=("$runner"); [[ "$runner" != *.sh ]] || cmd=(bash "$runner")
log=alpha30-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
"${cmd[@]}" --alpha30-ui-test > "$log" 2>&1 &
pid=$!
trap 'scrot "${log%.log}-failure.png" 2>/dev/null || true; kill "$pid" 2>/dev/null || true' EXIT
for attempt in $(seq 1 600); do
  grep -q '^ALPHA30_TOC_LINK ' "$log" && break
  kill -0 "$pid" || { cat "$log"; exit 1; }
  sleep 0.1
done
read -r _ x y < <(grep '^ALPHA30_TOC_LINK ' "$log")
window=$(xdotool search --onlyvisible --name '^Leaf Alpha30 Regression$' | head -n1)
xdotool windowfocus --sync "$window"
xdotool mousemove --window "$window" "$x" "$y"
sleep 0.2
xdotool mousemove --window "$window" "$((x+1))" "$((y+1))" click 1
wait "$pid" || { cat "$log"; exit 1; }
grep -q ALPHA30_UI_OK "$log"
trap - EXIT
