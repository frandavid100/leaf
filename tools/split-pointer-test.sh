#!/usr/bin/env bash
# Focus the intended toplevel before sending real input to its native popup.
set -euo pipefail
split_runner=${1:-./target/release/wordcraft-gnome}
GSK_RENDERER=cairo "$split_runner" --test-document --split-states-demo 2>split-states.log &
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
xdotool getwindowgeometry --shell "$split_window" >split-pointer-window.log
# This isolated Xvfb session has no window manager: the toplevel starts at 0,0.
# Keep the intended window explicit and send fresh motion with the final click.
xdotool mousemove --sync --window "$split_window" 600 150
sleep 0.4
scrot vista-dividido-reposo.png
xdotool mousemove --sync --window "$split_window" "$bx" "$by"
sleep 0.4
scrot vista-dividido-cuerpo.png
xdotool mousemove --sync --window "$split_window" "$ax" "$ay"
xdotool getmouselocation --shell >>split-pointer-window.log
sleep 0.4
scrot --pointer vista-dividido-flecha.png
# Re-enter the native dropdown and click in one input-client invocation.
xdotool mousemove --window "$split_window" "$bx" "$by" mousemove --sync --window "$split_window" "$ax" "$ay" click 1
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
