# Estado actual de Leaf

Actualizado: 1 de noviembre de 2026. Esta página describe el presente;
el [historial](docs/history/README.md) conserva entregas y decisiones anteriores.

## Dónde trabajar

- Repositorio: https://github.com/frandavid100/leaf
- Rama activa y destino de PR: **`wordcraft-saving`**. `main` conserva el antiguo
  prototipo ApexDesk; su README solo dirige al proyecto actual.
- [Cómo retomar el proyecto](docs/CONTINUING.md), [arquitectura y mapa de archivos](docs/ARCHITECTURE.md),
  [decisiones vigentes](docs/DECISIONS.md), [plan de trabajo](ROADMAP.md).

## En preparación: alpha.17

Nueva revisión autorizada de los 28 issues abiertos. Candidatos: #12, #18, #19,
#22, #25–#29 y #31–#39. #30 recibe mejoras parciales. Cierre y publicación
pendientes de validación nativa/instalada. [Alcance](docs/ISSUE_SWEEP_ALPHA17.md).

## Última versión publicada

[Alpha.16, Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.16).
ID `io.github.frandavid100.leaf`, runtime GNOME 49, actualización desde alpha.15.
Tag, paquete y fuente nativa `a7e3f349f8a4a2fdb3033acaecbc36d20d5f3cd2`.
Bundle: 10.894.408 bytes,
SHA256 `e1f08d048733bcdc6725066c81785df8bd65775db67e574242947fb157993120`.
Descarga pública, las ocho sumas/digests de assets y fuente verificadas.
No hay feed automático de actualizaciones ni publicación en Flathub.

**32 issues revisados**: los 24 iniciales y ocho añadidos durante el trabajo.
**14 cerrados; 18 abiertos**, incluido #12 parcialmente mejorado. El resto conserva
su trabajo pendiente, con notas de revisión en cada issue.
[Resultado por issue, evidencia y límites](docs/ISSUE_SWEEP_ALPHA16.md).

113 pruebas Rust, cuatro Python, 590 mensajes fuente gettext y la ronda gráfica
nativa completa superadas. Flatpak instalado: controles nuevos, actualización
desde alpha.15, reinstalación con datos, guardado/reapertura real mediante portal,
impresión y regresión Orca/AT-SPI.
[CI nativa e instalada](https://github.com/frandavid100/leaf/actions/runs/41357492415).
No equivale a pruebas humanas de todos los documentos, lectores o impresoras.

## Entrega acumulada completada

**Navegación** integra los marcadores bajo su título, con plegado de la sección;
los anteriores al primer título aparecen al principio. La tercera pestaña queda
solo para **Comentarios**: cada hilo comparte una burbuja, con separadores entre
mensajes y guías para las respuestas. Se conservan todas las acciones e historial.
[Detalle](docs/BOOKMARKS_COMMENTS.md).

**Ortografía y gramática** conserva la lista lateral, sin instrucciones iniciales
ni bloque de correcciones. Pulsar una fila solo selecciona y muestra el texto.
Se retira la entrada del menú de documento; F7 y el selector lateral la abren.
Las correcciones siguen en el contexto del texto y la revisión al escribir en
Preferencias. [Detalle](docs/PROOFING_SIDEBAR.md).

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
| Idiomas y apariencia | 600 mensajes gettext, inglés/español y páginas oscuras. [Traducciones](docs/TRANSLATING.md) |
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

David hará las capturas públicas y el alta de Weblate cuando pueda. No sustituir
las capturas con otras de CI ni repetir el intento de acceso en el navegador remoto.
Las traducciones por PR ya son posibles. La preparación de Flathub requerirá
revisar sus requisitos vigentes y participación humana; no se inicia ahora.

Consultar las incidencias actuales en GitHub; no inferir su estado de este resumen.
La [hoja de ruta](ROADMAP.md) distingue propuestas y trabajo aplazado: no añade
funciones automáticamente ni bloquea el desarrollo esperando probadores.
