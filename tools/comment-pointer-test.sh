#!/usr/bin/env bash
# Run under Xvfb. Both actions go through the document's actual GTK gestures.
set -euo pipefail
leaf_runner=${1:-./target/release/wordcraft-gnome}
"$leaf_runner" --comment-pointer-test 2>comment-pointer.log &
leaf_pid=$!
trap 'kill "$leaf_pid" 2>/dev/null || true' EXIT
for marker in COMMENT_ANCHOR_READY COMMENT_DRAG_READY; do
  for attempt in $(seq 1 300); do
    grep -q "$marker" comment-pointer.log && break
    kill -0 "$leaf_pid" || { cat comment-pointer.log; exit 1; }
    sleep 0.1
  done
  grep -q "$marker" comment-pointer.log || { cat comment-pointer.log; exit 1; }
  leaf_window=$(xdotool search --onlyvisible --pid "$leaf_pid" | head -n1)
  xdotool windowfocus --sync "$leaf_window"
  read -r _ leaf_x leaf_y leaf_end < <(grep "$marker" comment-pointer.log | tail -n1)
  xdotool mousemove --window "$leaf_window" "$leaf_x" "$leaf_y"
  if test "$marker" = COMMENT_ANCHOR_READY; then
    xdotool click 1
  else
    xdotool mousedown 1
    xdotool mousemove --sync --window "$leaf_window" "$leaf_end" "$leaf_y"
    sleep 0.15
    xdotool mouseup 1
  fi
done
wait "$leaf_pid" || { cat comment-pointer.log; exit 1; }
grep -q COMMENT_POINTER_UI_OK comment-pointer.log
