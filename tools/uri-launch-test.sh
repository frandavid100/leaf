#!/usr/bin/env bash
# Run in an isolated CI desktop/session. The handler records the URL; no network.
set -euo pipefail
leaf_runner=${1:-./target/release/wordcraft-gnome}
leaf_uri_test=$(mktemp -d /tmp/leaf-uri-test.XXXXXX)
leaf_apps="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
mkdir -p "$leaf_apps"
cat > "$leaf_uri_test/browser" <<'BROWSER'
#!/bin/sh
printf '%s' "$2" > "$1"
BROWSER
chmod +x "$leaf_uri_test/browser"
cat > "$leaf_apps/leaf-qa-browser.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Leaf QA Browser
Exec=$leaf_uri_test/browser $leaf_uri_test/captured-url %u
NoDisplay=true
MimeType=x-scheme-handler/https;
DESKTOP
gio mime x-scheme-handler/https leaf-qa-browser.desktop
leaf_cmd=("$leaf_runner")
if [[ "$leaf_runner" = *.sh ]]; then leaf_cmd=(bash "$leaf_runner"); fi
"${leaf_cmd[@]}" --uri-launch-test >uri-launch.log 2>&1 &
leaf_pid=$!
trap 'kill "$leaf_pid" 2>/dev/null || true' EXIT
for attempt in $(seq 1 200); do
  test -s "$leaf_uri_test/captured-url" && break
  kill -0 "$leaf_pid" || { cat uri-launch.log; exit 1; }
  sleep 0.1
done
test "$(cat "$leaf_uri_test/captured-url" 2>/dev/null)" = 'https://example.org/leaf-launch-test'
echo URI_SYSTEM_LAUNCH_OK | tee -a uri-launch.log
if [[ "$leaf_runner" = *flatpak* ]]; then flatpak kill io.github.frandavid100.leaf || true; fi
kill "$leaf_pid" 2>/dev/null || true
wait "$leaf_pid" || true
rm -f "$leaf_apps/leaf-qa-browser.desktop"
