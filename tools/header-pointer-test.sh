#!/usr/bin/env bash
# Run under xvfb-run: click a repeated header on page two, then click blank canvas to return to the body.
set -euo pipefail
GSK_RENDERER=cairo ./target/release/wordcraft-gnome --headers-pointer-test 2>headers-pointer.log &
leaf_pid=$!
trap 'kill "$leaf_pid" 2>/dev/null || true' EXIT
for marker in HEADER_POINTER_READY HEADER_RETURN_READY; do
  for attempt in $(seq 1 300); do
    if grep -q "$marker" headers-pointer.log; then break; fi
    kill -0 "$leaf_pid" || { cat headers-pointer.log; exit 1; }
    sleep 0.1
  done
  grep -q "$marker" headers-pointer.log || { cat headers-pointer.log; exit 1; }
  leaf_window=$(xdotool search --onlyvisible --pid "$leaf_pid" | head -n1)
  xdotool windowfocus --sync "$leaf_window"
  if test "$marker" = HEADER_POINTER_READY; then
    read -r _ leaf_x leaf_y < <(grep HEADER_POINTER_READY headers-pointer.log | tail -n1)
    xdotool mousemove --window "$leaf_window" "$leaf_x" "$leaf_y" click 1
  else
    read -r _ leaf_x leaf_y < <(grep HEADER_RETURN_READY headers-pointer.log | tail -n1)
    xdotool mousemove --window "$leaf_window" "$leaf_x" "$leaf_y" click 1
  fi
done
wait "$leaf_pid" || { cat headers-pointer.log; exit 1; }
grep -q 'HEADER_FOOTER_UI_OK pointer=true' headers-pointer.log
