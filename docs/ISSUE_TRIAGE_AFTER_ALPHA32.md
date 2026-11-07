# Revisión de issues después de alpha.32

La pasada posterior de #9/#99/#156 ya ha producido dos correcciones nativas y
una reproducción nueva del motor. Alpha.33 está en validación; los tres informes
siguen abiertos hasta resolver/verificar sus requisitos respectivos.
[Estado más reciente](ALPHA33_VALIDATION.md).

Revisados el 7 de noviembre de 2026 tras la petición de David de cerrar lo que
ya pueda considerarse resuelto. La revisión inicial no modifica el ejecutable publicado
ni exige otra release. Base: alpha.32, fuente publicada
`2524dbc789cfbfa0f3b6e153fde16f1df5887b8a`, WordCraft 0.16.0
`02a802cbafe45b5c8fecc00490c8f6b78c5ba251`.

## Cerrado: #100 — números y navegación de notas

[Cerrado como completado](https://github.com/frandavid100/leaf/issues/100).
Ya estaba implementado; el criterio anterior lo mantenía abierto por no conocer
la causa histórica de los números ausentes. No es necesario atribuir una causa
antigua para cerrar requisitos funcionales comprobables en la versión actual.
No se atribuye una nueva corrección a WordCraft 0.16.

- El diagnóstico existente Alpha29 reproduce exactamente «Hola» y «Probando los
  pies de página.» y exige ambos números en el display list. Se ha vuelto a
  ejecutar `cargo run --locked --example alpha29_notes_audit`; inspección visual
  del render confirma el superíndice en el cuerpo y el número junto a la nota.
- Una prueba local adicional inserta dos notas: 1 y 2 aparecen dos veces cada
  uno, en cuerpo y nota. Al borrar la primera, la segunda pasa a 1. Undo/Redo
  restaura numeración y documento canónico. Guardado/reapertura DOCX conserva
  las etiquetas; PDF verificado mediante extracción de texto y render visual.
- Las etapas 21/22 del controlador Alpha29 hacen clic real en ambos números y
  comprueban el destino correcto sin añadir historial. Alt+Intro es la ruta de
  teclado. Las entradas antiguas Edit Footnote / Next Footnote / Return to
  Reference ya no están en el menú visible; Insertar nota sigue disponible.
- [CI nativa completa](https://github.com/frandavid100/leaf/actions/runs/42371069496)
  e [instalaciones GNOME 49/51](https://github.com/frandavid100/leaf/actions/runs/42372618858)
  correctas para esos controles, incluidos inglés/español y escala 1×/2×.

La revisión local usa datos inventados. No se publican documentos privados ni
se presenta esto como una prueba humana en GNOME OS. Si reaparece un estado
que incumpla esos requisitos, se puede reabrir con esa reproducción.

## Los nueve restantes

| Issues | Estado y siguiente evidencia necesaria |
| --- | --- |
| #1, #2 | Maquetación de columnas. El trabajo de WordCraft continúa aparte; conservar anchuras ODT no prueba que se haya corregido su flujo/separador. No se integra otra rama ni se duplica ese trabajo. |
| #107 | Sigue la limitación de anclaje de imágenes. La batería de alpha.32 conserva `a_rejected_cross_page_anchor_restores_the_original_picture`: el rechazo no se maquilla como un movimiento correcto. Requiere corregir el anclaje público del motor. |
| #152 | Reproducido de nuevo en 0.16.0: Delete, Backspace y borrar el salto seleccionado dejan `Heading1` en el texto que era Normal. Necesita corrección del modelo upstream; no restilizar después desde GTK. |
| #9 | Controles nativos por debajo de 5,5, coma e incremento comprobados; alpha.33 explica el máximo entre espacios vecinos. El Portland original hereda After=8 pt; su copia editada exacta no está confirmada. Sigue abierto. |
| #99 | Ya reproducido con tabla y texto inventados, crecimiento de nota, Undo/Redo y layout nuevo: nota debajo del footer. Corrección del motor pendiente; informe upstream bloqueado por 403. [Reproducción pública](ISSUE99_TABLE_NOTES_REPRO.md). |
| #156 | Reproducidos dos fallos nativos: Home/End consumidos por GTK y zoom pendiente que restaura una página antigua. Alpha.33 corrige ambos y pasa los controles nuevos; cierre pendiente de la batería completa y del paquete. |
| #52 | Las hojas blancas de Files requieren identificar el proveedor y las entradas de caché/documentos afectados. Las miniaturas internas de Leaf son otra función; no se atribuye el defecto sin esa evidencia. |
| #84 | Descubrimiento de diccionarios del host y flujo de ajustes del sistema, en nativo/Flatpak y GNOME OS/Ubuntu. Importación y diccionarios incluidos funcionan, pero no prueban esa integración ni autorizan permisos amplios al host. |

## Nueva comprobación de #152

En la misma versión que alpha.32, crear `Document::from_text("\nBody paragraph.")`,
aplicar `para.style {"style":"Heading1"}` al párrafo vacío y ejecutar cada una de
las tres variantes de borrado desde una sesión nueva:

```text
Delete:         expected style=None; actual=Some("Heading1"); preserved=false
Backspace:      expected style=None; actual=Some("Heading1"); preserved=false
Selected break: expected style=None; actual=Some("Heading1"); preserved=false
```

La prueba no depende de GTK ni de formato de escritura pendiente. Conserva el
límite motor/cliente de AGENTS.md; esta revisión no introduce una bifurcación.
