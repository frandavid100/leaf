#!/usr/bin/env bash
# Real external clipboard, cancelled/file drags and delayed native FIFO loading.
set -euo pipefail
runner=${1:-./target/release/wordcraft-gnome}
source_runner=${LEAF35_NATIVE_SOURCE:-./target/release/wordcraft-gnome}
command=("$runner");[[ "$runner" != *.sh ]] || command=(bash "$runner")
root=$(cd "$(dirname "$0")/.." && pwd)
folder=$(mktemp -d)
python - "$folder/original.png" "$root/tests/fixtures/picture.b64" <<'PY'
from pathlib import Path
import sys,base64
Path(sys.argv[1]).write_bytes(base64.b64decode(Path(sys.argv[2]).read_text()))
PY
cp "$folder/original.png" "$folder/image.png"
if [[ -n ${LEAF35_APP_ID:-} ]];then
  flatpak run --command=sh "$LEAF35_APP_ID" -c 'test ! -r "$1"' sh "$folder/image.png"
  stages=2
else
  export LEAF36_DELAYED_DROP=1
  stages=3
fi
log=workflows36-pointer-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
source_log=workflows36-source-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
LEAF35_DROP_FILE="$folder/image.png" "$source_runner" --workflows35-drag-source > "$source_log" 2>&1 &
source_pid=$!
"${command[@]}" --workflows36-pointer-test > "$log" 2>&1 &
pid=$!
writer=
trap 'scrot "${log%.log}.png" 2>/dev/null || true; kill "$pid" "$source_pid" ${writer:-} 2>/dev/null || true; rm -rf "$folder"' EXIT
wait_marker(){
  local pattern=$1 file=$2
  for _ in $(seq 1 1000);do
    if grep -q "$pattern" "$file";then return;fi
    kill -0 "$pid" 2>/dev/null || { cat "$log";exit 1; }
    sleep .02
  done
  cat "$log";exit 1
}
wait_marker '^WORKFLOWS35_SOURCE ' "$source_log"
read -r _ sx sy < <(grep '^WORKFLOWS35_SOURCE ' "$source_log" | tail -n1)
source_window=$(xdotool search --onlyvisible --name '^Leaf Workflows35 Source$' | head -n1)
xdotool windowmove "$source_window" 2400 10
for stage in $(seq 0 "$stages");do
  wait_marker "^WORKFLOWS36_POINT $stage " "$log"
  read -r _ _ role ax ay ex ey < <(grep "^WORKFLOWS36_POINT $stage " "$log" | tail -n1)
  window=$(xdotool search --onlyvisible --name '^Leaf Workflows36 Pointer$' | head -n1)
  xdotool windowmove "$window" 0 100
  if [[ $role == query ]];then
    printf '\tneedle\nnext' | xclip -selection clipboard
    xdotool windowfocus --sync "$window" key ctrl+v
    continue
  fi
  if [[ $role == delayed ]];then rm "$folder/image.png";mkfifo "$folder/image.png";fi
  xdotool windowfocus --sync "$source_window" mousemove --window "$source_window" "$sx" "$sy" mousedown 1
  sleep .2
  xdotool mousemove_relative --sync 20 0
  sleep .2
  if [[ $role == image ]];then
    xdotool mousemove --sync --window "$window" "$ex" "$ey"
    sleep .2
    xdotool mousemove_relative --sync 2 0
    wait_marker "^WORKFLOWS36_MARKER $stage" "$log"
    wait_marker "^WORKFLOWS36_GAP_RESUMED $stage" "$log"
  fi
  xdotool mousemove --sync --window "$window" "$ax" "$ay"
  sleep .3
  xdotool mousemove_relative --sync 2 0
  sleep .3
  wait_marker "^WORKFLOWS36_MARKER $stage" "$log"
  if [[ $role == image ]];then wait_marker "^WORKFLOWS36_TARGET_READY $stage" "$log";fi
  scrot "${log%.log}-marker-$stage.png"
  if [[ $role == cancel ]];then
    xdotool key --window "$source_window" Escape
    wait_marker '^WORKFLOWS35_SOURCE_CANCEL ' "$source_log"
  fi
  xdotool mouseup 1
  if [[ $role == delayed ]];then
    wait_marker "^WORKFLOWS36_READ_PENDING $stage" "$log"
    xdotool windowfocus --sync "$window" key X
    sleep .2
    cat "$folder/original.png" > "$folder/image.png" &
    writer=$!
  fi
  wait_marker "^WORKFLOWS36_STAGE_OK $stage" "$log"
done
wait "$pid" || { cat "$log";exit 1; }
grep -q WORKFLOWS36_POINTER_OK "$log"
cat "$log"
