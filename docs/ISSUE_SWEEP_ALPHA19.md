# Cuarta revisión de issues — candidata alpha.19

Petición de David: intentar #35, #34, #44–#46, #28, #25, #26 y #47.
Base: `719907697ab4ecdb5c081f758b454a4ee58ec6d5`, rama main. Se conserva el
README actualizado para Weblate y su PR de traducciones; no se integra esa PR
como parte de esta revisión.

## Cambios de Leaf

| Issue | Implementación y aceptación |
| --- | --- |
| #34 | Ancho útil y acotado al abrir las listas, propagación de ancho en las vistas previas y tooltip con el nombre completo. Cada familia aparece una vez en su tipografía y tamaño actual. Los nombres largos pueden truncarse con tooltip; ya no se reduce la lista a una tira de puntos. |
| #44–#46 | Ctrl+K desde el documento crea o edita. Se resuelve el rango contiguo completo del enlace, incluidas líneas visuales y fragmentos de formato. El clic derecho conserva el destino pulsado con protección por documento/revisión. Insertar texto/enlace es un solo comando deshacible; abrir/cancelar no modifica el contenido. Hover muestra URL real como texto plano, con ajuste de línea; el área del enlace se contrasta con sus rectángulos reales, sin activar zonas vacías. |
| #28 | El tercer clic conserva la selección de párrafo del motor durante el arrastre. Extiende y revierte por párrafos completos, conserva la marca de final y usa el mismo camino durante autoscroll/cruce de páginas. Se conservan el arrastre de caracteres y el de palabras. |
| #25 | Navegar a un título usa las posiciones reales de hasta tres líneas precedentes, incluido el espaciado de párrafo. Limita el contexto a un cuarto de la vista para conservar visible el título y su sección, con alternativa acotada en límites de página/documento; no usa un margen fijo en píxeles ni la altura de una fuente de título grande. |
| #26 | Menú de revisión centrado en toda la fila, con márgenes simétricos y separación del borde derecho; conserva acciones y destino de cada entrada. |
| #47 | Peso normal aplicado solo a los nombres de marcadores, incluidos estados de foco/hover/selección; no cambia tipografía ni jerarquía de títulos. |

Los cambios usan API públicas. No se modifica WordCraft, Cargo.toml/Cargo.lock,
vendor ni parches. No se implementan regla, historial, imágenes flotantes,
índices, notas o ecuaciones en esta entrega.

## #35 permanece abierto

No se ha reproducido el fallo original de escritura. En el código previo ya
aceptaba entrada la apertura real ensayada, incluida sustitución y escritura
lenta de un tamaño. No se presenta el ajuste de anchura de #34 como corrección
del foco o de los métodos de entrada.

Se amplía la regresión con apertura por ratón, familia instalada, caret/selección,
popover fijado/no fijado, escritura lenta, pegar/borrar, decimal con punto/coma,
ventana estrecha, fuente grande y escala 200 %, y vuelta al documento. Su éxito
no demuestra el recorrido pendiente del equipo del mantenedor, especialmente
Wayland/IME. Se mantendrá abierto y no se cerrará solo por pasar la CI.

## Validación local

- 128 pruebas Rust (cinco nuevas): rango completo y límites de enlaces, un solo
  paso de deshacer, formato conservado, DOCX/ODT independientes, selección por
  párrafos y contexto real de los títulos, con fuente grande y espaciado previo.
- Cuatro pruebas Python; 588 mensajes gettext completos en inglés/español.
- Eventos reales de teclado/ratón: fuentes con selección y caret, ancho normal
  y estrecho, tamaños grandes y escala 200 %; triple clic adelante/atrás,
  autoscroll entre páginas; Ctrl+K, contexto de enlace, cancelar, insertar y hover.
- Pruebas gráficas de títulos, revisión y marcadores superadas; inspeccionadas
  las capturas internas de listas estrechas y grandes. No son capturas públicas.
- Formato, compilación GUI optimizada y CI nativa/Flatpak instalado: en curso.

Las nuevas pruebas están en `tests/alpha19_interactions.rs`,
`src/gnome/diagnostics/sweep19.rs` y `tools/followup19-pointer-test.sh`.
El controlador espera confirmaciones por etapa con límite de tiempo; no usa
cambios programáticos de texto como sustituto de teclear.

La validación instalada usa el cache ya accesible al Flatpak para esas
confirmaciones, sin ampliar los permisos del paquete. Se comprueba actualización
desde alpha.18, persistencia, portales, impresión y Orca existentes. No se afirma
prueba humana de GNOME OS, Wayland, impresora física o todos los documentos.

## Publicación y cierre

Pendientes del resultado de CI nativa y paquete instalado. Los ocho candidatos
se cerrarán solo tras verificar la entrega publicada. No se mueve ningún tag
previo ni se altera ninguna release existente.
