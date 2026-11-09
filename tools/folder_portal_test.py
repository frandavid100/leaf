#!/usr/bin/python3
"""Test-only OpenURI portal: inspect GTK's real directory FD without launching a file manager."""
import os
from pathlib import Path
import sys
from gi.repository import Gio, GLib

expected, ack, mode = Path(sys.argv[1]).resolve(), Path(sys.argv[2]), sys.argv[3]
interface = Gio.DBusNodeInfo.new_for_xml('''<node><interface name="org.freedesktop.portal.OpenURI">
<method name="OpenFile"><arg type="s" direction="in"/><arg type="h" direction="in"/><arg type="a{sv}" direction="in"/><arg type="o" direction="out"/></method>
<method name="OpenDirectory"><arg type="s" direction="in"/><arg type="h" direction="in"/><arg type="a{sv}" direction="in"/><arg type="o" direction="out"/></method>
<property name="version" type="u" access="read"/>
</interface></node>''').interfaces[0]
connection = Gio.bus_get_sync(Gio.BusType.SESSION, None)
calls = 0

def method(conn, sender, path, iface, name, params, invocation):
    global calls
    calls += 1
    assert calls == 1, "duplicate folder launch"
    assert name == "OpenFile", "OpenDirectory would open the folder's parent"
    _, descriptor, options = params.unpack()
    fd = invocation.get_message().get_unix_fd_list().get(descriptor)
    try:
        actual = Path(os.readlink(f"/proc/self/fd/{fd}")).resolve()
        assert actual == expected, (actual, expected)
        assert actual.is_dir()
        assert not options.get("writable", False), "folder launch requested write access"
    finally:
        os.close(fd)
    token = options["handle_token"]
    request = "/org/freedesktop/portal/desktop/request/" + sender[1:].replace(".", "_") + "/" + token
    invocation.return_value(GLib.Variant("(o)", (request,)))
    def respond():
        conn.emit_signal(sender, request, "org.freedesktop.portal.Request", "Response", GLib.Variant("(ua{sv})", (0 if mode == "success" else 2, {})))
        ack.write_text(str(actual))
        print("FOLDER_PORTAL_OK " + mode + " " + str(actual), flush=True)
        return False
    GLib.timeout_add(100, respond)

connection.register_object("/org/freedesktop/portal/desktop", interface, method, lambda *args: GLib.Variant("u", 3), None)
Gio.bus_own_name_on_connection(connection, "org.freedesktop.portal.Desktop", Gio.BusNameOwnerFlags.NONE, lambda *args: print("FOLDER_PORTAL_READY", flush=True), None)
GLib.MainLoop().run()
