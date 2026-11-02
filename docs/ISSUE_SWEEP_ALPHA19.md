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

## #35: reproducción instalada y corrección candidata

La primera validación del Flatpak en GNOME 49 reprodujo un fallo real: escribir
lentamente `18,75` acababa aplicando `75`. El entorno nativo GTK 4.14 no había
mostrado esa pérdida. La publicación se bloqueó antes de crear alpha.19.

Se desactiva el filtro numérico previo de GTK para que el parser de Leaf reciba
los dos separadores decimales. Se conserva la validación de rango y finitud.
La sincronización respeta el foco del delegado editable y no vuelve a seleccionar
todo el campo al aplicar cada carácter. La reconstrucción de páginas conserva
el foco de los controles en vez de devolverlo incondicionalmente al documento.

La regresión usa apertura por ratón, familia instalada, caret/selección,
popover fijado/no fijado, escritura lenta, pegar/borrar, decimal con punto/coma,
ventana estrecha, fuente grande y escala 200 %, y vuelta al documento. Un
caso adicional selecciona un documento largo y exige que cambie el número de
páginas durante la escritura sin perder el foco. La corrección sigue pendiente
de superar de nuevo la prueba instalada; no se afirma prueba humana Wayland/IME.

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
El controlador espera confirmaciones por etapa con límite de tiempo y drena
los eventos nativos antes de comprobarlas, para no evaluar un clic aún en cola
tras repaginar. No usa cambios programáticos de texto como sustituto de teclear.

La validación instalada usa el cache ya accesible al Flatpak para esas
confirmaciones, sin ampliar los permisos del paquete. Se comprueba actualización
desde alpha.18, persistencia, portales, impresión y Orca existentes. No se afirma
prueba humana de GNOME OS, Wayland, impresora física o todos los documentos.

## Publicación y cierre

Pendientes del resultado de CI nativa y paquete instalado. Los nueve candidatos
se cerrarán solo tras verificar la entrega publicada. No se mueve ningún tag
previo ni se altera ninguna release existente.
