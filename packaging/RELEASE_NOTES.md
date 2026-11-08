## Leaf 0.1.0-alpha.37 — Smoother navigation and integrated dialogs

- The personal template gallery scrolls together with the blank page. Starting to type dismisses it once and reclaims the space.
- Sidebar representation, filter and mode controls scroll with their content. Their background appears on hover, while keyboard focus and selected state remain accessible.
- Preferences and document tools use integrated adaptive dialogs that fit their parent window. File, folder and print choosers keep their native system workflows.
- Structure restores manual browsing after GTK allocation, cancels obsolete restoration on input and preserves independent sidebar positions. Modal dialogs keep document shortcuts out of their entries and restore editing focus.
- First Save suggests the document title, first heading or a localized generic name. Copy saves retain a localized copy suffix, with bounded Unicode filenames and the required document format.

WordCraft remains at 0.16.0. Existing engine-dependent issues and the deferred issue #52 are unchanged.

The Flatpak continues to use the read-only system Templates folder; custom template folder selection remains unavailable (#171).

## Leaf 0.1.0-alpha.37 — Navegación más fluida y diálogos integrados

- La galería de plantillas personales se desplaza junto con la página en blanco. Al empezar a escribir desaparece una sola vez y recupera ese espacio.
- Los selectores de vista, filtros y modos del panel lateral se desplazan con su contenido. El fondo aparece al pasar el puntero, conservando el foco de teclado y el estado seleccionado accesible.
- Las preferencias y las herramientas del documento usan diálogos adaptativos integrados en su ventana. Los selectores de archivos, carpetas e impresión conservan su funcionamiento nativo.
- Estructura recupera la posición de exploración tras estabilizarse el tamaño del panel y cancela las restauraciones pendientes si intervienes. Los diálogos conservan el foco de edición y evitan que sus atajos actúen sobre el documento.
- El primer guardado propone el título del documento, el primer encabezado o un nombre genérico traducido. Las copias conservan un sufijo traducido, con nombres Unicode de longitud limitada y el formato requerido.

WordCraft sigue en 0.16.0. Se mantienen los límites de los issues dependientes del motor y el aplazamiento del #52.

El Flatpak sigue usando la carpeta de plantillas del sistema en modo de solo lectura; la elección de carpetas personalizadas permanece desactivada (#171).
