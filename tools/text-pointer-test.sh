#!/usr/bin/env bash
set -euo pipefail
leaf_runner=${1:-./target/release/wordcraft-gnome}
leaf_cmd=("$leaf_runner")
if [[ "$leaf_runner" = *.sh ]]; then leaf_cmd=(bash "$leaf_runner"); fi
"${leaf_cmd[@]}" --text18-pointer-test 2>text18-pointer.log &
leaf_pid=$!
trap 'kill "$leaf_pid" 2>/dev/null || true' EXIT
for marker in TEXT18_INK_ARROW TEXT18_BLUE TEXT18_INK_BODY TEXT18_UNDERLINE_ARROW TEXT18_DOUBLE TEXT18_UNDERLINE_OFF TEXT18_UNDERLINE_ON; do
  for attempt in $(seq 1 250); do
    grep -q "$marker " text18-pointer.log && break
    kill -0 "$leaf_pid" || { cat text18-pointer.log; exit 1; }
    sleep 0.1
  done
  grep -q "$marker " text18-pointer.log || { tail -35 text18-pointer.log | cut -c1-240; exit 1; }
  read -r _ x y < <(grep "$marker " text18-pointer.log | tail -n1)
  leaf_window=$(xdotool search --onlyvisible --name '^Leaf Text Pointer Regression$' | head -n1)
  xdotool windowfocus --sync "$leaf_window"
  xdotool mousemove --sync --window "$leaf_window" "$x" "$y" click 1
  sleep 0.15
done
wait "$leaf_pid" || { cat text18-pointer.log; exit 1; }
grep -q TEXT18_POINTER_OK text18-pointer.log
trap - EXIT
