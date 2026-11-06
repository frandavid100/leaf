#!/usr/bin/env bash
set -euo pipefail
runner=${1:-./target/release/wordcraft-gnome}
cmd=("$runner"); [[ "$runner" != *.sh ]] || cmd=(bash "$runner")
log=alpha31-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
"${cmd[@]}" --alpha31-ui-test >"$log" 2>&1 &
pid=$!
trap 'scrot "${log%.log}-failure.png" 2>/dev/null || true; kill "$pid" 2>/dev/null || true' EXIT
for attempt in $(seq 1 600); do
  grep -q '^ALPHA31_REVIEW_READY' "$log" && break
  kill -0 "$pid" || { cat "$log"; exit 1; }
  sleep 0.05
done
grep -q '^ALPHA31_REVIEW_READY' "$log"
sleep 0.3
scrot "${log%.log}-review.png"
wait "$pid" || { cat "$log"; exit 1; }
grep -q ALPHA31_UI_OK "$log"
trap - EXIT
