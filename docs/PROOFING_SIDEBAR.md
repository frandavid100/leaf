# Ortografía, gramática e idioma

La tercera revisión sustituye el interruptor global de revisión al escribir.
Preferencias ofrece idioma y revisión predeterminados **solo para documentos
nuevos**; Propiedades → Información cambia la herencia del documento. Texto →
Idioma ofrece No revisar, Predeterminado y los idiomas. Predeterminado borra las
propiedades directas; un idioma explícito activa revisión en ese fragmento.
Los estilos y fragmentos explícitos no se reescriben al cambiar el documento.

La cuarta pestaña lateral muestra la lista sin instrucciones iniciales ni botón
Actualizar. Se renueva con debounce tras ediciones o cambios de configuración,
descartando resultados de otra identidad/revisión. Clic solo selecciona y revela
el texto; las correcciones están en el menú contextual. F7 abre la misma lista.
La revisión automática usa las propiedades efectivas de cada fragmento. Cambiar
Preferencias no altera documentos abiertos, errores visibles ni su historial.

DOCX conserva herencia y excepciones; el exportador ODT fijado aplana los idiomas
y pierde ciertos valores false. Leaf requiere DOCX al desactivar revisión en el
documento o crear una excepción explícita que la active. La herencia de idioma
por sí sola sigue siendo una limitación del ODT ordinario.

La configuración del documento se puede deshacer. La apertura, el análisis y
la navegación no cambian contenido ni estado modificado. Los diccionarios sin
instalar se indican en el selector; no se añaden motores alternativos.

[Alcance y validación alpha.18](ISSUE_SWEEP_ALPHA18.md) ·
[Panel](../src/gnome/spellcheck.rs) · [Modelo](../src/proofing_defaults.rs) ·
[Preferencias](../src/gnome/workspace.rs).
