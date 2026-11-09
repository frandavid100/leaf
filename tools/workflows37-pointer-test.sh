#!/usr/bin/env bash
set -euo pipefail
runner=${1:-./target/release/wordcraft-gnome}
command=("$runner"); [[ "$runner" != *.sh ]] || command=(bash "$runner")
export LEAF29_ACK_DIR
LEAF29_ACK_DIR=$(mktemp -d "${LEAF29_ACK_PARENT:-/tmp}/leaf37-ack.XXXXXX")
log=workflows37-${LANGUAGE:-en}-${GDK_SCALE:-1}.log
"${command[@]}" --workflows37-ui-test > "$log" 2>&1 &
pid=$!
trap 'scrot "${log%.log}-failure.png" 2>/dev/null || true; kill "$pid" 2>/dev/null || true; rm -rf "$LEAF29_ACK_DIR"' EXIT
wait_marker() {
  for _ in $(seq 1 1600);do
    grep -q "^WORKFLOWS37_POINT $1 " "$log" && return
    kill -0 "$pid" 2>/dev/null || { cat "$log";exit 1; }
    sleep .02
  done
  cat "$log";exit 1
}
copy_entry() {
  # GTK transfers clipboard ownership asynchronously. A fixed 100 ms delay can
  # return the previous copy on a busy HiDPI runtime. Require this copy's reply.
  local pending="leaf-entry-copy-pending-$$-$RANDOM" value
  printf '%s' "$pending" | xclip -selection clipboard >/dev/null 2>&1
  xdotool key ctrl+a ctrl+c
  for _ in $(seq 1 150); do
    if value=$(timeout 1s xclip -selection clipboard -o 2>/dev/null) && [[ "$value" != "$pending" ]]; then
      printf '%s' "$value"
      return 0
    fi
    sleep .02
  done
  echo 'Native entry copy did not reach the clipboard' >&2
  return 1
}
for stage in $(seq 0 10);do
  wait_marker "$stage"
  read -r _ _ x y < <(grep "^WORKFLOWS37_POINT $stage " "$log" | tail -n1)
  window=$(xdotool search --onlyvisible --name '^Leaf Workflows37$' | head -n1)
  xdotool windowfocus --sync "$window"
  case "$stage" in
    0)
      xdotool mousemove --window "$window" "$x" "$y" click 1 key ctrl+a
      xdotool type --clearmodifiers 'Leaf QA 37'
      # GTK groups typing by word; verify native Undo/Redo independently of
      # the later shortcut-blocking check, without assuming its group size.
      before=$(copy_entry)
      [[ "$before" == 'Leaf QA 37' ]] || { printf 'Native entry typing mismatch: <%s>\n' "$before";exit 1; }
      xdotool key ctrl+z
      after=$(copy_entry)
      [[ "$before" != "$after" ]] || { echo 'Native entry Undo did not change the field';exit 1; }
      xdotool key ctrl+shift+z
      redo=$(copy_entry)
      [[ "$redo" == "$before" ]] || { printf 'Native entry Redo mismatch: before=<%s> undo=<%s> redo=<%s>\n' "$before" "$after" "$redo";exit 1; }
      echo WORKFLOWS37_ENTRY_UNDO_OK
      xdotool key ctrl+n ctrl+s ctrl+w ctrl+z
      sleep .15
      xdotool key Escape
      ;;
    1)
      xdotool mousemove --window "$window" "$x" "$y" click 1
      sleep .15
      xdotool key Escape
      ;;
    2) xdotool key Escape ;;
    3) xdotool mousemove --window "$window" "$x" "$y" click 1 ;;
    4) xdotool key ctrl+z ;;
    6|7|8|9|10)
      if [[ $stage == 7 ]]; then scrot "${log%.log}-sidebar.png"; fi
      xdotool mousemove --window "$window" "$x" "$y" click 1
      ;;
    5)
      xdotool key ctrl+s
      chooser=
      for _ in $(seq 1 500);do
        chooser=$(xdotool search --onlyvisible --name '^(Save Document|Guardar documento) — DOCX or ODT$|^Guardar documento — DOCX u ODT$' 2>/dev/null | head -n1 || true)
        [[ -z $chooser ]] || break
        kill -0 "$pid" 2>/dev/null || { cat "$log";exit 1; }
        sleep .02
      done
      [[ -n $chooser ]] || { cat "$log";exit 1; }
      xdotool windowfocus --sync "$chooser" key Escape
      ;;
  esac
  touch "$LEAF29_ACK_DIR/workflows37-$stage"
done
wait "$pid" || { cat "$log";exit 1; }
grep -q WORKFLOWS37_UI_OK "$log"
cp "$LEAF29_ACK_DIR"/workflows37-dialog-*.png .
cat "$log"
rm -rf "$LEAF29_ACK_DIR"
trap - EXIT
