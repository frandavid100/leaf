# Tercera revisión de issues — alpha.18

Petición de David: otra pasada para resolver y cerrar todo lo verificable. Rama
wordcraft-saving; base publicada alpha.17. Esta fuente está en validación: no
supone una versión publicada ni un issue cerrado.

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
El exportador ODT fijado aplana idiomas resueltos y no conserva valores de
revisión false explícitos. Se mantiene el ODT ordinario; se bloquea guardar como
ODT cuando el documento desactiva revisión por defecto o necesita una excepción
explícita de revisión activada. El mensaje ofrece DOCX. La herencia de idioma
por sí sola sigue siendo una limitación del ODT; no prometer equivalencia DOCX.

#1/#2 dependen de defectos del motor fijado (columnas desiguales y separador).
#7 sigue la investigación de herencia de una historia par ausente; #9 necesita
el documento/reproductor original del espaciado. No cambiar dependencias ni
parches para cerrar informes sin evidencia.

La selección arrastrada de tabla utiliza la geometría de la página inicial,
como el cliente público de referencia. No afirmar selección continua a través
de páginas distintas. La interacción con tablas combina gestos y regiones
complejas; registrar aquí lo que realmente supere las pruebas antes de cerrar.

## Validación

Validación local: 123 pruebas Rust, cuatro Python, 587 mensajes gettext completos,
formato y enlaces. GTK real bajo Xvfb/TCP: controles nuevos, colores nativos,
navegación por caret, revisión automática, Propiedades y regresión de issues.
Ratón con xdotool: bordes, esquina, cancelación, filas, columnas, rectángulos de
celdas, tabla completa, inserciones, movimiento, combinadas y página posterior
con cabecera repetida. Texto comprueba chevrón, opción, reaplicación y toggle
mediante clics reales. Las coordenadas incluyen la transformación de superficie
nativa y las paletas esperan su asignación estable.

La tinta seleccionada conserva coordenadas lógicas incluso al reducir densidad
para páginas grandes: menos de 12 millones de píxeles, dos superficies en caché.
Restaurar Predeterminado borra también idioma directo complejo y de Asia oriental.

Pendiente antes de publicar/cerrar: build release de esta fuente, CI completa y
Flatpak instalado. Dos ejecuciones iniciales se detuvieron por representaciones
JSON incorrectas en las aserciones de color/resaltado (RGB objeto/array, no nombres).
Se corrigieron las pruebas; ninguna de esas compilaciones se distribuyó como release. No se presenta
el éxito de un test de comandos como prueba de interacción de ratón.


La siguiente ejecución superó los controles nuevos y los trece casos de tabla,
pero falló el script antiguo de sangría de Párrafo. La prueba añadía controladores
de captura y enviaba movimiento/clic en clientes separados. Se simplificó el
diagnóstico a coordenadas (sin controladores adicionales) y el script vuelve a
entrar en la flecha y pulsa en una misma invocación. Los metadatos X11 van a otro
log para no sobrescribir el stderr del proceso. Tres ejecuciones locales seguidas
superaron la navegación real a la subpágina con el código de producción.
Los cambios experimentales de controladores y menús se descartaron; no se alteró
la implementación de Párrafo para hacer pasar el test. Falta la CI final completa.
