#!/usr/bin/env bash
# Reuse the proven portal mount isolation, always dropping back to the runner UID.
set -euo pipefail
export XDG_CURRENT_DESKTOP=GNOME
export LEAF35_APP_ID=${1:?Missing app ID}
export LEAF35_BINARY=${2:?Missing binary}
output=${3:?Missing evidence folder}
root=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$output" "$HOME/.var/app/$LEAF35_APP_ID/cache"
output=$(realpath "$output")
export LEAF35_NATIVE_SOURCE=${LEAF35_NATIVE_SOURCE:-$root/debug-native/wordcraft-gnome}
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT
cat > "$temporary/installed.sh" <<'RUNNER'
#!/usr/bin/env bash
set -euo pipefail
if [[ -n ${LEAF35_TEST_UID:-} ]]; then test "$(id -u)" = "$LEAF35_TEST_UID"; fi
exec flatpak run --socket=x11 --nosocket=wayland \
  --env=LC_ALL=en_US.UTF-8 --env=LANGUAGE="${LANGUAGE:-en}" \
  --env=GDK_SCALE="${GDK_SCALE:-1}" --env=GSK_RENDERER=cairo \
  --env=LEAF29_ACK_DIR="${LEAF29_ACK_DIR:-}" --env=LEAF35_PROFILE="$LEAF35_PROFILE" --command=sh "$LEAF35_APP_ID" \
  -c 'export XDG_CONFIG_HOME="$LEAF35_PROFILE/config" XDG_STATE_HOME="$LEAF35_PROFILE/state"; exec "$@"' \
  leaf "$LEAF35_BINARY" "$@"
RUNNER
cat > "$temporary/check.sh" <<'CHECK'
#!/usr/bin/env bash
set -euo pipefail
root=$1;runner=$2;output=$3
cd "$output"
export LEAF29_ACK_PARENT="$HOME/.var/app/$LEAF35_APP_ID/cache"
bash "$root/tools/workflows38-pointer-test.sh" "$runner"
bash "$root/tools/workflows37-pointer-test.sh" "$runner"
"$runner" --workflows35-ui-test > "workflows35-ui-${LANGUAGE}.log" 2>&1 || { cat "workflows35-ui-${LANGUAGE}.log"; exit 1; }
grep -q WORKFLOWS35_UI_OK "workflows35-ui-${LANGUAGE}.log"
cat "workflows35-ui-${LANGUAGE}.log"
bash "$root/tools/workflows35-pointer-test.sh" "$runner"
"$runner" --workflows36-ui-test > "workflows36-ui-${LANGUAGE}.log" 2>&1 || { cat "workflows36-ui-${LANGUAGE}.log"; exit 1; }
grep -q WORKFLOWS36_UI_OK "workflows36-ui-${LANGUAGE}.log"
cat "workflows36-ui-${LANGUAGE}.log"
bash "$root/tools/workflows36-pointer-test.sh" "$runner"
bash "$root/tools/workflows36-template-folder-test.sh" "$runner"
CHECK
chmod +x "$temporary/installed.sh"
if [[ ${CI:-} == true ]]; then export LEAF35_TEST_UID; LEAF35_TEST_UID=$(id -u); fi
for language in en es; do
  scale=1; if [[ "$language" == es ]]; then scale=2;fi
  export LEAF35_PROFILE
  LEAF35_PROFILE=$(mktemp -d "$HOME/.var/app/$LEAF35_APP_ID/cache/workflows35.XXXXXX")
  session=()
  if [[ ${CI:-} == true ]]; then
    runtime=$(mktemp -d "${RUNNER_TEMP:-/tmp}/leaf35-runtime.XXXXXX")
    session=(sudo -E unshare --mount --propagation private --
      bash -euc 'mount --bind "$1" "$2"; shift 2; exec setpriv "$@"'
      leaf35-mount "$runtime" "${XDG_RUNTIME_DIR:?Missing runtime directory}"
      --reuid="$LEAF35_TEST_UID" --regid="$(id -g)" --init-groups --)
  fi
  LANGUAGE=$language GDK_SCALE=$scale xvfb-run -a -s '-screen 0 2800x2200x24' \
    "${session[@]}" dbus-run-session -- timeout 600s bash "$temporary/check.sh" "$root" "$temporary/installed.sh" "$output"
done
echo WORKFLOWS35_INSTALLED_OK
