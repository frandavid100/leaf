# Alpha.40 — panel inicial y barra de formato

Published and verified: #218, #219, #220, #221, #222 are completed. See [published identity and validation](../LATEST_RELEASE.md).

## Comportamiento

El folio conserva su posición normal. Inicio ocupa el panel lateral, con las
recuperaciones primero y las plantillas después. Cada fila presenta una miniatura
proporcionada, nombre y, para recuperaciones, fecha. La carpeta disponible mantiene
un botón y una explicación aunque no contenga documentos compatibles.

El primer contenido muestra Navegación; eliminarlo devuelve Inicio. Los párrafos
vacíos y el formato no impiden volver, mientras que tablas, imágenes y objetos
siguen contando como contenido. La transición usa el Stack nativo del panel y no
compensa ni desplaza la página. El título y la visibilidad completa de las dos
pestañas inferiores siguen la vista actual. Se respetan las animaciones desactivadas.

Descartar una recuperación exige confirmar el documento concreto. La operación
usa `file.recover` y comprueba la identidad y fecha de la copia antes de borrarla;
el motor rechaza los registros en uso. No se ofrece un Deshacer ficticio para un
borrado permanente. Cancelar conserva los bytes y no modifica el documento abierto.

La barra inferior pertenece al editor y conserva cuatro grupos separados:
Estilo, Párrafo, Texto y Elementos. Deshacer/Rehacer siguen en la cabecera.
El nombre del estilo abre el selector y su flecha abre acciones sobre ese estilo;
una selección mixta no permite actualizar un estilo arbitrario. Las vistas previas
y los nombres se conservan sin etiquetas de tipo.

Alineación, viñetas, numeración, negrita, cursiva y subrayado tienen accesos
directos. Los interruptores mixtos muestran un signo visible y estado accesible.
Imagen y Tabla abren sus herramientas existentes; insertar sigue siendo una acción
explícita dentro del panel. En estrecho se ocultan accesos, manteniendo las
operaciones en el menú correspondiente. No se reparentan menús abiertos al redimensionar.

## Fuentes y validación

Comprobaciones locales: 219 pruebas Rust y 15 Python correctas; formato de Leaf,
900 mensajes completos en inglés/español, metadatos bilingües, enlaces del proyecto,
auditoría de diferencias y compilación GTK de comprobación correctos. La ejecución
gráfica y la instalación se validan por separado en CI; no se dan por realizadas
por el mero éxito de estas comprobaciones locales.

Los adaptadores mantienen los comandos, documento e historial de WordCraft.
`src/gnome/format_bar.rs` contiene los accesos y sus estados derivados;
`src/gnome/templates.rs` controla la vista inicial y los resultados cancelables.
`src/templates.rs` determina si queda contenido real y `src/recovery.rs` descarta
una copia mediante la API pública. No se cambia el motor ni los permisos.

Se añaden regresiones de contenido vacío/no textual, estilos mixtos con párrafos
vacíos y descarte seguro de recuperaciones. La prueba nativa alpha40 recorre los
controles reales, selecciones mixtas, historial, menú de estilo, redimensionado
con panel fijado, herramientas de objetos, ciclos Inicio/Navegación y confirmación.
Las regresiones anteriores conservan sus comprobaciones útiles; se sustituyen
las expectativas de colocación o cierre irreversible que #218/#219 modifican.

La prueba de carpeta usa un portal OpenURI privado: recibe el descriptor real de
la carpeta elegida y comprueba su ruta, que sea directorio y que no se solicite
escritura. Comprueba respuestas de éxito y error, incluida una ruta con espacios.
No pretende simular una prueba humana de un gestor de archivos concreto.

El entorno local permite compilar y ejecutar las pruebas sin interfaz, pero
rechaza crear sockets de D-Bus/Xvfb. La validación nativa y de las instalaciones
GNOME 49/51 se realiza en GitHub Actions antes de publicar. El estado final,
ejecuciones, fuente y trece archivos públicos se registran tras la verificación
en [la última entrega](../LATEST_RELEASE.md).

## Límites

WordCraft se conserva en 0.16.0 con sus revisiones y parches existentes. #1/#99,
#107/#152, el aplazamiento de #52, la elección de carpetas del portal #171 y la
publicación externa de FlatPark #185 permanecen separados. #217 espera la
aclaración solicitada sobre los colores al pegar desde Chrome. Se mantienen las
capturas públicas a cargo de David y la publicación ocasional de Windows.

La validación automática no equivale a pruebas humanas de GNOME OS, accesibilidad,
impresión física ni gestores de archivos concretos. No se publica una alpha hasta
superar las comprobaciones completas y verificar los mismos bytes de la candidata.
