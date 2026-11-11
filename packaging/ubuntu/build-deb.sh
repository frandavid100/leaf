#!/usr/bin/env bash
# Repackage an authenticated, already tested upstream application tree unchanged.
set -euo pipefail
input=${1:?Usage: build-deb.sh RELEASE_INPUT OUTPUT}
out=${2:?Usage: build-deb.sh RELEASE_INPUT OUTPUT}
input=$(realpath "$input")
mkdir -p "$out"
out=$(realpath "$out")
version=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "$input/candidate.json")
[[ $version =~ ^0\.1\.0-alpha\.[0-9]+$ ]]
archive="Leaf-$version-flatpark-x86_64.tar.xz"
(cd "$input"; sha256sum -c SHA256SUMS --ignore-missing)
test -s "$input/$archive"
test "$(dpkg --print-architecture)" = amd64
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
mkdir -p "$stage/root/opt/leaf" "$stage/root/DEBIAN" "$stage/debian"
tar --extract --xz --file "$input/$archive" --directory "$stage/root/opt/leaf" --no-same-owner
binary="$stage/root/opt/leaf/bin/leaf"
test -x "$binary"
"$binary" --version | grep -Fx "Leaf $version"
id=io.github.frandavid100.leaf
install -Dm644 "$stage/root/opt/leaf/share/applications/$id.desktop" "$stage/root/usr/share/applications/$id.desktop"
install -Dm644 "$stage/root/opt/leaf/share/metainfo/$id.metainfo.xml" "$stage/root/usr/share/metainfo/$id.metainfo.xml"
for dir in scalable/apps symbolic/apps; do
  for icon in "$stage/root/opt/leaf/share/icons/hicolor/$dir/"*; do
    install -Dm644 "$icon" "$stage/root/usr/share/icons/hicolor/$dir/$(basename "$icon")"
  done
done
mkdir -p "$stage/root/usr/bin" "$stage/root/usr/share/doc/leaf"
ln -s /opt/leaf/bin/leaf "$stage/root/usr/bin/leaf"
cp "$input/candidate.json" "$stage/root/usr/share/doc/leaf/upstream-candidate.json"
cp "$input/source-commit.txt" "$stage/root/usr/share/doc/leaf/source-commit.txt"
cp "$input/release.json" "$stage/root/usr/share/doc/leaf/upstream-release.json"
cp "$input/packaging-commit.txt" "$stage/root/usr/share/doc/leaf/packaging-commit.txt"
cat > "$stage/root/usr/share/doc/leaf/copyright" <<'EOF'
Leaf contributors — GPL-3.0-or-later
Source: https://github.com/frandavid100/leaf
The unchanged upstream binary, translations, dictionaries, icons and notices
are installed under /opt/leaf. Full license texts and third-party notices:
/opt/leaf/share/licenses and /opt/leaf/share/doc/leaf/AI_ASSISTANCE.md.
This experimental package was prepared with AI assistance.
EOF
# Derive ABI dependencies from the installed Ubuntu libraries, not a guessed list.
cat > "$stage/debian/control" <<'EOF'
Source: leaf
Section: editors
Priority: optional
Maintainer: David Prieto <frandavid100@gmail.com>

Package: leaf
Architecture: amd64
Description: Native GTK document editor
EOF
depends=$(cd "$stage"; dpkg-shlibdeps -O -e"$binary" | sed -n 's/^shlibs:Depends=//p')
test -n "$depends"
deb_version="${version/-alpha./\~alpha.}-1ubuntu26.10.1"
cat > "$stage/root/DEBIAN/control" <<EOF
Package: leaf
Version: $deb_version
Section: editors
Priority: optional
Architecture: amd64
Maintainer: David Prieto <frandavid100@gmail.com>
Depends: $depends, librsvg2-common, shared-mime-info
Recommends: fonts-dejavu-core, fonts-liberation
Installed-Size: $(du -sk "$stage/root" | cut -f1)
Homepage: https://github.com/frandavid100/leaf
Description: Native GTK word processor (experimental Ubuntu package)
 Create and edit DOCX and ODT documents, and export to PDF.
 This package targets Ubuntu 26.10 amd64 and uses system GTK/libadwaita.
 Leaf is in alpha. Test with copies of important documents.
EOF
name="leaf_${deb_version}_amd64.deb"
dpkg-deb --root-owner-group --build "$stage/root" "$out/$name"
dpkg-deb --info "$out/$name"
sha256sum "$binary" | cut -d' ' -f1 > "$out/ubuntu-binary-sha256.txt"
(cd "$out"; sha256sum "$name" > "$name.sha256")
printf '%s\n' "$name" > "$out/ubuntu-package-name.txt"
