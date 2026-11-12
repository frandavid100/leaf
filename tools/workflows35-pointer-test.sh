#!/usr/bin/env bash
# Real GTK drag transfer, external X11 clipboard and native keyboard dispatch.
set -euo pipefail
runner=${1:-./target/release/wordcraft-gnome}
source_runner=${LEAF35_NATIVE_SOURCE:-./target/release/wordcraft-gnome}
cmd=("$runner"); [[ "$runner" != *.sh ]] || cmd=(bash "$runner")
root=$(cd "$(dirname "$0")/.." && pwd)
folder=$(mktemp -d)
python - "$folder/image.png" "$root/tests/fixtures/picture.b64" <<'PY'
import base64,pathlib,sys
pathlib.Path(sys.argv[1]).write_bytes(base64.b64decode(pathlib.Path(sys.argv[2]).read_text()))
PY
if [[ -n ${LEAF35_APP_ID:-} ]]; then
  flatpak run --command=sh "$LEAF35_APP_ID" -c 'test ! -r "$1"' sh "$folder/image.png"
fi
log=workflows35-pointer-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
source_log=workflows35-source-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
LEAF35_DROP_FILE="$folder/image.png" "$source_runner" --workflows35-drag-source > "$source_log" 2>&1 &
source_pid=$!
"${cmd[@]}" --workflows35-pointer-test > "$log" 2>&1 &
pid=$!
trap 'scrot "${log%.log}.png" 2>/dev/null || true; kill "$pid" "$source_pid" 2>/dev/null || true; rm -rf "$folder"' EXIT
for attempt in $(seq 1 300); do
  grep -q '^WORKFLOWS35_SOURCE ' "$source_log" && break
  kill -0 "$source_pid" || { cat "$source_log"; exit 1; }
  sleep .1
done
read -r _ sx sy < <(grep '^WORKFLOWS35_SOURCE ' "$source_log" | tail -n1)
source_window=$(xdotool search --onlyvisible --name '^Leaf Workflows35 Source$' | head -n1)
xdotool windowmove "$source_window" 2400 10
for stage in 0 1 2 3; do
  for attempt in $(seq 1 500); do
    grep -q "^WORKFLOWS35_POINTER $stage " "$log" && break
    kill -0 "$pid" || { cat "$log"; exit 1; }
    sleep .1
  done
  read -r _ _ role ax ay bx by < <(grep "^WORKFLOWS35_POINTER $stage " "$log" | tail -n1)
  window=$(xdotool search --onlyvisible --name '^Leaf Workflows35 Pointer$' | head -n1)
  xdotool windowmove "$window" 0 100
  case "$role" in
    file)
      xdotool windowfocus --sync "$source_window" mousemove --window "$source_window" "$sx" "$sy" mousedown 1
      sleep .2
      xdotool mousemove_relative --sync 20 0
      sleep .2
      xdotool mousemove --sync --window "$window" "$ax" "$ay"
      sleep .4
      xdotool mousemove_relative --sync 2 0
      sleep .3
      xdotool getmouselocation
      xdotool mouseup 1
      sleep .5
      scrot "${log%.log}-stage-$stage.png" ;;
    image)
      xclip -selection clipboard -t image/png -i "$folder/image.png"
      for attempt in $(seq 1 300); do
        grep -q '^WORKFLOWS35_IMAGE_CLIPBOARD_READY$' "$log" && break
        kill -0 "$pid" || { cat "$log"; exit 1; }
        sleep .05
      done
      grep -q '^WORKFLOWS35_IMAGE_CLIPBOARD_READY$' "$log"
      xdotool windowfocus --sync "$window" key ctrl+v ;;
    plain)
      printf Plain | xclip -selection clipboard
      xdotool windowfocus --sync "$window" key ctrl+shift+v ;;
    heading)
      xdotool windowfocus --sync "$window" mousemove --window "$window" "$ax" "$ay" mousedown 1
      sleep .2
      xdotool mousemove_relative --sync 20 0
      sleep .2
      xdotool mousemove --sync --window "$window" "$bx" "$by"
      sleep .4
      xdotool mousemove_relative --sync 2 0
      sleep .3
      xdotool getmouselocation
      xdotool mouseup 1
      sleep .5
      scrot "${log%.log}-stage-$stage.png" ;;
    *) cat "$log"; exit 1 ;;
  esac
done
wait "$pid" || { cat "$log"; exit 1; }
grep -q WORKFLOWS35_POINTER_OK "$log"
cat "$log"
