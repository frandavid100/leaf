#!/usr/bin/env bash
# Validate the actual host folder portal in either Flatpak runtime layout.
set -euo pipefail
export XDG_CURRENT_DESKTOP=GNOME
export LEAF84_APP_ID=${1:?Usage: dictionary84-installed-test.sh APP_ID BINARY OUTPUT}
export LEAF84_BINARY=${2:?Missing binary}
output=${3:?Missing output}
root=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$output" "$HOME/.var/app/$LEAF84_APP_ID/cache"
output=$(realpath "$output")
export LEAF84_PROFILE_PARENT="$HOME/.var/app/$LEAF84_APP_ID/cache"
export LEAF84_ACK_PARENT=$LEAF84_PROFILE_PARENT
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT
cat > "$temporary/installed.sh" <<'RUNNER'
#!/usr/bin/env bash
exec flatpak run --socket=x11 --nosocket=wayland \
  --env=LC_ALL=en_US.UTF-8 --env=LANGUAGE="${LANGUAGE:-en}" \
  --env=GDK_SCALE="${GDK_SCALE:-1}" --env=GSK_RENDERER=cairo \
  --env=LEAF84_ACK="${LEAF84_ACK:-}" \
  --env=LEAF84_PROFILE="$LEAF84_PROFILE" \
  --command=sh "$LEAF84_APP_ID" \
  -c 'export XDG_CONFIG_HOME="$LEAF84_PROFILE/config" XDG_STATE_HOME="$LEAF84_PROFILE/state"; exec "$@"' \
  leaf "$LEAF84_BINARY" "$@"
RUNNER
cd "$output"
for language in en es; do
  scale=1;screen=1400x1100x24
  if [[ "$language" == es ]]; then scale=2;screen=2800x2200x24;fi
  LANGUAGE=$language GDK_SCALE=$scale xvfb-run -a -s "-screen 0 $screen" \
    dbus-run-session -- timeout 240s bash "$root/tools/dictionary84-test.sh" "$temporary/installed.sh" folder
done
echo DICTIONARY84_INSTALLED_PORTAL_OK
