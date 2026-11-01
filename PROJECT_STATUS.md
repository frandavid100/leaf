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

## Trabajo autorizado en esta conversación

Revisión de claridad del código, organización, documentación para contribuidores
y continuidad entre chats, y actualización de la hoja de ruta.

Se reorganiza GTK en módulos privados bajo `src/gnome/`, se separan diagnósticos
bajo `src/gnome/diagnostics/` y se normaliza el formato de las fuentes Leaf.
El ejecutable interno y sus argumentos se conservan. WordCraft, renderizador,
parches existentes, dependencias y comportamiento de documento no se amplían.
Se mantiene alpha.13 como versión pública; esta revisión es de mantenimiento.

**Revisión completada y validada:** fuente `3d5e6826ec6311b23dcaac638a580cf76be2d04e`.
[CI nativa 41320879043](https://github.com/frandavid100/leaf/actions/runs/41320879043):
99 tests Rust, cuatro Python, formato, enlaces, catálogos y regresiones GTK/HiDPI.
[Orca/AT-SPI 41321948699](https://github.com/frandavid100/leaf/actions/runs/41321948699)
también correcto sobre ese ejecutable. Ambos artefactos descargados/verificados.
[Revisión y límites](docs/MAINTAINABILITY_REVIEW.md),
[evidencia](docs/MAINTAINABILITY_VALIDATION.json). La portada, CONTRIBUTING y
AGENTS de main dirigen a la rama actual. No se ha publicado otro instalable por
esta reorganización estructural.

## Qué está implementado

| Área | Estado y detalle |
| --- | --- |
| Documentos | DOCX/ODT, edición, formato, estilos, tablas, imágenes en línea, deshacer/rehacer y guardado. [Guía](docs/USER_GUIDE_ES.md) |
| Página | Columnas, propiedades, encabezados/pies, campos de página y opciones por sección. [Encabezados](docs/HEADERS_FOOTERS.md), [opciones e idioma](docs/PAGE_OPTIONS_PROOFING.md) |
| Interfaz | GTK4/libadwaita, pestañas/ventanas, búsqueda/reemplazo, miniaturas/títulos, contexto nativo y controles adaptables. |
| Salida y recuperación | PDF, impresión por portal y preferencias de autoguardado/recuperación. [Impresión](docs/KEYBOARD_PRINTING.md) |
| Idiomas y apariencia | 562 mensajes gettext, inglés/español y páginas oscuras. [Traducciones](docs/TRANSLATING.md) |
| Accesibilidad | Texto normal, cursor y selección mediante GtkAccessibleText; prueba real AT-SPI/Orca automatizada. [Alcance](docs/ACCESSIBLE_DOCUMENT_TEXT.md) |
| Rendimiento | Miniaturas acotadas, invalidación de capas afectadas y búsqueda cerrada sin trabajo redundante. [Mediciones](docs/HOST_PERFORMANCE_AUDIT.md) |

## Límites que siguen abiertos

- Fidelidad de documentos complejos; [ODT](docs/ODT_PRESERVATION_2026-10-31.md)
  y [límites observados con Writer](docs/INTEROPERABILITY_AUDIT.md).
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

No hay incidencias abiertas verificadas en GitHub al revisar esta página.
La [hoja de ruta](ROADMAP.md) distingue propuestas y trabajo aplazado: no añade
funciones automáticamente ni bloquea el desarrollo esperando probadores.
