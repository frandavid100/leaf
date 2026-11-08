#!/usr/bin/env bash
# Actual native chooser/portal grant, read-only source and process restart.
set -euo pipefail
runner=${1:?Missing installed runner}
root=$(cd "$(dirname "$0")/.." && pwd)
command=("$runner"); [[ "$runner" != *.sh ]] || command=(bash "$runner")
folder=$(mktemp -d /tmp/leaf36-template-host.XXXXXX)
trap 'kill "${pid:-}" 2>/dev/null || true; rm -rf "$folder"' EXIT
LEAF36_TEMPLATE_SOURCE="$folder" "${LEAF35_NATIVE_SOURCE:-$root/target/release/wordcraft-gnome}" --workflows36-template-create
before=$(sha256sum "$folder/Personal.docx" | cut -d' ' -f1)
if [[ -n ${LEAF35_APP_ID:-} ]]; then
 flatpak run --command=sh "$LEAF35_APP_ID" -c 'test ! -r "$1/Personal.docx"' sh "$folder"
fi
log=workflows36-template-folder-${LANGUAGE:-en}.log
"${command[@]}" --workflows36-template-folder-test > "$log" 2>&1 &
pid=$!
for _ in $(seq 1 600); do
 grep -q WORKFLOWS36_TEMPLATE_CHOOSER_READY "$log" && break
 kill -0 "$pid" || { cat "$log";exit 1; }
 sleep .05
done
for _ in $(seq 1 300); do
 dialog=$(xdotool search --onlyvisible --name '^(Choose Templates Folder|Elegir carpeta de plantillas)$' | head -n1 || true)
 [[ -z "$dialog" ]] || break
 sleep .05
done
test -n "${dialog:-}"
xdotool windowfocus --sync "$dialog" key --clearmodifiers alt+Home
sleep .5
xdotool key --clearmodifiers ctrl+l
sleep .3
xdotool key --clearmodifiers ctrl+a
printf '%s' "$folder" | xclip -selection clipboard
xdotool key --clearmodifiers ctrl+v
sleep .5
xdotool key --clearmodifiers alt+u
for _ in $(seq 1 1000); do
 grep -q WORKFLOWS36_TEMPLATE_FOLDER_OK "$log" && break
 kill -0 "$pid" || { cat "$log";exit 1; }
 sleep .05
done
grep -q WORKFLOWS36_TEMPLATE_FOLDER_OK "$log" || { cat "$log";exit 1; }
wait "$pid" || { cat "$log";exit 1; }
test "$before" = "$(sha256sum "$folder/Personal.docx" | cut -d' ' -f1)"
test "$(find "$folder" -type f | wc -l)" = 1
cat "$log"
