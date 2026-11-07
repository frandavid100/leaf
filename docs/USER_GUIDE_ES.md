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
  propiedades, PDF, imprimir, zoom y los modos Editar/Sugerir. Elementos está en la cabecera.
- Ctrl+S guarda y Ctrl+Mayús+S abre Guardar como. DOCX/ODT se eligen por el
  sufijo del archivo. Un documento abierto se guarda sobre su archivo; un documento
  nuevo pide su destino la primera vez. Guardar como elige otra copia. Al cerrar con cambios se
  puede cancelar, descartar o guardar. El indicador ● acompaña al nombre.
- Exportar a PDF es independiente del archivo editable y no cambia su destino
  ni historial. Imprimir está en el menú y en Ctrl+P.

Preferencias permite desactivar el autoguardado, usar un archivo de recuperación
(predeterminado) o guardar periódicamente en el propio archivo. Esta última opción
puede modificar el original abierto; actívala conscientemente. Recuperar documentos
está en el menú principal. Una operación cancelada o fallida conserva los cambios.

Para conservar ecuaciones editables, notas al pie, índice automático e imágenes
flotantes con ajuste del texto, usa DOCX. Leaf rechaza guardar esas estructuras
en ODT mientras el exportador actual las pierda o aplane. La recuperación usa
un formato que las conserve. Los cambios de formato sugeridos, movimientos y comentarios también requieren DOCX; las propuestas ordinarias de inserción/eliminación conservan su estructura en DOCX y ODT.

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

Cada botón de Deshacer/Rehacer tiene una flecha para abrir su historial; el menú
del documento contiene las mismas listas. Elegir una fila inferior recorre todos
los pasos hasta ella. Solo la primera muestra Ctrl+Z o Ctrl+Mayús+Z. Si no hay
nada que rehacer, el cuerpo de Rehacer puede mostrar «Repetir última acción»:
aplica el último formato de texto/párrafo admitido al destino actual. Por ejemplo,
aplica 14 pt a otro fragmento seleccionado. Escritura, archivos y operaciones
estructurales no se repiten; un cambio nuevo conserva las reglas normales de
invalidación de Rehacer.

La regla es transitoria: aparece al acercar el puntero a los bordes visibles de
la página; Ctrl+Alt+R permite enfocarla con el teclado. Sus números tienen más
contraste y los arrastres de márgenes, sangrías y tabulaciones usan un imán de
0,5 cm, medido desde el origen correspondiente. La vista previa y el valor
confirmado coinciden; Escape cancela y cada gesto admite Deshacer. Los valores
numéricos o importados no se redondean. Con foco en la regla, Arriba/Abajo eligen
marcador, Izquierda/Derecha lo ajustan (Mayús: 10 pt), Insertar añade tabulación,
Suprimir la elimina y Escape vuelve al texto.

Estilo muestra los estilos del documento con búsqueda, aplicación y gestión.
Párrafo reúne alineación, listas, sangría, interlineado, bordes/sombreado y columnas en filas con nombre y valor actual.
Texto conserva fuente y tamaño editables, opciones y vistas previas. Negrita, cursiva y otros interruptores usan casillas nativas: un estado mixto se activa para toda la selección al pulsarlo. Las flechas abren opciones internas con vuelta atrás; los botones de reaplicar usan el último ajuste aplicado de esa propiedad. Leer otra selección o cancelar no cambia ese recuerdo. Idioma muestra «Predeterminado» junto al idioma efectivo heredado. Los valores válidos de formato se aplican directamente. Los controles principales de los paneles
fijados siguen la selección y muestran valores mixtos. El editor de columnas se
sincroniza al cambiar de sección. Texto también contiene formato e idioma. Las opciones avanzadas están
un nivel dentro de sus paneles. Columnas admite el alcance que ofrece su editor;
las anchuras desiguales tienen limitaciones de conservación en ODT.
También hay [fallos visuales comunicados](TEST_RESULTS.md) con anchuras desiguales
y separadores al aplicar columnas a texto seleccionado. La conservación de sus
propiedades en DOCX no garantiza que esa disposición se dibuje correctamente.

## Marcadores y comentarios

Se añaden desde Elementos o el clic derecho. Los marcadores aparecen bajo su título en Navegación; Revisión reúne comentarios, correcciones y cambios sugeridos, con filtros y una burbuja por hilo. Pulsa un marcador o «Ir al comentario» para
seleccionar su texto. Clicar el texto anclado abre Revisión y resalta sus hilos.
Preferencias permite configurar la identidad de nuevos comentarios y revisiones;
no cambia los autores existentes. Los marcadores requieren un nombre
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

F7 abre las correcciones en Revisión. Cada tarjeta permite seleccionar el texto,
aplicar la propuesta disponible o ignorar solo esa aparición. El menú de la
corrección conserva otras sugerencias, Ignorar todas y Añadir al diccionario.
Una edición invalida los destinos anteriores antes de volver a comprobar.
La ortografía requiere un diccionario compatible para el idioma del texto;
la gramática utiliza las reglas disponibles para ese idioma.
La preferencia de revisión mientras se escribe establece el valor de documentos
nuevos; un documento ya abierto conserva el suyo.

En **Texto → Idioma → Ajustes de diccionarios…** puedes elegir una carpeta con
parejas Hunspell `.aff`/`.dic`, actualizar sus archivos, dejar de usarla o importar
archivos. En Flatpak, el selector concede acceso a la carpeta elegida. Los
diccionarios incluidos y las importaciones siguen disponibles; el idioma del
documento y el de los menús son independientes. Los idiomas sin archivos de
diccionario aparecen atenuados en la lista. La compatibilidad de los archivos se
comprueba al utilizar ese idioma.
[Prueba de la candidata](../tests/fixtures/dictionary84/README.md).

## Cambios sugeridos

El modo Sugerir registra las nuevas ediciones; Editar vuelve a la edición normal.
Revisión muestra las propuestas existentes en ambos modos, con Aceptar/Rechazar
y Deshacer/Rehacer. Resolver un comentario es independiente de aceptar cambios.
Los filtros Todos, Correcciones, Cambios y Comentarios no modifican el documento.

## Navegar y buscar

El panel lateral ofrece Navegación y Revisión. En Navegación, el selector permite
alternar entre Páginas y Estructura; la búsqueda es temporal. F9 muestra u oculta
el panel. Estructura permite filtrar por tipo de elemento y mantiene los títulos
y secciones como contexto. Las imágenes y tablas muestran su leyenda cuando
está disponible; la leyenda también tiene una entrada diferenciada debajo.
Seleccionar una tabla activa esa tabla, y seleccionar una imagen activa la imagen.
Navegación integra los marcadores bajo su título y los oculta al plegar la sección;
los anteriores al primer título se muestran al principio. Cada hilo de comentarios comparte una burbuja con
separadores entre mensajes. La lupa activa
la búsqueda (Ctrl+F); Ctrl+H muestra reemplazo. Las opciones incluyen distinguir
mayúsculas y reemplazar. Cada resultado muestra contexto y página; se puede
reemplazar uno o todos. La consulta se conserva al cerrar y se actualiza al abrirla.

Las páginas usan desplazamiento continuo y disposición doble cuando caben al
zoom actual. El menú de documento reúne +/−, porcentaje/restablecer al 100 % y ajustar al ancho.
La apariencia de páginas y miniaturas sigue el modo claro/oscuro de GNOME.

## Insertar y configurar

Elementos, en la cabecera adaptable y el menú contextual, agrupa tabla, imagen, índice y
ecuación; marcador, comentario, enlace y nota al pie; y los submenús de
encabezados/pies, campos automáticos y saltos. Las tablas e imágenes se editan
desde el mismo panel compartido de Elementos o su menú contextual. Las propiedades
aparecen primero y la acción Insertar permanece al final, también cuando hay un
elemento seleccionado. Sin un destino válido, las propiedades siguen visibles
pero desactivadas. Tab/Mayús+Tab recorren celdas y Tab al final puede
añadir una fila. Los campos automáticos incluyen página, total, autor, título,
nombre de archivo, fecha y hora. Fecha/hora no se recalculan al teclear texto.

Encabezados y pies se editan directamente en la página; clicar fuera de su zona o
pulsar Escape vuelve al cuerpo. El número de página se inserta en la posición
actual y conserva su alineación; el diálogo permite añadir el total de páginas.
Ancho y alto del papel solo se editan cuando se elige tamaño Personalizado.
Propiedades del documento incluye **Información** para editar título, asunto, autor,
palabras clave y categoría. Aplicar confirma todos los cambios; Cancelar los descarta.
El autor del documento no cambia la identidad usada en nuevos comentarios. La
categoría requiere DOCX: si se rellena, Leaf propone ese formato para conservarla.
Las otras pestañas reúnen papel, orientación, márgenes y opciones por sección:
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

El menú principal permite seguir el tema del sistema o elegir claro/oscuro. El
menú de documento reúne zoom y modo Edición/Revisión. Revisión registra los
cambios posteriores; cambiar de modo conserva las revisiones existentes y es
independiente de la revisión ortográfica.

## Enlaces y selección de párrafos

Ctrl+K crea un enlace sobre la selección o inserta texto y destino donde está el
caret. Dentro de un enlace abre su destino completo; también se puede editar
desde el clic derecho sin seleccionar todo el texto. Pasar el ratón muestra la
dirección y el gesto Ctrl+clic. Cancelar conserva el contenido; confirmar se
puede deshacer. Tras triple clic, arrastrar amplía la selección por párrafos.

## Notas al pie, índices y referencias

Elementos → Nota al pie (Ctrl+Alt+F) crea una nota numerada y lleva el cursor a su
texto. Clica su número para ir desde el cuerpo a la nota o desde la nota a la
referencia. Alt+Intro ofrece la misma navegación con teclado. La numeración,
el formato y los reinicios están en Propiedades del documento o de la sección;
los controles que no cambias conservan la configuración heredada.

Elementos → Índice de contenidos crea un índice de los títulos. El diálogo
permite elegir niveles, números de página y enlaces. Clic derecho sobre el
índice → Editar índice de contenidos cambia ese mismo índice y conserva las
opciones importadas que no modificas. Las referencias y el índice se actualizan
automáticamente tras editar y antes de guardar, exportar PDF o imprimir.

Selecciona una imagen y activa **Mostrar leyenda** en sus propiedades de Imagen.
Para una tabla, coloca el cursor en ella y usa la misma casilla en Tabla.
La etiqueta y numeración aparecen directamente debajo; escribe allí la descripción.
Desactivar la casilla oculta la leyenda y conserva su texto para recuperarlo al
reactivarla. Puedes deshacer estos cambios. Al pasar el ratón por el objeto o su
leyenda, o situar el cursor en esta, un fondo tenue indica su relación. Esta ayuda
visual no aparece en el PDF. Las leyendas numeradas requieren DOCX y objetos en
el cuerpo del documento; las leyendas importadas sin una asociación reconocible
siguen siendo texto editable.
Elementos → Enlace permite elegir una URL o un título, marcador, figura o tabla
del documento. Ctrl+clic sigue las referencias internas.

En Imagen puedes poner el texto a los lados o arriba y abajo, ajustar el espacio
y elegir si la imagen se mueve con el texto. Arrastra la imagen para moverla.
Los ocho tiradores cambian su tamaño; las esquinas conservan las proporciones y
los bordes cambian una dimensión. La vista previa muestra el contenido y puede
pasar entre páginas o desplazar el documento. Esc cancela; cada gesto se deshace
en un paso. Si el motor no puede colocar una imagen flotante en la página de un
salto manual, Leaf rechaza el movimiento y conserva su posición original.

Elementos → Ecuación (Alt+=) abre el editor básico UnicodeMath. Admite fracciones,
superíndices/subíndices, raíces y letras griegas. Doble clic o el menú contextual
edita una ecuación existente. No interpreta comandos LaTeX. Las ecuaciones
importadas demasiado complejas para este editor conservan su estructura sin
permitir una edición que la pierda.

Estas estructuras requieren DOCX. Leaf bloquea el guardado ODT que perdería notas,
campos automáticos, ecuaciones o imágenes flotantes. Ajustar al ancho adapta cada
sección a su papel; un porcentaje explícito mantiene la misma escala entre
secciones. Páginas muestra la miniatura activa con su posición física y el total;
redibujar el documento no interrumpe una consulta manual de otras miniaturas.

El motor actual admite un único índice automático por documento. Intentar crear otro muestra esta limitación y conserva el existente; sus propiedades se editan desde Elementos o el clic derecho.
