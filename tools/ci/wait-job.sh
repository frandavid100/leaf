#!/usr/bin/env bash
set -euo pipefail
name=${1:?Missing job name}
for attempt in $(seq 1 120); do
  state=$(gh api "repos/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID/jobs?per_page=100" --jq ".jobs[] | select(.name == \"$name\") | .status + \":\" + (.conclusion // \"pending\")")
  case "$state" in
    completed:success) exit 0 ;;
    completed:*) echo "$name ended with $state" >&2; exit 1 ;;
  esac
  sleep 15
done
echo "$name did not finish within 30 minutes." >&2
exit 1
