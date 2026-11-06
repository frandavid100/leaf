# Candidatas y publicación

Desde alpha.29, las pruebas automáticas y el empaquetado generan una candidata
sin crear una release ni cambiar la versión distribuida por FlatPark. David la
prueba antes de aprobar su publicación.

Cada candidata contiene la revisión fuente, el identificador de la ejecución,
las notas de versión y las sumas SHA-256 de los paquetes. Su versión interna es
la Alpha prevista; «candidata 1», «candidata 2», etc. distinguen las compilaciones
por su ejecución y hash, sin cambiar el paquete antes de publicarlo.

## Generar y probar

El commit de integración en `main` lleva `[candidate-alpha]`. La batería nativa
y las comprobaciones instaladas generan el artefacto `leaf-flatpak-alpha`.
Los paquetes no se suben automáticamente a Releases.

El commit de integración puede añadir `[bundle-only] [native-run:ID]`, con el
identificador real de una ejecución nativa del mismo repositorio/flujo. No se
cambian constantes de fuentes en el flujo para cada candidata.

Una compilación nativa puede ceder su ejecutable al empaquetado después de pasar
los controles nuevos, mientras continúa las regresiones anteriores. El flujo
comprueba identidad y equivalencia de fuentes; aplica el mismo strip optimizado,
valida las instalaciones en paralelo y exige el éxito completo de la ejecución
nativa antes de aceptar la candidata. Finalmente compara ambos ejecutables byte
a byte. El manifiesto conserva también la fuente y la ejecución nativas. La
promoción vuelve a comprobar ese resultado. Una prueba fallida no produce una
candidata aprobable.

Para instalar el bundle descargado:

```sh
flatpak install --user ./Leaf-0.1.0-alpha.29-x86_64.flatpak
flatpak run --user --branch=alpha io.github.frandavid100.leaf
```

Se conserva el mismo identificador de aplicación y sus datos. Una candidata
reemplaza la versión de esa misma rama instalada desde un bundle. La rama de
FlatPark puede estar instalada también; usar `--branch=alpha` elige explícitamente
la candidata. Para volver, reinstalar el bundle publicado anterior.

Las comprobaciones de la candidata deben cubrir los cambios indicados en sus
notas y los documentos donde aparecieron los problemas. Un fallo produce otra
candidata, con nueva fuente, ejecución y sumas.

## Promover después de la aprobación

El flujo `Publish an approved Alpha candidate without rebuilding` se ejecuta
manualmente con el identificador de la ejecución aprobada por David. Comprueba
que la ejecución terminó correctamente, pertenece a la rama y al flujo de
candidatas de este repositorio, y que coinciden la fuente, el manifiesto y todas
las sumas de verificación. Publica esos archivos y sus notas congeladas, creando
la etiqueta en la fuente que produjo la candidata.

No compila ni instala dependencias para producir otro ejecutable. No sustituir
los paquetes de una release ya publicada: cualquier corrección posterior genera
una nueva versión.
