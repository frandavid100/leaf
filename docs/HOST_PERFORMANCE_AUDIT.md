# Diagnóstico y optimización del host Leaf

Petición: medir rendimiento y corregir exclusivamente la capa Leaf. Cualquier
posible cambio de WordCraft debe proponerse a upstream, sin modificarlo aquí.

## Método

Se instrumenta primero el host de alpha.12 sin cambiar su comportamiento y se
conserva ese ejecutable optimizado como referencia. Se compara con el candidato
en el mismo runner Ubuntu 24.04, GTK/Xvfb y GSK Cairo, usando orden AB/BA y dos
procesos nuevos por versión y tamaño. Cada proceso usa un perfil temporal y
solo documentos sintéticos de 120, 600 o 1800 párrafos con texto acentuado.

La generación/guardado de fixtures queda fuera de los tiempos de importación.
Se importan DOCX y ODT desde archivos recién creados (caché de disco caliente),
se mide la primera maquetación y después los widgets/rasteres iniciales. Se
comprueban movimientos de cursor, desplazamiento, escritura/deshacer y búsqueda
activa/cerrada. El documento canónico debe acabar idéntico al inicial.

Los spans `leaf.*` son inclusivos. Los `boundary.*` delimitan llamadas a las
API públicas del motor a través de los adaptadores: incluyen su pequeño coste
Leaf, no son un perfil exclusivo de CPU de WordCraft. Los contadores añaden un pequeño coste de instrumentación a ambas versiones;
los tiempos describen este diagnóstico, no una sesión normal. No sumar spans anidados
ni atribuir todo el tiempo de una operación a un solo componente.

Además del tiempo se cuentan solicitudes de repintado, consultas de límites
GTK y filas de búsqueda creadas. Esos contadores distinguen trabajo eliminado
de fluctuaciones de CI. Se registra RSS por fase: incluye los documentos y
copias de comprobación de esta prueba, y no equivale al consumo de una sesión
normal ni a una promesa de menor memoria. Los rasteres ya tenían caché limitada
al área visible; se conserva esa política.

## Cambios exclusivos de Leaf

- La capa de selección repinta la unión de las páginas anteriores y actuales;
  así desaparecen las marcas al deseleccionar, cerrar búsqueda o cambiar de
  página. Durante la animación del cursor solo se invalida su página. Entrar o
  salir de encabezados/pies mantiene invalidación completa de sus guías.
- Las miniaturas consultan por búsqueda binaria los límites actuales de los
  widgets GTK. No se guardan alturas CSS que puedan quedar obsoletas al cambiar
  la ventana. Antes de la asignación se usan los mismos límites aproximados de
  la versión anterior, precalculados al construir la lista. Los límites aún
  coincidentes de widgets recién añadidos se consideran sin asignar; no todas
  las miniaturas visibles. Un callback débil y agrupado actualiza los límites
  después de cambiar el ajuste/tamaño del panel. Solo se recorren
  las miniaturas visibles y la pequeña caché para expulsar las anteriores.
- La búsqueda cerrada conserva la consulta, pero deja de ejecutar el buscador
  y reconstruir sus filas al editar. Al volver a abrirla se recalcula contra la
  revisión actual. Miniaturas ocultas por búsqueda/títulos no se rasterizan.

No cambia `Cargo.toml`, `Cargo.lock`, `vendor/`, `patches/`, la revisión de
WordCraft ni el renderizador. El workflow comprueba esa frontera por `git diff`.

## Corrección

Los tests sin GUI comparan la búsqueda binaria con un recorrido completo en
listas de alturas variables, huecos y bordes exactos, además de una lista de
10 000 entradas con número acotado de consultas. La prueba GTK compara la
caché con todos los límites reales en inicio, mitad y final del panel; comprueba
repintado de selección anterior, cierre y reapertura de búsqueda tras editar,
y restauración exacta del documento por deshacer. Se mantienen regresiones
existentes de encabezados, zoom/HiDPI, animación, búsqueda y ciclo de vida.

## Resultados

La comparación AB/BA pasa en los tres tamaños. Cada versión se ejecuta dos
veces por tamaño en el mismo runner. Medianas de la fase de construcción de
widgets y rasteres iniciales, **sin incluir importación ni primera maquetación**:

| Páginas | Miniaturas rasterizadas inicialmente | Fase inicial, ms (antes → después) |
| --- | --- | --- |
| 11 | 11 → 4 | 29.02 → 16.68 |
| 55 | 55 → 4 | 111.95 → 20.92 |
| 164 | 164 → 4 | 315.90 → 30.83 |

En 164 páginas, 80 movimientos de cursor: **13.120 → 80** operaciones (`overlay_queue`), 1.259 → 0.991 ms en esta muestra.

En 164 páginas, 80 actualizaciones de miniaturas en caché: **13.120 → 1.440** operaciones (`thumbnail_bounds`), 2.581 → 0.324 ms en esta muestra.

En 164 páginas, 4 actualizaciones solicitadas con búsqueda cerrada: **288 → 0** operaciones (`search_rows`), 103.335 → 0.002 ms en esta muestra.

La primera maquetación y las ocho inserciones/deshacer no presentan una
mejora consistente que deba atribuirse a estos cambios. No se modifica ni se
acelera internamente WordCraft. La menor RSS observada en los documentos grandes
es compatible con evitar los rasteres iniciales innecesarios, pero dos muestras
no justifican una promesa general de memoria. Los contadores de trabajo son la
evidencia más directa; no multiplicar sus ratios por la velocidad de toda la app.

No se ha aislado un defecto concreto del motor que justifique una incidencia
upstream. Las fases importación/maquetación/rasterización quedan registradas para
una futura reproducción si aparece un caso problemático; cualquier cambio allí
se propondrá a WordCraft, manteniendo esta rama sin modificaciones del motor.

[Evidencia durable, revisiones y resultados por fase](HOST_PERFORMANCE_VALIDATION.json).


## Repetir

```sh
cargo test --locked
cargo build --release --locked --features gui --bin wordcraft-gnome
XDG_CONFIG_HOME=$(mktemp -d) XDG_STATE_HOME=$(mktemp -d) \
  GSK_RENDERER=cairo xvfb-run -a \
  ./target/release/wordcraft-gnome --host-cache-ui-test
python tools/performance_compare.py /ruta/referencia /ruta/candidato
```

El marcador `--performance-ui-test` activa métricas exclusivamente para ese
diagnóstico sintético. No se registran documentos ni métricas en uso normal.
No es una nueva batería de compatibilidad por versión ni repite el corpus de
24 archivos. Tampoco sustituye la prueba humana Wayland/GNOME OS o con GPU.

Documentación GTK consultada:

- https://docs.gtk.org/gtk4/drawing-model.html
- https://docs.gtk.org/gtk4/method.Widget.queue_draw.html
- https://docs.gtk.org/gtk4/method.Widget.compute_bounds.html

## Entrega

[Alpha.13](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.13)
publicada después de superar la regresión de caché en el Flatpak instalado,
actualización desde alpha.12 y la prueba real AT-SPI/Orca. 99 tests y regresiones
GTK completos. Artefactos descargados/verificados y ocho assets contrastados;
revisiones, hashes y resultados instalados están en el JSON enlazado.
Las capturas públicas no se han sustituido y Weblate sigue a cargo del usuario.
