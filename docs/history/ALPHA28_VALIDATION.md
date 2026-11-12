# Alpha.28: alcance y comprobación

La entrega incorpora #53, #54, #55, #89, #94 y #95, más leyendas numeradas y
referencias cruzadas. Se usa el modelo y las API públicas de WordCraft 0.14.0;
no se cambian dependencias, motor, renderer ni parches de formatos.

## Fuente y validación nativa

Fuente publicada: `d2ba113d9ed2bc0dbb4794732c6035a195cba755`.
[CI](https://github.com/frandavid100/leaf/actions/runs/42057994674): batería nativa
completa superada, con 151 pruebas Rust, cuatro Python y 766 mensajes completos
en inglés y español. Incluye controles GTK, puntero, historial, reglas,
liberación de ventanas, teclado, impresión, vistas estrechas/oscuras y HiDPI.

La prueba de retorno desde un encabezado fija zoom de 80 % para que su clic
caiga en un fondo real, fuera de las reglas transitorias del borde. El ajuste
por sección sigue comprobándose por separado en el diagnóstico de alpha28.

## Paquetes publicados

[Release alpha.28](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.28),
publicada el 5 de noviembre de 2026 a las 16:18 UTC. La misma CI pasa actualización
desde alpha.27, instalación y reinstalación, interacciones y apariencia dentro
del sandbox, portales de archivos/impresión, texto AT-SPI y salida Orca. El
archivo relocatable pasa con GNOME 51 bajo `/app/extra/leaf`. No son pruebas
humanas de accesibilidad, Wayland, IME ni impresoras físicas.

Se descargaron de la release el bundle, el archivo FlatPark, `source-commit.txt`
y `SHA256SUMS`; sus sumas coinciden. La etiqueta y el archivo de fuente señalan
la revisión indicada. La [galería y su procedencia](../screenshots/README.md)
corresponden al Flatpak instalado de esa misma revisión.

## Documentos externos

Se descargaron estas muestras públicas independientes el 5 de noviembre de 2026.
La procedencia se comprueba con sus metadatos, no se infiere de la extensión.

| Muestra | Productor declarado | SHA-256 |
| --- | --- | --- |
| [sample.docx](https://github.com/apache/poi/blob/trunk/test-data/document/sample.docx) | Microsoft Office Word | `be4fc1bf778ad8183f714e09b9df40669de34e9ecbc3adecfc8953f00749036f` |
| [tdf133507_contextualSpacingSection.odt](https://github.com/LibreOffice/core/blob/master/sw/qa/extras/odfexport/data/tdf133507_contextualSpacingSection.odt) | LibreOfficeDev 24.2.0.0.alpha0 | `7793ffea5e755bb197020dc81eb48c2b01827c7a2128a268d1d9e10e74b9faf9` |
| [footnotes.docx](https://github.com/apache/poi/blob/trunk/test-data/document/footnotes.docx) | Microsoft Office Word 12 | `9070094b27ff5201477f4de4b2c027730c66cb889106a1609c5adeed207a59c1` |

En las dos primeras se abre el archivo con Leaf, se inserta una marca de texto,
se guarda en su formato original y se reabre. El texto completo del cuerpo
reabierto coincide con el texto editado. La exportación PDF conserva la marca,
comprobada con `pdftotext`. DOCX: 1544 caracteres originales, una página y PDF de
15920 bytes. ODT: 611 caracteres originales, dos páginas y PDF de 17159 bytes.

En la tercera se encuentra una nota importada, se navega a ella, se añade texto,
se vuelve al cuerpo y se guarda/reabre como DOCX. Se conserva una referencia y
el texto editado de la nota coincide. El PDF de 11882 bytes contiene la edición.

Estas son comprobaciones de apertura/edición/guardado/reapertura en Leaf y PDF.
No son una comparación visual exhaustiva ni una reapertura en las aplicaciones
originales. Las muestras no se redistribuyen dentro de la aplicación.

## Regresiones automatizadas

`tests/references.rs` cubre inserción, edición, navegación, eliminación de
referencias, historial, DOCX y nota larga con paginación/PDF. El índice y la
referencia cruzada siguen siendo campos actualizables tras reabrir DOCX y
renombrar el título de destino.

`tests/document_tools.rs` cubre ajuste flotante, separación, anclaje,
DOCX e historial. Una regresión adicional detectó una caché de líneas obsoleta
al pasar de una imagen detrás del texto a una imagen en línea: la orden del
cliente ahora toca la revisión del párrafo, como las órdenes públicas del motor.
La prueba compara el resultado ya maquetado con una sesión de caché nueva.

Los diagnósticos GTK de alpha28 activan las acciones y sus diálogos reales,
comprueban zoom por sección y miniaturas, y usan xdotool para mover/redimensionar
imágenes, con vistas previas y deshacer/rehacer. Se ejecutan en inglés a escala 1
y español a escala 2. El resto de la batería comprueba las regresiones anteriores.

## Límites

Las notas, los índices automáticos, las referencias/leyendas y las imágenes
flotantes requieren DOCX para conservar su estructura. Las guardas de Leaf
rechazan los guardados ODT que las perderían. El guardado ODT ordinario sigue
cubierto por el recorrido externo anterior.

La ficha AppStream y la galería de Leaf pueden actualizarse en este repositorio.
FlatPark conserva su propia copia de la ficha; esa galería requiere que sus
responsables incorporen la actualización. No se da por publicada en su catálogo
por el hecho de publicar una release de Leaf.
