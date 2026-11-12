#!/usr/bin/env bash
# Clean CI account only: the host's real font viewer performs the installation.
set -euo pipefail
[[ ${CI:-} == true ]] || { echo 'Use a clean CI account for the system-font installer test.' >&2; exit 1; }
app_id=${1:?Missing installed application ID}
binary=${2:?Missing installed executable}
output=$(realpath -m "${3:?Missing evidence folder}")
root=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$output" "$HOME/.var/app/$app_id/cache"
if [[ ${LEAF279_ABSENT_FONT_DIR:-} == 1 ]]; then
  test ! -e "$HOME/.local/share/fonts"
  echo FONT279_START_WITHOUT_USER_FONT_DIRECTORY
else
  mkdir -p "$HOME/.local/share/fonts"
fi
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
python3 - <<'PY'
from gi.repository import Gio
apps = Gio.AppInfo.get_all_for_type('font/ttf')
assert any(app.get_id() == 'org.gnome.font-viewer.desktop' for app in apps), [app.get_id() for app in apps]
print('FONT279_HOST_FONT_HANDLER_OK')
PY
export LEAF279_ACK_DIR="$acks" LEAF279_APP_ID="$app_id" LEAF279_BINARY="$binary" LEAF279_ROOT="$root" LEAF279_OUTPUT="$output"
cat > "$temporary/test.sh" <<'RUN'
#!/usr/bin/env bash
set -euo pipefail
dbus-update-activation-environment DISPLAY XAUTHORITY XDG_CONFIG_HOME XDG_CURRENT_DESKTOP GTK_A11Y NO_AT_BRIDGE
flatpak run --socket=x11 --nosocket=wayland --env="LANGUAGE=${LANGUAGE:-en}" --env=LC_ALL=C.UTF-8 --env="GDK_SCALE=${GDK_SCALE:-1}" \
  --env=GSK_RENDERER=cairo --env=GTK_A11Y=atspi --env=LEAF279_ACK_DIR="$LEAF279_ACK_DIR" \
  --env="LEAF279_REFERENCE=${LEAF279_REFERENCE:-0}" \
  --env="LEAF279_FIRST_USE_RECOVERY=${LEAF279_ABSENT_FONT_DIR:-0}" \
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
  find "$HOME/.var/app/$LEAF279_APP_ID/cache/leaf/font-downloads" -name "Poppins-$style.ttf" -exec gio info -a standard::content-type '{}' \;
  if ! python3 "$LEAF279_ROOT/tools/font279-viewer-install.py" | tee "$LEAF279_OUTPUT/viewer-$style.log"; then
    scrot "$LEAF279_OUTPUT/viewer-failure-$style.png"
    xwininfo -root -tree > "$LEAF279_OUTPUT/windows-$style.txt"
    cat "$LEAF279_OUTPUT/leaf.log"
    exit 1
  fi
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
wait "$app" || { cat "$LEAF279_OUTPUT/leaf.log"; exit 1; }
if grep -q 'FONT279_RESTART_REQUIRED styles=Regular,Bold,Italic,BoldItalic' "$LEAF279_OUTPUT/leaf.log"; then
  test "${LEAF279_ABSENT_FONT_DIR:-0}" = 1
  flatpak run --socket=x11 --nosocket=wayland --env="LANGUAGE=${LANGUAGE:-en}" --env=LC_ALL=C.UTF-8 \
    --env="GDK_SCALE=${GDK_SCALE:-1}" --env=GSK_RENDERER=cairo --env=GTK_A11Y=atspi \
    --env="LEAF279_ACK_DIR=$LEAF279_ACK_DIR" --env="LEAF279_REFERENCE=${LEAF279_REFERENCE:-0}" \
    --command="$LEAF279_BINARY" "$LEAF279_APP_ID" --font279-restart-test > "$LEAF279_OUTPUT/restart.log" 2>&1 || { cat "$LEAF279_OUTPUT/restart.log"; exit 1; }
  grep -q 'FONT279_RESTART_RECOVERY_OK styles=Regular,Bold,Italic,BoldItalic' "$LEAF279_OUTPUT/restart.log"
  cat "$LEAF279_OUTPUT/restart.log"
else
  grep -q 'FONT279_SYSTEM_INSTALL_OK styles=Regular,Bold,Italic,BoldItalic' "$LEAF279_OUTPUT/leaf.log"
fi
if [[ ${LEAF279_REFERENCE:-} == 1 ]]; then
  mkdir -p "$LEAF279_OUTPUT/reference"
  cp "$LEAF279_ACK_DIR/font279.docx" "$LEAF279_ACK_DIR/font279-leaf.pdf" "$LEAF279_OUTPUT/"
  timeout 60s libreoffice -env:UserInstallation="file://$LEAF279_OUTPUT/lo-profile" --headless \
    --convert-to pdf --outdir "$LEAF279_OUTPUT/reference" "$LEAF279_OUTPUT/font279.docx"
  python3 "$LEAF279_ROOT/tools/font279-reference-compare.py" \
    "$LEAF279_OUTPUT/font279-leaf.pdf" "$LEAF279_OUTPUT/reference/font279.pdf" | tee "$LEAF279_OUTPUT/reference.log"
fi
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
xvfb-run -a -s '-screen 0 2800x2200x24' sudo -E unshare --mount --propagation private -- \
  bash -euc 'mount --bind "$1" "$2"; shift 2; exec setpriv "$@"' font279-mount "$runtime" "${XDG_RUNTIME_DIR:?Missing runtime directory}" \
  --reuid="$uid" --regid="$(id -g)" --init-groups -- dbus-run-session -- timeout 600s bash "$temporary/test.sh"
echo FONT279_INSTALLED_PORTAL_OK
