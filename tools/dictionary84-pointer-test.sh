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
      xdotool key --clearmodifiers ctrl+l
      xdotool type --clearmodifiers --delay 5 "${LEAF84_SOURCE:?Missing synthetic source}"
      xdotool key Return
      sleep 0.4
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
  last=$((last+1))
done
wait "$pid" || { cat "$log"; exit 1; }
cat "$log"
grep -q "DICTIONARY84_UI_OK mode=$mode" "$log"
