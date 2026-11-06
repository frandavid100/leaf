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
| #131 | Navegación reúne Páginas y Estructura bajo dos representaciones internas; conserva filtro, expansión y desplazamientos independientes por pestaña, restaura la representación al volver y revela destinos modificados mientras estaba oculta. |
| #121 | Parcial: propiedades visibles/insensibles, explicación mediante tooltip, inserción inferior, transición nativa y herramientas compartidas. La dependencia de #127 sigue pendiente. |
| #124 | Parcial: índice canónico de todos los tipos disponibles, lectura de tablas anidadas y filtro nativo. Filtros con contexto de sección/títulos y selección activa por extensión real de enlaces, objetos y notas. Falta la asociación de leyendas. |
| #126/#127 | Sin cierre. Los objetos imagen no tienen una identidad persistente de leyenda; `references.caption` inserta un párrafo Caption/SEQ contiguo, sin propietario. No se dibujan asociaciones adivinadas ni se oculta/elimina texto para simular una propiedad reversible. |
| #107 | Reproducido de nuevo: el objeto queda en página 0 al moverlo a un párrafo cuyo primer carácter es U+000C. Reinsertarlo después del salto mediante las API públicas conserva el fallo. Se mantiene rollback atómico y el issue abierto. |
| #1/#2 | Los casos fijados del motor y sus reproducciones anteriores siguen vigentes; no se modifica ni actualiza el motor en esta pasada. |
| #9/#52/#99/#100 | Los controles conocidos pasan; no se dispone del caso original suficiente para atribuir o cerrar los síntomas pendientes. |
| #84 | La exposición de diccionarios del host requiere integración específica de plataforma; permanece la importación explícita. |

Cerrados tras la publicación verificada: #122, #123, #125, #128, #129, #130 y #131.
Los restantes conservan el estado parcial/pendiente indicado en la tabla.

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

173 pruebas Rust y cuatro Python; 855 mensajes completos inglés/español.
Formato, catálogos y enlaces locales comprobados. Pruebas GTK locales con Xvfb,
interacción de Texto y Fuente, revisión, Párrafo y la nueva pasada superadas.
CI optimizada e instalaciones GNOME 49/51 completadas con éxito antes de la
promoción. La publicación conserva los mismos paquetes, sin recompilar.

Las capturas diagnósticas no sustituyen la galería pública ni una prueba humana.

La batería de propiedades ejercita también por separado repetición de alineación,
tipo y nivel de lista, las tres sangrías, interlineado y columnas entre documentos.
La regresión de lista reproduce la pérdida de tipo en un párrafo nuevo y exige
conservar tipo/nivel con un solo paso de Deshacer. La numeración importada ligada
a una definición canónica no se reaplica como un identificador de otro documento.
La prueba general de interacción abre ahora Fuente en su página interna real.

## Identidad de la revisión final

Fuente nativa: `3f898df0f42f280d932eaa2bb3790dd6b45d1190`.
[Validación nativa](https://github.com/frandavid100/leaf/actions/runs/42184270093).
La candidata exige éxito completo y equivalencia de fuentes, y compara byte a
byte el ejecutable empaquetado con el ejecutable optimizado validado.

La regresión de Navegación verifica con controles nativos los desplazamientos
independientes, filtro y estado por pestaña, regreso desde Revisión/Búsqueda,
cierre/apertura de la barra y cambio de destino mientras Estructura está oculta.
Superada en inglés 1× y español 2×. Cambiar de representación no modifica el
documento, historial, selección ni viewport. La prueba antigua de Sangrías usa
ahora el par vigente configuración/repetición, con ratón real.

Las ejecuciones iniciales fallidas no son candidatas publicables. La CI detectó
supuestos antiguos de los conductores (coordenadas 2×, destino del clic fuera del
viewport y la antigua ubicación de Fuente) y la dependencia de la regla respecto
a la página activa por viewport. Las regresiones se actualizaron a los requisitos
vigentes y la regla conserva geometría visible independiente de la miniatura/caret.
La prueba local completa de reglas supera sus 60 pasos.

La primera candidata detectó que el conductor FlatPark cambiaba de directorio
sin llevar consigo el nuevo script auxiliar de Alpha.30. Se incorpora al entorno
de diagnóstico y se exige repetir la instalación GNOME 51 antes de publicar.

## Publicación verificada

- [Candidata e instalaciones](https://github.com/frandavid100/leaf/actions/runs/42187179877): éxito completo.
- Fuente publicada: `85d0134e413f1cb4587e2cf2de9d9928a1e85a1c`.
- [Promoción automática](https://github.com/frandavid100/leaf/actions/runs/42189125319): éxito; autorización explícita de David.
- [Release v0.1.0-alpha.30](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.30): pública, no borrador; 12 archivos.
- Artefacto descargado `13489200539`: SHA-256 `92b2f16566527b92de4b112cd63d8b8ca3e789e5193a38dd7c4874da17b549fc`.
- Todas las sumas internas verificadas; los 12 digests públicos coinciden con la candidata.
- Bundle Flatpak: `f868786d7c1ff750a49a3329916e2b0e129f043cdff3910304904e1dcc5eff77`.
- Archivo FlatPark: `3dad6b82a8d58f91a6e727543c29f17f0385b0eee17c22ca44fac0419672fc84`.

Los registros `alpha29-flatpark/alpha30-en-1.log` y `alpha30-es-2.log` contienen
`NAVIGATION131_UI_OK` y `ALPHA30_UI_OK`; verifican también la pasada nueva en
GNOME 51, en inglés 1× y español 2×. La primera candidata fallida no se publicó.
