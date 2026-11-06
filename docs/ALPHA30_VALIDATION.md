# Alpha.30: alcance, validación y límites

David autoriza esta pasada y su publicación automática después de las pruebas.
Se acumulan los cambios de la candidata alpha.29, que no tuvo release pública.
Motor, renderer, dependencias y parches permanecen en la revisión fijada de WordCraft 0.14.

| Issue | Estado de implementación |
| --- | --- |
| #122 | Visibilidad basada en la imagen seleccionada, ancla visual durante remaquetación y guardas de identidad/revisión para resultados diferidos. |
| #123 | Arbitraje de cursores imagen/tabla/texto, ocho tiradores, selección de imágenes no seleccionadas, movimiento/arrastre y marcadores/márgenes de las reglas. |
| #125 | Correcciones y revisiones comparten la superficie plana de los comentarios; sin sombra de tarjeta elevada ni selección universal. |
| #128 | La miniatura activa deriva del foco de selección maquetado; scroll y hover no cambian el destino. |
| #129 | Clic ordinario sobre el enlace estructurado del índice, puntero y decoración temporal localizada por páginas; sin inferir destino por título o página impresa. |
| #130 | Iconos, propiedades enlazadas, Fuente interna con entrada/previsualizaciones, tamaño expandido, repetición propia de fuente/idioma, Superíndice en Más, Sangrías internas y Bordes/Sombreado compartidos. |
| #121 | Parcial: propiedades visibles/insensibles, explicación mediante tooltip, inserción inferior, transición nativa y herramientas compartidas. La dependencia de #127 sigue pendiente. |
| #124 | Parcial: índice canónico de todos los tipos disponibles, lectura de tablas anidadas y filtro nativo. Filtros con contexto de sección/títulos y selección activa por extensión real de enlaces, objetos y notas. Falta la asociación de leyendas. |
| #126/#127 | Sin cierre. Los objetos imagen no tienen una identidad persistente de leyenda; `references.caption` inserta un párrafo Caption/SEQ contiguo, sin propietario. No se dibujan asociaciones adivinadas ni se oculta/elimina texto para simular una propiedad reversible. |
| #107 | Reproducido de nuevo: el objeto queda en página 0 al moverlo a un párrafo cuyo primer carácter es U+000C. Reinsertarlo después del salto mediante las API públicas conserva el fallo. Se mantiene rollback atómico y el issue abierto. |
| #1/#2 | Los casos fijados del motor y sus reproducciones anteriores siguen vigentes; no se modifica ni actualiza el motor en esta pasada. |
| #9/#52/#99/#100 | Los controles conocidos pasan; no se dispone del caso original suficiente para atribuir o cerrar los síntomas pendientes. |
| #84 | La exposición de diccionarios del host requiere integración específica de plataforma; permanece la importación explícita. |

## Comprobaciones

Regresión nueva `--alpha30-ui-test` con ratón real mediante
`tools/alpha30-pointer-test.sh`: miniatura independiente del scroll, imagen lejos
_del inicio_, conservación de viewport, apertura sin mutación, repetición de Fuente/
Idioma y Bordes/Sombreado con historial, propiedades sin destino, filtro de
navegación y clic real del índice. Se ejecuta también contra el paquete instalado.
La regresión de referencias comprueba destinos distintos para títulos duplicados,
bookmarks reales e invariancia de documento/historial durante navegación.
Las regresiones anteriores adaptan explícitamente los requisitos sustituidos
(Fuente interna, tamaño expandido, Sangrías internas y controles insensibles).

173 pruebas Rust y cuatro Python; 854 mensajes completos inglés/español.
Formato, catálogos y enlaces locales comprobados. Pruebas GTK locales con Xvfb,
interacción de Texto y Fuente, revisión, Párrafo y la nueva pasada superadas.
La CI optimizada y las instalaciones GNOME 49/51 deben terminar correctamente
antes de la promoción; sus identificadores se registran al finalizar.

Las capturas diagnósticas no sustituyen la galería pública ni una prueba humana.

La batería de propiedades ejercita también por separado repetición de alineación,
tipo y nivel de lista, las tres sangrías, interlineado y columnas entre documentos.
La regresión de lista reproduce la pérdida de tipo en un párrafo nuevo y exige
conservar tipo/nivel con un solo paso de Deshacer. La numeración importada ligada
a una definición canónica no se reaplica como un identificador de otro documento.
La prueba general de interacción abre ahora Fuente en su página interna real.
