# Ortografía y gramática en el panel lateral

Alpha.14 introduce la lista lateral y la revisión al escribir en Preferencias.
La revisión de alpha.15 simplifica la lista a navegación, según la prueba de David.
WordCraft permanece intacto.

## Interacción

- Cuarta pestaña, después de Páginas, Navegación y Comentarios. F7 y el selector
  lateral la abren; no hay entrada de revisión en el menú de documento.
- Cada fila muestra texto y motivo. Un clic o activación por teclado selecciona
  el fragmento del documento y lo lleva a la vista. No abre menús ni controles
  adicionales. La lista con resultados no muestra instrucciones al principio.
  Publicar los resultados de un análisis nunca mueve el cursor automáticamente.
- Las correcciones, ignorar todas y diccionario personal siguen en el menú
  contextual del documento y usan los comandos actuales. El panel solo navega.
- Solo Preferencias ofrece revisar ortografía/gramática al escribir. Cambiarlo
  afecta a los documentos abiertos y se conserva para aperturas futuras. F7 y
  Volver a revisar funcionan aunque la revisión automática esté desactivada.
- Al editar, los resultados anteriores se invalidan. Si la revisión automática
  está activa y la pestaña visible, se espera una pausa de 300 ms antes de revisar.
  Si está desactivada, se solicita una revisión manual para renovar la lista.

## Trabajo en segundo plano y límites

Una instantánea derivada alimenta un único trabajador por pestaña. Se agrupan
solicitudes nuevas; identidad, revisión y generación invalidan resultados viejos
tras edición, cambio de idioma/diccionario o sustitución del documento. El cierre
libera temporizadores y callbacks débiles; no se añade un modelo editable paralelo.

Se mantienen las herramientas de WordCraft: gramática basada en sus reglas,
ortografía según idioma/diccionarios disponibles, fragmentos sin revisión y un
límite de 500 avisos por análisis. No se añade un servicio remoto ni dependencias.

## Validación

- Regresión headless: revisión explícita con la automática desactivada; selección
  del error sin cambiar documento, revisión, historial ni estado modificado.
- `--proofing-ui-test`: Preferencias real, lista de varios errores, clic/navegación,
  ausencia de instrucciones/controles de corrección, navegación sin edición,
  revisión manual con live desactivado, actualización automática y sustitución
  del documento durante un análisis para comprobar el descarte de resultados viejos.
- CI ejecuta inglés/español y captura el panel ancho, estrecho y oscuro mediante
  `--proofing-demo`; el Flatpak instalado repite la prueba funcional. Capturas solo
  para QA, sin sustituir las públicas a cargo de David.

La base alpha.14 pasó la validación gráfica nativa en la
[CI de la fuente final](https://github.com/frandavid100/leaf/actions/runs/41341927920),
incluidas las capturas anchas/estrechas/oscuras y el descarte de resultados obsoletos.
El ajuste automático del zoom conserva el panel abierto; solo la navegación explícita
por página cierra la superposición estrecha. El [Flatpak instalado](https://github.com/frandavid100/leaf/actions/runs/41343316411)
ha pasado la misma prueba funcional y la regresión Orca/AT-SPI.
[Alpha.14 está publicada](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.14);
la descarga y sus sumas se han verificado.

La regresión adaptada de alpha.15 se valida de nuevo antes de publicar el paquete.

Código: [panel](../src/gnome/spellcheck.rs), [Preferencias](../src/gnome/workspace.rs),
[adaptador](../src/proofing.rs), [diagnóstico](../src/gnome/diagnostics/spellcheck.rs).
