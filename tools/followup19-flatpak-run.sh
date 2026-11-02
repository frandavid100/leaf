#!/usr/bin/env bash
set -euo pipefail
leaf_environment=()
for variable in LANGUAGE GDK_SCALE LEAF19_ACK_DIR LEAF19_RANGE LEAF19_NARROW LEAF19_LARGE; do
  if [[ -v "$variable" ]]; then leaf_environment+=(--env="$variable=${!variable}"); fi
done
exec flatpak run "${leaf_environment[@]}" --socket=x11 --nosocket=wayland io.github.frandavid100.leaf "$@"
