# Alpha.45 — fuentes, posición de lectura y herramientas

## Alcance y autorización

El 11/11/2026 David pide cerrar todos los issues resolubles, priorizar el más
reciente y publicar una nueva release. Autoriza publicación directa de esta
entrega tras validarla. No se cambia el procedimiento habitual de RC para otras
entregas. La fuente exacta y los archivos públicos quedan registrados por la
publicación en [LATEST_RELEASE](LATEST_RELEASE.md).

## Cambios

#278 conserva una referencia de lectura independiente del cursor durante zoom,
ajuste al ancho y cambios de tamaño. Las restauraciones diferidas respetan la
identidad y revisión del documento y ceden ante navegación o entrada posterior.
Siguen las asignaciones reales de tamaño de GTK, también con escalado alto; la
regresión comprueba que el cursor final queda dentro del visor. El borde superior
permanece fijo si la vista ya estaba al principio: Inicio conserva la posición del
folio al escribir, deshacer y rehacer.
#280 conserva esa referencia al borrar/restaurar una imagen flotante, por teclado,
menús y Deshacer/Rehacer, también cuando hay imágenes consecutivas y el motor
retiene como objetivo la vecina. Distingue el borrado de imágenes de escribir
sobre la selección o borrar texto ordinario: esas acciones revelan el cursor.
No cambia el anclaje ni la selección del motor.

#259 completa el catálogo de herramientas actuales: galerías y opciones de listas,
operaciones de selección/inserción/borrado/división de tablas, inserción independiente
de imagen/tabla/índice, campos automáticos y saltos. Se reutilizan las acciones y
páginas existentes; las nuevas identidades empiezan en Más opciones, sin alterar
el orden ni la colocación guardados. Imagen y Tabla mantienen sus primeras
posiciones y las nuevas inserciones avanzadas se añaden después. Copiar normalmente sigue proporcionando
Pegar formato, conservando los contratos de alpha.43 y sus pruebas.

#262 mantiene Aplicar a encima de las opciones desplazables, muestra estados
mixtos y reconoce diseños importados que coinciden con un preajuste. Los valores
válidos se aplican directamente. Abrir, elegir alcance o actualizar la vista previa
no crea una edición. Las entradas numéricas sincronizan su valor al cambiar de
alcance y cancelan la entrada diferida del destino anterior. El diseño recordado se aplica al destino actual, sin retener
secciones del documento de origen. La vista previa representa geometría: #1,
flujo de texto en columnas desiguales, sigue siendo un límite del motor. ODT
conserva el límite previo de exportación de anchuras desiguales.

#279 añade aviso consolidado al abrir, sustitutas explícitas, archivos locales y
un catálogo pequeño de Poppins fijado a una revisión de Google Fonts. Descarga
TTF y licencia OFL tras una acción del usuario; valida tamaño, formato y familia,
con TLS, cancelación y tiempos acotados. GtkFileLauncher abre el archivo real en
el visor del sistema. Cada variante requiere su instalación normal. Releer las
fuentes instaladas invalida la caché del proveedor público y actualiza las
pestañas sin editar texto, formato, selección, historial o estado modificado.
No se crea una colección privada de fuentes instaladas en Leaf.

## Validación

Las regresiones nuevas ejercitan los controles GTK y los comandos reales:
`--zoom278-ui-test`, `--picture280-ui-test`, `--catalog259-ui-test` y
`--columns-ui-test`. La CI las ejecuta en inglés/español y desde los dos
paquetes instalados GNOME 49/51, conservando toda la batería anterior. La prueba
`tools/font279-installed-test.sh` utiliza un host limpio y el verdadero visor de
fuentes, pulsa Instalar y compara los archivos del host con las descargas,
comprobando después las cuatro variantes y el cambio real de renderizado dentro
del Flatpak. Su éxito es obligatorio para esta entrega. No equivale a una
prueba personal de GNOME OS, Windows, Orca o impresora física.

Localmente pasan 247 pruebas Rust, 16 Python y 1076 mensajes completos en
inglés/español. Han pasado las pruebas nativas de zoom, borrado flotante, catálogo y
columnas, Inicio en inglés/español al 100/200 % y apertura de herramientas
visibles al 200 %, incluida la reutilización de valores entre alcances y la cancelación
de entradas pendientes. También pasa la regresión completa de controles de
alpha.43, con apertura no mutante de propiedades de tabla; la instalación real de cursiva y negrita cursiva también pasó en un
perfil de prueba. El host ya tenía normal/negrita: la aceptación completa de las
cuatro variantes se reserva expresamente al host limpio de CI. Las verificaciones
finales del código, traducciones, paquete optimizado y publicación quedan en el
run enlazado desde LATEST_RELEASE; este informe no sustituye ese resultado.

## Límites y pendientes

WordCraft sigue fijado en 0.17.0, revisión
`5574b32600aa7d782c64f5a6c6bb24696152220d`; engine/render y vendor no cambian.
Se declaran directamente dos dependencias ya bloqueadas: wordcraft-fonts de esa
misma revisión y ureq 3.5. No cambia el perfil optimizado ni el unwinding.

#279 permanece abierto hasta completar también la distribución de tienda y las
comprobaciones restantes. El bundle propio permite red para las descargas
solicitadas. El manifiesto vigente de FlatPark no la permite: las fuentes locales
siguen disponibles allí, pero la descarga dentro de Leaf necesita una actualización
de permisos separada. Leaf detecta ese permiso real y ofrece el archivo local,
sin mostrar una descarga que el paquete no puede realizar. No se presenta la prueba GNOME 51 como evidencia de un
permiso que la tienda aún no concede.

#281 está remitido a WordCraft #6261; la discusión de bordes parciales es #6262.
La unión de tablas se propone en WordCraft PR #6251. No se integran ramas de
trabajo ajenas ni se cierran #1/#99/#152/#252/#261/#265/#267/#268/#277 por parches
que aún no estén en la dependencia de Leaf. #231 conserva su reapertura y las
copias de recuperación; no hay una reproducción nueva que justifique cerrarlo.
#185 conserva la comprobación de la ficha publicada en su conversación. Windows
se publica ocasionalmente y no se regenera en esta entrega.
