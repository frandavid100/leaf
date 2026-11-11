#!/usr/bin/env python3
"""Exercise the real system font viewer's accessible Install button in CI."""
import time
import gi
gi.require_version('Atspi', '2.0')
from gi.repository import Atspi


def descendants(root):
    todo = [root]
    for _ in range(4000):
        if not todo:
            return
        node = todo.pop()
        yield node
        try:
            todo.extend(node.get_child_at_index(i) for i in range(min(node.get_child_count(), 300)))
        except Exception:
            pass


deadline = time.monotonic() + 60
while time.monotonic() < deadline:
    desktop = Atspi.get_desktop(0)
    for i in range(desktop.get_child_count()):
        app = desktop.get_child_at_index(i)
        try:
            name = app.get_name().lower()
            viewer = 'font' in name or 'tipograf' in name
            portal = 'portal' in name
            if not (viewer or portal):
                continue
            for node in descendants(app):
                if node.get_role() != Atspi.Role.PUSH_BUTTON:
                    continue
                label = node.get_name()
                wanted = label in (('Install', 'Instalar') if viewer else ('Open', 'Abrir'))
                if wanted and node.get_state_set().contains(Atspi.StateType.SENSITIVE):
                    action = node.get_action_iface()
                    if action and action.do_action(0):
                        if viewer:
                            print('FONT279_VIEWER_INSTALL_ACTIVATED', flush=True)
                            raise SystemExit(0)
                        print('FONT279_PORTAL_CHOOSER_ACCEPTED', flush=True)
                        time.sleep(1)
                        break
        except SystemExit:
            raise
        except Exception:
            pass
    time.sleep(0.2)
raise SystemExit('The system font viewer did not expose an enabled Install button')
