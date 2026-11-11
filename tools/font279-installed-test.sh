#!/usr/bin/env bash
# Clean CI account only: the host's real font viewer performs the installation.
set -euo pipefail
[[ ${CI:-} == true ]] || { echo 'Use a clean CI account for the system-font installer test.' >&2; exit 1; }
app_id=${1:?Missing installed application ID}
binary=${2:?Missing installed executable}
output=$(realpath -m "${3:?Missing evidence folder}")
root=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$output" "$HOME/.local/share/fonts" "$HOME/.var/app/$app_id/cache"
temporary=$(mktemp -d)
acks=$(mktemp -d "$HOME/.var/app/$app_id/cache/font279.XXXXXX")
export XDG_CONFIG_HOME="$temporary/config" XDG_CURRENT_DESKTOP=GNOME GTK_A11Y=atspi NO_AT_BRIDGE=0
mkdir -p "$XDG_CONFIG_HOME"
for style in Regular Bold Italic BoldItalic; do
  test ! -e "$HOME/.local/share/fonts/Poppins-$style.ttf"
done
cat > "$XDG_CONFIG_HOME/mimeapps.list" <<'MIME'
[Default Applications]
font/ttf=org.gnome.font-viewer.desktop;
application/x-font-ttf=org.gnome.font-viewer.desktop;
MIME
export LEAF279_ACK_DIR="$acks" LEAF279_APP_ID="$app_id" LEAF279_BINARY="$binary" LEAF279_ROOT="$root" LEAF279_OUTPUT="$output"
cat > "$temporary/test.sh" <<'RUN'
#!/usr/bin/env bash
set -euo pipefail
dbus-update-activation-environment DISPLAY XAUTHORITY XDG_CONFIG_HOME XDG_CURRENT_DESKTOP GTK_A11Y NO_AT_BRIDGE
flatpak run --socket=x11 --nosocket=wayland --env=LANGUAGE=en --env=LC_ALL=C.UTF-8 --env=GDK_SCALE=1 \
  --env=GSK_RENDERER=cairo --env=GTK_A11Y=atspi --env=LEAF279_ACK_DIR="$LEAF279_ACK_DIR" \
  --command="$LEAF279_BINARY" "$LEAF279_APP_ID" --font279-ui-test > "$LEAF279_OUTPUT/leaf.log" 2>&1 &
app=$!
trap 'kill "$app" 2>/dev/null || true' EXIT
for style in Regular Bold Italic BoldItalic; do
  for n in $(seq 1 1200); do
    if [[ -f "$LEAF279_ACK_DIR/ready-$style" ]]; then break; fi
    kill -0 "$app" || { cat "$LEAF279_OUTPUT/leaf.log"; exit 1; }
    sleep 0.1
  done
  test -f "$LEAF279_ACK_DIR/ready-$style"
  python3 "$LEAF279_ROOT/tools/font279-viewer-install.py" | tee "$LEAF279_OUTPUT/viewer-$style.log"
  installed="$HOME/.local/share/fonts/Poppins-$style.ttf"
  for n in $(seq 1 150); do test ! -s "$installed" || break; sleep 0.1; done
  test -s "$installed"
  FONT279_STYLE="$style" python3 - <<'PY'
import hashlib, os, pathlib
style = os.environ['FONT279_STYLE']
name = f'Poppins-{style}.ttf'
cache = pathlib.Path.home() / '.var/app' / os.environ['LEAF279_APP_ID'] / 'cache/leaf/font-downloads'
sources = list(cache.glob(f'*/Poppins/*/{name}'))
target = pathlib.Path.home() / '.local/share/fonts' / name
assert any(hashlib.sha256(p.read_bytes()).digest() == hashlib.sha256(target.read_bytes()).digest() for p in sources)
print('FONT279_HOST_BYTES_OK', style)
PY
  scrot "$LEAF279_OUTPUT/installed-$style.png"
  xdotool search --onlyvisible --class 'gnome-font-viewer' windowclose || true
  touch "$LEAF279_ACK_DIR/installed-$style"
done
wait "$app"
grep -q 'FONT279_SYSTEM_INSTALL_OK styles=Regular,Bold,Italic,BoldItalic' "$LEAF279_OUTPUT/leaf.log"
RUN
cleanup() {
  # Delete only this test's exact downloaded bytes; never a pre-existing font.
  LEAF279_APP_ID="$app_id" python3 - <<'PY'
import hashlib, os, pathlib
cache = pathlib.Path.home() / '.var/app' / os.environ['LEAF279_APP_ID'] / 'cache/leaf/font-downloads'
for style in ['Regular', 'Bold', 'Italic', 'BoldItalic']:
    name = f'Poppins-{style}.ttf'
    target = pathlib.Path.home() / '.local/share/fonts' / name
    if target.exists() and any(hashlib.sha256(p.read_bytes()).digest() == hashlib.sha256(target.read_bytes()).digest() for p in cache.glob(f'*/Poppins/*/{name}')):
        target.unlink()
PY
  fc-cache -f >/dev/null 2>&1 || true
  rm -rf "$temporary"
}
trap cleanup EXIT
runtime=$(mktemp -d "${RUNNER_TEMP:-/tmp}/font279-runtime.XXXXXX")
uid=$(id -u)
xvfb-run -a -s '-screen 0 1800x1400x24' sudo -E unshare --mount --propagation private -- \
  bash -euc 'mount --bind "$1" "$2"; shift 2; exec setpriv "$@"' font279-mount "$runtime" "${XDG_RUNTIME_DIR:?Missing runtime directory}" \
  --reuid="$uid" --regid="$(id -g)" --init-groups -- dbus-run-session -- timeout 600s bash "$temporary/test.sh"
echo FONT279_INSTALLED_PORTAL_OK
