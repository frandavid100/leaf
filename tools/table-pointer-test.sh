#!/usr/bin/env bash
set -euo pipefail
leaf_runner=${1:-./target/release/wordcraft-gnome}
leaf_cmd=("$leaf_runner")
if [[ "$leaf_runner" = *.sh ]]; then leaf_cmd=(bash "$leaf_runner"); fi
for leaf_case in column row corner cancel select insert move merged later; do
  leaf_log="table18-$leaf_case.log"
  "${leaf_cmd[@]}" "--table18-$leaf_case" 2>"$leaf_log" &
  leaf_pid=$!
  trap 'kill "$leaf_pid" 2>/dev/null || true' EXIT
  for attempt in $(seq 1 200); do
    grep -q TABLE18_READY "$leaf_log" && break
    kill -0 "$leaf_pid" || { cat "$leaf_log"; exit 1; }
    sleep 0.1
  done
  read -r _ sx sy ex ey < <(grep TABLE18_READY "$leaf_log" | tail -n1)
  leaf_window=$(xdotool search --onlyvisible --name '^Leaf Table Pointer Regression$' | head -n1)
  xdotool windowfocus --sync "$leaf_window"
  xdotool mousemove --sync --window "$leaf_window" "$sx" "$sy"
  sleep 0.1
  if test "$leaf_case" = insert; then
    xdotool click 1
  else
    xdotool mousedown 1
    sleep 0.1
    xdotool mousemove --sync --window "$leaf_window" "$ex" "$ey"
    sleep 0.2
    if test "$leaf_case" = cancel; then xdotool key Escape; sleep 0.1; fi
    xdotool mouseup 1
  fi
  wait "$leaf_pid" || { cat "$leaf_log"; exit 1; }
  grep -q "TABLE18_OK case=$leaf_case" "$leaf_log"
done
trap - EXIT
