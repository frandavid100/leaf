# Alpha.31: última pasada de issues

David autoriza el 7/11/2026 resolver los informes verificables y publicar una
release automáticamente, sin un nuevo visto bueno, después de las pruebas
nativas e instaladas. Esta autorización sustituye el requisito de aprobación
de las candidatas alpha.31 anteriores para esta entrega.

## Correcciones en validación

| Issue | Cambio | Regresión |
| --- | --- | --- |
| #135 | Hacer visible el selector Navegación antes de seleccionarlo en GtkStack; GTK rechaza seleccionar un hijo oculto. | Recorrido Revisión → Navegación, selector real mapeado, modelo de dos opciones, filtros y scroll independientes, búsqueda y pestañas. |
| #155 | Invalidar tras instalar la sesión nueva y programar el trabajador aunque Revisión esté oculta. | Apertura con panel lateral oculto y edición desde Navegación; resultados ya preparados antes de abrir Revisión, identidad/revisión e historial conservados. |
| #149 | Orden solicitado, Bordes/Sombreado separados y repetición independiente, iconos/unidades, paleta GTK completa. | Recorrido de subpáginas, selección de color personalizada con teclado/ratón, propiedades e historial; controles claros/oscuros a escala 1×/2×. |
| #150 | Tamaño en una fila nativa plana con icono/etiqueta; pie de opciones compacto y coherente con Párrafo. | Se conserva la regresión de edición de fuente/tamaño y recorridos de formato. |
| #151 | Seleccionar/Insertar/Eliminar/Combinar/Dividir, propiedades y configuración de división independientes; creación al final. | Activación GTK de Insertar fila, estructura canónica y un Deshacer; estado sin tabla y orden del cuerpo compartido. |

No se modifican WordCraft, render, dependencias, parches ni el controlador original
de regresión de fuente/pegado/canvas. Los menús compartidos conservan cierre/fijación,
selección y comandos del motor. Catálogos inglés/español completos.

## Informes que conservan trabajo

| Issues | Resultado y límite |
| --- | --- |
| #1, #2 | Columnas: trabajo de WordCraft en otra conversación. Esta pasada no cambia el motor. |
| #152 | Reproducido en el motor fijado `5cd53f184aab11def13b7fced18727159b9bc96a`: párrafo vacío Heading1 seguido de texto normal; Delete desde el vacío, Backspace al inicio del siguiente y selección del salto trasladan Heading1 al texto. `Document::delete_range` conserva las propiedades del primer párrafo incondicionalmente. Requiere corrección del modelo upstream, no formatear después desde GTK. |
| #156 | La ruta pública utilizada por Leaf pasó 2.085 límites de línea en páginas posteriores, tanto en un párrafo largo (36 páginas) como en párrafos separados (21 páginas), sin destino anterior. No reproduce aún el salto informado ni confirma su causa. Falta el caso concreto/estado gráfico del informe; no se cierra por pasar controles sintéticos. |
| #9 | Medidas con Before 0/1/3/5/5,5/6/24: con After precedente 0 el desplazamiento responde a cada valor; con After 5,5 permanece constante hasta Before 5,5 y luego aumenta. Se demuestra el comportamiento del máximo de espacios vecinos; no confirma que sea la causa en el DOCX editado Portland-Head-Light(1), aún no disponible. No se impone espaciado aditivo. |
| #99 | Comprobado el DOCX privado Muestra aportado, SHA-256 `f9b223fd6a437819f7735d4f9eea81dd1d8aed1b0400aa8358e23054d1c72049`. El render público actual sitúa en la segunda página la nota en y=756,564–771,813 pt y el footer en y=781,201–796,450 pt: nota por encima, con separación de 9,388 pt. Fuente y render no se publican. Este resultado no reproduce la captura informada, por lo que no se atribuye una corrección ni se cierra el caso. |
| #100 | Muestra muestra referencia y número de nota; la navegación ya existe desde alpha.29. El estado original con números ausentes sigue sin reproducirse; no duplicar la navegación existente ni afirmar un arreglo nuevo. |
| #107 | Se conserva la comprobación de anclaje real y rollback cuando el motor rechaza el destino; no se convierte un movimiento rechazado en un éxito aparente. |
| #52, #84 | Continúan las comprobaciones específicas de proveedor/caché de miniaturas y diccionarios del host; el éxito del paquete aislado no sustituye esos entornos. |

## Reproducción mínima de #152

Con `Session::new(Document::from_text("\nBody paragraph."))`, ejecutar
`para.style {"style":"Heading1"}` sobre el primer párrafo y `text.delete`.
El párrafo superviviente contiene el texto del segundo y `style: Heading1`.
Repetir con `caret.set` a `Pos::body(1,0)` y `text.backspace`, o seleccionar
`Pos::body(0,0)..Pos::body(1,0)` antes de borrar, produce el mismo resultado.
El control no depende de GTK, selección pendiente ni del panel de estilos.

## Identidad y validación

La validación final y la identidad de los archivos publicados se completarán
tras las comprobaciones nativas, GNOME 49 y FlatPark/GNOME 51. Solo se cerrarán
los issues de las correcciones que superen ese proceso.
