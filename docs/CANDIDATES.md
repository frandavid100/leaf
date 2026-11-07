# Candidatas y publicación

Desde alpha.29, las pruebas automáticas y el empaquetado generan una candidata
sin crear una release ni cambiar la versión distribuida por FlatPark. El proceso normal permite probarla antes de aprobar su publicación. Para la
pasada alpha.30 y la última pasada alpha.31 del 7/11/2026, David autoriza
expresamente la publicación automática tras las comprobaciones.

Cada candidata contiene la revisión fuente, el identificador de la ejecución,
las notas de versión y las sumas SHA-256 de los paquetes. Su versión interna es
la Alpha prevista; «candidata 1», «candidata 2», etc. distinguen las compilaciones
por su ejecución y hash, sin cambiar el paquete antes de publicarlo.

## Alpha.31 publicada

La [candidata 42361339472](https://github.com/frandavid100/leaf/actions/runs/42361339472)
superó las instalaciones GNOME 49/51 y exigió el éxito de toda la batería nativa
42360443648. La [promoción automática](https://github.com/frandavid100/leaf/actions/runs/42362957503)
publicó los mismos archivos el 7/11/2026. Verificados los doce digests públicos.
[Fuente, sumas y alcance](ALPHA31_REMAINING.md).

## Generar y probar

El commit de integración en `main` lleva `[candidate-alpha]`. El mismo marcador
en el título de su PR omite solo la comparación opcional de rendimiento; no
omite pruebas funcionales nativas ni instaladas. La batería nativa
y las comprobaciones instaladas generan el artefacto `leaf-flatpak-alpha`.
Sin otra autorización, los paquetes no se suben automáticamente a Releases.
Un commit de candidata que añade `[auto-publish-alpha]`, dentro de una publicación
expresamente autorizada, activa la promoción al terminar correctamente la ejecución.

El commit de integración puede añadir `[bundle-only] [native-run:ID]`, con el
identificador real de una ejecución nativa del mismo repositorio/flujo. No se
cambian constantes de fuentes en el flujo para cada candidata.

Una compilación nativa puede ceder su ejecutable al empaquetado después de pasar
los controles nuevos y las regresiones de menú contextual, fijación y Párrafo,
ejecutadas una sola vez antes de crear el artefacto reutilizable, mientras continúa las regresiones anteriores. El flujo
comprueba identidad y equivalencia de fuentes; aplica el mismo strip optimizado,
valida las instalaciones en paralelo y exige el éxito completo de la ejecución
nativa antes de aceptar la candidata. Finalmente compara ambos ejecutables byte
a byte. El manifiesto conserva también la fuente y la ejecución nativas. La
promoción vuelve a comprobar ese resultado. Una prueba fallida no produce una
candidata aprobable.

Para instalar el bundle descargado:

```sh
flatpak install --user ./Leaf-0.1.0-alpha.31-x86_64.flatpak
flatpak run --user --branch=alpha io.github.frandavid100.leaf
```

Al probarla, confirma que «Acerca de Leaf» muestra `0.1.0-alpha.31`, especialmente
si tienes abiertas ventanas de varias versiones. El paquete completo incluye
`SHA256SUMS`; `sha256sum -c SHA256SUMS` verifica los archivos descargados.

Se conserva el mismo identificador de aplicación y sus datos. Una candidata
reemplaza la versión de esa misma rama instalada desde un bundle. La rama de
FlatPark puede estar instalada también; usar `--branch=alpha` elige explícitamente
la candidata. Para volver, reinstalar el bundle publicado anterior.

Las comprobaciones de la candidata deben cubrir los cambios indicados en sus
notas y los documentos donde aparecieron los problemas. Un fallo produce otra
candidata, con nueva fuente, ejecución y sumas.

## Promover una candidata validada

El flujo `Publish a validated Alpha candidate without rebuilding` se ejecuta
manualmente con el identificador aprobado por David, o automáticamente mediante
el marcador autorizado descrito arriba. Comprueba
que la ejecución terminó correctamente, pertenece a la rama y al flujo de
candidatas de este repositorio, y que coinciden la fuente, el manifiesto y todas
las sumas de verificación. Publica esos archivos y sus notas congeladas, creando
la etiqueta en la fuente que produjo la candidata.

No compila ni instala dependencias para producir otro ejecutable. No sustituir
los paquetes de una release ya publicada: cualquier corrección posterior genera
una nueva versión.
