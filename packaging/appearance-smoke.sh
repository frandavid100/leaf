#!/usr/bin/env bash
# Actual native/installed rendering; internal evidence, never README screenshots.
set -euo pipefail
mode=${1:?native or flatpak}
scale=${2:-1}
binary=${3:-./target/release/wordcraft-gnome}
app_id=io.github.frandavid100.leaf
out=${LEAF_APPEARANCE_OUT:-dist}
mkdir -p "$out"
for variant in light dark high-contrast high-contrast-dark; do
  contrast=0
  [[ "$variant" != high-contrast* ]] || contrast=1
  log="$out/theme-$mode-$variant-$scale.log"
  config=$(mktemp -d)
  state=$(mktemp -d)
  if [[ "$mode" = native ]]; then
    LC_ALL=en_US.UTF-8 LANGUAGE=es GDK_SCALE=$scale ADW_DEBUG_HIGH_CONTRAST=$contrast XDG_CONFIG_HOME=$config XDG_STATE_HOME=$state "$binary" --main-menu-demo >"$log" 2>&1 &
  elif [[ "$mode" = flatpak ]]; then
    flatpak run --env=LC_ALL=en_US.UTF-8 --env=LANGUAGE=es --env=GDK_SCALE="$scale" --env=ADW_DEBUG_HIGH_CONTRAST="$contrast" --socket=x11 --nosocket=wayland --command=sh "$app_id" -c 'export XDG_CONFIG_HOME=$(mktemp -d) XDG_STATE_HOME=$(mktemp -d); exec /app/bin/leaf --main-menu-demo' >"$log" 2>&1 &
  else
    exit 2
  fi
  pid=$!
  trap 'kill "$pid" 2>/dev/null || true; rm -rf "$config" "$state"' EXIT
  window=
  for attempt in $(seq 1 150); do
    kill -0 "$pid" || { cat "$log"; exit 1; }
    window=$(xdotool search --onlyvisible --name '^Leaf$' 2>/dev/null | head -n1 || true)
    [[ -z "$window" ]] || break
    sleep 0.1
  done
  test -n "$window"
  xdotool windowfocus --sync "$window"
  sleep 1.8
  # The established three 44 logical-pixel GNOME previews retain generous targets.
  x=219
  [[ "$variant" != *dark ]] || x=316
  xdotool mousemove --sync --window "$window" "$((x*scale))" "$((96*scale))" click 1
  sleep 0.2
  scrot "$out/theme-$mode-$variant-$scale-hover.png"
  xdotool mousemove --sync --window "$window" "$((600*scale))" "$((300*scale))"
  sleep 0.25
  scrot "$out/theme-$mode-$variant-$scale-selected.png"
  # Nine stock menu entries precede the selected theme in reverse tab order.
  # Capture the menu focus and the actual native checkbutton focus ring.
  for step in $(seq 1 10); do
    xdotool key shift+Tab
    sleep 0.1
    if [[ "$step" = 1 || "$step" = 10 ]]; then
      scrot "$out/theme-$mode-$variant-$scale-keyboard-$step.png"
    fi
  done
  if grep -E 'Invalid value for ADW_DEBUG|panicked at|Gtk-CRITICAL' "$log"; then exit 1; fi
  if [[ "$mode" = flatpak ]]; then flatpak kill "$app_id"; else kill "$pid"; fi
  wait "$pid" || true
  rm -rf "$config" "$state"
  trap - EXIT
  echo "THEME_APPEARANCE_OK mode=$mode variant=$variant scale=$scale"
done
