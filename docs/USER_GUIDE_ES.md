# Usar Leaf

Leaf crea y edita documentos con GTK4/libadwaita y el motor WordCraft.
Es una aplicación experimental: prueba con copias y comprueba los guardados
importantes en otro editor. [Instalación Flatpak](../packaging/INSTALL.md).

## Documentos y guardado

- El menú principal del panel lateral contiene Abrir, pestaña/ventana nueva,
  todas las pestañas, recuperación, preferencias y ayuda.
- Ctrl+O abre; Ctrl+N/Ctrl+T crean otra pestaña; Ctrl+Mayús+N crea otra ventana;
  Ctrl+W cierra la pestaña. Los documentos conservan sus estados independientes.
- El menú de tres puntos corresponde al documento: Guardar, Guardar como,
  Insertar, propiedades, PDF, imprimir y revisión ortográfica, entre otras acciones.
- Ctrl+S guarda y Ctrl+Mayús+S abre Guardar como. DOCX/ODT se eligen por el
  sufijo del archivo; el selector confirma sobrescrituras. La primera propuesta
  de guardado protege el original mediante una copia. Al cerrar con cambios se
  puede cancelar, descartar o guardar. El indicador ● acompaña al nombre.
- Exportar a PDF es independiente del archivo editable y no cambia su destino
  ni historial. Imprimir está en el menú y en Ctrl+P.

Preferencias permite desactivar el autoguardado, usar un archivo de recuperación
(predeterminado) o guardar periódicamente en el propio archivo. Esta última opción
puede modificar el original abierto; actívala conscientemente. Recuperar documentos
está en el menú principal. Una operación cancelada o fallida conserva los cambios.

## Edición e interfaz

Escritura Unicode, selección con ratón/teclado, copiar/cortar/pegar y deshacer/rehacer.
Intro crea párrafo; Mayús+Intro, salto de línea. El portapapeles dentro de Leaf
conserva texto enriquecido, tablas e imágenes; el texto externo también se admite.

Deshacer/Rehacer están a la izquierda de la cabecera; Estilo → Párrafo → Texto,
con iconos, a la derecha. En una ventana estrecha esos controles pasan a la barra
inferior. Los desplegables comparten cabecera, vuelta atrás y chincheta: fijar
permite seguir aplicando opciones sin cerrarlos; abrir otro sustituye al anterior.
El clic derecho abre opciones nativas de texto, tablas o imágenes; sus páginas
de formato quedan contenidas en el menú contextual.

Estilo muestra los estilos del documento con búsqueda, aplicación y gestión.
Párrafo reúne alineación, listas, sangría, interlineado, bordes/sombreado y columnas.
Texto contiene fuente, tamaño, formato e idioma. Las opciones avanzadas están
un nivel dentro de sus paneles. Columnas admite el alcance que ofrece su editor;
las anchuras desiguales tienen limitaciones de conservación en ODT.
También hay [fallos visuales comunicados](TEST_RESULTS.md) con anchuras desiguales
y separadores al aplicar columnas a texto seleccionado. La conservación de sus
propiedades en DOCX no garantiza que esa disposición se dibuje correctamente.

## Marcadores y comentarios

Se añaden desde Insertar o el clic derecho. La tercera pestaña lateral muestra
los marcadores por nombre y los comentarios con sus respuestas. Pulsa un marcador
o «Ir al comentario» para seleccionar su texto. Los marcadores requieren un nombre
único de hasta 40 caracteres, empezando por letra, sin espacios.

En cada comentario puedes responder; su menú permite editar, resolver/reabrir o
eliminar. La eliminación pide confirmación e incluye las respuestas. Deshacer y
rehacer conservan el historial. Ctrl+Mayús+F5 añade marcador; Ctrl+Alt+M añade
comentario; Ctrl+Intro envía el comentario y Escape cancela el diálogo.

**Guarda los comentarios en DOCX.** El exportador actual de WordCraft no los
conserva en ODT: Leaf bloquea ese guardado y propone DOCX. Si el autoguardado directo
apuntaba a ODT, mantiene una copia de recuperación. Los marcadores ODT conservan
nombre y posición, pero no el intervalo seleccionado. [Alcance](BOOKMARKS_COMMENTS.md).

## Ortografía y gramática

La cuarta pestaña lateral muestra una lista de errores. Pulsa una fila para ir al
texto y seleccionar el fragmento. La lista solo navega; las sugerencias y las
opciones de corrección siguen en el menú contextual del texto.
F7 abre la pestaña; el botón de volver a revisar actualiza la lista manualmente.

**Preferencias → Ortografía y gramática** contiene la opción de revisar mientras
se escribe. Al activarla se actualizan los subrayados y la lista visible tras una
pausa breve. Al desactivarla puedes seguir revisando manualmente con F7. Una
edición invalida las posiciones anteriores para impedir correcciones equivocadas.
Se conservan los idiomas por fragmento y la exclusión de revisión. Los idiomas
sin diccionario instalado se omiten y la gramática usa las reglas de WordCraft.

## Navegar y buscar

El panel lateral se abre por defecto y ofrece cuatro pestañas: Páginas, Navegación,
Comentarios y Ortografía y gramática. Navegación integra los marcadores bajo su
título y los oculta al plegar la sección; los anteriores al primer título se
muestran al principio. Cada hilo de comentarios comparte una burbuja con
separadores entre mensajes. La lupa activa
la búsqueda (Ctrl+F); Ctrl+H muestra reemplazo. Las opciones incluyen distinguir
mayúsculas y reemplazar. Cada resultado muestra contexto y página; se puede
reemplazar uno o todos. La consulta se conserva al cerrar y se actualiza al abrirla.

Las páginas usan desplazamiento continuo y disposición doble cuando caben al
zoom actual. Los controles +/− y su menú ofrecen zoom, 100 % y ajustar al ancho.
La apariencia de páginas y miniaturas sigue el modo claro/oscuro de GNOME.

## Insertar y configurar

Insertar, en los tres puntos, contiene tabla, imagen, marcador, comentario,
encabezado, pie y número de página. Las tablas e imágenes se editan desde su menú
contextual; no hay botones
permanentes Tabla/Imagen en la barra. Tab/Mayús+Tab recorren celdas y Tab al final
puede añadir una fila. Las imágenes son en línea, con tamaño/proporción, sustitución
y eliminación; no se ofrece posicionamiento flotante.

Encabezados y pies se editan directamente en la página; Escape vuelve al cuerpo.
Propiedades del documento reúne papel, orientación, márgenes y opciones por sección:
variantes, distancias, vínculos y numeración. [Uso y límites de encabezados](HEADERS_FOOTERS.md)
y [opciones de página/idioma](PAGE_OPTIONS_PROOFING.md).

F7 abre la revisión local de ortografía/gramática. Hay herramientas inglesas y
diccionarios españoles incluidos; otros idiomas pueden carecer de diccionario.
Texto → Idioma del texto permite aplicar idioma o excluir revisión.

## Acceso y comprobación

F6/Mayús+F6 cambian entre documento, panel y formato; F10 abre el menú principal.
La ayuda y los atajos del teclado están en el menú principal. Orca dispone de
acceso inicial al texto normal, cursor y selección; no implica semántica completa
de tablas, imágenes o campos. [Alcance de accesibilidad](ACCESSIBLE_DOCUMENT_TEXT.md).

La fidelidad de archivos complejos, fuentes y paginación necesita más muestras.
[Guía para probar copias](TESTING.md) · [Estado y límites](../PROJECT_STATUS.md).
Los tests automáticos no sustituyen la experiencia humana en cada entorno.

Para modificar el programa: [arquitectura](ARCHITECTURE.md), [contribuir](../CONTRIBUTING.md)
y [continuar en otra conversación](CONTINUING.md). El [historial](history/README.md)
conserva las versiones anteriores de esta guía y del plan de desarrollo.
