# Intercambio Leaf ↔ LibreOffice — 1 de noviembre de 2026

Auditoría puntual autorizada por el usuario. Los archivos son sintéticos y se
generan automáticamente; no se han solicitado documentos privados ni repetido
el corpus completo de 24 archivos. Esta comprobación sigue siendo opcional.

## Método

`examples/interop_audit.rs` crea cuatro archivos mediante las API públicas de
WordCraft y los adaptadores reales de Leaf: DOCX/ODT, con una sección y con tres.
Incluyen primera página diferente, pares/impares, encabezados independientes y
heredados, PAGE/NUMPAGES, reinicio de numeración en VII, distancias de encabezado
21pt/pie 27pt, márgenes del cuerpo 72pt, inglés británico, español y un fragmento
marcado «no revisar». Se recorren todas las páginas, no solo la primera.

`tools/interop_audit.py` arranca Writer 24.2.7 en Ubuntu 24.04 mediante UNO, con
perfil temporal propio y macros deshabilitadas. Abre cada archivo, sustituye un
marcador de texto en cada sección, guarda en el mismo formato y exporta un PDF.
Leaf abre ese resultado, lo guarda y lo vuelve a abrir. Se conservan los estados
de ambas aplicaciones, archivos, PDF, texto extraído y una captura interna.

La comprobación adicional convierte los DOCX a ODT mediante Writer y elimina
exclusivamente los atributos del espacio de nombres Leaf antes de importarlos.
También se hace con los ODT guardados por Writer. Así se distingue la geometría
estándar de la recuperación mediante metadatos propios. La ejecución completa
no se interpreta como que todos los criterios hayan pasado: cada diferencia
figura expresamente en `results.json` y `REPORT.md`.

## Hallazgos y corrección

En la versión anterior, un encabezado ODT configurado a 21pt del borde aparecía
en Writer a 72pt; el pie a 27pt aparecía a 72pt. Writer conservaba los atributos
privados de distancia y Leaf recuperaba sus valores al reabrirlo, ocultando la
diferencia visible. Al retirar esos atributos, Leaf tampoco interpretaba bien
las distancias y márgenes de un ODT independiente creado por Writer.

El cuarto parche aislado de formatos escribe reservas y espaciado dinámico
estándar ODF y convierte las medidas al importar. El mínimo del encabezado/pie
incluye el espaciado con el cuerpo, como en los archivos exportados por Writer.
Solo se aplican reservas de las partes activas de la página maestra. Geometría
estándar y formatos de numeración comunes prevalecen sobre valores privados
obsoletos. Las revisiones del motor y renderizador no cambian.

Los tests de `tests/odt_geometry.rs` usan tres archivos pequeños incluidos en el
repositorio: estándar, partes desactivadas y metadatos deliberadamente obsoletos.
También comprueban conservación del documento vivo y márgenes nominales que
se solapan con las zonas del encabezado/pie. Los cuatro parches aplicados sobre
el WordCraft fijado reproducen exactamente el código `src` vendorizado.

## Resultado de la repetición corregida

Código probado: `125b6ab96db54169e9f652517d2e255039a66e36`.
[Ejecución completa](https://github.com/frandavid100/leaf/actions/runs/41299085581),
artefacto `13080537464`. Descargado de forma independiente; SHA256 del ZIP:
`5578d93f0088e1407dd495d5afc4bf41a338fb3cdd8f440624a2e8c1c385740c`.

| Comprobación | DOCX, una sección | DOCX, tres secciones | ODT, una sección | ODT, tres secciones |
|---|---|---|---|---|
| Texto tras editar/guardar/reabrir | Conservado | Conservado | Conservado | Conservado |
| Variantes de encabezados y campos automáticos | Conservadas | Conservadas | Conservadas | Conservadas |
| Inglés y español reconocidos | Sí | Sí | Sí | Sí |
| Fragmento «no revisar» | Writer lo elimina | Writer lo elimina | Conservado | Conservado |
| Vínculos entre secciones | No procede | Writer crea copias | No procede | Writer crea copias |
| Distancias/márgenes y ODT sin atributos Leaf | Correctos | Correctos | Correctos | Correctos |

Los 108 criterios producen 100 resultados correctos y ocho avisos. Los avisos
corresponden a la exclusión de revisión en DOCX y a los vínculos en las muestras
con varias secciones; no se han omitido del informe para obtener un resultado
global sin diferencias. La captura interna corregida de Writer se ha revisado.

Referencia anterior sin corregir: código
`f1da96ba6a5c39f5a54b23aca36fd5a714a6193c`,
[ejecución](https://github.com/frandavid100/leaf/actions/runs/41298485536),
artefacto `13080586471`, SHA256
`6184493bdf54fbd31d7d54a181e777b2e8feed44293b0e40f18ae792d216c85e`.
El número de criterios creció después para medir explícitamente los vínculos;
no comparar porcentajes agregados de ambas pasadas como una mejora global.

La corrección se distribuye en [alpha.11](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.11). Pasaron 91 tests, compilación GTK optimizada, regresiones de interfaz y comprobaciones del Flatpak instalado/actualizado desde alpha.10. Los hashes del artefacto descargado y de todos los assets publicados coinciden.

## Límites confirmados

- Writer 24.2.7 elimina `w:noProof` al guardar los DOCX de la prueba. La entrada
  contiene ese elemento estándar; la salida carece de él. Inglés y español se
  conservan; el fragmento deja de estar excluido de la revisión. En ODT se
  conserva la exclusión mediante el idioma estándar `zxx`.
- Writer materializa las partes vinculadas de las secciones en copias
  independientes, tanto en DOCX como en ODT. Los textos siguen presentes y la
  presentación se comprueba por separado; editar una copia ya no propaga el
  cambio a la anterior. No se deducen vínculos a partir de textos idénticos,
  porque también pueden ser copias independientes intencionadas.
- No se implementa toda la geometría ODF: bordes, rellenos, altura fija,
  espaciado no dinámico o secciones continuas pueden requerir trabajo adicional.
  Las distancias de partes inactivas, formatos exóticos y márgenes nominales
  solapados conservan las extensiones necesarias para sus conceptos propios.
- Esta muestra no contiene tablas ni imágenes; las pruebas existentes cubren
  esos elementos. No sustituye un corpus representativo de documentos reales,
  Microsoft Word, sesiones manuales en GNOME OS, Orca o una impresora física.

## Repetir la comprobación

En un equipo con Rust, Writer, `python3-uno` y `poppler-utils`:

```sh
cargo build --release --locked --example interop_audit
/usr/bin/python3 tools/interop_audit.py
```

También puede ejecutarse manualmente `interop-audit.yml` en GitHub Actions.
El marcador `[interop-audit]` permite una ejecución deliberada al modificar el
trabajador o el conversor; `[interop-audit-only]` evita repetir las pruebas GTK
cuando solo se ajusta el diagnóstico. Las versiones normales no lo ejecutan.
Los archivos de salida se guardan en `interop-results/`; la evidencia resumida
y sus identificadores se recogen en `INTEROPERABILITY_VALIDATION.json`.
