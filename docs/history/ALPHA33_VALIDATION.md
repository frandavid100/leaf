# Alpha.33: teclado, zoom y revisión de #9/#99

Estado: **publicada y verificada** en [alpha.33](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.33). Trabajo solicitado por
David el 7/11/2026, con autorización previa para publicar automáticamente después
de las comprobaciones. [PR #161](https://github.com/frandavid100/leaf/pull/161).

## #156: dos fallos nativos reproducidos y corregidos

La prueba anterior solo llamaba al comando del motor. La nueva prueba envía
teclas y clics reales y mide por separado posición canónica y viewport.

- En [la base sin corrección](https://github.com/frandavid100/leaf/actions/runs/42378804400),
  Fin deja el cursor en el párrafo 108, offset 308, en vez del fin visual 350.
  El ScrolledWindow consume la tecla antes del controlador de Leaf en burbujeo.
- Recibir en captura corrige Fin/Inicio y Mayús al 50/100/150 %. La [segunda ejecución](https://github.com/frandavid100/leaf/actions/runs/42380295294)
  reproduce después otro fallo: un zoom pendiente restaura la página antigua
  tras una navegación nueva. Cursor y=14043,7; viewport y=1404, altura 748.
- La restauración ahora comprueba documento, revisión y zoom. Si una navegación
  más reciente cambia el destino, revela el cursor actual. Un contador registra
  también Fin cuando ya estaba al final de línea y la selección no cambia.
  El callback conserva propietario débil; no retiene ni sustituye el documento.

La fuente final supera las nuevas pruebas gráficas: Fin/Inicio con Mayús,
Ctrl+Fin/Ctrl+Inicio, repaginación por edición, campos nativos con foco, selección,
documento/historial sin cambios al navegar, zoom pendiente y Fin repetido sin
cambio de selección. La captura interna muestra el destino y su miniatura activa.
No se sustituye la galería pública ni el controlador original de fuente/pegado/canvas.

## #9: controles medidos y explicación visible

El campo GTK conserva 0/1/3/5/5,5/6/24 pt en ambos sentidos, acepta coma decimal
y la acción nativa de incremento. Con Después anterior=0, la distancia medida
es 15,868652 + Antes; con Después=5,5 es 15,868652 + máximo(5,5, Antes).
No se impone un mínimo de 5,5. La llamada directa inicial a spin() era una prueba
incorrecta del botón porque omite la actualización previa del texto de GTK;
se sustituyó por la acción nativa.

Párrafo muestra una explicación breve del máximo entre espacios contiguos y de
su posible supresión por estilos, traducida a inglés/español. No cambia la
semántica del documento. El Portland original conservado tiene Después=8 pt en
Normal/Standfirst y Antes=11/Después=7 en Heading2. No es la copia editada con los
párrafos de prueba de la captura; el origen exacto de su umbral sigue sin confirmar.
**#9 permanece abierto.**

## #99: reproducción del motor, pendiente de corrección

[Reproducción pública y análisis](ISSUE99_TABLE_NOTES_REPRO.md), con texto
inventado: una nota en una tabla puede quedar debajo del footer. Se reproduce
con sesión nueva, crecimiento de nota, Undo/Redo y diferentes distancias del pie.
El render y PDF confirman el orden incorrecto; no es solo una medición de textura.

La reserva de notas posterior a colocar filas es la causa investigada, sin
presentarla como un arreglo probado. Crear el informe upstream devuelve 403
“Resource not accessible by integration”. No se creó un issue allí ni se añade
un fork del motor a Leaf. El documento privado no se publica.
**#99 permanece abierto.**

## Fuentes y validación

- Motor/dependencias/parches: sin cambios, WordCraft 0.16.0
  `02a802cbafe45b5c8fecc00490c8f6b78c5ba251`.
- Fuente nativa: `ce16577ba85139f9f0d8c10551b103d4c8c3f5fe`;
  [ejecución completa](https://github.com/frandavid100/leaf/actions/runs/42381417100).
- Fuente de candidata: `529e086d7e6aec85c1ed4409af8c7e55aaa6c8b7`;
  [empaquetado e instalaciones](https://github.com/frandavid100/leaf/actions/runs/42382604979).
  Diff de todos los inputs de la app vacío frente a la fuente nativa.
- Compilación GUI optimizada y check GUI locales correctos; 189 pruebas Rust,
  cuatro Python, 811 mensajes gettext completos, formato y enlaces correctos.
  La CI final también supera las 189 pruebas sin interfaz y los controles nuevos.
- Batería nativa completa, GNOME 49 instalado, FlatPark/GNOME 51 y comparación
  final del ejecutable: **superados**. El mismo controlador nuevo se ejecuta en
  ambos runtimes, en inglés/español y escala 1×/2×.
- [Publicación automática](https://github.com/frandavid100/leaf/actions/runs/42384540865)
  correcta el 7/11/2026 a las 15:04 UTC. Tag/source coinciden con la candidata;
  doce archivos públicos coinciden en tamaño y SHA-256 con la descarga validada,
  incluidas las once entradas del manifiesto. #156 cerrado después de comprobarlo.
  [Registro verificable de los archivos](ALPHA33_ASSET_VERIFICATION.json).

Bundle: 12 932 008 bytes, SHA-256
`8aaee728e01a9e2c35ff999cf68431a8d01f6f910a4ca2dabe6d5928b85f17b2`.
Archivo FlatPark: 13 213 348 bytes, SHA-256
`24af9ce8630ad76549592ba9b7c3e797570aab8f0bbf97f9bc9f4abcfe3d300d`.

Implementación y verificación asistidas por IA. Las comprobaciones automatizadas
no se presentan como una sesión humana en GNOME OS/Wayland.

