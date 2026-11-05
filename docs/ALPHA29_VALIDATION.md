# Alpha.29: candidata y alcance

La candidata continúa la pasada autorizada de issues con prioridad #99–#109.
La [PR #110](https://github.com/frandavid100/leaf/pull/110) contiene los cambios.
Todavía no se ha publicado una release: David debe probar el paquete y aprobar
su promoción sin recompilar. [Proceso y comandos de instalación](CANDIDATES.md).

## Resultado por issue

| Issue | Implementación y comprobación prevista en la candidata |
| --- | --- |
| #109 | Insertar compartido y agrupado, un único Enlace para URL/destinos internos y siete campos automáticos estructurados. Pruebas de encabezado/pie, metadatos, historial, DOCX y PDF. |
| #108 | Revelar miniatura activa con desplazamiento mínimo y etiquetas completas localizadas. Redibujar no interrumpe la exploración manual del panel. |
| #107 | Movimiento hacia la página real, escala/grab offset, vista previa, autoscroll y cancelación. Parcial: el motor no siempre acepta anclar una imagen flotante al párrafo de un salto manual; se rechaza ese destino y se restaura el original. Sigue abierto. |
| #106 | Ocho tiradores y cursores, bordes de una dimensión/esquinas proporcionales; vista previa, cancelación y un paso de historial. Las imágenes en línea conservan su anclaje al texto. |
| #105 | Selección de imagen con acento translúcido y contenido visible; decoración solo de vista. |
| #104 | Leyenda directa de la imagen seleccionada/tabla actual, sin diálogo ni selector manual; caret listo para la descripción y disponibilidad contextual. |
| #103 | Actualización agrupada de valores derivados, índice y referencias; caret/historial estables, recálculo tras deshacer/rehacer y preparación antes de guardar/PDF/impresión. Fechas no se recalculan al teclear. |
| #102 | Notas en Propiedades del documento/sección, ancladas a la sección solicitada. Los valores sin cambios conservan su herencia. |
| #101 | Menú contextual del índice real, edición de sus opciones y conservación de los switches que no se modifican. |
| #100 | Navegación directa por ambos números y Alt+Intro; retiradas las órdenes rutinarias del menú. El ejemplo nuevo “Hola” muestra ambos números en el render público. El síntoma original de números ausentes no se reproduce; sigue abierto para contrastar el caso informado. |
| #99 | Los controles nuevo y poblado sitúan la nota dentro de la hoja. No está disponible el DOCX original del informe; su maquetación de tabla/columnas sigue sin reproducirse. Sigue abierto. |
| #56 | Editor nativo básico UnicodeMath, plantillas, inserción/edición estructurada, DOCX, historial, PDF y protección de construcciones importadas que no sobreviven a la conversión lineal. |
| #84 | Sin cierre: la instalación de idiomas y exposición de diccionarios del sistema dependen de la plataforma; se conserva la importación explícita existente. |
| #52 | Sin cierre: faltan el archivo y el proveedor/caché original para atribuir la miniatura vacía de Archivos. |
| #9 | Sin cierre: las regresiones ordinarias de espaciado pasan, pero no se ha aislado el caso original. |
| #2 | Sin cierre: defecto confirmado de separadores en secciones continuas del motor; upstream #5178 ya registrado. |
| #1 | Sin cierre: continuación natural desigual con desbordamiento del motor; upstream #2382 ya registrado. |

## Pruebas

La ejecución local completa supera 161 pruebas Rust, cuatro Python, formato y
enlaces; catálogos inglés/español completos con 792 mensajes. La comprobación GUI
local es de tipos; la ejecución real GTK depende de CI con las bibliotecas GNOME.

`tests/references_alpha29.rs` comprueba actualizaciones derivadas, fecha congelada,
caret, historial/redo, leyendas e índice; Guardar como actualiza FILENAME al destino
real y un fallo de escritura conserva la sesión. Los siete campos conservan su
historia/documento al guardar/reabrir, incluidos PAGE/NUMPAGES en encabezado/pie y
metadatos presentes únicamente en un encabezado. La guarda ODT recorre todas las
historias. PDF/impresión usan una copia y conservan la ruta del documento.

`tests/document_tools.rs` verifica movimiento en línea/flotante a otra sección,
DOCX, PDF y un paso de historial, y rechazo atómico del destino con salto manual.
`tests/equations.rs` verifica inserción, edición, errores, DOCX/PDF y protección de
propiedades matemáticas que el editor básico no puede representar fielmente.

`src/gnome/diagnostics/alpha29.rs` y `tools/alpha29-pointer-test.sh` ejercitan
controles GTK y eventos reales: actualización automática, miniaturas, enlaces,
índice, leyendas, ecuaciones, notas, Propiedades, los ocho tiradores en línea y
flotantes, movimiento entre páginas, cancelación, huecos inválidos, clic de ida y
vuelta en los números y autoscroll. Inglés/escala 1 y español/escala 2, nativo y
dentro del Flatpak. Se conserva la batería de regresiones anterior.

La primera ejecución gráfica pasó los controles/gestos en inglés y detectó que
la prueba asumía abierto el panel tras navegar desde una vista colapsada. Ahora
se abre explícitamente el panel para comprobar también ese recorrido; no se
presenta ese fallo de preparación del test como una corrección de maquetación.

## Identidad y límites

El manifiesto `candidate.json`, `source-commit.txt`, las notas congeladas y
`SHA256SUMS` identifican el paquete final. La ejecución de empaquetado debe pasar
actualización desde alpha.28, portales y pruebas instaladas GNOME 49/FlatPark 51
antes de presentar la candidata como verificada. El éxito local descrito arriba
no afirma por sí solo éxito instalado ni publicación.

No cambian WordCraft, las dependencias ni los parches. No se sustituyen capturas
públicas ni se amplían Orca, Weblate o Flathub. DOCX sigue siendo necesario para
las estructuras que el exportador ODT pierde. Las comprobaciones automáticas no
sustituyen pruebas humanas en Wayland, métodos de entrada o impresoras físicas.
