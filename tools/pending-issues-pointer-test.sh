#!/usr/bin/env bash
set -euo pipefail
leaf_runner=${1:-./target/release/wordcraft-gnome}
leaf_cmd=("$leaf_runner")
if [[ "$leaf_runner" = *.sh ]]; then leaf_cmd=(bash "$leaf_runner"); fi
export LEAF_PENDING_ACK
LEAF_PENDING_ACK=$(mktemp -d "${LEAF_PENDING_ACK_PARENT:-/tmp}/leaf-pending.XXXXXX")
"${leaf_cmd[@]}" --pending-end-ui-test >pending-end.log 2>&1 &
leaf_pid=$!
trap 'kill "$leaf_pid" 2>/dev/null || true; rm -rf "$LEAF_PENDING_ACK"' EXIT
last=0
while kill -0 "$leaf_pid" 2>/dev/null; do
  count=$(grep -c '^PENDING_KEY ' pending-end.log || true)
  if ((count<=last)); then sleep 0.05; continue; fi
  read -r _ stage key ax ay < <(grep '^PENDING_KEY ' pending-end.log | sed -n "$((last+1))p")
  window=$(xdotool search --onlyvisible --name '^Leaf Pending Issues$' | head -n1)
  xdotool windowfocus --sync "$window"
  if ((ax>=0)); then xdotool mousemove --window "$window" "$ax" "$ay" click 1; sleep 0.1; fi
  xdotool key --clearmodifiers "$key"
  if ((stage==0)); then scrot pending-end-first-key.png; fi
  touch "$LEAF_PENDING_ACK/$stage"
  last=$((last+1))
done
wait "$leaf_pid" || { cat pending-end.log; exit 1; }
cat pending-end.log
grep -q PENDING_END_UI_OK pending-end.log
