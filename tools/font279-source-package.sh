#!/usr/bin/env bash
# Package this run's validated executable with unchanged alpha45 resources.
set -euo pipefail
runtime=${1:?49 or 51}
binary=$(realpath "${2:?validated optimized executable}")
[[ "$runtime" == 49 || "$runtime" == 51 ]]
app_id=io.github.frandavid100.leaf
work="$RUNNER_TEMP/font279-build-$runtime"
archive="$RUNNER_TEMP/Leaf-0.1.0-alpha.45-flatpark-x86_64.tar.xz"
gh release download v0.1.0-alpha.45 --repo frandavid100/leaf --pattern 'Leaf-0.1.0-alpha.45-flatpark-x86_64.tar.xz' --dir "$RUNNER_TEMP" --clobber
echo "138cdb67333384a086cd56190d5fe2f146c2202708fa6ac75751c8db31199203  $archive" | sha256sum -c -
flatpak --user remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak --user install -y --noninteractive flathub "org.gnome.Platform//$runtime"
flatpak build-init "$work" "$app_id" org.gnome.Platform org.gnome.Platform "$runtime"
prefix="$work/files"
executable=/app/bin/leaf
flags=()
if [[ "$runtime" == 51 ]]; then
  # Flatpak reserves /app/extra for install-time extra-data; build-export does
  # not publish a hand-populated files/extra directory. Use the real route.
  prefix="$RUNNER_TEMP/font279-payload-$runtime"
  executable=/app/extra/leaf/bin/leaf
else
  flags+=(--filesystem=xdg-templates:ro)
fi
mkdir -p "$prefix"
tar -xJf "$archive" -C "$prefix"
install -m0755 "$binary" "$prefix/bin/leaf"
cmp "$binary" "$prefix/bin/leaf"
server_pid=
trap 'test -z "$server_pid" || kill "$server_pid" 2>/dev/null || true' EXIT
if [[ "$runtime" == 51 ]]; then
  served="$RUNNER_TEMP/font279-http-$runtime"
  mkdir -p "$served" "$work/files/bin"
  tar -cJf "$served/font279.tar.xz" -C "$prefix" .
  python3 - "$served" <<'PY' &
import functools, http.server, pathlib, sys
root = pathlib.Path(sys.argv[1])
handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(root))
with http.server.ThreadingHTTPServer(('127.0.0.1', 0), handler) as server:
    (root / 'port').write_text(str(server.server_port))
    server.serve_forever()
PY
  server_pid=$!
  for n in $(seq 1 100); do test ! -s "$served/port" || break; kill -0 "$server_pid"; sleep 0.1; done
  port=$(cat "$served/port")
  checksum=$(sha256sum "$served/font279.tar.xz" | cut -d' ' -f1)
  size=$(stat -c %s "$served/font279.tar.xz")
  cat > "$work/files/bin/apply_extra" <<'APPLY'
#!/bin/sh
set -eu
mkdir leaf
tar --no-same-owner -xJf font279.tar.xz -C leaf
rm font279.tar.xz
APPLY
  chmod +x "$work/files/bin/apply_extra"
  mkdir -p "$work/files/share"
  for folder in applications metainfo icons; do
    cp -R "$prefix/share/$folder" "$work/files/share/"
  done
  cat > "$work/files/bin/leaf" <<'LAUNCH'
#!/bin/sh
exec /app/extra/leaf/bin/leaf "$@"
LAUNCH
  chmod +x "$work/files/bin/leaf"
  flags+=(--extra-data="font279.tar.xz:$checksum:$size:0:http://127.0.0.1:$port/font279.tar.xz" --metadata='Extra Data=NoRuntime=false')
fi
flatpak build-finish --command="$executable" --share=network --share=ipc \
  --socket=wayland --socket=fallback-x11 --device=dri "${flags[@]}" "$work"
flatpak build-export "$RUNNER_TEMP/font279-repo-$runtime" "$work"
flatpak --user remote-add --no-gpg-verify "font279-source-$runtime" "file://$RUNNER_TEMP/font279-repo-$runtime"
flatpak --user install -y --noninteractive "font279-source-$runtime" "$app_id"
mkdir -p dist/font279
flatpak --user info --show-metadata "$app_id" > "dist/font279/metadata-$runtime.ini"
printf '%s\n' "source=${LEAF279_NATIVE_SOURCE:-$GITHUB_SHA}" "validation_source=$GITHUB_SHA" "binary=$(sha256sum "$binary" | cut -d' ' -f1)" "resource_archive=138cdb67333384a086cd56190d5fe2f146c2202708fa6ac75751c8db31199203" > "dist/font279/identity-$runtime.txt"
echo "FONT279_SOURCE_PACKAGE_OK runtime=$runtime app_id=$app_id"
