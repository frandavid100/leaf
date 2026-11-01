# Marcadores y comentarios

## Alcance autorizado

Crear desde **Insertar** y el menú contextual; gestionar ambos en la tercera
pestaña **Marcadores y comentarios**. Modelo, historial y serialización pertenecen
a WordCraft; Leaf solo añade adaptadores, controles y protección de guardado.

- Marcadores: nombre único visible, creación sobre el cursor o selección del cuerpo,
  navegación al intervalo y eliminación. Los nombres reservados que empiezan por
  `_` siguen ocultos y no se crean desde esta interfaz.
- Comentarios: crear sobre selección/palabra, editar, responder, resolver/reabrir
  y eliminar con confirmación. Eliminar un hilo elimina también sus respuestas;
  las operaciones de documento se pueden deshacer/rehacer.
- La navegación selecciona el texto de referencia sin ensuciar el documento.
  Las respuestas heredan el anclaje; comentarios sin anclaje visible se muestran
  sin botón de navegación activo. Se toleran padres ausentes y ciclos importados.
- Diálogos GTK modales, cancelación con Escape y envío de texto con Ctrl+Enter.
  Ctrl+Mayús+F5 añade marcador; Ctrl+Alt+M añade comentario desde el documento.
- Las vistas se reconstruyen tras cambios o al abrir el panel. Callbacks débiles,
  revisión/identidad del documento y cierre del editor al liberar una pestaña.

El autor sigue siendo el de la sesión de WordCraft; no se añade una preferencia
nueva de identidad. No se implementan globos en los márgenes ni otra etapa Orca.

## Formatos: límite importante

**DOCX:** comprobar nombres/intervalos de marcadores, comentarios, respuestas,
texto Unicode y estado resuelto tras guardar/reabrir.

**ODT con el WordCraft fijado:** el modelo de intercambio reduce un marcador a
`Inline::Anchor`, escrito como `text:bookmark`; al importar se reconstruye como
marcador puntual. Conserva nombre y ubicación, pero pierde el intervalo original.
Los objetos de comentario no tienen representación en ese modelo y el importador
ODT omite las anotaciones. No afirmar compatibilidad de comentarios ODT.

Leaf rechaza el guardado ODT si la sesión contiene comentarios, antes de escribir
el archivo o limpiar el estado modificado. Guardar propone extensión DOCX. El
mismo guardado protegido sirve al autoguardado directo; cuando el destino ODT es
incompatible, se mantiene la recuperación nativa y se avisa una vez por error.
El diálogo de comentarios avisa de la necesidad de DOCX.

### Reproducción lista para upstream

`tests/annotations.rs::odt_bookmark_locations_survive_but_comments_must_not_be_silently_saved`
construye un documento pequeño, crea un marcador sobre «Café», exporta ODT y lo
reabre: el nombre sigue, pero la selección ya es puntual. Después añade un
comentario, comprueba que Leaf protege el archivo original y usa deliberadamente
`wordcraft_engine::io::save_bytes("upstream.odt", &session.doc)` para reproducir
la omisión del comentario fuera del guardado de Leaf.

Código upstream afectado, revisión `165d5725dc25b4a7887d8025c3b1fe506ccc3dcc`:
`crates/formats/src/model/from_doc.rs`, `model/to_doc.rs`, `odt/writer.rs` y
`odt/body.rs`. No se modifica motor, formatos, parches, revisiones o dependencias.
La reproducción está preparada; no se ha abierto una issue upstream desde aquí.

## Comprobaciones

Las 104 pruebas Rust (incluidas cinco de anotaciones), cuatro Python, formato,
catálogos y enlaces han pasado localmente. La validación GTK está bloqueada
localmente porque este entorno prohíbe crear sockets (Xvfb no puede arrancar).
La comprobación de tipos GTK y la compilación optimizada de la fuente final han
pasado, conservando panic unwinding. Se ha verificado `Leaf 0.1.0-alpha.14` con
`--version`. No publicar sin los checks GTK/Flatpak de CI.
No hay una nueva release publicada. [Registro](BOOKMARKS_COMMENTS_VALIDATION.json).

```sh
cargo test --locked --test annotations
cargo build --release --locked --features gui --bin wordcraft-gnome
LANGUAGE=es GSK_RENDERER=cairo xvfb-run -a ./target/release/wordcraft-gnome --annotations-ui-test
```

Cinco regresiones headless cubren historial, nombres duplicados, Unicode,
navegación, restricciones de edición, conservación DOCX y protección ODT.
La comprobación GTK activa la acción real de Insertar, una fila del menú
contextual, botones de responder y acciones de editar/resolver, deshacer/rehacer,
cambio búsqueda/panel y rechazo de un diálogo obsoleto. Comprueba además que el
editor se cierre al liberar el documento. CI ejecuta inglés y español y conserva
capturas internas de ventana ancha, estrecha y oscura. Las capturas públicas
siguen siendo responsabilidad de David.

Mapa: [adaptador](../src/annotations.rs), [panel](../src/gnome/annotations.rs),
[diálogos](../src/gnome/annotation_dialog.rs), [pruebas](../tests/annotations.rs),
[diagnóstico GTK](../src/gnome/diagnostics/annotations.rs).
