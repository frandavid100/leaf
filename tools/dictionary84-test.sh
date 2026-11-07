#!/usr/bin/env bash
# Prepare a private fixture/profile, then verify chooser, refresh and restart.
set -euo pipefail
runner=${1:-./target/release/wordcraft-gnome}
mode=${2:-folder}
root=$(cd "$(dirname "$0")/.." && pwd)
temporary=$(mktemp -d "${LEAF84_PROFILE_PARENT:-/tmp}/leaf84-test.XXXXXX")
export LEAF84_SOURCE
LEAF84_SOURCE=$(mktemp -d /tmp/leaf84-host.XXXXXX)
trap 'rm -rf "$temporary" "$LEAF84_SOURCE"' EXIT
cp "$root/tests/fixtures/dictionary84/it_IT.aff" "$root/tests/fixtures/dictionary84/it_IT.dic" "$LEAF84_SOURCE/"
touch "$LEAF84_SOURCE/LEAF_TEST_FIXTURE"
export LEAF84_PROFILE="$temporary/profile"
mkdir -p "$LEAF84_PROFILE" "$temporary/evidence"
if [[ -z ${LEAF84_APP_ID:-} ]]; then
  export XDG_CONFIG_HOME="$LEAF84_PROFILE/config" XDG_STATE_HOME="$LEAF84_PROFILE/state"
else
  # Raw host paths are not granted by the folder portal. Only its returned URI
  # may be used; this assertion also catches accidentally widened permissions.
  flatpak run --command=sh "$LEAF84_APP_ID" -c 'test ! -r "$1"' sh "$LEAF84_SOURCE/it_IT.dic"
fi
bash "$root/tools/dictionary84-pointer-test.sh" "$runner" "$mode"
if [[ "$mode" == folder ]]; then
  bash "$root/tools/dictionary84-pointer-test.sh" "$runner" reopen
  grep -qx 'SET UTF-8' "$LEAF84_SOURCE/it_IT.aff"
  python3 - "$LEAF84_SOURCE" <<'PY'
from pathlib import Path
import sys
p=Path(sys.argv[1])/'it_IT.dic'
assert p.read_text()=='4\nfoglio\nprova\nfoglioprovaextra\nfoglioprovanuova\n'
print('DICTIONARY84_SOURCE_FILES_OK')
PY
fi
