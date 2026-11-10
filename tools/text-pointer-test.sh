#!/usr/bin/env bash
set -euo pipefail
log=${LEAF_TEXT_POINTER_LOG:-text18-pointer.log}
leaf_runner=${1:-./target/release/wordcraft-gnome}
leaf_cmd=("$leaf_runner")
if [[ "$leaf_runner" = *.sh ]]; then leaf_cmd=(bash "$leaf_runner"); fi
"${leaf_cmd[@]}" --text18-pointer-test 2>"$log" &
leaf_pid=$!
trap 'scrot "${log%.log}-failure.png" 2>/dev/null || true; kill "$leaf_pid" 2>/dev/null || true' EXIT
for marker in TEXT18_INK_ARROW TEXT18_BLUE TEXT18_INK_BACK TEXT18_INK_REOPEN TEXT18_INK_BODY TEXT18_INK_AGAIN TEXT18_UNDERLINE_ARROW TEXT18_DOUBLE TEXT18_UNDERLINE_OFF TEXT18_UNDERLINE_ON; do
  for attempt in $(seq 1 350); do
    grep -q "^$marker " "$log" && break
    kill -0 "$leaf_pid" || { cat "$log"; exit 1; }
    if [[ "$attempt" = 150 ]]; then scrot "${log%.log}-$marker-stalled.png" 2>/dev/null || true; fi
    sleep 0.1
  done
  grep -q "^$marker " "$log" || { tail -35 "$log" | cut -c1-240; exit 1; }
  read -r _ x y < <(grep "^$marker " "$log" | tail -n1)
  leaf_window=$(xdotool search --onlyvisible --name '^Leaf Text Pointer Regression$' | head -n1)
  xdotool windowfocus --sync "$leaf_window"
  # Wait for GTK to observe genuine motion on this popup's target before
  # sending the single click; X11 pointer warps can overtake surface entry.
  for attempt in $(seq 1 50); do
    xdotool mousemove --window "$leaf_window" "$((x + attempt % 2))" "$((y + attempt % 2))"
    sleep 0.1
    grep -Fq "POINTER_TARGET_ENTERED $marker" "$log" && break
    kill -0 "$leaf_pid" || { cat "$log"; exit 1; }
  done
  grep -Fq "POINTER_TARGET_ENTERED $marker" "$log" || { cat "$log"; exit 1; }
  { echo "$marker before window=$leaf_window"; xdotool getwindowgeometry --shell "$leaf_window"; xdotool getmouselocation; } >> "${log%.log}-driver.log"
  xdotool mousedown 1
  sleep 0.12
  xdotool mouseup 1
  { echo "$marker after"; xdotool getmouselocation; } >> "${log%.log}-driver.log"
  sleep 0.15
done
wait "$leaf_pid" || { cat "$log"; exit 1; }
grep -q TEXT18_POINTER_OK "$log"
trap - EXIT
