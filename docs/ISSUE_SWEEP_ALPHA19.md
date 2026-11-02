# Cuarta revisión de issues — alpha.19 publicada

Petición de David: intentar #35, #34, #44–#46, #28, #25, #26 y #47.
Base integrada: `4e212a97721de1913def6d3ce406b828847b7ab4`, rama main. Se conserva el
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

## #35: reproducción instalada y corrección verificada

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
páginas durante la escritura sin perder el foco. La reproducción instalada y
la matriz completa pasan con la corrección. El recorrido original específico de Fuente no se reprodujo por separado;
ambos campos sí se verificaron con eventos reales. No se afirma prueba humana
Wayland/IME.

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
- Formato, compilación GUI optimizada y CI nativa/Flatpak instalado superados.

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

[Alpha.19 publicada](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.19), fuente/tag `1f8694dd8cf51c8f981baf8aa23093ea4e09b625`.
[CI nativa](https://github.com/frandavid100/leaf/actions/runs/41439845294) y [CI instalada](https://github.com/frandavid100/leaf/actions/runs/41442198035) superadas. Descargados los ocho assets
públicos; fuente, tamaños y SHA256 comprobados. Bundle: 10.984.832 bytes;
SHA256 `71656fe9c658fff1f24804f2ee7c2ed9d9086de72919b66053594efa9ffdf39c`. Actualización desde alpha.18, persistencia,
portales, impresión y regresión AT-SPI/Orca superadas en el paquete instalado.
[Evidencia estructurada](ISSUE_SWEEP_ALPHA19_VALIDATION.json).

Cerrados tras verificar la entrega: #25, #26, #28, #34, #35 y #44–#47.
Quedan 18 informes abiertos y 41 cerrados al terminar esta pasada;
las incidencias posteriores y funciones nuevas no se incorporan por aparecer aquí.
El paquete reutiliza el binario validado de `2fd69ff222d2f3f0b5ba9c64289af5d8cd7da2e4`.
La fuente final solo ajusta la conservación de capturas y la referencia del artefacto;
el workflow exige identidad de código, dependencias, pruebas, catálogos, datos y
herramientas antes de reutilizarlo. La reutilización queda registrada en la evidencia.
Las 17 etiquetas y releases previas se conservan, junto al archivo ApexDesk y
`wordcraft-saving`. La PR #57 de Weblate queda intacta y fuera de esta revisión.
