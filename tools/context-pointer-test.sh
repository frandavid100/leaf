#!/usr/bin/env bash
set -euo pipefail

GSK_RENDERER=cairo ./target/release/wordcraft-gnome --context-ui-test 2> context-menu.log &
context_pid=$!
for attempt in $(seq 1 300); do
  if grep -q CONTEXT_POINTER_READY context-menu.log; then break; fi
  kill -0 "$context_pid"
  sleep 0.1
done
grep -q CONTEXT_POINTER_READY context-menu.log
read marker cx cy < <(grep CONTEXT_POINTER_READY context-menu.log | tail -n1)
context_window=$(xdotool search --onlyvisible --pid "$context_pid" | head -n1)
xdotool windowfocus --sync "$context_window"
xdotool mousemove --window "$context_window" "$cx" "$cy" click 3
sleep 0.3
scrot vista-contexto-texto.png
for attempt in $(seq 1 100); do
  if grep -q CONTEXT_ACTION_READY context-menu.log; then break; fi
  kill -0 "$context_pid"
  sleep 0.1
done
grep -q CONTEXT_ACTION_READY context-menu.log
read marker ax ay < <(grep CONTEXT_ACTION_READY context-menu.log | tail -n1)
xdotool mousemove --window "$context_window" "$ax" "$ay" click 1
wait "$context_pid" || { cat context-menu.log; exit 1; }
grep -q CONTEXT_UI_OK context-menu.log
