#!/usr/bin/env bash
# Fetch the exact Language data revision used by the pinned WordCraft release.
set -euo pipefail
leaf_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
leaf_revision=$(cat "$leaf_root/craft-i18n.rev")
[[ "$leaf_revision" =~ ^[0-9a-f]{40}$ ]]
leaf_data="$leaf_root/target/language-data"
if [[ ! -d "$leaf_data/.git" ]]; then
  mkdir -p "$leaf_data"
  git -C "$leaf_data" init --quiet
fi
if ! git -C "$leaf_data" cat-file -e "$leaf_revision^{commit}" 2>/dev/null; then
  git -C "$leaf_data" fetch --depth=1 https://github.com/storytold/craft-i18n "$leaf_revision"
fi
git -C "$leaf_data" checkout --quiet --detach "$leaf_revision"
[[ "$(git -C "$leaf_data" rev-parse HEAD)" == "$leaf_revision" ]]
test -d "$leaf_data/language-data"
if [[ -n "${GITHUB_ENV:-}" ]]; then
  printf 'CRAFT_I18N_DIR=%s\nCRAFT_I18N_REQUIRED=1\n' "$leaf_data" >> "$GITHUB_ENV"
fi
printf 'Language data ready: %s\n' "$leaf_data"
printf 'Local builds: export CRAFT_I18N_DIR="%s" CRAFT_I18N_REQUIRED=1\n' "$leaf_data"
