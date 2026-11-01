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

[Alpha.13, Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.13).
ID `io.github.frandavid100.leaf`, runtime GNOME 49, actualización desde alpha.12.
Tag/paquete `dc37ec8d0a9aef9c9635fe40ca43f7947bb01193`; fuente nativa
`e5b3380f014ee0babf2c6d890f5fb15de43bd486`. Bundle: 10.773.920 bytes,
SHA256 `7914c4b85c6b93e770799b4ba40b8ba8358f9aa728e169e2b2c06046265f5c43`.
No hay feed automático de actualizaciones ni publicación en Flathub.

99 pruebas Rust y regresiones GTK/Flatpak superadas, incluida Orca automatizada.
[Validación y límites de alpha.13](docs/HOST_PERFORMANCE_VALIDATION.json).
No equivale a pruebas humanas de todos los documentos, lectores o impresoras.

## Trabajo autorizado en curso

David ha autorizado subir los cambios acumulados y publicar alpha.14 tras la
validación, incluyendo **revisión en el panel lateral** y **marcadores y comentarios**: creación desde Insertar y el menú
contextual, y una tercera pestaña compartida del panel lateral. Se usa el modelo,
los comandos y el historial de WordCraft, sin modificar motor ni formatos.

La implementación incluye una cuarta pestaña con la lista de errores, navegación
por clic, correcciones y revisión manual con F7. La revisión mientras se escribe
se configura exclusivamente en Preferencias. [Detalle](docs/PROOFING_SIDEBAR.md).

Está preparada y pendiente de validación gráfica: marcadores (crear, navegar y eliminar),
comentarios y respuestas (crear, editar, resolver/reabrir y eliminar), navegación
al texto anotado, nombres únicos y guardas de identidad/revisión en diálogos.
DOCX debe conservar los hilos. El exportador ODT fijado **omite comentarios y
convierte intervalos de marcador en posiciones**; Leaf bloquea guardado ODT con
comentarios y mantiene recuperación si el autoguardado directo resulta incompatible.
[Alcance, pruebas y reproducción para upstream](docs/BOOKMARKS_COMMENTS.md).
Han pasado 105 pruebas Rust, cuatro Python, formato, traducciones, enlaces,
comprobación de tipos GTK y compilación optimizada.
El entorno local impide crear sockets para Xvfb: las pruebas GTK, el Flatpak y
la actualización desde alpha.13 quedan pendientes. No hay una nueva versión
publicada; alpha.13 sigue siendo la pública. La subida y publicación acumulada
ya están autorizadas; no se requiere otra confirmación para esta entrega.

La revisión anterior de organización y continuidad está terminada y validada:
[revisión](docs/MAINTAINABILITY_REVIEW.md), [evidencia](docs/MAINTAINABILITY_VALIDATION.json).

## Qué está implementado

| Área | Estado y detalle |
| --- | --- |
| Documentos | DOCX/ODT, edición, formato, estilos, tablas, imágenes en línea, deshacer/rehacer y guardado. [Guía](docs/USER_GUIDE_ES.md) |
| Página | Columnas, propiedades, encabezados/pies, campos de página y opciones por sección. [Encabezados](docs/HEADERS_FOOTERS.md), [opciones e idioma](docs/PAGE_OPTIONS_PROOFING.md) |
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
