## Leaf 0.1.0-alpha.46 — Smoother typing in long documents and a clearer codebase

- The Review panel, Outline and page thumbnails now refresh once typing pauses instead of after every keystroke. With Review open in a long document, the work done between keystrokes dropped to about a seventh.
- Fields, cross-references and the table of contents are recomputed on a background thread after a typing pause. In a 131-page document with a table of contents, the longest freeze went from about 460 ms to about 50 ms.
- Autosave no longer rereads preferences from disk or rechecks ODT documents every second while nothing changes.
- Diagnostic checks are gathered behind a single module, contributor documentation is in English and historical reports are archived, so the code is easier to review.

WordCraft remains at the same 0.17.0 revision and its open engine issues are unchanged. Undo and Redo still refresh fields synchronously.

Speed measurements used synthetic documents in a virtual machine with software rendering; gains on real hardware will differ. No issues are closed by this release.

## Leaf 0.1.0-alpha.46 — Escritura más fluida en documentos largos y un código más claro

- Los paneles Revisión, Esquema y las miniaturas se actualizan al hacer una pausa al escribir, no tras cada tecla. Con Revisión abierto en un documento largo, el trabajo entre pulsaciones baja a una séptima parte.
- Los campos, las referencias cruzadas y el índice se recalculan en segundo plano tras una pausa al escribir. En un documento de 131 páginas con índice, el bloqueo más largo baja de unos 460 ms a unos 50 ms.
- El autoguardado ya no vuelve a leer las preferencias ni a comprobar los documentos ODT cada segundo cuando no hay cambios.
- Las comprobaciones de diagnóstico se reúnen en un solo módulo, la documentación para colaboradores está en inglés y los informes históricos se archivan, para que el código sea más fácil de revisar.

WordCraft conserva la misma revisión de 0.17.0 y sus issues abiertos del motor no cambian. Deshacer y Rehacer siguen actualizando los campos de forma síncrona.

Las mediciones de velocidad usaron documentos sintéticos en una máquina virtual con renderizado por software; la mejora en equipos reales será distinta. Esta versión no cierra ningún issue.
