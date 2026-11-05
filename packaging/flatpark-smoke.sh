#!/usr/bin/env bash
# Run the release archive at FlatPark's prefix on its GNOME runtime.
set -euo pipefail
archive=${1:?Usage: flatpark-smoke.sh RELEASE_ARCHIVE}
app_id=io.github.frandavid100.leaf.FlatparkTest
runtime_version=51
if flatpak --user info "$app_id" >/dev/null 2>&1; then
  echo "The test application is already installed: $app_id" >&2
  exit 1
fi
temporary=$(mktemp -d)
trap 'flatpak kill "$app_id" >/dev/null 2>&1 || true; flatpak --user uninstall -y --noninteractive "$app_id" >/dev/null 2>&1 || true; rm -rf "$temporary"' EXIT
flatpak --user install -y --noninteractive flathub "org.gnome.Platform//$runtime_version"
flatpak build-init "$temporary/build" "$app_id" org.gnome.Platform org.gnome.Platform "$runtime_version"
mkdir -p "$temporary/build/files/extra/leaf"
tar -xJf "$archive" -C "$temporary/build/files/extra/leaf"
test -x "$temporary/build/files/extra/leaf/bin/leaf"
test -s "$temporary/build/files/extra/leaf/share/hunspell/es_ES.dic"
test -s "$temporary/build/files/extra/leaf/share/locale/es/LC_MESSAGES/leaf.mo"
flatpak build-finish --command=/app/extra/leaf/bin/leaf --share=ipc --socket=wayland --socket=fallback-x11 --device=dri "$temporary/build"
flatpak build-export "$temporary/repo" "$temporary/build"
flatpak build-bundle "$temporary/repo" "$temporary/leaf-test.flatpak" "$app_id"
flatpak --user install -y --noninteractive "$temporary/leaf-test.flatpak"
flatpak run "$app_id" --version
for language in en es; do
  log="$temporary/localization-$language.log"
  timeout 90s flatpak run --socket=x11 --nosocket=wayland \
    --env=GSK_RENDERER=cairo --env=LC_ALL=en_US.UTF-8 --env="LANGUAGE=$language" \
    "$app_id" --localization-ui-test >"$log" 2>&1 || { cat "$log"; exit 1; }
  cat "$log"
  grep -q "LOCALIZATION_UI_OK language=$language" "$log"
done
log="$temporary/proofing.log"
timeout 90s flatpak run --socket=x11 --nosocket=wayland \
  --env=GSK_RENDERER=cairo --env=LC_ALL=en_US.UTF-8 --env=LANGUAGE=es \
  "$app_id" --proofing-ui-test >"$log" 2>&1 || { cat "$log"; exit 1; }
cat "$log"
grep -q PROOFING_UI_OK "$log"
echo FLATPARK_GNOME51_RELOCATION_OK
