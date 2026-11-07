#!/usr/bin/env bash
# Real keyboard and native numeric controls in either installed runtime layout.
set -euo pipefail
export LEAF_PENDING_APP_ID=${1:?Usage: pending-installed-test.sh APP_ID BINARY OUTPUT}
export LEAF_PENDING_BINARY=${2:?Missing installed binary path}
output=${3:?Missing evidence directory}
root=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$output" "$HOME/.var/app/$LEAF_PENDING_APP_ID/cache"
output=$(realpath "$output")
export LEAF_PENDING_ACK_PARENT="$HOME/.var/app/$LEAF_PENDING_APP_ID/cache"
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT
cat > "$temporary/installed.sh" <<'RUNNER'
#!/usr/bin/env bash
exec flatpak run --socket=x11 --nosocket=wayland --env=LC_ALL=en_US.UTF-8 \
  --env=LANGUAGE="${LANGUAGE:-en}" --env=GDK_SCALE="${GDK_SCALE:-1}" \
  --env=GSK_RENDERER=cairo --env=LEAF_PENDING_ACK="${LEAF_PENDING_ACK:-}" \
  --command=sh "$LEAF_PENDING_APP_ID" \
  -c 'export XDG_CONFIG_HOME=$(mktemp -d) XDG_STATE_HOME=$(mktemp -d); exec "$@"' \
  leaf "$LEAF_PENDING_BINARY" "$@"
RUNNER
cd "$output"
for language in en es; do
  scale=1; screen=1400x1100x24
  if [[ "$language" == es ]]; then scale=2; screen=2800x2200x24; fi
  export LANGUAGE=$language GDK_SCALE=$scale
  xvfb-run -a -s "-screen 0 $screen" timeout 90s bash "$temporary/installed.sh" \
    --pending-spacing-ui-test >"spacing-$language-$scale.log" 2>&1 || {
      cat "spacing-$language-$scale.log"; exit 1;
    }
  grep -q PENDING_SPACING_UI_OK "spacing-$language-$scale.log"
  xvfb-run -a -s "-screen 0 $screen" timeout 90s bash \
    "$root/tools/pending-issues-pointer-test.sh" "$temporary/installed.sh"
  mv pending-end.log "end-$language-$scale.log"
  mv pending-end-first-key.png "end-$language-$scale.png"
done
echo PENDING_INSTALLED_CONTROLS_OK
