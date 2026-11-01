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

## Completado: alpha.14 acumulada — anotaciones y revisión lateral

Publicada con marcadores/comentarios desde Insertar y contexto en la tercera
pestaña lateral, e hilos conservados en DOCX. Leaf protege el guardado ODT con
comentarios frente a la omisión del exportador. [Detalle](docs/BOOKMARKS_COMMENTS.md).

Ortografía y gramática ocupa la cuarta pestaña lateral con lista navegable;
la revisión al escribir está en Preferencias. [Detalle](docs/PROOFING_SIDEBAR.md).
Han pasado 105 tests Rust, CI nativa completa y Flatpak instalado, actualización
desde alpha.13, portales y Orca automatizada. [Registro](docs/BOOKMARKS_COMMENTS_VALIDATION.json).

La protección general frente a cambios externos sigue aplazada. No iniciar
nuevas etapas de Orca.

## Completado: revisión de issues y alpha.16

Publicada y verificada tras revisar 32 issues (24 iniciales y ocho nuevos).
14 cerrados; 18 abiertos. La sincronización #12 mejora los controles principales,
pero queda abierta por el editor de columnas y la cobertura restante. Columnas,
tablas avanzadas, pares/impares, el caso de espaciado, herencia de revisión y los
ocho nuevos informes conservan sus tareas pendientes.
[Resultado y comprobaciones](docs/ISSUE_SWEEP_ALPHA16.md).

## Completado: refinamiento de paneles y metadatos para alpha.15

Marcadores integrados bajo títulos de Navegación, una burbuja por hilo de
comentarios y revisión lateral solo para navegar. Retirar la entrada de revisión
del menú de documento. Añadir título, asunto, autor, palabras clave y categoría
editables en Propiedades. Publicada tras 108 pruebas Rust, validación nativa y
Flatpak instalado, actualización desde alpha.14, portales y regresión Orca/AT-SPI.
[Comprobaciones](docs/SIDEBAR_REFINEMENT_VALIDATION.json).

## Orden de prioridad después de esta revisión

Estas son propuestas; requieren un fallo concreto o una petición antes de actuar.
La revisión y entrega están completadas. Los issues abiertos conservan sus tareas; no iniciar otra ampliación automáticamente.

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
| Alpha.15 | Navegación con marcadores, burbujas de comentarios, revisión simplificada y metadatos. | [Validación](docs/SIDEBAR_REFINEMENT_VALIDATION.json) |
| Alpha.14 | Marcadores/comentarios y revisión ortográfica lateral. | [Anotaciones](docs/BOOKMARKS_COMMENTS.md), [revisión](docs/PROOFING_SIDEBAR.md) |
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
