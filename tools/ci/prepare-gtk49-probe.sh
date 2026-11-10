#!/usr/bin/env bash
# Run the unchanged format136 probe on Leaf's supported GNOME runtime.
# GTK <=4.16 has GNOME/gtk#6817: gtk_window_native_layout dereferences a
# missing surface. The official fix shipped in GTK 4.17.6; no Leaf shim is used.
set -euo pipefail
leaf_binary=$(realpath "${1:?Missing exact native executable}")
leaf_root=${GITHUB_WORKSPACE:?Missing checkout}
leaf_work="${RUNNER_TEMP:?Missing temporary directory}/leaf-native-gtk49"
leaf_runner="${RUNNER_TEMP}/leaf-native-gtk49-run.sh"
command -v flatpak >/dev/null || sudo env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends flatpak
sudo sysctl -w kernel.apparmor_restrict_unprivileged_userns=0
flatpak --user remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak --user install -y --noninteractive flathub org.gnome.Platform//49
flatpak build-init "$leaf_work" io.github.frandavid100.leaf.NativeProbe org.gnome.Platform org.gnome.Platform 49
cat > "$RUNNER_TEMP/leaf-native-gtk-version.c" <<'C'
#include <gtk/gtk.h>
#include <stdio.h>
int main(void) {
  printf("NATIVE_GTK_RUNTIME %u.%u.%u\n", gtk_get_major_version(), gtk_get_minor_version(), gtk_get_micro_version());
  return gtk_get_major_version() != 4 || gtk_get_minor_version() < 18;
}
C
cc "$RUNNER_TEMP/leaf-native-gtk-version.c" -o "$RUNNER_TEMP/leaf-native-gtk-version" $(pkg-config --cflags --libs gtk4)
flatpak build --filesystem="$RUNNER_TEMP:ro" "$leaf_work" "$RUNNER_TEMP/leaf-native-gtk-version"
flatpak --user info --show-commit org.gnome.Platform//49
cat > "$leaf_runner" <<'RUNNER'
#!/usr/bin/env bash
set -euo pipefail
exec flatpak build --socket=x11 --nosocket=wayland \
  --filesystem="$GITHUB_WORKSPACE:ro" \
  --bind-mount="/leaf-probe-acks=${LEAF29_ACK_DIR:?Missing pointer acknowledgements}" \
  --env=LEAF29_ACK_DIR=/leaf-probe-acks \
  --env=LANGUAGE="${LANGUAGE:-en}" --env=LC_ALL=C.UTF-8 \
  --env=GDK_SCALE="${GDK_SCALE:-1}" --env=GSK_RENDERER=cairo \
  --env=CRAFT_I18N_DIR="$CRAFT_I18N_DIR" --env=CRAFT_I18N_REQUIRED=1 \
  --env=LEAF_GTK49_BINARY="$LEAF_GTK49_BINARY" \
  "$LEAF_GTK49_BUILD" /bin/sh -c \
  'export XDG_CONFIG_HOME=$(mktemp -d) XDG_STATE_HOME=$(mktemp -d); exec "$LEAF_GTK49_BINARY" "$@"' leaf-probe "$@"
RUNNER
chmod 0755 "$leaf_runner"
{
  printf 'LEAF_GTK49_BINARY=%s\n' "$leaf_binary"
  printf 'LEAF_GTK49_BUILD=%s\n' "$leaf_work"
  printf 'LEAF_FORMAT136_RUNNER=%s\n' "$leaf_runner"
} >> "${GITHUB_ENV:?Missing workflow environment}"
