#!/usr/bin/env bash
set -euo pipefail
leaf_runner=${1:-./target/release/wordcraft-gnome}
leaf_cmd=("$leaf_runner")
if [[ "$leaf_runner" = *.sh ]]; then leaf_cmd=(bash "$leaf_runner"); fi
"${leaf_cmd[@]}" --interaction-pointer-test 2>interactions-pointer.log &
leaf_pid=$!
trap 'kill "$leaf_pid" 2>/dev/null || true' EXIT
for marker in SHIFT_POINTER_READY WORD_POINTER_READY WORD_DRAG_READY LINK_POINTER_READY FONT_INPUT_READY_4 FONT_INPUT_READY_5 FONT_INPUT_READY_6; do
  for attempt in $(seq 1 300); do
    grep -q "$marker" interactions-pointer.log && break
    kill -0 "$leaf_pid" || { cat interactions-pointer.log; exit 1; }
    sleep 0.1
  done
  grep -q "$marker" interactions-pointer.log || { cat interactions-pointer.log; exit 1; }
  for attempt in $(seq 1 20); do
    leaf_window=$(xdotool search --onlyvisible --name '^Leaf Interaction Regression$' | head -n1 || true)
    test -n "$leaf_window" && break
    sleep 0.05
  done
  test -n "$leaf_window"
  xdotool windowfocus --sync "$leaf_window"
  read -r _ ax ay bx by < <(grep "$marker" interactions-pointer.log | tail -n1)
  xdotool mousemove --window "$leaf_window" "$ax" "$ay"
  case "$marker" in
    SHIFT_POINTER_READY)
      xdotool click 1 keydown Shift mousemove --window "$leaf_window" "$bx" "$by" click 1 keyup Shift ;;
    WORD_POINTER_READY) xdotool click --repeat 2 --delay 80 1 ;;
    WORD_DRAG_READY)
      xdotool click 1
      sleep 0.08
      xdotool mousedown 1 mousemove --sync --window "$leaf_window" "$bx" "$by"
      sleep 0.1
      xdotool mouseup 1 ;;
    LINK_POINTER_READY) xdotool keydown Control click 1 keyup Control ;;
    FONT_INPUT_READY_4|FONT_INPUT_READY_5|FONT_INPUT_READY_6)
      scrot vista-texto.png
      xdotool getmouselocation
      xdotool click 1 key ctrl+a
      if test "$marker" = FONT_INPUT_READY_5; then
        xdotool key ctrl+v
        sleep 0.25
        xdotool key BackSpace
      else
        if test "$marker" = FONT_INPUT_READY_4; then family='serif '; else family=17.5; fi
        xdotool type --clearmodifiers --delay 20 "$family"
      fi ;;
  esac
done
wait "$leaf_pid" || { cat interactions-pointer.log; exit 1; }
grep -q INTERACTION_POINTER_OK interactions-pointer.log
grep -q 'LINK_ACTIVATED https://example.org/leaf-test' interactions-pointer.log
