#!/usr/bin/env python3
"""Real AT-SPI requests, keyboard events and unmodified Orca speech transcripts.

Run under xvfb-run/dbus-run-session with system python3 + pyatspi. Only generated
test text is used. Orca's own debug output records generated announcements; this
does not claim a human listening test or a physical braille display test.
"""
import json
import os
import pty
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import time
import traceback

import gi
gi.require_version("Atspi", "2.0")
from gi.repository import GLib, Atspi
import pyatspi

OUT = Path("a11y-results").resolve()
FIXTURE = "First paragraph for Orca.\nCafé, español y 😀.\nFinal paragraph."
events = []


def spin():
    context = GLib.MainContext.default()
    while context.pending():
        context.iteration(False)


def wait(predicate, seconds=15):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        spin()
        try:
            result = predicate()
            if result:
                return result
        except Exception:
            pass
        time.sleep(.05)
    raise RuntimeError("Timed out waiting for assistive technology or application")


def record(event):
    try:
        if event.source.name == "Document Content":
            events.append({"type": event.type, "detail1": event.detail1, "detail2": event.detail2, "text": str(event.any_data)})
    except Exception:
        pass


def find_named(name):
    for app in pyatspi.Registry.getDesktop(0):
        try:
            app.set_cache_mask(Atspi.Cache.NONE)
            found = pyatspi.findDescendant(app, lambda obj: obj.name == name)
            if found is not None:
                return found
        except Exception:
            continue


def find_canvas():
    return find_named("Document Content")


def dump_tree():
    nodes = []
    def visit(obj, depth):
        if depth > 40 or len(nodes) > 500:
            return
        try:
            if obj.getRole() == pyatspi.ROLE_APPLICATION:
                obj.set_cache_mask(Atspi.Cache.NONE)
            nodes.append({"depth": depth, "name": obj.name, "role": obj.getRoleName(), "interfaces": list(pyatspi.listInterfaces(obj))})
            for child in obj:
                visit(child, depth+1)
        except Exception as error:
            nodes.append({"depth": depth, "error": str(error)})
    visit(pyatspi.Registry.getDesktop(0), 0)
    (OUT / "tree.json").write_text(json.dumps(nodes, indent=2, ensure_ascii=False))


def command(*args):
    return subprocess.run(list(map(str, args)), check=True, text=True, capture_output=True, timeout=20).stdout


def main():
    OUT.mkdir(exist_ok=True)
    baseline = "--baseline" in sys.argv
    sandbox = "--sandbox" in sys.argv
    binary = Path(sys.argv[1]).resolve()
    for kind in ["object:children-changed", "object:text-changed", "object:text-caret-moved", "object:text-selection-changed", "object:state-changed:focused"]:
        pyatspi.Registry.registerEventListener(record, kind)
    with tempfile.TemporaryDirectory(prefix="leaf-a11y-") as profile:
        env = os.environ.copy()
        env.update(XDG_CONFIG_HOME=profile, XDG_CACHE_HOME=profile, LEAF_CONFIG_HOME=profile)
        # A bare Xvfb has no window activation protocol; use a real WM so Orca
        # receives the same active-window/focus sequence as on a desktop.
        wm_out = (OUT / "window-manager.log").open("w")
        wm = subprocess.Popen(["openbox", "--sm-disable"], env=env, stdout=wm_out, stderr=subprocess.STDOUT)
        wait(lambda: command("xdotool", "get_desktop").strip(), 15)
        command("gsettings", "set", "org.gnome.desktop.interface", "toolkit-accessibility", "true")
        orca_log = OUT / "orca-debug.log"
        def speech_lines():
            return [line for line in orca_log.read_text(errors="replace").splitlines() if "SPEECH OUTPUT:" in line]
        orca_out = (OUT / "orca-process.log").open("w")
        # Orca buffers --debug-file on a regular file. A real terminal makes
        # its unmodified logger line-buffered, so we can await actual utterances
        # rather than racing a buffered log and interrupting speech with the
        # next keystroke. No speech generator or AT event is mocked.
        master, slave = pty.openpty()
        def capture():
            with orca_log.open("wb", buffering=0) as output:
                while True:
                    try:
                        chunk = os.read(master, 65536)
                        if not chunk:
                            break
                        output.write(chunk)
                    except OSError:
                        break
        collector = threading.Thread(target=capture, daemon=True)
        collector.start()
        orca = subprocess.Popen(["orca", "--replace", "--enable=speech,echo-by-character", "--disable=braille", "--user-prefs=" + profile + "/orca", "--debug-file=/dev/stdout"], env=env, stdout=slave, stderr=orca_out)
        os.close(slave)
        app = None
        try:
            wait(lambda: orca_log.exists() and "ORCA: Initialized" in orca_log.read_text(errors="replace"), 30)
            with (OUT / "leaf.log").open("w") as log:
                finish = Path("a11y-finish")
                launch = [str(binary), "--test-document" if baseline else "--a11y-ui-test"]
                if sandbox:
                    app_id = "io.github.frandavid100.leaf"
                    data = Path(command("flatpak", "run", "--command=sh", app_id, "-c", 'printf %s "$XDG_DATA_HOME"'))
                    assert data.is_relative_to(Path.home() / ".var/app" / app_id), data
                    finish = data / "a11y-finish"
                    launch = ["flatpak", "run", "--socket=x11", "--nosocket=wayland", "--env=GTK_A11Y=atspi", "--env=LANGUAGE=en", "--command=sh", app_id, "-c", 'mkdir -p "$XDG_DATA_HOME"; cd "$XDG_DATA_HOME"; exec /app/bin/leaf --a11y-ui-test']
                finish.unlink(missing_ok=True)
                app = subprocess.Popen(launch, env=env, stdout=log, stderr=subprocess.STDOUT)
                if not baseline:
                    wait(lambda: "ACCESSIBLE_DOCUMENT_READY" in (OUT / "leaf.log").read_text(), 25)
                else:
                    # GtkFixed's original generic canvas is omitted from the
                    # exported tree altogether. Prove the client is working by
                    # finding and focusing a native control, then record absence.
                    menu = wait(lambda: find_named("Main Menu"), 25)
                    dump_tree()
                    canvas = find_canvas()
                    window = wait(lambda: command("xdotool", "search", "--onlyvisible", "--pid", app.pid).splitlines()[0])
                    command("xdotool", "windowactivate", "--sync", window)
                    command("xdotool", "key", "--clearmodifiers", "F10", "Escape")
                    time.sleep(.5)
                    spin()
                    result = {"baseline": True, "orca_version": command("orca", "--version").strip(), "native_control_exposed": True, "canvas_exposed": canvas is not None, "text_supported": canvas is not None and "Text" in pyatspi.listInterfaces(canvas), "speech_is_generated_transcript_not_human_listening": True}
                    assert not result["text_supported"]
                    (OUT / "result.json").write_text(json.dumps(result, indent=2) + "\n")
                    (OUT / "orca-speech.txt").write_text("\n".join(line for line in orca_log.read_text(errors="replace").splitlines() if "SPEECH OUTPUT:" in line) + "\n")
                    print("A11Y_BASELINE_NO_TEXT_CONFIRMED", flush=True)
                    return
                try:
                    canvas = wait(find_canvas, 25)
                finally:
                    dump_tree()
                window = wait(lambda: command("xdotool", "search", "--onlyvisible", *( ["--name", "^Leaf$"] if sandbox else ["--pid", app.pid] )).splitlines()[0])
                command("xdotool", "windowactivate", "--sync", window)
                command("xdotool", "key", "--clearmodifiers", "ctrl+Home")
                wait(lambda: canvas.getState().contains(pyatspi.STATE_FOCUSED))
                result = {"baseline": baseline, "orca_version": command("orca", "--version").strip(), "role": canvas.getRoleName(), "interfaces": list(pyatspi.listInterfaces(canvas)), "speech_is_generated_transcript_not_human_listening": True}
                try:
                    text = canvas.queryText()
                    result.update(text_supported=True, text=text.getText(0, -1), count=text.characterCount, caret=text.caretOffset)
                except NotImplementedError:
                    result["text_supported"] = False
                result["sandbox"] = sandbox
                (OUT / "initial.json").write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n")
                if baseline:
                    assert not result["text_supported"], "Baseline unexpectedly exposes Text; inspect before adapting the test"
                    command("xdotool", "key", "--clearmodifiers", "Down", "Right")
                    time.sleep(.5)
                    spin()
                    print("A11Y_BASELINE_NO_TEXT_CONFIRMED", flush=True)
                else:
                    assert result["text_supported"]
                    assert result["text"] == FIXTURE, repr(result["text"])
                    assert result["count"] == len(FIXTURE), (result["count"], len(FIXTURE))
                    wait(lambda: text.caretOffset == 0)
                    command("xdotool", "key", "--clearmodifiers", "Down")
                    second = len(FIXTURE.split("\n")[0]) + 1
                    wait(lambda: text.caretOffset == second)
                    wait(lambda: any("Café" in line for line in speech_lines()))
                    value = Atspi.Text.get_string_at_offset(canvas, second, Atspi.TextGranularity.WORD)
                    word, start, end = value.content, value.start_offset, value.end_offset
                    assert "Café" in word, (word, start, end)
                    command("xdotool", "key", "--delay", "200", "--clearmodifiers", "shift+Right", "shift+Right", "shift+Right", "shift+Right")
                    wait(lambda: text.getNSelections() == 1)
                    a, b = text.getSelection(0)
                    assert (a,b) == (second,second+4), (a,b)
                    wait(lambda: any("selected" in line.lower() for line in speech_lines()))
                    command("xdotool", "type", "--clearmodifiers", "--delay", "150", "X")
                    expected = FIXTURE.replace("Café", "X")
                    wait(lambda: text.getText(0,-1) == expected)
                    wait(lambda: any("SPEECH OUTPUT: 'X'" in line or "SPEECH OUTPUT: 'capital X'" in line for line in speech_lines()))
                    command("xdotool", "key", "--clearmodifiers", "ctrl+z")
                    wait(lambda: text.getText(0,-1) == FIXTURE)
                    command("xdotool", "key", "--clearmodifiers", "ctrl+a")
                    wait(lambda: text.getSelection(0) == (0,len(FIXTURE)))
                    command("xdotool", "key", "--clearmodifiers", "ctrl+Home", "Down", "Down", "Up")
                    wait(lambda: sum("Café" in line for line in speech_lines()) >= 2)
                    result["orca_announcements"] = {"paragraph": True, "selection": True, "typed_character": True, "restored_paragraph": True}
                    command("xdotool", "key", "--clearmodifiers", "ctrl+Home")
                    wait(lambda: text.caretOffset == 0)
                    finish.touch()
                    wait(lambda: app.poll() is not None)
                    assert app.returncode == 0, app.returncode
                    assert any("text-changed:insert" in e["type"] for e in events), events
                    assert any("text-changed:delete" in e["type"] for e in events), events
                    assert any("text-caret-moved" in e["type"] for e in events), events
                    assert any("text-selection-changed" in e["type"] for e in events), events
                    result["keyboard_read_edit_select_undo"] = True
                    print("AT_SPI_DOCUMENT_TEXT_OK", flush=True)
                result["events"] = events
                (OUT / "result.json").write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n")
                speech = [line for line in orca_log.read_text(errors="replace").splitlines() if "SPEECH OUTPUT:" in line]
                (OUT / "orca-speech.txt").write_text("\n".join(speech) + "\n")
                if not baseline:
                    assert any("Café" in line for line in speech), "Orca did not generate paragraph speech; inspect debug log"
                    print("ORCA_DOCUMENT_SPEECH_OK", flush=True)
        finally:
            (OUT / "events.json").write_text(json.dumps(events, indent=2, ensure_ascii=False) + "\n")
            for process in [app, orca, wm]:
                if process is not None and process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
            orca_out.close()
            wm_out.close()
            collector.join(timeout=2)
            os.close(master)


if __name__ == "__main__":
    try:
        main()
    except Exception:
        OUT.mkdir(exist_ok=True)
        (OUT / "ERROR.txt").write_text(traceback.format_exc())
        raise
