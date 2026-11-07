#!/usr/bin/env bash
# Exercise actual chooser permissions and controls with synthetic data only.
set -euo pipefail
runner=${1:-./target/release/wordcraft-gnome}
mode=${2:-folder}
cmd=("$runner"); [[ "$runner" != *.sh ]] || cmd=(bash "$runner")
flag=--dictionary84-ui-test
[[ "$mode" != reopen ]] || flag=--dictionary84-reopen-test
[[ "$mode" != native ]] || flag=--dictionary84-native-test
export LEAF84_ACK
LEAF84_ACK=$(mktemp -d "${LEAF84_ACK_PARENT:-/tmp}/leaf84-ack.XXXXXX")
log="dictionary84-$mode-${LANGUAGE:-en}-${GDK_SCALE:-1}.log"
"${cmd[@]}" "$flag" >"$log" 2>&1 &
pid=$!
trap 'scrot "${log%.log}-final.png" 2>/dev/null || true; kill "$pid" 2>/dev/null || true; rm -rf "$LEAF84_ACK"' EXIT
last=0
chooser_window() {
  for _ in $(seq 1 200); do
    dialog=$(xdotool search --onlyvisible --name '^(Choose a Dictionary Folder|Seleccionar una carpeta de diccionarios)$' | head -n1 || true)
    if [[ -n "$dialog" ]]; then printf '%s\n' "$dialog"; return; fi
    kill -0 "$pid" || { cat "$log" >&2; return 1; }
    sleep 0.05
  done
  cat "$log" >&2; return 1
}
while kill -0 "$pid" 2>/dev/null; do
  count=$(grep -c '^DICTIONARY84_READY ' "$log" || true)
  if ((count<=last)); then sleep 0.05; continue; fi
  read -r _ stage role x y < <(grep '^DICTIONARY84_READY ' "$log" | sed -n "$((last+1))p")
  window=$(xdotool search --onlyvisible --name '^Leaf Dictionary Probe$' | head -n1)
  if [[ "$role" == install-system ]]; then
    [[ ${CI:-} == true ]] || { echo 'System installation probe runs only on CI.' >&2; exit 1; }
    sudo timeout 120s apt-get -o Acquire::Retries=1 -o Acquire::http::Timeout=20 -o Acquire::https::Timeout=20 install -y --no-install-recommends hunspell-cs
  elif [[ "$role" == update-and-refresh ]]; then
    test -f "${LEAF84_SOURCE:?Missing synthetic source}/LEAF_TEST_FIXTURE"
    python3 - "$LEAF84_SOURCE" <<'PY'
from pathlib import Path
import sys
p=Path(sys.argv[1])/'it_IT.dic'
p.write_text('4\nfoglio\nprova\nfoglioprovaextra\nfoglioprovanuova\n')
PY
  fi
  xdotool windowfocus --sync "$window"
  xdotool mousemove --window "$window" "$x" "$y" click 1
  case "$role" in
    choose)
      dialog=$(chooser_window)
      xdotool windowfocus --sync "$dialog"
      test -d "${LEAF84_SOURCE:?Missing synthetic source}"
      printf 'DICTIONARY84_DRIVER_SOURCE %s\n' "$LEAF84_SOURCE"
      if [[ -n ${LEAF84_APP_ID:-} ]]; then
        # GTK3's portal chooser disables acceptance in Recent mode. Enter the
        # folder browser before supplying the requested source; grant only it.
        xdotool key --clearmodifiers alt+Home
        sleep 0.5
      fi
      # GTK's inline completion can change a path during synthetic typing or
      # Return. Paste it atomically and activate the actual accept mnemonic.
      xdotool key --clearmodifiers ctrl+l
      sleep 0.3
      xdotool key --clearmodifiers ctrl+a
      printf '%s' "$LEAF84_SOURCE" | xclip -selection clipboard
      xdotool key --clearmodifiers ctrl+v
      sleep 0.5
      scrot "${log%.log}-location.png"
      xdotool key --clearmodifiers alt+u
      ;;
    cancel)
      dialog=$(chooser_window)
      xdotool windowfocus --sync "$dialog"
      xdotool key Escape
      ;;
    close-owner) chooser_window >/dev/null ;;
    install-system|update-and-refresh|forget) ;;
    *) echo "Unknown dictionary probe stage: $role" >&2; exit 1 ;;
  esac
  sleep 0.3
  scrot "${log%.log}-$stage.png"
  touch "$LEAF84_ACK/$stage"
  if [[ "$role" == update-and-refresh && ${LEAF84_GDB:-} == 1 ]]; then
    (
      sleep 15
      if kill -0 "$pid" 2>/dev/null && ! grep -q DICTIONARY84_REFRESH_OK "$log"; then
        touch "${log%.log}-hang-detected"
        app_pid=$(pgrep -f '^/app/bin/leaf --dictionary84-ui-test' | head -n1 || true)
        if [[ -n "$app_pid" ]]; then
          sudo timeout 25s gdb --batch "$LEAF84_GDB_BINARY" -ex 'set pagination off' -ex 'thread apply all bt' -p "$app_pid" > "${log%.log}-backtrace.log" 2>&1 || true
        fi
        folder=$(python3 - "$LEAF84_PROFILE/config/wordcraft-gnome/preferences.json" <<'PYTHON'
import json,sys
print(json.load(open(sys.argv[1]))['dictionary_folder'])
PYTHON
        )
        timeout 8s flatpak run --command=sh "$LEAF84_APP_ID" -c 'test -f "$1/1" && cat "$2/it_IT.dic"' leaf "$LEAF84_ACK" "$folder" > "${log%.log}-portal-read.log" 2>&1 || true
        cat "${log%.log}-backtrace.log" "${log%.log}-portal-read.log" || true
      fi
    ) &
  fi
  last=$((last+1))
done
wait "$pid" || { cat "$log"; exit 1; }
cat "$log"
grep -q "DICTIONARY84_UI_OK mode=$mode" "$log"
test ! -f "${log%.log}-hang-detected"
