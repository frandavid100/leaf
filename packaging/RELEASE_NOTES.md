## Leaf 0.1.0-alpha.38 — Recovery in the start view and smoother transitions

- Navigation and Review stay fixed at the bottom of every sidebar view, including Search; upper selectors and filters keep scrolling with their content.
- New blank windows and tabs show labeled template and pending-recovery rows only when they have items. The editable blank page is always available; the separate recovery menu and picker are removed.
- Choose a pending recovery directly from its row. It opens as an unsaved document, while late results cannot replace newer edits or steal focus from another tab.
- Starting to type gently retracts the start rows upward and moves the page into the reclaimed space. Typing, paste and input methods remain responsive; reduced-animation settings are respected.

WordCraft remains at 0.16.0. Existing engine-dependent issues and the deferred issue #52 are unchanged.

The Flatpak uses the read-only system Templates folder; custom template folder selection remains unavailable (#171).

## Leaf 0.1.0-alpha.38 — Recuperación en la vista inicial y transiciones más suaves

- Navegación y Revisión permanecen fijas al pie de todas las vistas del panel lateral, incluida Búsqueda; los selectores y filtros superiores siguen desplazándose con el contenido.
- Las ventanas y pestañas nuevas muestran filas con título para plantillas y recuperaciones solo cuando contienen elementos. El folio editable siempre está disponible; se eliminan el menú y el selector separado de recuperación.
- Elige una recuperación directamente desde su fila. Se abre como documento con cambios por guardar, sin que una respuesta tardía sustituya ediciones posteriores ni quite el foco a otra pestaña.
- Al empezar a escribir, las filas iniciales se retiran suavemente hacia arriba y el folio ocupa ese espacio. La escritura, el pegado y los métodos de entrada siguen respondiendo; se respeta la preferencia del sistema sobre animaciones.

WordCraft sigue en 0.16.0. Se mantienen los límites de los issues dependientes del motor y el aplazamiento del #52.

El Flatpak usa la carpeta de plantillas del sistema en modo de solo lectura; la elección de carpetas personalizadas sigue desactivada (#171).
