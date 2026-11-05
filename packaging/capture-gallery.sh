#!/usr/bin/env bash
set -euo pipefail
mode=${1:-native}
output=${2:-alpha28-gallery}
mkdir -p "$output"
xcompmgr -c > "$output/compositor.log" 2>&1 &
compositor=$!
pid=
cleanup() {
  [[ -z "$pid" ]] || kill "$pid" 2>/dev/null || true
  if [[ "$mode" == flatpak ]]; then flatpak kill io.github.frandavid100.leaf 2>/dev/null || true; fi
  kill "$compositor" 2>/dev/null || true
}
trap cleanup EXIT
for variant in main notes review format; do
  if [[ "$mode" == flatpak ]]; then
    flatpak run --socket=x11 --nosocket=wayland --command=sh \
      --env=LANGUAGE=en --env=LC_ALL=en_US.UTF-8 --env=GSK_RENDERER=cairo --env="LEAF_GALLERY=$variant" \
      io.github.frandavid100.leaf -c \
      'export XDG_CONFIG_HOME=$(mktemp -d) XDG_STATE_HOME=$(mktemp -d); exec /app/bin/leaf --alpha28-gallery' \
      > "$output/$variant.log" 2>&1 &
  else
    XDG_CONFIG_HOME=$(mktemp -d) XDG_STATE_HOME=$(mktemp -d) LANGUAGE=en \
      GSK_RENDERER=cairo LEAF_GALLERY=$variant ./target/release/wordcraft-gnome \
      --alpha28-gallery > "$output/$variant.log" 2>&1 &
  fi
  pid=$!
  for attempt in $(seq 1 200); do
    grep -q ALPHA28_GALLERY_READY "$output/$variant.log" && break
    kill -0 "$pid" 2>/dev/null || { cat "$output/$variant.log"; exit 1; }
    sleep 0.1
  done
  grep -q ALPHA28_GALLERY_READY "$output/$variant.log"
  sleep 1
  window=$(xdotool search --onlyvisible --name '^Leaf$' | head -n1)
  xdotool windowfocus --sync "$window"
  geometry=$(xdotool getwindowgeometry --shell "$window")
  leaf_x=$(sed -n 's/^X=//p' <<< "$geometry")
  leaf_y=$(sed -n 's/^Y=//p' <<< "$geometry")
  leaf_width=$(sed -n 's/^WIDTH=//p' <<< "$geometry")
  leaf_height=$(sed -n 's/^HEIGHT=//p' <<< "$geometry")
  # Capture the composited desktop rectangle, including native popup surfaces.
  scrot --autoselect="$leaf_x,$leaf_y,$leaf_width,$leaf_height" "$output/leaf-alpha28-$variant.png"
  if [[ "$mode" == flatpak ]]; then flatpak kill io.github.frandavid100.leaf; else kill "$pid"; fi
  wait "$pid" || true
  pid=
done
