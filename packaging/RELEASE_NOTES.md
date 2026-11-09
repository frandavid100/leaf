## Leaf 0.1.0-alpha.41 — Personal formatting tools and readable web paste

- Choose and reorder Paragraph, Text and Elements buttons in Preferences. Changes apply to open windows and survive restarting; other commands stay in each group’s overflow menu. Start uses compact paper previews and two-line template names without widening the sidebar. Live recovery snapshots remain protected and are excluded from recovery choices; superseded scans cannot restore stale rows.
- Direct formatting pickers open above their own buttons. Overflow menus omit visible controls, List keeps bullets and numbering together, and insertion actions stay in a fixed footer.
- Toggle buttons, property pickers and toggles with options retain their behavior in the toolbar and overflow. Text Color opens its picker; reapplying the last color is available inside it.
- Pasted Chrome web text adapts its predominant neutral theme colors for readability while retaining local colors, highlights and formatting. Firefox, Undo/Redo and document roundtrips are covered by regression tests.
- Update to WordCraft 0.17, including upstream fixes for moving floating pictures across manual page breaks and preserving a following heading when deleting an empty paragraph. Replacing a picture retains its width and uses the replacement’s proportions.

Discarding a recovery copy is permanent and requires confirmation. No Undo is offered for that deletion.

WordCraft remains at 0.16.0. Engine-dependent issues, the deferred issue #52, custom template-folder selection in the sandbox (#171), FlatPark publication and the Chrome paste-color investigation (#217) remain separate.

## Leaf 0.1.0-alpha.41 — Herramientas de formato personales y pegado web legible

- Elige y ordena los botones de Párrafo, Texto y Elementos en Preferencias. Los cambios se aplican a las ventanas abiertas y se conservan al reiniciar; las demás acciones siguen en el menú de cada grupo. Inicio usa vistas de papel compactas y nombres de plantilla de dos líneas sin ensanchar el panel lateral. Las copias del trabajo abierto se conservan protegidas y quedan fuera de la lista de recuperación; las consultas sustituidas no pueden mostrar filas obsoletas.
- Los selectores directos se abren sobre su botón. Los menús omiten los controles visibles, Lista reúne viñetas y numeración, y las acciones de inserción permanecen en un pie fijo.
- Los botones de activación, los selectores y los botones con opciones mantienen su comportamiento en la barra y en el menú. Color del texto abre su selector; dentro se puede volver a aplicar el último color.
- El texto web pegado desde Chrome adapta los colores neutros predominantes del tema para resultar legible y conserva los colores locales, el resaltado y el formato. Las regresiones comprueban Firefox, Deshacer/Rehacer y el guardado y reapertura.
- Actualización a WordCraft 0.17, con las correcciones del motor para mover imágenes flotantes entre páginas tras un salto manual y conservar el título siguiente al borrar un párrafo vacío. Sustituir una imagen conserva su anchura y usa las proporciones de la nueva.

Descartar una copia de recuperación es permanente y exige confirmación. Ese borrado no ofrece Deshacer.

WordCraft sigue en 0.16.0. Se mantienen por separado los issues del motor, el aplazamiento del #52, la elección de carpetas de plantillas en el sandbox (#171), la publicación en FlatPark y la investigación de los colores pegados desde Chrome (#217).
