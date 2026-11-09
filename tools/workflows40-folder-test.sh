#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
runner=${1:-$root/target/release/wordcraft-gnome}
if [[ ${LEAF40_PRIVATE_BUS:-} != 1 ]]; then
  exec dbus-run-session -- env LEAF40_PRIVATE_BUS=1 bash "$0" "$runner"
fi
command=("$runner"); [[ "$runner" != *.sh ]] || command=(bash "$runner")
export LEAF40_FOLDER LEAF29_ACK_DIR GTK_USE_PORTAL=1
parent=${LEAF29_ACK_PARENT:-/tmp}
LEAF40_FOLDER=$(mktemp -d "$parent/Plantillas personales.XXXXXX")
LEAF29_ACK_DIR=$(mktemp -d "$parent/leaf40-folder-ack.XXXXXX")
cleanup() { kill "${portal_pid:-}" 2>/dev/null || true; rm -rf "$LEAF40_FOLDER" "$LEAF29_ACK_DIR"; }
trap cleanup EXIT
for mode in success error; do
  rm -f "$LEAF29_ACK_DIR/folder40-response"
  /usr/bin/python3 "$root/tools/folder_portal_test.py" "$LEAF40_FOLDER" "$LEAF29_ACK_DIR/folder40-response" "$mode" > "folder40-portal-$mode.log" 2>&1 &
  portal_pid=$!
  for _ in $(seq 100); do grep -q FOLDER_PORTAL_READY "folder40-portal-$mode.log" && break; sleep .03; done
  grep -q FOLDER_PORTAL_READY "folder40-portal-$mode.log" || { cat "folder40-portal-$mode.log"; exit 1; }
  "${command[@]}" --workflows40-folder-test > "folder40-ui-$mode.log" 2>&1 || { cat "folder40-ui-$mode.log" "folder40-portal-$mode.log"; exit 1; }
  grep -q 'WORKFLOWS40_FOLDER_UI_OK' "folder40-ui-$mode.log"
  result=true; [[ "$mode" != error ]] || result=false
  grep -q "WORKFLOWS40_FOLDER_RESULT success=$result" "folder40-ui-$mode.log"
  grep -q "FOLDER_PORTAL_OK $mode" "folder40-portal-$mode.log"
  cat "folder40-portal-$mode.log" "folder40-ui-$mode.log"
  kill "$portal_pid"; wait "$portal_pid" || true
done
echo WORKFLOWS40_FOLDER_OK
