# Seguimiento de fuentes #279 — rama de revisión

El refresco del proveedor compartido debe incluir las familias usadas por todas
las pestañas vivas antes de reconstruir sus maquetaciones. Antes solo se
recargaban las familias de la pestaña que iniciaba la instalación; una fuente
local usada únicamente por otra pestaña podía seguir mostrando su sustituta.
Se conservan WordCraft 0.17, dependencias, parches y perfil release con unwinding.

## Recuperación del entorno y autorización

El 12/11/2026 David autoriza subir una rama de revisión y ejecutar GitHub Actions.
El entorno anterior se reinició antes de subir su commit local `940e475`; ese
commit, sus diagnósticos y logs no están disponibles en GitHub ni en el nuevo
entorno. Se reconstruye la corrección sobre `ee6c1dc`, conservando los cambios
de macOS incorporados después de alpha.45. Las 250 pruebas del entorno anterior
no se presentan como validación de esta reconstrucción.

La rama `fix/font279-acceptance` ejecuta la batería nativa existente y después
las pruebas instaladas. Se añade ese nombre concreto a `push` porque el conector
no ofrece dispatch de workflows; no se cambia `main`. Las condiciones de
publicación siguen exigiendo `main` y sus marcadores: esta rama no publica nada.

## Comprobaciones preparadas

- Fuente QA original CC0, generada solo para la prueba: el usuario abre su TTF
  mediante el selector real y el visor del sistema instala sus bytes exactos.
- Cerrar el visor sin instalar conserva el estado no resuelto y el renderizado.
- Instalar desde una pestaña que no usa esa familia actualiza otra pestaña,
  sin cambiar documento, selección, revisión, historial ni dirty. Se comprueban
  posición de lectura, texto nuevo/Undo, DOCX/ODT y archivo original intacto.
- GNOME 49 y prefijo trasladado GNOME 51, inglés normal y español al 200 %.
- Caso separado con directorio de fuentes de usuario ausente al arrancar.
  Un fallo no se oculta: hay un gate explícito independiente del resto.
- Cuatro estilos Poppins instalados por el visor, PDF real de Leaf y comparación
  con el mismo DOCX exportado por LibreOffice: fuentes, una página, texto y
  anchuras por línea (tolerancia 2 % o 1 punto).

Los paquetes de prueba contienen el ejecutable optimizado de la misma ejecución
validada y los recursos inmutables de alpha.45. Su identidad y metadatos quedan
en los artefactos `font279-source-49/51`; no son una release pública.

## FlatPark

La rama externa
[`fix/leaf-font-downloads`](https://github.com/flatpark/flatpark/compare/main...frandavid100:fix/leaf-font-downloads)
sí se conservó, commit `32c2bcaf41f3d99c4b967d29e1f1cac85792aa96`.
Añade red para las descargas explícitas de fuentes/licencias; no da acceso
estático a documentos ni al host. La CI instala ese manifiesto mediante el
repositorio local firmado, comprueba el desempaquetado de sistema y ejercita
el alpha.45 oficial sin sustituir su ejecutable. Ese resultado es independiente
de la corrección de código de esta rama.

La política de FlatPark exige probar el paquete y enseñar un borrador antes de
pedir autorización para abrir la PR. No se ha abierto esa PR ni modificado la
tienda. La autorización de CI no se interpreta como aprobación de esa PR.
[Borrador concreto, pruebas y límite de primer uso](FONT279_FLATPARK_PR.md).

## Estado de aceptación

La ejecución instalada
[43173821014](https://github.com/frandavid100/leaf/actions/runs/43173821014)
comprueba el mismo ejecutable `f7cc1e7` tras corregir requisitos del runner:
lector SVG, caché MIME del visor y clon completo para el generador de FlatPark.
El paquete exacto FlatPark `32c2bca` supera desempaquetado como root sin
capacidades, instalación desde repositorio local firmado, permisos, datos entre
procesos, cuatro estilos Poppins mediante el visor, arranque y captura observada.
No está enviado ni publicado en la tienda.

En GNOME 49/51 pasan instalación con directorio existente, comparación con
LibreOffice (máxima diferencia de anchura 0,036 puntos), cierre sin instalar,
archivo local, refresco de otra pestaña, lectura, texto nuevo/Undo y DOCX/ODT;
los dos últimos casos se ejercitan en EN 100 % y ES 200 %.

Queda demostrado un límite real: si el directorio de fuentes no existía al
crear el sandbox, instalar en el host no lo vuelve visible en ese proceso. Se
añade una indicación EN/ES para guardar y reiniciar Leaf cuando sigue habiendo
fuentes ausentes después de instalarlas. La nueva prueba conserva el directorio
ausente de partida, instala los cuatro estilos por el visor, comprueba el aviso
y estado intacto y abre un proceso nuevo para verificar fuentes, renderizado y
comparación con Writer. No se amplían permisos ni se registran fuentes privadas.
Esta corrección y el recorrido de reinicio requieren validación de la nueva
fuente. Se conserva la prueba del paquete FlatPark oficial, cuyos bytes y
manifiesto no cambian, sin repetirla para una modificación del código de Leaf.
El paquete de prueba usa los recursos base de alpha.45 con el catálogo EN/ES
compilado de la fuente actual; registra las sumas de ambos MO. La recuperación
en español se comprueba también a través del texto accesible del diálogo, para
evitar aceptar silenciosamente el inglés por faltar una traducción empaquetada.

La ejecución [43167291346](https://github.com/frandavid100/leaf/actions/runs/43167291346)
valida `f7cc1e7`: 248 pruebas Rust, compilación optimizada, cinco grupos
gráficos y gate nativo correctos. El grupo de edición pasó al repetirlo sin
cambiar el ejecutable; el primer intento se atascó en el pegado de una imagen
en español al 200 %, después de superar el arrastre de archivo.

La primera preparación instalada falló: el comprobador FlatPark necesitaba
instalar antes su runtime; el exportador Flatpak omite un `files/extra` añadido
a mano; y el controlador GNOME 49 no encontró el botón Install. Se corrigen las
dos primeras causas usando instalación de runtime y extra-data real, y se
añaden capturas/árbol accesible para el visor. Se usa la identidad real de la
app en cada runner limpio, con escritorio/iconos exportados, sin perfiles
personales. Esto todavía no demuestra el caso de directorio de fuentes ausente.

La repetición `[font279-ci-only]` reutiliza el ejecutable de esa ejecución:
comprueba por API los siete gates nativos, la revisión exacta y que no cambió
ninguna entrada de compilación. Registra por separado fuente del ejecutable y
fuente del controlador; solo se repiten paquetes y pruebas afectadas. No se
recompila ni se repite una batería nativa ya superada por cambios de CI.

Formato Rust, 16 pruebas Python, 1076 mensajes completos EN/ES, sintaxis de
scripts y enlaces de documentación comprobados localmente; actionlint también
superado. Compilación y batería nativa de esta reconstrucción superadas en CI.
Falta validar la indicación y recuperación de primer uso. #279 permanece abierto. Los casos negativos
adicionales de descarga/cancelación, visor ausente y fuentes incrustadas del
entorno perdido no se dan por revalidados con estos nuevos diagnósticos.

Trabajo preparado con asistencia de IA para el mantenedor de Leaf.
