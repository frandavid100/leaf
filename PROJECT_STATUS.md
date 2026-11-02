# Estado actual de Leaf

Actualizado: 2 de noviembre de 2026. Esta página describe el presente;
el [historial](docs/history/README.md) conserva entregas y decisiones anteriores.

## Dónde trabajar

- Repositorio: https://github.com/frandavid100/leaf
- Rama activa, compilación y destino de PR: **`main`**. ApexDesk se conserva en
  `archive/apexdesk-main-2026-11-02`; `wordcraft-saving` sigue disponible.
- [Integración de historiales y validación](docs/MAIN_MIGRATION.md).
- [Cómo retomar el proyecto](docs/CONTINUING.md), [arquitectura y mapa de archivos](docs/ARCHITECTURE.md),
  [decisiones vigentes](docs/DECISIONS.md), [plan de trabajo](ROADMAP.md).

## Última versión publicada: alpha.19

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
| Interfaz | GTK4/libadwaita, pestañas/ventanas, búsqueda/reemplazo, miniaturas/títulos, contexto nativo y controles adaptables. |
| Salida y recuperación | PDF, impresión por portal y preferencias de autoguardado/recuperación. [Impresión](docs/KEYBOARD_PRINTING.md) |
| Idiomas y apariencia | 588 mensajes gettext, inglés/español y páginas oscuras. [Traducciones](docs/TRANSLATING.md) |
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
