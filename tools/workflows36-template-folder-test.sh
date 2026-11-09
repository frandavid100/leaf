#!/usr/bin/env bash
# Real native custom chooser, read-only sandbox XDG mount and process restart.
set -euo pipefail
runner=${1:?Missing installed runner}
root=$(cd "$(dirname "$0")/.." && pwd)
command=("$runner"); [[ "$runner" != *.sh ]] || command=(bash "$runner")
folder=$(mktemp -d /tmp/leaf36-template-host.XXXXXX)
xdg_folder= backup= had_dirs=false
cleanup() {
 kill "${pid:-}" 2>/dev/null || true
 if [[ -n "$backup" ]]; then
  if [[ "$had_dirs" == true ]]; then cp "$backup" "$HOME/.config/user-dirs.dirs"; else rm -f "$HOME/.config/user-dirs.dirs"; fi
  rm -f "$backup"
 fi
 rm -rf "$folder"
 [[ -z "$xdg_folder" ]] || rm -rf "$xdg_folder"
}
trap cleanup EXIT
LEAF36_TEMPLATE_SOURCE="$folder" "${LEAF35_NATIVE_SOURCE:-$root/target/release/wordcraft-gnome}" --workflows36-template-create
before=$(sha256sum "$folder/Personal.docx" | cut -d' ' -f1)
if [[ -n ${LEAF35_APP_ID:-} ]]; then
 flatpak run --command=sh "$LEAF35_APP_ID" -c 'test ! -r "$1/Personal.docx"' sh "$folder"
 # Use the production xdg-templates:ro permission with an actual localized
 # host XDG assignment. Keep the isolated application profile in agreement.
 test "${CI:-}" == true
 xdg_folder=$(mktemp -d "$HOME/Plantillas personales.XXXXXX")
 LEAF36_TEMPLATE_SOURCE="$xdg_folder" "${LEAF35_NATIVE_SOURCE:-$root/target/release/wordcraft-gnome}" --workflows36-template-create
 xdg_before=$(sha256sum "$xdg_folder/Personal.docx" | cut -d' ' -f1)
 mkdir -p "$HOME/.config" "$LEAF35_PROFILE/config"
 backup=$(mktemp)
 if [[ -f "$HOME/.config/user-dirs.dirs" ]]; then had_dirs=true;cp "$HOME/.config/user-dirs.dirs" "$backup";fi
 python3 - "$HOME/.config/user-dirs.dirs" "$xdg_folder" <<'PYDIRS'
from pathlib import Path
import sys
p=Path(sys.argv[1])
lines=p.read_text().splitlines() if p.exists() else []
p.write_text('\n'.join(l for l in lines if not l.lstrip().startswith('XDG_TEMPLATES_DIR='))+'\nXDG_TEMPLATES_DIR="'+sys.argv[2]+'"\n')
PYDIRS
 cp "$HOME/.config/user-dirs.dirs" "$LEAF35_PROFILE/config/user-dirs.dirs"
fi
log=workflows36-template-folder-${LANGUAGE:-en}.log
"${command[@]}" --workflows36-template-folder-test > "$log" 2>&1 &
pid=$!
if [[ -z ${LEAF35_APP_ID:-} ]]; then
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
fi
for _ in $(seq 1 1000); do
 grep -q WORKFLOWS36_TEMPLATE_FOLDER_OK "$log" && break
 kill -0 "$pid" || { cat "$log";exit 1; }
 sleep .05
done
grep -q WORKFLOWS36_TEMPLATE_FOLDER_OK "$log" || { cat "$log";exit 1; }
wait "$pid" || { cat "$log";exit 1; }
test "$before" = "$(sha256sum "$folder/Personal.docx" | cut -d' ' -f1)"
test "$(find "$folder" -type f | wc -l)" = 1
if [[ -n "$xdg_folder" ]]; then
 test "$xdg_before" = "$(sha256sum "$xdg_folder/Personal.docx" | cut -d' ' -f1)"
 test "$(find "$xdg_folder" -type f | wc -l)" = 1
 bash "$root/tools/workflows40-folder-test.sh" "$runner" "$xdg_folder"
fi
cat "$log"
