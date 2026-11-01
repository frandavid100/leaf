# Estado actual de Leaf

Actualizado: 1 de noviembre de 2026. Esta página describe el presente;
el [historial](docs/history/README.md) conserva entregas y decisiones anteriores.

## Dónde trabajar

- Repositorio: https://github.com/frandavid100/leaf
- Rama activa y destino de PR: **`wordcraft-saving`**. `main` conserva el antiguo
  prototipo ApexDesk; su README solo dirige al proyecto actual.
- [Cómo retomar el proyecto](docs/CONTINUING.md), [arquitectura y mapa de archivos](docs/ARCHITECTURE.md),
  [decisiones vigentes](docs/DECISIONS.md), [plan de trabajo](ROADMAP.md).

## Última versión publicada

[Alpha.14, Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.14).
ID `io.github.frandavid100.leaf`, runtime GNOME 49, actualización desde alpha.13.
Tag/paquete `0f9d8dab49b4e60c693012ed929345b12ef606fe`; fuente nativa
`727b3434bc4df2da1ec62c17e04353ac28b87955`. Bundle: 10.629.136 bytes,
SHA256 `1df0c0144ddd4c6559e49bb9a9ee395c86a078fa88f1f5cfe66e0620ca3b7965`.
No hay feed automático de actualizaciones ni publicación en Flathub.

105 pruebas Rust, cuatro Python y la ronda gráfica nativa completa superadas,
incluidos inglés/español, ventanas estrechas/oscuras e HiDPI. El Flatpak instalado
ha pasado los nuevos paneles, actualización desde alpha.13, reinstalación con datos,
arranque de escritorio, portales de documentos/impresión y Orca/AT-SPI automatizados.
[CI nativa](https://github.com/frandavid100/leaf/actions/runs/41341927920) ·
[CI del paquete](https://github.com/frandavid100/leaf/actions/runs/41343316411) ·
[Registro y límites](docs/BOOKMARKS_COMMENTS_VALIDATION.json).
La descarga pública y sus sumas se han verificado. No equivale a pruebas humanas
de todos los documentos, lectores o impresoras.

## Trabajo actual: refinamiento de los paneles para alpha.15

David pide integrar marcadores bajo los títulos de Navegación, dedicar la tercera
pestaña a Comentarios con una burbuja por hilo y separadores, y simplificar
Revisión a navegación sin instrucciones ni bloque de correcciones. Se retira
Ortografía y gramática del menú de documento. Implementado localmente; validación
gráfica y publicación aún pendientes. David añade cinco campos editables en
Propiedades: título, asunto, autor, palabras clave y categoría. Se acumulan en
la misma entrega; no publicar el paquete anterior mientras se implementan.
[Detalle y comprobaciones](docs/SIDEBAR_REFINEMENT_VALIDATION.json).

## Base publicada: entrega acumulada alpha.14

Los **marcadores y comentarios** se crean desde Insertar y el menú contextual,
y comparten la tercera pestaña lateral. Incluye creación, navegación, eliminación,
edición/respuesta de comentarios y resolución/reapertura de hilos, con historial
WordCraft y guardas frente a diálogos obsoletos. [Detalle](docs/BOOKMARKS_COMMENTS.md).

**Ortografía y gramática** ocupa la cuarta pestaña lateral, con lista de errores,
navegación por clic y correcciones. F7 permite revisión manual; revisar mientras
se escribe se configura exclusivamente en Preferencias. Los análisis descartan
posiciones obsoletas. [Detalle](docs/PROOFING_SIDEBAR.md).

DOCX conserva los hilos. El exportador ODT fijado **omite comentarios y convierte
intervalos de marcador en posiciones**; Leaf bloquea el guardado ODT con comentarios,
propone DOCX y conserva recuperación si el autoguardado directo es incompatible.
Motor, revisiones, dependencias, vendor y parches existentes permanecen intactos.
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
| Idiomas y apariencia | 599 mensajes gettext, inglés/español y páginas oscuras. [Traducciones](docs/TRANSLATING.md) |
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
