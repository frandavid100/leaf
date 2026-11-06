# Alpha.29: candidata y alcance

La candidata continúa la pasada autorizada de issues con prioridad #99–#109 y los nuevos #111–#121.
La [PR #110](https://github.com/frandavid100/leaf/pull/110) contiene los cambios.
Todavía no se ha publicado una release: David debe probar el paquete y aprobar
su promoción sin recompilar. [Proceso y comandos de instalación](CANDIDATES.md).

## Resultado por issue

| Issue | Implementación y comprobación prevista en la candidata |
| --- | --- |
| #111–#116 | Cabecera contextual del panel, iconos simbólicos de 16 px y ¶, idioma heredado resuelto en la misma fila, traducciones y verbos Insertar/Aplicar. Pendiente de validación GTK final. |
| #117/#119 | Revisión unificada con filtros, cambios del motor, correcciones y comentarios, acciones propias e indentación reducida. Pruebas de historial, DOCX/ODT y objetivos obsoletos; pendiente de validación GTK final. |
| #118 | Imán de 0,5 cm en magnitudes del documento con histéresis para los gestos de regla; valores importados y numéricos intactos. Pendiente de gestos GTK finales. |
| #120 | Filas de formato con etiquetas y casillas nativas de tres estados, opciones separadas y memoria por propiedad. Pendiente de validación GTK final. |
| #121 | Elementos en la cabecera adaptable y herramientas compartidas. Parcial: el motor solo admite un índice por documento; una creación explícita nunca reemplaza silenciosamente el existente. Se conserva el acceso a crear/editar con la limitación explicada. |
| #109 | Elementos compartido y agrupado (nombre/ubicación actualizados por #121), un único Enlace para URL/destinos internos y siete campos automáticos estructurados. Pruebas de encabezado/pie, metadatos, historial, DOCX y PDF. |
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

La ejecución local completa supera 171 pruebas Rust, cuatro Python, formato y
enlaces; catálogos inglés/español completos con 837 mensajes. La comprobación GUI local incluye GTK 4.14/libadwaita 1.5 con Xvfb: revisión unificada, comentarios, correcciones, formato y gestos reales de imagen y reglas. La candidata aún debe superar CI y sus instalaciones.

`tests/references_alpha29.rs` comprueba actualizaciones derivadas, fecha congelada,
caret, historial/redo, leyendas e índice; Guardar como actualiza FILENAME al destino
real y un fallo de escritura conserva la sesión. La regresión de índice importado también comprueba edición del mismo campo,
historial y nueva reapertura DOCX. Los siete campos conservan su
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
dentro del Flatpak GNOME 49 y del archivo relocatable GNOME 51. Se conserva la batería de regresiones anterior.

La primera ejecución gráfica pasó los controles/gestos en inglés. La revisión
HiDPI aisló una miniatura dos píxeles fuera del área visible: el borde CSS de la
selección cambiaba su tamaño después del ajuste inicial. La corrección sigue ese
cambio de página durante dos frames de maquetación, con identidad/página guardadas;
los redibujados ordinarios no la programan. La prueba también abre explícitamente
el panel tras una navegación que cierre su variante colapsada.

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

La prueba de Texto usa dos movimientos de ratón antes del clic: en GTK 4.14/X11,
un salto instantáneo entre superficies puede producir una notificación tardía de
la ventana padre. Una reproducción mínima independiente confirma el efecto; el
segundo movimiento reproduce la entrada normal en el panel sin cambiar su
modalidad ni sustituir el clic por activación programática. Las etiquetas actuales
se actualizan en su widget existente para conservar el foco durante refrescos.

La revisión visual en español también corrige el nombre «Revisión» y localiza
las explicaciones ortográficas y las reglas inglesas comunes del motor en el
idioma de la interfaz. Los mensajes desconocidos se conservan íntegros. El
interlineado actual muestra decimales legibles sin redondear el valor guardado.

La regresión histórica de fuente/tamaño cierra la lista de familias y su panel
Texto antes de pulsar el caret: las filas nuevas pueden cubrir ese punto en una
ventana estrecha. La escritura, los decimales con coma, el pegado y la selección
con panel fijado siguen comprobándose con entrada real; el panel conserva su pin
al volver a abrirlo.
