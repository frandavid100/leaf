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

## Estado de aceptación — comprobado el 12/11/2026

La fuente compilada es `a32c9e8291d93f9ea62e679baf31ed1200818acf`.
La [validación nativa](https://github.com/frandavid100/leaf/actions/runs/43181264250)
supera 248 pruebas Rust, 16 Python, 1077 mensajes completos EN/ES, compilación
optimizada, los cinco grupos gráficos y el gate agregado, a la primera.

La [validación instalada final](https://github.com/frandavid100/leaf/actions/runs/43186108526)
usa el controlador `f79bdbe8ffcd243840bc961aa884f21798e0669c` y el mismo ejecutable.
Antes de reutilizarlo verifica los siete gates nativos y que las entradas de
compilación son idénticas. SHA-256 del ejecutable:
`aad68c541e93f8ff23eb9e60d53bc7c4900bac9ca4ef3d90f8788e2fe6449e26`.
Los catálogos EN/ES actuales se compilan y sus sumas quedan en los metadatos del
paquete; los recursos base son los de alpha.45. No es una release pública.

| Comprobación instalada | GNOME 49 | GNOME 51 |
| --- | --- | --- |
| Carpeta de fuentes ausente al arrancar; cuatro instalaciones reales | Correcto | Correcto |
| Aviso de guardar/reiniciar y proceso nuevo con Poppins disponible | Correcto, EN 100 % | Correcto, ES 200 % |
| Carpeta existente: cuatro estilos con refresco en vivo | Correcto | Correcto |
| Visor cerrado sin instalar; fallback y estado conservados | EN/ES | EN/ES |
| Archivo local: refresco de otra pestaña y posición de lectura | EN/ES | EN/ES |
| Texto nuevo/Undo, DOCX/ODT, nombres y dirty intactos | Correcto | Correcto |
| Comparación del mismo DOCX con LibreOffice | Correcto | Correcto |

El log GNOME 51 contiene cuatro comprobaciones del texto accesible español
`FONT279_RESTART_MESSAGE_LOCALIZED_OK language=es`, además de
`FONT279_RESTART_RECOVERY_OK styles=Regular,Bold,Italic,BoldItalic`.
El gate independiente exige éxito real del primer uso: no se oculta un fallo
por permitir continuar con los casos restantes.

La comparación con Writer verifica cuatro fuentes PDF reales, una página,
texto coincidente y anchura de cada línea. La máxima diferencia observada es
0,036 puntos. Es un DOCX sintético de cuatro líneas, no una afirmación de fidelidad
general ni una prueba del documento original de David.

## Límite de primer uso

Si la carpeta de fuentes de usuario no existía cuando Flatpak creó el sandbox,
la instalación en el host no hace visible esa nueva carpeta al proceso abierto.
Leaf mantiene la fuente como no disponible y explica que hay que guardar el
trabajo y reiniciar. La prueba conserva esa condición inicial, instala los cuatro
estilos por el visor, comprueba el aviso y el estado intacto, y abre un proceso
nuevo: Poppins está disponible, el renderizado cambia y el PDF coincide con las
comprobaciones de referencia. No se reinicia automáticamente, amplían permisos
de archivos ni registran fuentes privadas.

## Propuesta FlatPark comprobada

La [ejecución 43173821014](https://github.com/frandavid100/leaf/actions/runs/43173821014/job/131033158433)
comprueba el paquete exacto `32c2bcaf41f3d99c4b967d29e1f1cac85792aa96`:
auditoría, bytes fijados, desempaquetado como root sin capacidades, instalación
desde repositorio local firmado mediante publish.sh --verify, permisos efectivos,
datos entre procesos, cuatro instalaciones reales, arranque y captura observada.
Se usa el ejecutable oficial alpha.45, separado de la corrección de código.

FlatPark actualizó automáticamente el pin a alpha.45. La propuesta actual
`438eca97cde7134179b1fa622b6437ab627e0188` integra su main
`9804ff685fe1b3da047dd245f1946c5922b87273` y resuelve el conflicto de notas.
Los archivos completos del registro Leaf, scripts y herramientas del sitio son
idénticos a la revisión probada. La propuesta está sin conflictos; cambia solo
tres archivos frente a ese main. No está enviada ni desplegada.

## Incidencias del controlador resueltas

El controlador alpha35 enviaba Ctrl+V antes de que el proceso GTK observara el
portapapeles externo image/png, bloqueándose en ES 200 %. Ahora el diagnóstico
anuncia su disponibilidad antes de recibir el gesto real; se conservan las
aserciones de imagen, documento e historial. Los dos idiomas pasan a la primera
en la compilación nativa final. No cambia el pegado del producto.

La primera comprobación del aviso ES detectó que LC_ALL=C.UTF-8 mantenía toda
la interfaz en inglés pese a disponer del MO completo. Los controladores finales
usan en_US.UTF-8 con LANGUAGE=es y comprueban texto accesible realmente traducido.
Este ajuste de entorno no cambia el código ni exige recompilarlo.

## Pendiente y alcance

Todo permanece en `fix/font279-acceptance`; main no se ha modificado y no hay
una nueva release. La PR FlatPark requiere aprobación explícita y sigue en
borrador. #279 está abierto. Antes de cerrarlo faltan integración, entrega y los
casos de aceptación adicionales: cancelación/fallo de descarga, visor ausente,
fuentes incrustadas y otros sistemas. Las pruebas del entorno perdido no se dan
por revalidadas. No se afirma una prueba humana de GNOME OS, Bazaar ni del
archivo original de David.

Trabajo preparado con asistencia de IA para el mantenedor de Leaf.
