#!/usr/bin/env bash
# Exercise FlatPark's extra-data installation and relocated GNOME runtime.
set -euo pipefail
archive=${1:?Usage: flatpark-smoke.sh RELEASE_ARCHIVE}
app_id=io.github.frandavid100.leaf.FlatparkTest
remote=leaf-flatpark-test
runtime_version=51
if flatpak --user info "$app_id" >/dev/null 2>&1 || flatpak --user remotes --columns=name | grep -Fxq "$remote"; then
  echo 'The FlatPark test application or remote already exists.' >&2
  exit 1
fi
temporary=$(mktemp -d)
server_pid=
trap 'flatpak kill "$app_id" >/dev/null 2>&1 || true; flatpak --user uninstall -y --noninteractive "$app_id" >/dev/null 2>&1 || true; flatpak --user remote-delete "$remote" >/dev/null 2>&1 || true; kill "$server_pid" >/dev/null 2>&1 || true; rm -rf "$temporary"' EXIT
flatpak --user install -y --noninteractive flathub "org.gnome.Platform//$runtime_version"
flatpak build-init "$temporary/build" "$app_id" org.gnome.Platform org.gnome.Platform "$runtime_version"
mkdir -p "$temporary/download" "$temporary/build/files/bin"
cp "$archive" "$temporary/download/leaf.tar.xz"
# Serve the exact archive to Flatpak's real extra-data downloader on this runner.
python3 - "$temporary" <<'PY' &
import functools
import http.server
import pathlib
import sys
root = pathlib.Path(sys.argv[1])
handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(root / 'download'))
with http.server.ThreadingHTTPServer(('127.0.0.1', 0), handler) as server:
    (root / 'port').write_text(str(server.server_port))
    server.serve_forever()
PY
server_pid=$!
for attempt in $(seq 1 100); do
  test ! -s "$temporary/port" || break
  kill -0 "$server_pid"
  sleep 0.1
done
port=$(cat "$temporary/port")
checksum=$(sha256sum "$archive" | cut -d ' ' -f 1)
size=$(stat -c %s "$archive")
cat > "$temporary/build/files/bin/apply_extra" <<'APPLY'
#!/bin/sh
set -eu
mkdir leaf
tar -xJf leaf.tar.xz -C leaf
rm leaf.tar.xz
APPLY
chmod +x "$temporary/build/files/bin/apply_extra"
flatpak build-finish --filesystem=xdg-templates:ro --command=/app/extra/leaf/bin/leaf \
  --extra-data="leaf.tar.xz:$checksum:$size:0:http://127.0.0.1:$port/leaf.tar.xz" \
  --metadata='Extra Data=NoRuntime=false' \
  --share=ipc --socket=wayland --socket=fallback-x11 --device=dri "$temporary/build"
flatpak build-export "$temporary/repo" "$temporary/build"
flatpak --user remote-add --no-gpg-verify "$remote" "file://$temporary/repo"
flatpak --user install -y --noninteractive "$remote" "$app_id"
flatpak run --command=sh "$app_id" -c '
  test -x /app/extra/leaf/bin/leaf
  test -s /app/extra/leaf/share/hunspell/es_ES.dic
  test -s /app/extra/leaf/share/locale/es/LC_MESSAGES/leaf.mo
'
flatpak run "$app_id" --version
for language in en es; do
  log="$temporary/localization-$language.log"
  timeout 90s flatpak run --socket=x11 --nosocket=wayland \
    --env=GSK_RENDERER=cairo --env=LC_ALL=en_US.UTF-8 --env="LANGUAGE=$language" \
    "$app_id" --localization-ui-test >"$log" 2>&1 || { cat "$log"; exit 1; }
  cat "$log"
  grep -q "LOCALIZATION_UI_OK language=$language" "$log"
done
bash tools/workflows35-installed-test.sh "$app_id" /app/extra/leaf/bin/leaf dist/workflows35-flatpark
log="$temporary/proofing.log"
timeout 90s flatpak run --socket=x11 --nosocket=wayland \
  --env=GSK_RENDERER=cairo --env=LC_ALL=en_US.UTF-8 --env=LANGUAGE=es \
  "$app_id" --proofing-ui-test >"$log" 2>&1 || { cat "$log"; exit 1; }
cat "$log"
grep -q PROOFING_UI_OK "$log"
log="$temporary/clipboard.log"
timeout 90s flatpak run --socket=x11 --nosocket=wayland \
  --env=GSK_RENDERER=cairo --env=LC_ALL=en_US.UTF-8 --env=LANGUAGE=es \
  "$app_id" --clipboard-ui-test >"$log" 2>&1 || { cat "$log"; exit 1; }
cat "$log"
grep -q CLIPBOARD_FORMATS_UI_OK "$log"
bash tools/pending-installed-test.sh "$app_id" /app/extra/leaf/bin/leaf dist/pending-flatpark
bash tools/dictionary84-installed-test.sh "$app_id" /app/extra/leaf/bin/leaf dist/dictionary84-flatpark
# Exercise the new candidate interactions in the relocated GNOME 51 build too.
leaf29_root=$PWD
leaf29_output=${2:-dist/alpha29-flatpark}
mkdir -p "$leaf29_output" "$HOME/.var/app/$app_id/cache"
leaf29_output=$(realpath "$leaf29_output")
# The combined pointer driver invokes this child relative to its working directory.
mkdir -p "$leaf29_output/tools"
cp "$leaf29_root/tools/alpha30-pointer-test.sh" "$leaf29_root/tools/alpha31-pointer-test.sh" "$leaf29_root/tools/format136-pointer-test.sh" "$leaf29_root/tools/caption-pointer-test.sh" "$leaf29_output/tools/"
cat > "$temporary/leaf29-flatpark.sh" <<'FLATPARK29'
#!/usr/bin/env bash
exec flatpak run --socket=x11 --nosocket=wayland --env=LC_ALL=en_US.UTF-8 --env=LANGUAGE="${LANGUAGE:-en}" --env=GDK_SCALE="${GDK_SCALE:-1}" --env=GSK_RENDERER=cairo --env=LEAF29_ACK_DIR="$LEAF29_ACK_DIR" --command=sh io.github.frandavid100.leaf.FlatparkTest -c 'export XDG_CONFIG_HOME=$(mktemp -d) XDG_STATE_HOME=$(mktemp -d); exec /app/extra/leaf/bin/leaf "$@"' leaf "$@"
FLATPARK29
(
  cd "$leaf29_output"
  export LEAF29_ACK_PARENT="$HOME/.var/app/$app_id/cache"
  LANGUAGE=en GDK_SCALE=1 xvfb-run -a -s '-screen 0 1400x1100x24' timeout 180s bash "$leaf29_root/tools/alpha29-pointer-test.sh" "$temporary/leaf29-flatpark.sh"
  LANGUAGE=es GDK_SCALE=2 xvfb-run -a -s '-screen 0 2800x2200x24' timeout 180s bash "$leaf29_root/tools/alpha29-pointer-test.sh" "$temporary/leaf29-flatpark.sh"
)
echo FLATPARK_ALPHA29_INTERACTIONS_OK
echo FLATPARK_GNOME51_RELOCATION_OK
