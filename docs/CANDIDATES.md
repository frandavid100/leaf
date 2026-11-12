# Candidatas y publicación

El procedimiento habitual acordado con David el 9/11/2026 es generar y validar
primero una RC, entregársela para probarla y publicar cuando dé el visto bueno
a esa candidata concreta. Si se aprueba sin cambios, se promueven los mismos
archivos y notas congeladas, sin recompilar ni repetir la batería ya superada.
Al publicar se verifican fuente, tag y sumas de los archivos descargados.

Si aparecen fallos que exigen correcciones, se genera otra RC con nueva fuente,
ejecución y sumas; se valida y se entrega de nuevo para probarla. Preparar una RC
no autoriza por sí solo su publicación. Las autorizaciones automáticas de
entregas anteriores pertenecen a su historial y no se aplican a futuras versiones.

Cada candidata contiene la revisión fuente, el identificador de la ejecución,
las notas de versión y las sumas SHA-256 de los paquetes. Su versión interna es
la Alpha prevista; «candidata 1», «candidata 2», etc. distinguen las compilaciones
por su ejecución y hash, sin cambiar el paquete antes de publicarlo.

## Alpha.32 publicada

La [candidata 42372618858](https://github.com/frandavid100/leaf/actions/runs/42372618858)
superó GNOME 49/51 y la batería nativa completa 42371069496. La
[promoción automática](https://github.com/frandavid100/leaf/actions/runs/42374275590)
publicó los mismos archivos el 7/11/2026. Verificados el tag y los doce archivos
públicos; fuente `2524dbc789cfbfa0f3b6e153fde16f1df5887b8a`.
[Actualización, mediciones y límites](WORDCRAFT_016_REVIEW.md).

## Alpha.31 publicada

La [candidata 42361339472](https://github.com/frandavid100/leaf/actions/runs/42361339472)
superó las instalaciones GNOME 49/51 y exigió el éxito de toda la batería nativa
42360443648. La [promoción automática](https://github.com/frandavid100/leaf/actions/runs/42362957503)
publicó los mismos archivos el 7/11/2026. Verificados los doce digests públicos.
[Fuente, sumas y alcance](history/ALPHA31_REMAINING.md).

## Generar y probar

El commit de integración en `main` lleva `[candidate-alpha]`. El mismo marcador
en el título de su PR omite solo la comparación opcional de rendimiento; no
omite pruebas funcionales nativas ni instaladas. La batería nativa
y las comprobaciones instaladas generan el artefacto `leaf-flatpak-alpha`.
El marcador habitual es solo `[candidate-alpha]`, sin `[auto-publish-alpha]`:
los paquetes quedan disponibles para probar y esperan el visto bueno de David.
El mecanismo `[auto-publish-alpha]` se conserva para una excepción posterior
expresamente autorizada para una entrega concreta, no como procedimiento habitual.

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
flatpak install --user ./Leaf-0.1.0-alpha.32-x86_64.flatpak
flatpak run --user --branch=alpha io.github.frandavid100.leaf
```

Al probarla, confirma que «Acerca de Leaf» muestra `0.1.0-alpha.32`, especialmente
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
manualmente con el identificador de la RC aprobada por David. La promoción
automática queda reservada a la excepción expresa descrita arriba. Comprueba
que la ejecución terminó correctamente, pertenece a la rama y al flujo de
candidatas de este repositorio, y que coinciden la fuente, el manifiesto y todas
las sumas de verificación. Publica esos archivos y sus notas congeladas, creando
la etiqueta en la fuente que produjo la candidata.

La promoción reutiliza el artefacto aprobado y sus pruebas completas, con las
verificaciones de identidad e integridad de la publicación. No vuelve a ejecutar
la compilación ni la batería funcional de esa misma candidata. No sustituir
los paquetes de una release ya publicada: cualquier corrección posterior genera
una nueva versión.
