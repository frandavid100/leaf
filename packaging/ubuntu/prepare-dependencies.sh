#!/usr/bin/env bash
# Download only: never install Ubuntu 26.10 packages on the build runner.
set -euo pipefail
root=$(realpath deb-dependencies)
mkdir -p "$root/lists/partial" "$root/cache/partial"
test -s "$root/status"
cat > "$root/sources.list" <<'EOF'
deb [signed-by=/usr/share/keyrings/ubuntu-archive-keyring.gpg] https://archive.ubuntu.com/ubuntu stonking main universe
deb [signed-by=/usr/share/keyrings/ubuntu-archive-keyring.gpg] https://archive.ubuntu.com/ubuntu stonking-updates main universe
deb [signed-by=/usr/share/keyrings/ubuntu-archive-keyring.gpg] https://security.ubuntu.com/ubuntu stonking-security main universe
EOF
opts=(
  -o "Dir::State::status=$root/status"
  -o "Dir::State::lists=$root/lists"
  -o "Dir::Cache::archives=$root/cache"
  -o "Dir::Etc::sourcelist=$root/sources.list"
  -o Dir::Etc::sourceparts=-
  -o APT::Sandbox::User=root
  -o Acquire::Retries=1
  -o Acquire::https::Timeout=30
)
apt-get "${opts[@]}" -o APT::Update::Error-Mode=any update
apt-get "${opts[@]}" --download-only --no-install-recommends -y install \
  python3 dpkg-dev libgtk-4-1 libadwaita-1-0 librsvg2-common shared-mime-info \
  desktop-file-utils xvfb xauth dbus-x11 fonts-dejavu-core fonts-liberation \
  locales ca-certificates
