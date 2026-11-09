## Leaf 0.1.0-alpha.41 — Clearer start previews and native formatting controls

- Start uses a continuous list of full-size page previews, with recoveries first and templates second, newest first within each kind. Names appear below the previews; the Templates folder button remains fixed in the sidebar footer, including when the list is empty.
- Choose styles from one fixed-width, subtly framed dropdown. Direct formatting pickers belong to their launcher, open without internal page-slide animations and close on a second click. Independent controls have native spacing; underline and strikethrough use native split buttons. Bullet, numbering and Remove List choices have distinct visible icons.
- Choose and reorder Paragraph, Text and Elements buttons in Preferences. Choices apply to open windows and survive restarting; other commands remain in overflow. Insertion actions stay in fixed popup footers. Live recovery snapshots are protected and omitted from recovery choices; obsolete scans cannot restore stale rows.
- Pasted Chrome web text adapts its predominant neutral theme colors for readability while retaining local colors, highlights and formatting. Firefox, Undo/Redo and document roundtrips are covered by regression tests.
- Update to WordCraft 0.17, including upstream fixes for moving floating pictures across manual page breaks and preserving a following heading when deleting an empty paragraph. Replacing a picture retains its width and follows the replacement’s proportions.

Discarding a recovery copy is permanent and requires confirmation. No Undo is offered for that deletion.

WordCraft is 0.17.0. Whole-paragraph deletion can still transfer formatting to the following paragraph (#152); unequal-column continuation (#1) and table footnotes (#99) remain engine issues. The deferred Files thumbnail report (#52), custom read-only template-folder portal support (#171), and final store screenshots/client validation (#185) remain open.

## Leaf 0.1.0-alpha.41 — Miniaturas iniciales más claras y controles de formato nativos

- Inicio muestra una lista continua de miniaturas de página grandes: primero las recuperaciones y después las plantillas, de más reciente a más antigua dentro de cada tipo. Los nombres aparecen debajo; el botón de la carpeta de plantillas permanece fijo al pie del panel, incluso si la lista está vacía.
- Elige estilos desde un único desplegable de anchura fija y fondo suave. Los selectores directos pertenecen a su botón, se abren sin animaciones internas de cambio de página y se cierran al pulsar de nuevo. Los controles independientes tienen separación nativa; subrayado y tachado usan botones divididos nativos. Viñetas, numeración y Quitar lista tienen iconos distintos y visibles.
- Elige y ordena los botones de Párrafo, Texto y Elementos en Preferencias. Los cambios se aplican a las ventanas abiertas y se conservan al reiniciar; las demás acciones siguen en los menús. Las acciones de inserción permanecen en pies fijos. Las copias del trabajo abierto se conservan protegidas y quedan fuera de recuperación; las consultas sustituidas no pueden restaurar filas obsoletas.
- El texto web pegado desde Chrome adapta los colores neutros predominantes del tema para resultar legible y conserva los colores locales, el resaltado y el formato. Las regresiones comprueban Firefox, Deshacer/Rehacer y el guardado y reapertura.
- Actualización a WordCraft 0.17, con las correcciones del motor para mover imágenes flotantes entre páginas tras un salto manual y conservar el título siguiente al borrar un párrafo vacío. Sustituir una imagen conserva su anchura y usa las proporciones de la nueva.

Descartar una copia de recuperación es permanente y exige confirmación. Ese borrado no ofrece Deshacer.

WordCraft es 0.17.0. Borrar un párrafo completo todavía puede transferir su formato al siguiente (#152); siguen pendientes la continuación entre columnas desiguales (#1) y las notas al pie tras tablas (#99). Continúan abiertos las miniaturas de Archivos aplazadas (#52), el portal de carpetas de plantillas personalizadas de solo lectura (#171) y las capturas finales y comprobación en la tienda (#185).
