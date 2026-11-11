#!/usr/bin/env bash
# Download only: never install Ubuntu 26.10 packages on the build runner.
set -euo pipefail
root=$(realpath deb-dependencies)
mkdir -p "$root/lists/partial" "$root/cache/partial"
test -s "$root/status"
cat > "$root/sources.list" <<'EOF'
deb [signed-by=/usr/share/keyrings/ubuntu-archive-keyring.gpg] https://mirrors.edge.kernel.org/ubuntu stonking main universe
deb [signed-by=/usr/share/keyrings/ubuntu-archive-keyring.gpg] https://mirrors.edge.kernel.org/ubuntu stonking-updates main universe
deb [signed-by=/usr/share/keyrings/ubuntu-archive-keyring.gpg] https://mirrors.edge.kernel.org/ubuntu stonking-security main universe
EOF
opts=(
  -o "Dir::State::status=$root/status"
  -o "Dir::State::lists=$root/lists"
  -o "Dir::Cache::archives=$root/cache"
  -o "Dir::Cache::pkgcache=$root/pkgcache.bin"
  -o "Dir::Cache::srcpkgcache=$root/srcpkgcache.bin"
  -o "Dir::Etc::sourcelist=$root/sources.list"
  -o Dir::Etc::sourceparts=-
  -o APT::Sandbox::User=root
  -o Acquire::Retries=1
  -o Acquire::https::Timeout=30
)
apt-get "${opts[@]}" -o APT::Update::Error-Mode=any update
mapfile -t packages < packaging/ubuntu/test-packages.txt
apt-get "${opts[@]}" --download-only --no-install-recommends -y install "${packages[@]}"
