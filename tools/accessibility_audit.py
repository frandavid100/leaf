#!/usr/bin/env python3
"""Real AT-SPI requests, keyboard events and unmodified Orca speech transcripts.

Run under xvfb-run/dbus-run-session with system python3 + pyatspi. Only generated
test text is used. Orca's own debug output records generated announcements; this
does not claim a human listening test or a physical braille display test.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import traceback

import gi
gi.require_version("Atspi", "2.0")
from gi.repository import GLib
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


def find_canvas():
    for app in pyatspi.Registry.getDesktop(0):
        try:
            found = pyatspi.findDescendant(app, lambda obj: obj.name == "Document Content")
            if found is not None:
                return found
        except Exception:
            continue


def dump_tree():
    nodes = []
    def visit(obj, depth):
        if depth > 14 or len(nodes) > 500:
            return
        try:
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
    binary = Path(sys.argv[1]).resolve()
    for kind in ["object:text-changed", "object:text-caret-moved", "object:text-selection-changed", "object:state-changed:focused"]:
        pyatspi.Registry.registerEventListener(record, kind)
    with tempfile.TemporaryDirectory(prefix="leaf-a11y-") as profile:
        env = os.environ.copy()
        env.update(XDG_CONFIG_HOME=profile, XDG_CACHE_HOME=profile, LEAF_CONFIG_HOME=profile)
        command("gsettings", "set", "org.gnome.desktop.interface", "toolkit-accessibility", "true")
        orca_log = OUT / "orca-debug.log"
        orca_out = (OUT / "orca-process.log").open("w")
        orca = subprocess.Popen(["orca", "--replace", "--enable=speech,echo-by-character", "--disable=braille", "--user-prefs=" + profile + "/orca", "--debug-file=" + str(orca_log)], env=env, stdout=orca_out, stderr=subprocess.STDOUT)
        app = None
        try:
            wait(lambda: orca_log.exists() and "ORCA: Initialized" in orca_log.read_text(errors="replace"), 30)
            with (OUT / "leaf.log").open("w") as log:
                app = subprocess.Popen([str(binary), "--test-document" if baseline else "--a11y-ui-test"], env=env, stdout=log, stderr=subprocess.STDOUT)
                try:
                    canvas = wait(find_canvas, 25)
                finally:
                    dump_tree()
                window = wait(lambda: command("xdotool", "search", "--onlyvisible", "--pid", app.pid).splitlines()[0])
                command("xdotool", "windowfocus", "--sync", window)
                canvas.queryComponent().grabFocus()
                command("xdotool", "key", "--clearmodifiers", "ctrl+Home")
                wait(lambda: canvas.getState().contains(pyatspi.STATE_FOCUSED))
                result = {"baseline": baseline, "orca_version": command("orca", "--version").strip(), "role": canvas.getRoleName(), "interfaces": list(pyatspi.listInterfaces(canvas)), "speech_is_generated_transcript_not_human_listening": True}
                try:
                    text = canvas.queryText()
                    result.update(text_supported=True, text=text.getText(0, -1), count=text.characterCount, caret=text.caretOffset)
                except NotImplementedError:
                    result["text_supported"] = False
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
                    word, start, end = text.getTextAtOffset(second, pyatspi.TEXT_BOUNDARY_WORD_START)
                    assert "Café" in word, (word, start, end)
                    command("xdotool", "key", "--clearmodifiers", "shift+Right", "shift+Right", "shift+Right", "shift+Right")
                    wait(lambda: text.getNSelections() == 1)
                    a, b = text.getSelection(0)
                    assert (a,b) == (second,second+4), (a,b)
                    command("xdotool", "type", "--clearmodifiers", "--delay", "150", "X")
                    expected = FIXTURE.replace("Café", "X")
                    wait(lambda: text.getText(0,-1) == expected)
                    command("xdotool", "key", "--clearmodifiers", "ctrl+z")
                    wait(lambda: text.getText(0,-1) == FIXTURE)
                    command("xdotool", "key", "--clearmodifiers", "ctrl+a")
                    wait(lambda: text.getSelection(0) == (0,len(FIXTURE)))
                    command("xdotool", "key", "--clearmodifiers", "ctrl+Home", "Down", "Down", "Up")
                    wait(lambda: orca_log.exists() and "Café" in orca_log.read_text(errors="replace"))
                    command("xdotool", "key", "--clearmodifiers", "ctrl+Home")
                    Path("a11y-finish").touch()
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
            for process in [app, orca]:
                if process is not None and process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
            orca_out.close()


if __name__ == "__main__":
    try:
        main()
    except Exception:
        OUT.mkdir(exist_ok=True)
        (OUT / "ERROR.txt").write_text(traceback.format_exc())
        raise
