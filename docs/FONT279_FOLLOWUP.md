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

## Estado de aceptación

Antes de subir: formato Rust, 16 pruebas Python, 1076 mensajes completos EN/ES,
sintaxis de scripts y enlaces de documentación comprobados localmente. La
compilación, las pruebas Rust y las pruebas de Flatpak de esta reconstrucción
requieren resultados nuevos de CI. #279 permanece abierto. Los casos negativos
adicionales de descarga/cancelación, visor ausente y fuentes incrustadas del
entorno perdido no se dan por revalidados con estos nuevos diagnósticos.

Trabajo preparado con asistencia de IA para el mantenedor de Leaf.
