#!/usr/bin/env bash
set -euo pipefail
exec flatpak run --user --branch=alpha --socket=x11 --nosocket=wayland \
  --env=GSK_RENDERER=cairo --env=LANGUAGE="${LANGUAGE:-en}" --env=GDK_SCALE="${GDK_SCALE:-1}" \
  --env=LEAF29_ACK_DIR="$LEAF29_ACK_DIR" --command=sh io.github.frandavid100.leaf \
  -c 'export XDG_CONFIG_HOME=$(mktemp -d) XDG_STATE_HOME=$(mktemp -d); exec /app/bin/leaf "$@"' leaf "$@"
