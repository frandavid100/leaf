#!/usr/bin/env bash
# Only a disposable CI account may install this original test-only font.
set -euo pipefail
[[ ${CI:-} == true ]] || { echo 'Requires a clean CI account.' >&2; exit 1; }
app_id=${1:?application ID}
binary=${2:?installed binary}
output=$(realpath -m "${3:?evidence directory}")
root=$(cd "$(dirname "$0")/.." && pwd)
mode=${LEAF279_MODE:-local}
case "$mode" in local|dismiss) ;; *) exit 1 ;; esac
mkdir -p "$output" "$HOME/.var/app/$app_id/cache"
temporary=$(mktemp -d)
acks=$(mktemp -d "$HOME/.var/app/$app_id/cache/font279-local.XXXXXX")
font="$acks/Leaf279QA-Regular.ttf"
target="$HOME/.local/share/fonts/Leaf279QA-Regular.ttf"
test ! -e "$target"
python3 "$root/tools/font279-fixture.py" "$font"
export XDG_CONFIG_HOME="$temporary/config" XDG_CURRENT_DESKTOP=GNOME GTK_A11Y=atspi NO_AT_BRIDGE=0
mkdir -p "$XDG_CONFIG_HOME"
cat > "$XDG_CONFIG_HOME/mimeapps.list" <<'MIME'
[Default Applications]
font/ttf=org.gnome.font-viewer.desktop;
application/x-font-ttf=org.gnome.font-viewer.desktop;
MIME
export LEAF279_ACK_DIR="$acks" LEAF279_APP_ID="$app_id" LEAF279_BINARY="$binary"
export LEAF279_OUTPUT="$output" LEAF279_ROOT="$root" LEAF279_FONT="$font" LEAF279_TARGET="$target" LEAF279_MODE="$mode"
cat > "$temporary/test.sh" <<'RUN'
#!/usr/bin/env bash
set -euo pipefail
dbus-update-activation-environment DISPLAY XAUTHORITY XDG_CONFIG_HOME XDG_CURRENT_DESKTOP GTK_A11Y NO_AT_BRIDGE
flatpak run --socket=x11 --nosocket=wayland --env="LANGUAGE=${LANGUAGE:-en}" --env=LC_ALL=C.UTF-8 \
  --env="GDK_SCALE=${GDK_SCALE:-1}" --env=GSK_RENDERER=cairo --env=GTK_A11Y=atspi \
  --env="LEAF279_ACK_DIR=$LEAF279_ACK_DIR" --env="LEAF279_MODE=$LEAF279_MODE" \
  --command="$LEAF279_BINARY" "$LEAF279_APP_ID" --font279-acceptance-test > "$LEAF279_OUTPUT/leaf.log" 2>&1 &
app=$!
trap 'scrot "$LEAF279_OUTPUT/final.png" 2>/dev/null || true; xwininfo -root -tree > "$LEAF279_OUTPUT/windows.txt" 2>/dev/null || true; kill "$app" 2>/dev/null || true' EXIT
for n in $(seq 1 900); do
  test ! -f "$LEAF279_ACK_DIR/ready-local" || break
  kill -0 "$app" || { cat "$LEAF279_OUTPUT/leaf.log"; exit 1; }
  sleep 0.1
done
test -f "$LEAF279_ACK_DIR/ready-local"
for n in $(seq 1 200); do
  chooser=$(xdotool search --onlyvisible --name '^(Open Font File|Abrir archivo de fuente)$' | head -1 || true)
  test -z "$chooser" || break
  sleep 0.1
done
test -n "$chooser"
xdotool windowfocus --sync "$chooser" key ctrl+l
printf '%s' "$LEAF279_FONT" | xclip -selection clipboard
sleep 0.3
xdotool key ctrl+a ctrl+v
sleep 0.6
xdotool key Return
sleep 1
if xdotool search --onlyvisible --name '^(Open Font File|Abrir archivo de fuente)$' >/dev/null 2>&1; then
  xdotool key Return
fi
if [[ "$LEAF279_MODE" == local ]]; then
  python3 "$LEAF279_ROOT/tools/font279-viewer-install.py" | tee "$LEAF279_OUTPUT/viewer.log"
  for n in $(seq 1 150); do test ! -s "$LEAF279_TARGET" || break; sleep 0.1; done
  cmp "$LEAF279_FONT" "$LEAF279_TARGET"
else
  for n in $(seq 1 300); do
    xdotool search --onlyvisible --class 'gnome-font-viewer' >/dev/null 2>&1 && break
    sleep 0.1
  done
  xdotool search --onlyvisible --class 'gnome-font-viewer' >/dev/null
  test ! -e "$LEAF279_TARGET"
fi
scrot "$LEAF279_OUTPUT/viewer.png"
xdotool search --onlyvisible --class 'gnome-font-viewer' windowclose || true
touch "$LEAF279_ACK_DIR/viewer-done"
wait "$app" || { cat "$LEAF279_OUTPUT/leaf.log"; exit 1; }
grep -q "FONT279_ACCEPTANCE_OK mode=$LEAF279_MODE" "$LEAF279_OUTPUT/leaf.log"
RUN
cleanup() {
  if [[ -f "$target" ]] && cmp -s "$font" "$target"; then rm "$target"; fi
  fc-cache -f >/dev/null 2>&1 || true
  rm -rf "$temporary"
}
trap cleanup EXIT
runtime=$(mktemp -d "${RUNNER_TEMP:-/tmp}/font279-runtime.XXXXXX")
uid=$(id -u)
xvfb-run -a -s '-screen 0 2800x2200x24' sudo -E unshare --mount --propagation private -- \
  bash -euc 'mount --bind "$1" "$2"; shift 2; exec setpriv "$@"' font279-mount "$runtime" "${XDG_RUNTIME_DIR:?}" \
  --reuid="$uid" --regid="$(id -g)" --init-groups -- dbus-run-session -- timeout 480s bash "$temporary/test.sh"
echo "FONT279_LOCAL_INSTALLED_OK mode=$mode"
