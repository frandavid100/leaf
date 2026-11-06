# Hoja de ruta de Leaf

Actualizada: 6 de noviembre de 2026. [Estado actual](PROJECT_STATUS.md) ·
[Decisiones](docs/DECISIONS.md) · [Historial](docs/history/README.md).
Esta es la lista vigente; los planes antiguos archivados no añaden tareas.

## Pendiente: prueba y aprobación de la candidata alpha.29

Candidata generada y [verificada](https://github.com/frandavid100/leaf/actions/runs/42143216380), con 19 issues resueltos.
Incluye los cambios recientes y conserva abiertos los casos parciales de
[la revisión](docs/ALPHA29_VALIDATION.md). David debe probarla antes de publicar;
la promoción usa los paquetes exactos, sin recompilar. [Proceso](docs/CANDIDATES.md).

## Completado: alpha.28

Autorizada por David. Incluye zoom por sección y miniaturas (#89, #94, #95),
imágenes flotantes (#53), notas al pie (#55), índice automático (#54), leyendas,
referencias cruzadas y capturas públicas nuevas. Se comprueban DOCX/PDF,
historial e interacción. [Publicada](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.28)
y verificada en Flatpak GNOME 49 y archivo FlatPark GNOME 51. Se mantienen las
guardas ODT; la ficha independiente de FlatPark requiere actualización externa.

## Completado: alpha.27 para FlatPark

Publicados archivo relocatable, checksum y registro de la prueba instalada con
GNOME 51, con traducciones y revisión ortográfica verificadas. El bundle GNOME 49
y la batería nativa también superaron sus comprobaciones. [Release](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.27).
La inclusión en el catálogo queda pendiente del empaquetado de FlatPark.

## Completado: alpha.23

Reglas dentro de los cuatro bordes, menú de saltos y navegación por secciones, encabezados y pies pares independientes y pestañas nativas de Propiedades.

## Completado: alpha.22

[Publicada y verificada](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.22).
Incluye la revisión de formato, los comentarios en la barra lateral, Insertar
compartido y los cambios de impresión y optimización. Columnas (#1, #2),
reglas interiores (#64), saltos (#65) y diccionarios (#69) siguen pendientes.

## Completado: integración de WordCraft 0.14 y limpieza del cliente

Autorizada por David tras alpha.21: investigar las novedades y limpiar código,
estructura y formato. API de impresión específica, separación de responsabilidades
y eliminación del recorrido de estilos ocultos; sin nuevas funciones ni release.
[Investigación y validación](docs/WORDCRAFT_014_REVIEW.md).

## Completado: sexta revisión y alpha.21 — WordCraft 0.14.0

Publicada y verificada. Motor 0.14.0 con formatos recopiado y los cuatro parches
ODT reaplicados. Cuatro cierres (#18, #48, #51, #61): selector de tema revisado,
reglas blancas finas transitorias, historial con AdwSplitButton nativo y listas
Fuente/Tamaño acotadas. Corregidos también mínimos de fuentes grandes y captura
de clics por reglas ocultas, con regresiones reales.

141 pruebas Rust, cuatro Python, 678 mensajes gettext y [CI nativa/instalada](https://github.com/frandavid100/leaf/actions/runs/41496070439)
completa. Assets públicos, fuente, sumas y capturas instaladas comprobados.
Estado final: **50 cerrados / 10 abiertos**. #1, #2, #7, #9, #37 y #52–#56
conservan sus requisitos; actualización del motor no equivale a autoría avanzada.
[PR #62](https://github.com/frandavid100/leaf/pull/62) ·
[Alcance](docs/ISSUE_SWEEP_ALPHA21.md) · [Evidencia](docs/ISSUE_SWEEP_ALPHA21_VALIDATION.json).
Weblate #57 permanece separada; sin Flathub, nuevas etapas de Orca ni cambio de
capturas públicas. Historiales y etiquetas anteriores conservados.

## Quinta revisión: alpha.20 (histórico)

Seis cierres verificados (#49–#51, #58–#60): cursor, historial/Repetir y presentación
lateral. Regla (#48) y selector GNOME (#18) implementados y probados automáticamente;
la comparación visual instalada quedó pendiente al desconectarse el entorno y se
completó durante alpha.21.
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
