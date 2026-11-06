# Estado actual de Leaf

Actualizado: 6 de noviembre de 2026. Esta página describe el presente;
el [historial](docs/history/README.md) conserva entregas y decisiones anteriores.

## Dónde trabajar

- Repositorio: https://github.com/frandavid100/leaf
- Rama activa, compilación y destino de PR: **`main`**. ApexDesk se conserva en
  `archive/apexdesk-main-2026-11-02`; `wordcraft-saving` sigue disponible.
- [Integración de historiales y validación](docs/MAIN_MIGRATION.md).
- [Cómo retomar el proyecto](docs/CONTINUING.md), [arquitectura y mapa de archivos](docs/ARCHITECTURE.md),
  [decisiones vigentes](docs/DECISIONS.md), [plan de trabajo](ROADMAP.md).

## En curso: leyendas, navegación y revisión de #132

David autoriza resolver #132 y completar conjuntamente #121/#124/#126/#127.
Las asociaciones explícitas de leyendas, su ocultación sin pérdida, propiedades
compartidas, navegación tipada y decoración temporal están implementadas bajo
validación. Nueve regresiones específicas sin GUI pasan, junto con controles
GTK por clic/teclado en inglés y español, tema claro/oscuro y escala 1×/2×.
Faltan la batería gráfica completa, la instalación y la revisión final antes
de cualquier cierre.
[Modelo, comprobaciones y límites](docs/CAPTION_OWNERSHIP.md).

El documento original de #132 no reproduce el solapamiento en los controles
Alpha.30/Alpha.31 de este entorno, con emoji en color; sus baselines y exportación
también están separados. El issue sigue abierto, sin atribuirle una corrección.
La segunda candidata ya entregada se conserva intacta; no hay nueva publicación.

## Preparada: segunda candidata alpha.31 — Interfaz nativa

[Candidata para probar](https://github.com/frandavid100/leaf/actions/runs/42265503961/artifacts/13527444066), sin publicación automática.
Cerrados #134, #135 y #136: cabeceras alineadas, selector nativo Páginas/Estructura
y refinamientos comunes de Texto, Párrafo, Estilo y Elementos. Se conserva #133.
La regla rechaza eventos de superficies flotantes, evitando que el selector de
fuentes pierda el foco y desvíe el teclado al documento.

178 pruebas Rust, cuatro Python y 824 mensajes completos inglés/español.
[CI nativa](https://github.com/frandavid100/leaf/actions/runs/42263872786),
[instalaciones GNOME 49 / FlatPark GNOME 51](https://github.com/frandavid100/leaf/actions/runs/42265503961)
y [menús realmente traducidos en ambos entornos](https://github.com/frandavid100/leaf/actions/runs/42269593968)
superados. Descarga, once sumas y capturas instaladas comprobadas; la regresión
original de fuente/pegado mantiene su secuencia y aserciones.

Fuente del paquete `79292491050943559df21d9082657fe38ee46254`; ejecutable de
`179ef4bec1609c355c3dae325c3358fd6bb948a8`, con equivalencia y bytes comprobados.
[Alcance, identidad, sumas y límites](docs/ALPHA31_UI_VALIDATION.md).
Quedan 13 issues: #1, #2, #9, #52, #84, #99, #100, #107, #121, #124, #126, #127 y #132.
Alpha.30 sigue siendo la última versión pública. David debe probar la candidata
y aprobar la promoción de estos mismos archivos, sin recompilar.

## Primera candidata alpha.31 — Histórica, sustituida por la segunda

La [primera candidata](https://github.com/frandavid100/leaf/actions/runs/42230422027/artifacts/13510771784)
solo cerró #133 sobre la interfaz de Alpha.30. Superó 175 pruebas Rust, cuatro
Python y las instalaciones GNOME 49/51. En aquella entrega se retiraron los
experimentos de #134–#136 tras fallar la entrada instalada; quedaron 16 issues.
Su [identidad y evidencia](docs/ALPHA31_VALIDATION.md) se conservan como historial.
La segunda candidata incorpora esa corrección y resuelve los tres informes de
interfaz después de aislar la captura de eventos de la regla.

## Última versión publicada: alpha.30

[Release y descargas](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.30).
Publicada automáticamente con autorización de David, tras superar la
[CI nativa](https://github.com/frandavid100/leaf/actions/runs/42184270093),
[instalaciones GNOME 49 y FlatPark/GNOME 51](https://github.com/frandavid100/leaf/actions/runs/42187179877)
y [promoción sin recompilar](https://github.com/frandavid100/leaf/actions/runs/42189125319).
Fuente publicada `85d0134e413f1cb4587e2cf2de9d9928a1e85a1c`; ejecutable de
`3f898df0f42f280d932eaa2bb3790dd6b45d1190`, con equivalencia y bytes comprobados.
Los 12 archivos públicos coinciden con la candidata por SHA-256.

Cerrados siete issues de esta pasada: #122, #123, #125, #128, #129, #130 y #131.
Incluye viewport de imágenes estable, cursores, miniaturas por caret, índice
navegable, tarjetas de revisión, Texto/Párrafo y Navegación Páginas/Estructura.
Acumula toda la candidata alpha.29. 173 pruebas Rust, cuatro Python y 855 mensajes
completos inglés/español. [Evidencia, sumas y límites](docs/ALPHA30_VALIDATION.md).

Al finalizar alpha.30 quedaron abiertos 12: #121/#124 parciales, #126/#127 pendientes de asociación canónica
de leyendas, y los ocho antiguos #1, #2, #9, #52, #84, #99, #100 y #107. No se
actualiza el motor ni se ocultan requisitos pendientes mediante cierres parciales.

## Candidata alpha.29: incorporada a alpha.30

La [PR #110](https://github.com/frandavid100/leaf/pull/110) integra la pasada
#99–#109 y #111–#121, además del editor básico de ecuaciones (#56).
[Validación nativa completa](https://github.com/frandavid100/leaf/actions/runs/42142299796) y
[empaquetado e instalaciones GNOME 49/FlatPark 51](https://github.com/frandavid100/leaf/actions/runs/42143216380) superados.
Fuente de la candidata `361a25c286a1a06f6c007b9abdf23d7d1d36b8e3`; fuente del ejecutable
`25ae0ba569bc374e84ad78d0a314fe7b9c611b5b`, con equivalencia comprobada.
171 pruebas Rust, cuatro Python y 837 mensajes completos inglés/español.

Resueltos 19 issues: #56, #101–#106, #108–#109 y #111–#120. Revisión reúne
correcciones, cambios sugeridos y comentarios; Texto/Párrafo usan filas y
Elementos comparte las herramientas del objeto. Incluye reglas con imán de
0,5 cm, referencias automáticas, notas, imágenes y miniaturas mejoradas.
Los casos #99, #100, #107 y #121 conservan trabajo pendiente o limitaciones;
#1, #2, #9, #52 y #84 siguen abiertos. [Evidencia y límites](docs/ALPHA29_VALIDATION.md).

La candidata alpha.29 no se publicó por separado: sus cambios se distribuyen en
alpha.30, autorizada y verificada. El manifiesto y SHA256SUMS identifican los
archivos. [Proceso de candidatas](docs/CANDIDATES.md).

## Entrega anterior: alpha.28

[Release y descargas](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.28).
[CI nativa, Flatpak GNOME 49 y archivo FlatPark GNOME 51](https://github.com/frandavid100/leaf/actions/runs/42057994674)
completas. Fuente `d2ba113d9ed2bc0dbb4794732c6035a195cba755`.

Incluye zoom por sección y navegación/miniaturas (#89, #94, #95), imágenes
flotantes con ajuste, separación y gestos de ratón (#53), notas al pie (#55),
índice automático (#54), leyendas y referencias cruzadas. La [PR #98](https://github.com/frandavid100/leaf/pull/98)
integra controles nativos mediante las API públicas de WordCraft. Se corrigió
además la invalidación de maquetación al cambiar el ajuste de una imagen.

151 pruebas Rust, cuatro Python y 766 mensajes completos inglés/español.
Verificados historial, interfaz, idiomas, HiDPI, actualización desde alpha.27,
portales de archivos e impresión, reinstalación, texto AT-SPI y salida Orca.
Son comprobaciones automáticas; no sustituyen pruebas humanas de dispositivos.

La edición/guardado/reapertura y el PDF se comprobaron con muestras externas de
Word y Writer, incluida una nota importada de Word. [Evidencia y límites](docs/ALPHA28_VALIDATION.md).
Estas estructuras avanzadas requieren DOCX: las guardas ODT siguen evitando
pérdidas. No se modificaron motor, renderer, dependencias ni parches.

La [galería pública](docs/screenshots/README.md) usa cuatro capturas reales del
Flatpak instalado, expresamente autorizadas por David. La ficha AppStream de
Leaf está actualizada; FlatPark mantiene otra copia que sus responsables deben
actualizar para incorporar la galería y la descripción nuevas.

## Entrega anterior: alpha.27

[Release y descargas](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.27).
[CI nativa completa](https://github.com/frandavid100/leaf/actions/runs/41995941521)
y [empaquetado e instalaciones GNOME 49/51](https://github.com/frandavid100/leaf/actions/runs/42000544951)
superados. Fuente de la release `1259aa95fbde9650fe17d5a9476cb4d1e73685cc`;
el ejecutable reutiliza la compilación validada de `74865717f99d63bab8000f1470db7b42718ba761`,
con equivalencia de fuentes comprobada por CI.

Incluye el tarball solicitado por FlatPark, con `bin/` y `share/` en la raíz,
su checksum y `flatpark-smoke.log`. La prueba instala el archivo mediante
extra-data en `/app/extra/leaf` con GNOME 51 y verifica traducciones inglés/español
y revisión ortográfica. El bundle habitual usa GNOME 49 y verifica actualización
desde alpha.26, interacción, guardado mediante portal, impresión y Orca/AT-SPI.
La primera prueba del tarball falló por omitir Flatpak el directorio reservado
`extra` al exportar el paquete de prueba. El comprobador corregido usa descarga
HTTP local con checksum y `apply_extra` durante la instalación.

FlatPark ha aceptado la inclusión; su publicación en el catálogo queda pendiente
del empaquetado de sus responsables. Avisarles de cambios de desktop/iconos/
metainfo y futuros requisitos de runtime. Las pruebas anteriores son automáticas.

## Entrega anterior: alpha.25

[Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.25).
[CI nativa e instalada](https://github.com/frandavid100/leaf/actions/runs/41649488931)
superada. Incluye comentarios plegables, vistas previas de estilos y cambios
en miniaturas y reglas. La integración con los ajustes de idioma del sistema
(#84) y las comprobaciones específicas de desplazamiento y arrastre de
márgenes (#83, #87) siguen pendientes.

[Alpha.24](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.24)
incorporó propiedades de sección, reglas translúcidas e importación de diccionarios.

## Validación de compatibilidad para el MVP

[Auditoría con LibreOffice Writer](https://github.com/frandavid100/leaf/actions/runs/41630136413).
La ficha DOCX creada en Writer conserva la edición de Leaf, las celdas de tabla
y el espaciado de 24 puntos al reabrirla. El PDF exportado conserva el texto
editado y el contenido de la tabla. El informe de CI recoge diferencias
en los vínculos entre secciones y en la exclusión de texto de la revisión
ortográfica tras guardar desde Writer. El caso original de espaciado (#9)
sigue pendiente de reproducción.

## Entrega anterior: alpha.23

[Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.23), fuente `9cda589709aff7d0c21446c9b9b105b0d0750acc`. [CI nativa e instalada](https://github.com/frandavid100/leaf/actions/runs/41606977263) superada y sumas de las descargas verificadas. Cerrados #64, #65, #7 y #37.

## Entrega anterior: alpha.22 — formato, comentarios e impresión

[Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.22).
Fuente: `1cda6b2df2eeabd46dbc3331c5adb64c452b679f`. GNOME 49 y actualización
desde alpha.21. [CI nativa e instalada](https://github.com/frandavid100/leaf/actions/runs/41597951947)
superada. Verificados los ocho archivos públicos, las sumas y la fuente del paquete.

Incluye WordCraft 0.14.0, su API específica para impresión, la optimización del
panel Estilos y la limpieza del cliente. Siete cierres: #66–#68, #70, #71,
#73 y #74. #69 sigue abierto para el enlace de instalación de diccionarios.

## Revisión de formato, comentarios e inserción

La [PR #72](https://github.com/frandavid100/leaf/pull/72) compacta Fuente,
Tamaño, Texto, Párrafo y Estilos (#66–#71), añade un editor de comentarios y
respuestas en el panel lateral (#73) y comparte el submenú Insertar entre
Documento y el menú contextual (#74). El [informe de revisión](docs/FORMAT_POPOVER_REVIEW.md)
recoge la comparación de prototipos y las reproducciones pendientes de
columnas (#1, #2), saltos (#65) y diccionarios (#69).

## Revisión adicional: integración y claridad del código

David autoriza revisar el aprovechamiento de WordCraft 0.14 y limpiar estructura,
formato y trabajo redundante del cliente. Se integra la API de PDF para impresión,
se separan controladores y diálogos y se evita recorrer estilos ocultos. La
[PR #63](https://github.com/frandavid100/leaf/pull/63) está integrada tras superar la
[CI nativa](https://github.com/frandavid100/leaf/actions/runs/41508412922). [Investigación y alcance](docs/WORDCRAFT_014_REVIEW.md).

## Entrega anterior: alpha.21 — WordCraft 0.14.0

[Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.21), publicado y verificado el 2 de noviembre de 2026.
Fuente/tag: `3849d2973670ba05093f178108bab182778579f1`. GNOME 49; actualización desde alpha.20.
Descarga independiente de los ocho assets públicos, sumas y fuente comprobadas.
Bundle: 11028040 bytes; SHA256 `6b332768aeb50d26c8db44f769a5e014fa71fb546fffc413591bcf2ad2290f40`.

Motor actualizado a WordCraft **0.14.0**, commit
`5cd53f184aab11def13b7fced18727159b9bc96a`, con copia nueva de formatos y los
cuatro parches ODT reaplicados. Perfil optimizado habitual conservado.

**Cuatro cierres:** #18, #48, #51 y #61. Selector de tema revisado en las ocho
variantes instaladas anteriores; reglas blancas finas por hoja y solo transitorias;
Deshacer/Rehacer con el mismo AdwSplitButton que Párrafo; Fuente/Tamaño con
anchuras acotadas y Tamaño a fila completa. Las reglas ocultas dejan pasar clics.

141 pruebas Rust, cuatro Python, 678 mensajes gettext, formato y enlaces correctos.
[CI completa nativa e instalada](https://github.com/frandavid100/leaf/actions/runs/41496070439) superada, incluidas las pruebas de interacción,
HiDPI, actualización/reinstalación, portales, impresión y regresión Orca existente.
Capturas finales instaladas revisadas en claro/oscuro y ventanas estrechas a 1×/2×.
[Resultado y límites](docs/ISSUE_SWEEP_ALPHA21.md) ·
[evidencia](docs/ISSUE_SWEEP_ALPHA21_VALIDATION.json).

Estado: **50 cerrados y 10 abiertos**: #1, #2, #7, #9, #37 y #52–#56.
La auditoría sintética del motor actualizado conserva en DOCX índice automático,
notas, ecuaciones e imágenes flotantes; ODT sigue perdiendo esas estructuras y
se mantienen las guardas. Sus interfaces completas no están implementadas.
No se afirma validación humana de Wayland, IME, dispositivos o documentos arbitrarios.
La PR #57 de Weblate permanece separada; archivo, rama de referencia y etiquetas
anteriores se conservan.

## Entrega anterior: alpha.20 (estado histórico)

[Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.20),
publicado el 2 de noviembre de 2026. Fuente/tag: `e5d65dde0c788ef563b5263dbcf3e4e823df92ba`.
GNOME 49; actualización desde alpha.19. Ocho assets públicos y digests registrados
por GitHub. Bundle: 11.059.320 bytes; SHA256
`0e178093f421f08be493086198943c042c4e719d431506a8381a41de89e90e5b`.

**Seis informes cerrados:** #49–#51 y #58–#60. Cursor con fundido, Repetir,
historiales Deshacer/Rehacer, iconos de Navegación/Comentarios y tooltip de páginas.
La regla (#48) y el selector GNOME (#18) están implementados y pasan las pruebas
automáticas; permanecen abiertos para completar la comparación visual instalada.
Leaf también bloquea guardar en ODT notas, ecuaciones, índice automático e imágenes
flotantes que el exportador perdería; sus interfaces completas (#53–#56) siguen pendientes.

141 pruebas Rust, cuatro Python, 678 mensajes gettext y build optimizado correctos.
[CI nativa](https://github.com/frandavid100/leaf/actions/runs/41466739518) y
[Flatpak instalado](https://github.com/frandavid100/leaf/actions/runs/41471767320)
completos, con equivalencia de fuentes comprobada antes de reutilizar el ejecutable.
Incluye gestos, actualización/reinstalación, portales, impresión y la regresión
Orca existente; las ocho capturas de tema verifican también el cambio real por teclado.

Al cerrar alpha.20, el entorno de ejecución se desconectó. Quedaron pendientes
la revisión visual completa de sus capturas instaladas y la descarga
independiente de sus assets; publicación, fuente/tag y digests sí están confirmados
por GitHub. No se afirma validación humana de Wayland, IME o dispositivos.
[Alcance y pendientes](docs/ISSUE_SWEEP_ALPHA20.md) ·
[evidencia](docs/ISSUE_SWEEP_ALPHA20_VALIDATION.json).

Estado histórico de aquella entrega: **47 cerrados y 12 abiertos**. Se preservan releases, etiquetas, archivo de
ApexDesk, wordcraft-saving y la PR de Weblate separada.

## Entrega anterior: alpha.19

[Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.19), publicado y verificado el 2 de noviembre
de 2026. Fuente/tag: `1f8694dd8cf51c8f981baf8aa23093ea4e09b625`. Runtime GNOME 49;
actualización desde alpha.18, ocho assets públicos y sumas verificados.
Bundle: 10.984.832 bytes; SHA256
`71656fe9c658fff1f24804f2ee7c2ed9d9086de72919b66053594efa9ffdf39c`.

**Nueve informes cerrados:** #25, #26, #28, #34, #35 y #44–#47. Listas de fuentes
legibles y escritura estable, enlaces con Ctrl+K/contexto/URL visible, arrastre
por párrafos, contexto de títulos y ajustes de revisión/marcadores.
Superadas 128 pruebas Rust, cuatro Python, 588 mensajes gettext y compilación
optimizada. La [CI nativa](https://github.com/frandavid100/leaf/actions/runs/41439845294) y la [validación instalada](https://github.com/frandavid100/leaf/actions/runs/41442198035) también han pasado, incluidas actualización,
portales, impresión y Orca automatizada. [Alcance](docs/ISSUE_SWEEP_ALPHA19.md) ·
[evidencia](docs/ISSUE_SWEEP_ALPHA19_VALIDATION.json).

#35 se reprodujo realmente en GNOME 49 (`18,75` acababa en `75`) antes de corregir
el filtro numérico y el foco de los delegados durante la repaginación. Ambos
campos se verificaron con teclado real; no equivale a prueba humana Wayland/IME.
Estado al terminar: 41 cerrados y 18 abiertos. Las funciones nuevas,
los límites del motor y los informes posteriores conservan su alcance propio.

## Entrega anterior: alpha.18

[Alpha.18, Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.18),
publicada y verificada el 2 de noviembre de 2026. Fuente y tag:
`a8207cbdb5b11f050373287a44aecd835aefd50c`.
ID `io.github.frandavid100.leaf`, runtime GNOME 49; actualización desde alpha.17.
Bundle: 10.956.624 bytes; SHA256
`2c3aeff8a530f7ba644825a099922f53c30a93b743487e79146fde9974c783e5`.
Descarga pública de los ocho assets, sumas y fuente comprobadas.
No hay feed automático ni publicación en Flathub.

**Diez informes cerrados:** #5, #6, #14–#16, #30 y #40–#43. Incluye gestos de
ratón de tablas, idioma/revisión heredados, Texto compacto, iconos GNOME exactos,
selección nativa y título activo según caret. [Alcance](docs/ISSUE_SWEEP_ALPHA18.md)
y [evidencia](docs/ISSUE_SWEEP_ALPHA18_VALIDATION.json).

123 pruebas Rust, cuatro Python, 587 mensajes gettext y la ronda gráfica nativa
completa superadas. Flatpak instalado: actualización/reinstalación, portales,
entrada de texto, impresión, enlaces y regresión Orca/AT-SPI automatizada.
[CI completa](https://github.com/frandavid100/leaf/actions/runs/41402369921).
No equivale a pruebas humanas de todos los documentos o dispositivos.

DOCX conserva la herencia y las excepciones de revisión. ODT aplana la herencia
de idioma; Leaf exige DOCX para revisión predeterminada desactivada o excepciones
explícitas de revisión activada que ODT perdería. No se alteró WordCraft.

**Estado histórico tras alpha.18: 32 cerrados y 20 abiertos, de 52 informes.** Quedaban
#1, #2, #7 y #9 de la pasada inicial. Durante el trabajo aparecieron #44–#52 y
se reabrieron #18, #25, #26, #28, #34, #35 y #37. Sus nuevos requisitos quedaron
pendientes de aquella entrega; alpha.18 no los daba por resueltos. La cuarta
revisión y su resultado actualizado están descritos arriba.

La [alpha.17 anterior](docs/ISSUE_SWEEP_ALPHA17.md) y sus resultados se conservan
como historial. La prueba de flecha de Párrafo se simplificó y ahora envía movimiento
y clic de forma coherente, sin añadir controladores de captura al caso ensayado.

## Entrega acumulada completada

**Navegación** integra los marcadores bajo su título, con plegado de la sección;
los anteriores al primer título aparecen al principio. La tercera pestaña queda
solo para **Comentarios**: cada hilo comparte una burbuja, con separadores entre
mensajes y guías para las respuestas. Se conservan todas las acciones e historial.
[Detalle](docs/BOOKMARKS_COMMENTS.md).

**Ortografía y gramática** conserva la lista lateral, sin instrucciones iniciales
ni bloque de correcciones. Pulsar una fila solo selecciona y muestra el texto.
Se retira la entrada del menú de documento; F7 y el selector lateral la abren.
Las correcciones siguen en el contexto del texto; la nueva pasada cambia la
revisión a herencia por documento con preferencias solo para documentos nuevos. [Detalle](docs/PROOFING_SIDEBAR.md).

**Propiedades → Información** permite editar título, asunto, autor, palabras clave
y categoría. Aplicar confirma metadatos, página y encabezados en un solo paso de
deshacer; Cancelar descarta el formulario. El autor del documento no cambia la
identidad de nuevos comentarios. [Uso](docs/USER_GUIDE_ES.md).

DOCX conserva los cinco campos y los hilos. El exportador ODT fijado **omite
comentarios y categoría, y convierte intervalos de marcador en posiciones**.
Leaf impide perder comentarios/categoría al guardar ODT, propone DOCX y conserva
recuperación si el autoguardado directo es incompatible. Los otros cuatro campos
se conservan en ODT. Motor, dependencias, vendor y parches permanecen intactos.
La reproducción upstream está documentada; no se ha abierto una issue desde aquí.

La revisión anterior de organización y continuidad está terminada y validada:
[revisión](docs/MAINTAINABILITY_REVIEW.md), [evidencia](docs/MAINTAINABILITY_VALIDATION.json).

## Qué está implementado

| Área | Estado y detalle |
| --- | --- |
| Documentos | DOCX/ODT, edición, formato, estilos, tablas, imágenes en línea, deshacer/rehacer y guardado. [Guía](docs/USER_GUIDE_ES.md) |
| Página | Columnas, propiedades, encabezados/pies, campos de página y opciones por sección. [Encabezados](docs/HEADERS_FOOTERS.md), [opciones e idioma](docs/PAGE_OPTIONS_PROOFING.md) |
| Revisión y anotaciones | Lista lateral de errores, revisión al escribir en Preferencias, marcadores e hilos de comentarios. [Revisión](docs/PROOFING_SIDEBAR.md), [anotaciones](docs/BOOKMARKS_COMMENTS.md) |
| Interfaz | Regla superpuesta, historiales y repetición de formato; GTK4/libadwaita, pestañas/ventanas, búsqueda/reemplazo, miniaturas/títulos, contexto nativo y controles adaptables. |
| Salida y recuperación | PDF, impresión por portal y preferencias de autoguardado/recuperación. [Impresión](docs/KEYBOARD_PRINTING.md) |
| Idiomas y apariencia | 678 mensajes gettext, inglés/español y páginas oscuras. [Traducciones](docs/TRANSLATING.md) |
| Accesibilidad | Texto normal, cursor y selección mediante GtkAccessibleText; prueba real AT-SPI/Orca automatizada. [Alcance](docs/ACCESSIBLE_DOCUMENT_TEXT.md) |
| Rendimiento | Miniaturas acotadas, invalidación de capas afectadas y búsqueda cerrada sin trabajo redundante. [Mediciones](docs/HOST_PERFORMANCE_AUDIT.md) |

## Límites que siguen abiertos

- Fidelidad de documentos complejos; [ODT](docs/ODT_PRESERVATION_2026-10-31.md)
  y [límites observados con Writer](docs/INTEROPERABILITY_AUDIT.md).
- Columnas: David comunica fallos de anchuras desiguales y separadores al aplicar
  columnas a texto seleccionado; también los observa en WordCraft. [Informe humano
  y alcance de las pruebas](docs/TEST_RESULTS.md). Pendiente de reproducción mínima;
  conservar propiedades DOCX no demuestra una maquetación visual correcta.
- Imágenes flotantes, tiradores de tablas, fórmulas y otras funciones avanzadas
  aún no implementadas. No se consideran tareas autorizadas por esta lista.
- Semántica accesible completa de tablas/imágenes/campos y operaciones AT
  avanzadas; ampliaciones y validación humana aplazadas por el usuario.
- Confirmación humana de Wayland/escalado fraccionario/multimonitor,
  impresora física y sesiones largas, cuando haya usuarios disponibles.

## Quién tiene lo pendiente

David ha aportado la captura principal del README, incorporada con su
[procedencia](docs/screenshots/README.md). La presentación pública y la guía de
instalación se han simplificado; las notas de versiones siguen enlazadas en
[docs](docs/RELEASE_NOTES.md). El alta de Weblate sigue pendiente de David.
No sustituir capturas públicas por otras de CI ni repetir el intento de acceso
en el navegador remoto.
Las traducciones por PR ya son posibles. La preparación de Flathub requerirá
revisar sus requisitos vigentes y participación humana; no se inicia ahora.

Consultar las incidencias actuales en GitHub; no inferir su estado de este resumen.
La [hoja de ruta](ROADMAP.md) distingue propuestas y trabajo aplazado: no añade
funciones automáticamente ni bloquea el desarrollo esperando probadores.
