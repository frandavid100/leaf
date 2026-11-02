#!/usr/bin/env bash
# Focus the intended toplevel before sending real input to its native popup.
set -euo pipefail
GSK_RENDERER=cairo ./target/release/wordcraft-gnome --test-document --split-states-demo 2>split-states.log &
split_pid=$!
trap 'kill "$split_pid" 2>/dev/null || true' EXIT
for attempt in $(seq 1 300); do
  grep -q SPLIT_POINTER_READY split-states.log && break
  kill -0 "$split_pid" || { cat split-states.log; exit 1; }
  sleep 0.1
done
grep -q SPLIT_POINTER_READY split-states.log
read -r _ bx by ax ay < <(grep SPLIT_POINTER_READY split-states.log | tail -n1)
split_window=$(xdotool search --onlyvisible --name '^Leaf Split Button Regression$' | head -n1)
xdotool windowfocus --sync "$split_window"
xdotool getwindowgeometry --shell "$split_window" >>split-states.log
# This isolated Xvfb session has no window manager: the toplevel starts at 0,0.
# Popup positions are already in that root coordinate space. Do not translate
# them a second time through an arbitrary X window returned by a title search.
xdotool mousemove --sync 600 150
sleep 0.4
scrot vista-dividido-reposo.png
xdotool mousemove --sync "$bx" "$by"
sleep 0.4
scrot vista-dividido-cuerpo.png
xdotool mousemove --sync "$ax" "$ay"
xdotool getmouselocation --shell >>split-states.log
sleep 0.4
scrot --pointer vista-dividido-flecha.png
xdotool click 1
for attempt in $(seq 1 50); do
  grep -q SPLIT_SUBPAGE_OK split-states.log && break
  kill -0 "$split_pid" || { cat split-states.log; exit 1; }
  sleep 0.1
done
if ! grep -q SPLIT_SUBPAGE_OK split-states.log; then
  scrot vista-dividido-fallo.png
  cat split-states.log
  exit 1
fi
scrot vista-dividido-subpagina.png
