#!/usr/bin/env bash
# Run only inside the disposable Ubuntu 26.10 container, never on a user host.
set -euo pipefail
. /etc/os-release
test "$ID" = ubuntu && test "$VERSION_ID" = 26.10
export DEBIAN_FRONTEND=noninteractive
mkdir -p /etc/ssl/certs
cp /tmp/ubuntu-ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
sed -i 's|http://|https://|g' /etc/apt/sources.list.d/ubuntu.sources
apt-get -o APT::Update::Error-Mode=any -o Acquire::Retries=1 update
apt-get install -y --no-install-recommends python3 dpkg-dev libgtk-4-1 libadwaita-1-0 \
  librsvg2-common shared-mime-info desktop-file-utils xvfb xauth dbus-x11 \
  fonts-dejavu-core fonts-liberation locales ca-certificates
localedef -i en_US -f UTF-8 en_US.UTF-8
export LC_ALL=en_US.UTF-8
mkdir -p dist/ubuntu
bash packaging/ubuntu/build-deb.sh deb-input dist/ubuntu
deb="dist/ubuntu/$(cat dist/ubuntu/ubuntu-package-name.txt)"
apt-get install -y --no-install-recommends "$(realpath "$deb")"
desktop-file-validate /usr/share/applications/io.github.frandavid100.leaf.desktop
ldd /usr/bin/leaf | tee dist/ubuntu/ubuntu-ldd.txt
! grep -q 'not found' dist/ubuntu/ubuntu-ldd.txt
dpkg-query -W leaf libgtk-4-1 libadwaita-1-0 libc6 > dist/ubuntu/ubuntu-dependencies.txt
test "$(readlink -f /usr/bin/leaf)" = /opt/leaf/bin/leaf
test "$(sha256sum /usr/bin/leaf | cut -d' ' -f1)" = "$(cat dist/ubuntu/ubuntu-binary-sha256.txt)"
useradd --create-home leaf-test
mkdir -p /home/leaf-test/.config /home/leaf-test/.cache /home/leaf-test/.local/state
chown -R leaf-test:leaf-test /home/leaf-test
for language in en es; do
  runuser -u leaf-test -- env HOME=/home/leaf-test LC_ALL=en_US.UTF-8 LANGUAGE="$language" \
    GSK_RENDERER=cairo xvfb-run -a -s '-screen 0 1600x1200x24' dbus-run-session -- \
    timeout 120s /usr/bin/leaf --localization-ui-test 2>&1 | tee "dist/ubuntu/ubuntu-locale-$language.log"
  grep -q LOCALIZATION_UI_OK "dist/ubuntu/ubuntu-locale-$language.log"
done
# The existing package diagnostic is platform-neutral except for its Windows
# storage assertions. On Linux it checks relative resources, SVG decoding,
# dictionaries, DOCX/ODT save/reopen/overwrite, Unicode paths, PDF and settings.
runuser -u leaf-test -- env HOME=/home/leaf-test LC_ALL=en_US.UTF-8 LANGUAGE=en \
  GSK_RENDERER=cairo xvfb-run -a dbus-run-session -- \
  timeout 120s /usr/bin/leaf --windows-package-test 2>&1 | tee dist/ubuntu/ubuntu-resources.log
grep -q WINDOWS_PACKAGE_RESOURCES_OK dist/ubuntu/ubuntu-resources.log
test -s /home/leaf-test/.config/wordcraft-gnome/preferences.json
cp /home/leaf-test/.config/wordcraft-gnome/preferences.json /tmp/leaf-preferences-before
apt-get remove -y leaf
test ! -e /usr/bin/leaf && test ! -e /opt/leaf/bin/leaf
cmp /tmp/leaf-preferences-before /home/leaf-test/.config/wordcraft-gnome/preferences.json
apt-get install -y --no-install-recommends "$(realpath "$deb")"
cmp /tmp/leaf-preferences-before /home/leaf-test/.config/wordcraft-gnome/preferences.json
test "$(sha256sum /usr/bin/leaf | cut -d' ' -f1)" = "$(cat dist/ubuntu/ubuntu-binary-sha256.txt)"
echo UBUNTU_26_10_DEB_INSTALLED_TEST_OK
