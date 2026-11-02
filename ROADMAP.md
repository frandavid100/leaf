# Hoja de ruta de Leaf

Actualizada: 2 de noviembre de 2026. [Estado actual](PROJECT_STATUS.md) ·
[Decisiones](docs/DECISIONS.md) · [Historial](docs/history/README.md).
Esta es la lista vigente; los planes antiguos archivados no añaden tareas.

## Sexta revisión en curso: WordCraft 0.14.0 y alpha.21

Autorizada la actualización del motor. Copia nueva de formatos 0.14.0 con los
cuatro parches ODT reaplicados; 141 pruebas Rust y release optimizada de GitHub
correctos. #18 cerrado tras completar su revisión visual instalada. #48, #51 y
#61 están implementados, con validación gráfica/instalada pendiente y una
regresión de anchura a tamaño grande detectada por CI en corrección.
[PR #62](https://github.com/frandavid100/leaf/pull/62) ·
[Alcance y límites](docs/ISSUE_SWEEP_ALPHA21.md).

## Quinta revisión: alpha.20 publicada

Seis cierres verificados (#49–#51, #58–#60): cursor, historial/Repetir y presentación
lateral. Regla (#48) y selector GNOME (#18) implementados y probados automáticamente;
la comparación visual instalada sigue pendiente tras desconectarse el entorno.
141 pruebas Rust, cuatro Python, 678 mensajes y CI nativa/instalada correctos.
ODT queda protegido ante cuatro pérdidas estructurales adicionales; #53–#56 no
se consideran completos. Estado: 47 cerrados / 12 abiertos.
[Alcance](docs/ISSUE_SWEEP_ALPHA20.md) · [evidencia](docs/ISSUE_SWEEP_ALPHA20_VALIDATION.json).
Motor/dependencias intactos; no se amplía Weblate, Flathub, Orca ni el corpus.

## Completado: cuarta revisión y alpha.19

Publicada y verificada: nueve cierres (#25, #26, #28, #34, #35 y #44–#47).
128 pruebas Rust, cuatro Python, 588 mensajes gettext y CI nativa/Flatpak
instalado completas. #35 se reprodujo en el runtime instalado y se verificó
tras corregir entrada y foco. [Alcance](docs/ISSUE_SWEEP_ALPHA19.md) ·
[evidencia](docs/ISSUE_SWEEP_ALPHA19_VALIDATION.json).

Quedan 18 informes abiertos al terminar; consultar GitHub antes de elegir
otra tarea. No ampliar automáticamente con funciones nuevas, cambios del motor,
Weblate, Flathub o nuevas etapas de Orca.

## Consolidación en main y preparación de Weblate

Integración de los historiales de ApexDesk y Leaf en `main`, con archivo del
main anterior y `wordcraft-saving` conservada. Referencias de desarrollo, CI,
contribuciones y configuración preparada de Weblate actualizadas. No es una
nueva release. [Revisión y validación](docs/MAIN_MIGRATION.md) ·
[Activación de Weblate a cargo de David](docs/WEBLATE_SETUP.md).

## Completado: tercera revisión y alpha.18

Publicada y verificada. Diez informes cerrados de los catorce iniciales: #5, #6,
#14–#16, #30 y #40–#43. 123 pruebas Rust, cuatro Python y CI nativa/Flatpak
instalado completas. [Alcance y pendientes](docs/ISSUE_SWEEP_ALPHA18.md) ·
[evidencia](docs/ISSUE_SWEEP_ALPHA18_VALIDATION.json).

Estado histórico tras alpha.18: 32 cerrados y 20 abiertos, de 52. Los cuatro
pendientes de la pasada inicial eran #1, #2, #7 y #9. Los nueve nuevos (#44–#52)
y siete reabiertos (#18, #25, #26, #28, #34, #35, #37) conservaron sus requisitos
para revisiones posteriores.
No se incorporaron esas ampliaciones a alpha.18 ni se cerraron por pruebas de
requisitos anteriores. No se actualizaron motor/dependencias de WordCraft,
Weblate, Flathub ni main.

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

## Completado: segunda revisión y alpha.17

Publicada y verificada: 18 informes resueltos (#12, #18, #19, #22, #25–#29,
#31–#39). La entrega incluye selección por palabras y con Mayús, entrada de texto,
enlaces utilizables, menús de revisión, navegación y refinamientos nativos.
117 pruebas Rust y CI nativa/Flatpak instalado completas, incluida actualización,
portales y Orca automatizada. [Resultado](docs/ISSUE_SWEEP_ALPHA17.md) y
[evidencia](docs/ISSUE_SWEEP_ALPHA17_VALIDATION.json).

Al terminar aquella entrega quedaban 14 issues abiertos, incluidos #40–#43.
La tercera revisión y el estado actual están descritos arriba. La rama de aquella entrega fue `wordcraft-saving`; el desarrollo actual está en `main`.

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
| Capturas públicas | Captura principal proporcionada por David e incorporada al README; conservar procedencia y no sustituir por imágenes de CI. |
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
