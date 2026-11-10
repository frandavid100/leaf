#!/usr/bin/env bash
set -euo pipefail
leaf_runner=${1:-./target/release/wordcraft-gnome}
mode=${2:-font}
leaf_cmd=("$leaf_runner")
if [[ "$leaf_runner" = *.sh ]]; then leaf_cmd=(bash "$leaf_runner"); fi
export LEAF19_ACK_DIR
LEAF19_ACK_DIR=$(mktemp -d "${LEAF19_ACK_PARENT:-/tmp}/leaf19-ack.XXXXXX")
log="followup19-${mode}-${LEAF19_RANGE:-range}-${LEAF19_NARROW:-wide}-${GDK_SCALE:-1}-${LEAF19_REFLOW:-normal}.log"
"${leaf_cmd[@]}" "--followup19-$mode" >"${log%.log}-out.log" 2>"$log" &
pid=$!
trap 'kill "$pid" 2>/dev/null || true; rm -rf "$LEAF19_ACK_DIR"' EXIT
last=0
while kill -0 "$pid" 2>/dev/null; do
  count=$(grep -c '^ISSUE19_READY ' "$log" || true)
  if (( count <= last )); then sleep 0.05; continue; fi
  line=$(grep '^ISSUE19_READY ' "$log" | sed -n "$((last+1))p")
  read -r _ stage role ax ay bx by cx cy <<< "$line"
  window=$(xdotool search --onlyvisible --name '^Leaf Followup Regression$' | head -n1)
  # Link is now an integrated dialog. The diagnostic emits coordinates in
  # this same parent surface for both editor and dialog controls.
  if [[ "$mode" != font ]] || (( last == 0 )); then
    xdotool windowfocus --sync "$window"
  fi
  case "$role" in
    open)
      for attempt in $(seq 1 50); do
        xdotool mousemove --window "$window" "$((ax + attempt % 2))" "$((ay + attempt % 2))"
        sleep 0.1
        grep -Fq 'POINTER_TARGET_ENTERED FOLLOWUP19_OPEN' "$log" && break
      done
      grep -Fq 'POINTER_TARGET_ENTERED FOLLOWUP19_OPEN' "$log" || { cat "$log"; exit 1; }
      xdotool mousemove --window "$window" "$ax" "$ay" mousedown 1
      sleep 0.12
      xdotool mouseup 1 ;;
    choices|pin) xdotool mousemove --window "$window" "$ax" "$ay" click 1 ;;
    escape) xdotool key Escape ;;
    font)
      family=$(sed -n 's/^ISSUE19_FAMILY //p' "$log" | head -n1)
      xdotool mousemove --window "$window" "$ax" "$ay" click 1 key ctrl+a
      xdotool key Delete
      xdotool type --clearmodifiers --delay 80 "$family"
      # Filtering is non-mutating; Enter explicitly chooses the typed family.
      xdotool key Return ctrl+k ;;
    paste) xdotool mousemove --window "$window" "$ax" "$ay" click 1 key ctrl+a ctrl+v ;;
    backspace) xdotool key BackSpace ;;
    size|comma)
      xdotool mousemove --window "$window" "$ax" "$ay" click 1 key ctrl+a
      if [[ "$role" = size ]]; then value=17.5; elif [[ -n "${LEAF19_LARGE:-}" ]]; then value=64,5; else value=18,75; fi
      xdotool type --clearmodifiers --delay 450 "$value" ;;
    return)
      # Dismiss the font list, then toggle its Text launcher explicitly.
      # In narrow windows the floating panel covers this fixture's caret;
      # Escape routing for nonmodal popovers differs between GTK versions.
      xdotool key Escape
      sleep 0.25
      xdotool mousemove --window "$window" "$bx" "$by" click 1
      sleep 0.25
      xdotool mousemove --sync --window "$window" "$ax" "$ay" click 1
      sleep 0.15
      xdotool type --clearmodifiers Q ;;
    triple|autoscroll)
      xdotool mousemove --window "$window" "$ax" "$ay" click --repeat 2 --delay 80 1
      sleep 0.08
      xdotool mousedown 1 mousemove --sync --window "$window" "$bx" "$by"
      if [[ "$role" = autoscroll ]]; then sleep 4; else sleep 0.2; fi
      xdotool mouseup 1 ;;
    shortcut) xdotool key ctrl+k ;;
    edit)
      xdotool mousemove --window "$window" "$ax" "$ay" click 1 key ctrl+a
      xdotool type --clearmodifiers 'https://example.org/edited?a=1&b=<value>'
      xdotool mousemove --window "$window" "$bx" "$by" click 1 ;;
    context)
      xdotool mousemove --window "$window" "$ax" "$ay" click 3
      sleep 0.25
      xdotool key Home Return ;;
    cancel) xdotool mousemove --window "$window" "$ax" "$ay" click 1 ;;
    insert)
      xdotool mousemove --window "$window" "$ax" "$ay" click 1
      xdotool type --clearmodifiers 'New link'
      xdotool mousemove --window "$window" "$bx" "$by" click 1
      xdotool type --clearmodifiers 'https://example.org/new'
      xdotool mousemove --window "$window" "$cx" "$cy" click 1 ;;
    hover)
      xdotool mousemove --window "$window" "$ax" "$ay"
      sleep 2 ;;
    *) echo "Unknown role $role" >&2; exit 1 ;;
  esac
  sleep 0.7
  scrot "${log%.log}-${stage}.png"
  touch "$LEAF19_ACK_DIR/$stage"
  last=$((last+1))
done
wait "$pid" || { cat "$log"; exit 1; }
grep -q "ISSUE19_${mode^^}_OK" "$log"
if [[ "$mode" = link ]]; then grep -Fq 'ISSUE19_TOOLTIP https://example.org/edited?a=1&b=<value>' "$log"; fi
