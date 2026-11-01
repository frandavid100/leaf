# Hoja de ruta de Leaf

Actualizada: 1 de noviembre de 2026. [Estado actual](PROJECT_STATUS.md) ·
[Decisiones](docs/DECISIONS.md) · [Historial](docs/history/README.md).
Esta es la lista vigente; los planes antiguos archivados no añaden tareas.

## Completado: mantenimiento y facilidad de contribución

**Autorizado por David:** revisar estructura y claridad, separar responsabilidades,
facilitar la incorporación de personas y nuevos chats, y actualizar este plan.
La reorganización queda validada: 99 tests Rust, cuatro Python, formato,
traducciones, enlaces, GTK/HiDPI y Orca automatizada. [Informe](docs/MAINTAINABILITY_REVIEW.md).
[Mapa de código](docs/ARCHITECTURE.md) · [Continuidad](docs/CONTINUING.md).

## En curso: alpha.14 acumulada — anotaciones y revisión lateral

**Autorizado por David:** añadirlos desde Insertar y el menú contextual; compartir
una tercera pestaña del panel lateral. Conectar creación, navegación, gestión de
hilos e historial a WordCraft. Validar los controles GTK y la conservación DOCX.
El ODT actual pierde comentarios: impedir esa pérdida en Leaf, documentar la
reproducción para upstream y conservar recuperación. [Detalle](docs/BOOKMARKS_COMMENTS.md).

David ha autorizado además trasladar Ortografía y gramática a una cuarta pestaña
lateral con lista de errores navegable y mover la revisión al escribir a
Preferencias. Subir y publicar una única release tras la validación.
[Alcance de la revisión lateral](docs/PROOFING_SIDEBAR.md).

La protección general frente a cambios externos se ha aplazado para priorizar
esta función visible. No iniciar nuevas etapas de Orca.

## Orden de prioridad después de esta revisión

Estas son propuestas; requieren un fallo concreto o una petición antes de actuar.
Los marcadores y comentarios anteriores son la ampliación elegida actualmente.

| Prioridad | Trabajo | Qué permite darlo por resuelto |
| --- | --- | --- |
| 1 | Fallos reproducibles de pérdida de contenido, guardado o cierres. | Reproducción pequeña, corrección en Leaf y regresión pertinente. Si el defecto es de WordCraft, propuesta upstream. |
| 2 | Problemas de interacción, importación/guardado o rendimiento comunicados por usuarios. | Comparación antes/después y límites documentados; motor intacto en cambios de Leaf. |
| 3 | Mejoras de uso que David elija: p. ej. edición avanzada de tablas o imágenes. | Alcance acordado, integración GNOME y comprobaciones propias de la función. No implementar toda la lista. |

Mientras no haya informes, el mantenimiento y las comprobaciones automáticas
pueden continuar sin exigir una base de probadores. No inventar incidencias:
consultar las incidencias vigentes en GitHub antes de elegir un fallo.

## Aplazado o dependiente de otra persona

| Asunto | Decisión vigente |
| --- | --- |
| Nuevas ampliaciones de Orca | Aplazadas por David. Conservar el soporte y la prueba automática actuales. |
| Pruebas humanas con Orca/braille, impresoras y GNOME OS | Esperar a disponer de usuarios; no bloquear otras tareas ni presentarlas como ya realizadas. |
| Capturas públicas | Las proporcionará David; mantener las actuales hasta entonces. |
| Weblate alojado | David completará la cuenta/autorización. Catálogos PO y aportaciones por PR ya disponibles. |
| Flathub | Más adelante, con revisión de requisitos y colaboración humana. La distribución GitHub Flatpak ya existe. |
| Actualización/optimización de WordCraft | Solo con autorización específica. Proponer allí los cambios del motor; no mezclarlos con optimizaciones de Leaf. |
| Corpus completo DOCX/ODT | Manual/opcional cuando se justifique; no obligatorio antes de cada versión. |

## Base completada

| Entrega | Resultado | Evidencia |
| --- | --- | --- |
| Alpha.13 | Diagnóstico y optimización exclusiva del host Leaf. | [Informe](docs/HOST_PERFORMANCE_AUDIT.md) |
| Alpha.12 | Acceso inicial al texto con Orca. | [Alcance y prueba](docs/ACCESSIBLE_DOCUMENT_TEXT.md) |
| Alpha.11 | Comprobación independiente con Writer y geometría ODT. | [Interoperabilidad](docs/INTEROPERABILITY_AUDIT.md) |
| Alpha.10 | Secciones, revisión ortográfica e idioma del texto. | [Opciones e idioma](docs/PAGE_OPTIONS_PROOFING.md) |
| Alpha.9 | Encabezados, pies y numeración editable. | [Uso y límites](docs/HEADERS_FOOTERS.md) |
| Alpha.8 | Inglés/español y documento oscuro. | [Localización](docs/LOCALIZATION_DARK_MODE.md) |
| Alpha.6 | Corrección de ciclos de vida GTK. | [Auditoría](docs/GTK_LIFECYCLE_AUDIT_2026-10-31.md) |
| Alpha.5 | Impresión y navegación de teclado. | [Impresión/teclado](docs/KEYBOARD_PRINTING.md) |
| Alpha.1–4 | Flatpak, columnas, HiDPI y correcciones de conservación ODT. | [Historial](docs/history/README.md) |

La edición básica y de formato, estilos, tablas/imágenes en línea, propiedades,
PDF, pestañas, búsqueda y recuperación forman parte de esa base. Sus límites
siguen en [PROJECT_STATUS](PROJECT_STATUS.md); «completado» no significa fidelidad
universal ni prueba humana de cada entorno.
