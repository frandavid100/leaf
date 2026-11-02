# Tercera revisión de issues — alpha.18 publicada

Publicada y verificada el 2 de noviembre de 2026:
[alpha.18](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.18).
Fuente y tag: `a8207cbdb5b11f050373287a44aecd835aefd50c`.
Árbol: `f64eb01f30914e42e252ccc4c7d2a59733dc44ed`.
[Validación verificable](ISSUE_SWEEP_ALPHA18_VALIDATION.json).

La pasada inicial revisó 14 informes y cerró diez: **#5, #6, #14, #15, #16,
#30 y #40–#43**. Permanecen #1, #2, #7 y #9. Durante el trabajo entraron nueve
informes nuevos (#44–#52) y se reabrieron siete (#18, #25, #26, #28, #34, #35,
#37). El estado consultado tras estos cierres es **32 cerrados y 20 abiertos,
de 52 informes**. Los dieciséis posteriores se revisaron para continuidad y
siguen abiertos; no se presentan como implementados en alpha.18.

## Implementación

- #14–#16: Preferencias fija idioma y revisión de documentos nuevos. Propiedades
  cambia los valores heredados del documento, sin reescribir estilos ni fragmentos.
  Texto → Idioma ofrece primero No revisar, luego Predeterminado y los idiomas.
  Predeterminado borra las dos propiedades directas; un idioma explícito activa
  revisión incluso si el documento la desactiva. Selección y punto de inserción
  mantienen su alcance; cambios de documento son deshacibles y protegidos.
- #30: la revisión automática usa esos valores efectivos, sin un segundo
  interruptor global. Lista lateral con debounce y descarte por identidad/revisión.
- #40: Texto se organiza en tres filas (B/I/x²; subrayado/tachado; color/resaltado),
  con SplitButton nativos. Las paletas recuerdan valores independientes de la
  selección. Fuente y Tamaño tienen filas propias, Cambio de mayúsculas abre
  opciones y Limpiar formato queda al final, separado. Más opciones conserva las
  herramientas avanzadas sin repetir acciones básicas.
- #41: iconos exactos headings-allcap-symbolic y font-symbolic del Icon Development
  Kit oficial GNOME, incluidos con licencia CC0 y procedencia, para ambas barras.
- #42: selección con fondo y texto del tema GTK, incluidos foco/inactividad.
  Una lista de dibujo pública desechable recolorea solo la tinta seleccionada;
  el documento, salida guardada e historial no cambian. Caché de dos páginas como
  máximo, con identidad, revisión, zoom, densidad y color. No se modifica render.
- #43: el título activo es el último título precedente al caret del cuerpo.
  Navegación actualiza la fila y revela su posición lateral sin activar goto,
  robar foco ni desplazar el documento. Respeta texto anterior al primer título.
- #5/#6: puente GTK para table.resizeDrag/selectAt/moveDrag/insertAt/fitColumn.
  Los hit targets y planes son públicos del motor: bordes, esquina, tirador,
  selección de fila/columna/celda y marcas de inserción. Vista previa sin cambios,
  Escape cancela; un redimensionado completado es una edición deshacible.

## Formatos y límites

DOCX conserva valores heredados y excepciones (incluido noProof=false).
El exportador ODT fijado aplana idiomas resueltos y no conserva ciertos valores
false explícitos. Leaf mantiene el ODT ordinario; requiere DOCX cuando el documento
desactiva revisión por defecto o necesita una excepción explícita que la active.
La herencia de idioma por sí sola sigue siendo una limitación del ODT. No se
modificaron motor, renderizador, dependencias ni parches de WordCraft.

La selección arrastrada de tabla utiliza la página inicial, como el cliente
público de referencia. No se afirma arrastre continuo de selección entre páginas.
La tinta de selección mantiene coordenadas lógicas al reducir densidad para
páginas grandes; menos de 12 millones de píxeles por superficie y dos en caché.

## Validación completada

- 123 pruebas Rust, cuatro Python, 587 mensajes gettext completos, formato y
  enlaces actuales; build GUI release optimizada con panic unwinding.
- [CI nativa e instalada](https://github.com/frandavid100/leaf/actions/runs/41402369921):
  controles nuevos en inglés/español, trece casos de ratón de tablas, cuerpos y
  chevrones de Texto, revisión automática, navegación por caret y regresiones
  anteriores. Ventanas ancha/estrecha, claro/oscuro, HiDPI, cierre y memoria.
- Flatpak instalado GNOME 49: actualización desde alpha.17, datos conservados al
  reinstalar, apertura/guardado mediante portal, impresión, enlaces y regresión
  existente AT-SPI/Orca. No se afirma validación humana de GNOME OS o impresoras.
- Descarga pública de los ocho assets, digests SHA256 y SHA256SUMS comprobados.
  Bundle: **10.956.624 bytes**, SHA256
  `2c3aeff8a530f7ba644825a099922f53c30a93b743487e79146fde9974c783e5`.
  El tag y source-commit.txt coinciden con la fuente ensayada.

## Intentos previos y diagnóstico de ratón

Las dos primeras ejecuciones se detuvieron por aserciones con representaciones
JSON incorrectas de color/resaltado. La tercera pasó los controles nuevos y las
tablas, pero falló el script antiguo de la flecha de sangría. Ninguna compilación
fallida se distribuyó como release.

Se simplificó ese diagnóstico para aportar solo coordenadas, sin controladores
extra de captura. El script vuelve a entrar en la flecha y pulsa en una misma
invocación de xdotool; sus metadatos X11 van a otro log para no sobrescribir stderr.
Pasó tres veces en debug y tres en release, y después en la CI completa. Los
cambios experimentales de controladores y menús se descartaron. No se atribuye
el fallo a un único mecanismo interno de GTK sin prueba adicional.

## Trabajo que continúa abierto

| Informes | Estado concreto |
| --- | --- |
| #1, #2 | Defectos de columnas en la base fijada; requieren solución del motor, sin parches del cliente. |
| #7 | Historia par ausente hereda la predeterminada; reproducción reducida en ISSUE7_TRIAGE.md. |
| #9 | El caso ordinario cambia la geometría; falta aislar el DOCX original con su espaciado contextual/automático. |
| #18 | Se pide un selector de tema establecido (libpanel o patrón oficial), sustituyendo el dibujo actual. |
| #25 | El salto debe dejar dos o tres líneas de contexto, además de la alineación implementada anteriormente. |
| #26 | Centrado vertical y separación del menú de cada entrada de revisión. |
| #28 | El doble clic funciona; falta arrastre con granularidad de párrafo tras triple clic. |
| #34, #35 | Nuevos informes de lista de fuentes ilegible y escritura en Fuente/Tamaño. Las pruebas de recorridos concretos no cierran automáticamente estos informes; requieren su reproducción ampliada. |
| #37 | Se pide AdwInlineViewSwitcher real, con API y mínimo libadwaita 1.7; alpha.18 conserva ViewSwitcher y el mínimo anterior. |
| #44–#46 | Ctrl+K, edición del enlace completo desde cualquier punto y destino visible al pasar el ratón. |
| #47 | Peso regular de nombres de marcadores. |
| #48 | Prototipo de regla interactiva superpuesta, con hover, fade y fijación. |
| #49 | Caret negro con fade basado en la referencia y preferencias del sistema. |
| #50 | Repetir última acción, separado de la pila de Rehacer. |
| #51 | Historiales en controles y menús: se confirmó undo_history/undoTo y redo_label; falta resolver la enumeración completa de Rehacer sin inventar otra pila. |
| #52 | Miniaturas DOCX de GNOME Files: falta identificar miniaturizador, caché y paquete afectado; no hay causa atribuida ni se borró la caché del usuario. |
