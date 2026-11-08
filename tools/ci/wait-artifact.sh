#!/usr/bin/env bash
# A concurrent producer can fail before uploading; stop promptly in that case.
set -euo pipefail
name=${1:?Missing artifact name}
for attempt in $(seq 1 120); do
  available=$(gh api "repos/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID/artifacts" --jq "[.artifacts[] | select(.name == \"$name\" and .expired == false)] | length")
  if [[ "$available" -gt 0 ]]; then exit 0; fi
  failure=$(gh api "repos/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID/jobs?per_page=100" --jq '[.jobs[] | select(.name == "Build and validate unpublished Alpha candidate" and (.conclusion == "failure" or .conclusion == "cancelled" or .conclusion == "skipped"))] | length')
  if [[ "$failure" -gt 0 ]]; then echo 'Candidate producer did not succeed.' >&2; exit 1; fi
  sleep 15
done
echo 'Candidate archive was not produced within 30 minutes.' >&2
exit 1
