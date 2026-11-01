# Accesibilidad inicial del texto del documento

Esta primera etapa permite exponer texto normal, cursor y selección a lectores
de pantalla mediante la interfaz nativa `GtkAccessibleText`, implementada por
el propio lienzo visible. No se añade un editor oculto ni se sustituye WordCraft.

## Diseño

`src/accessible_text.rs` construye un índice de lectura de la historia activa.
Traduce caracteres Unicode de AT-SPI a posiciones UTF-8 de WordCraft, conserva
los párrafos vacíos y evita truncar una cadena por un NUL incrustado. Palabras
y oraciones usan las reglas Unicode de la biblioteca de segmentación que ya
utilizaba el motor; no se actualiza ninguna dependencia transitiva. Las líneas
visuales se consultan en `DocLayout::line_span`, incluida la división por ajuste
de línea y el salto entre párrafos.

`src/gnome_canvas.rs` añade la interfaz al lienzo `GtkFixed` que ya contiene las
páginas. Su función accesible es cuadro de texto multilínea. Las consultas leen
el índice; la escritura, el movimiento, la selección y deshacer continúan por
los controladores y comandos de WordCraft existentes. Las posiciones del cursor
y de la selección proceden de la sesión, no de una selección mantenida aparte.

El índice se reconstruye solo al cambiar revisión, documento o historia activa;
mover el cursor no vuelve a copiar el texto. Es una representación derivada y
no editable. Cada cambio compara prefijo/sufijo Unicode y notifica la mínima
sustitución contigua: primero borrado, mientras sigue disponible el texto
anterior, después inserción con el texto nuevo. No se mantiene un préstamo
RefCell durante esas notificaciones. Los callbacks y el widget solo guardan
referencias débiles al estado de la pestaña; el índice se vacía al cerrarla.

El estado de solo lectura refleja las restricciones generales del documento;
las comprobaciones reales de edición y de regiones protegidas siguen siendo
responsabilidad del motor. La interfaz no concede permisos adicionales.

## Comprobación

Los seis tests de `tests/accessible_text.rs` comprueban caracteres multibyte,
emoji con ZWJ, espacios y párrafos vacíos, límites inválidos, palabras/oraciones,
coordenadas de líneas ajustadas, selecciones, encabezados y ausencia de cambios
en documento/historial durante las consultas.

La prueba independiente usa AT-SPI desde Python, teclas reales mediante xdotool
y Orca sin modificar. Consulta el texto y sus desplazamientos, selecciona una
palabra con acento, la sustituye y deshace; el proceso Leaf comprueba que su
documento canónico vuelve exactamente al inicial. Se registran eventos de
inserción, borrado, cursor y selección, y el texto de los anuncios generados por
Orca. Se usa un perfil temporal con contenido sintético, sin documentos privados.

La comparación anterior usa el ejecutable publicado de alpha.11: los controles
nativos aparecen en el árbol AT-SPI, pero su lienzo genérico está omitido. La
prueba nueva exige que aparezca como área de texto y que responda a `Text`.
Los resultados, revisiones y hashes verificados quedan en
[`ACCESSIBLE_DOCUMENT_VALIDATION.json`](ACCESSIBLE_DOCUMENT_VALIDATION.json).
La validación nativa supera 97 tests y la prueba independiente con Orca 46.1
confirma anuncios de párrafo, selección, carácter escrito y texto restaurado.
F6 recorre panel, herramientas y documento antes de iniciar la lectura.
El Flatpak instalado supera la misma prueba con Orca 46.1 y se publica como
[alpha.12](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.12).
Se han descargado y verificado independientemente ambos ZIP de auditoría/paquete,
los siete checksums y los ocho assets publicados.
La prueba anterior confirma que alpha.11 no exponía el texto del lienzo;
la nueva confirma 61 caracteres Unicode, selección Café (26–30), sustitución
por X y restauración exacta del documento canónico al deshacer.

Las transcripciones son evidencia de los anuncios generados automáticamente;
no equivalen a escuchar una sesión completa ni a probar una línea braille.
Las capturas públicas y Weblate continúan a cargo del usuario.

## Alcance de esta entrega

Se prioriza lectura y edición de texto normal mediante teclado. El índice
expone el texto fuente de la historia activa: todavía no es una representación
estructural completa del documento. Los campos y objetos conservan sus marcas
en el texto; tablas, imágenes, relaciones entre celdas, atributos ricos, filtrado
según revisiones/texto oculto y otras estructuras necesitan trabajo posterior.

No se anuncian como implementadas las operaciones remotas de `EditableText`,
la colocación del cursor/selección mediante las API de GTK 4.22 ni la geometría
por carácter de las API posteriores a GTK 4.14. Se conserva GTK mínimo 4.14 y el
runtime GNOME 49. Las limitaciones de esos métodos deben distinguirse de mover
el cursor, seleccionar y escribir con las teclas normales, que siguen usando
el editor real. El acceso completo mediante pantalla táctil/braille y lectores
más recientes requiere validación específica.

Documentación primaria consultada:

- https://docs.gtk.org/gtk4/iface.AccessibleText.html
- https://docs.gtk.org/gtk4/section-accessibility.html
- https://github.com/gtk-rs/gtk4-rs/blob/0.10.3/gtk4/src/subclass/accessible_text.rs
- https://github.com/GNOME/gtk/blob/4.14.5/gtk/gtkaccessibletext.c
- https://github.com/GNOME/orca/tree/ORCA_46_1

## Repetir el diagnóstico

En una sesión de pruebas con Orca y `python3-pyatspi`:

```sh
GTK_A11Y=atspi NO_AT_BRIDGE=0 GSK_RENDERER=cairo LANGUAGE=en \
  xvfb-run -a dbus-run-session -- /usr/bin/python3 \
  tools/accessibility_audit.py ./target/release/wordcraft-gnome
```

El flujo opcional `accessibility-audit.yml` instala los clientes y usa el
ejecutable previamente validado. No exige volver a ejecutar el corpus de
documentos. No usar el marcador de fin del test sobre una sesión de trabajo
normal: la opción `--a11y-ui-test` crea exclusivamente el documento sintético.
