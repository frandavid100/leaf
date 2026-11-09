# Decisiones vigentes

## 9/11/2026 — procedimiento habitual: RC, prueba y promoción sin cambios

David establece como procedimiento habitual entregar primero una RC validada
para probarla. Con su visto bueno para esa candidata y sin correcciones, publicar
los mismos archivos y notas congeladas, sin recompilar ni repetir la batería ya
superada. Se conservan las verificaciones de fuente, tag y SHA-256 al publicar.
Si encuentra fallos y hay cambios, preparar otra RC validada para volver a probar.
Las autorizaciones automáticas anteriores fueron para aquellas entregas; no son
una autorización permanente para publicar futuras versiones. La alpha.41 sigue
como candidata hasta recibir su visto bueno. [Procedimiento](CANDIDATES.md).

## 9/11/2026 — RC alpha.41 verificada y diez cierres

Se cierran #107, #152, #217 y #226–#232 después de validar código, interfaz y
paquetes instalados GNOME 49/51. El ejecutable de `2d9b1fd` se conserva sin
recompilar; la candidata `d6adbe5` verifica equivalencia de fuente y bytes.
Los ajustes posteriores afectan solo a la descarga de artefactos y a perfiles/
sincronización de los controladores de pruebas. La RC queda descargable para
probarla; la última alpha pública sigue siendo la 40. Se mantienen los cinco
informes pendientes y los aplazamientos. [Identidad y evidencia](ALPHA41_INTERFACE.md).

## 9/11/2026 — candidata 0.17 y nuevos issues

David pide una RC con WordCraft 0.17 y #217, añadiendo los issues recientes
#226–#232. Se adapta el par neutro predominante del HTML externo de Chrome;
los colores locales intencionados se conservan. WordCraft sigue siendo el
modelo, el renderizador y el historial. Preferencias solo guarda disposición
de controles, nunca datos del documento. Las copias de documentos vivos siguen
escribiéndose pero no se ofrecen como recuperación ni para descarte. No publicar
ni cerrar por una validación parcial; registrar pruebas y límites de la candidata.


## Publicada y verificada automáticamente: 0.1.0-alpha.40

Se publican los mismos bytes de la candidata tras superar todas las pruebas
nativas y de instalación. Se verifican fuente, tag y trece archivos públicos antes
de cerrar #218, #219, #220, #221, #222. [Identidad y límites](LATEST_RELEASE.md).

## 9/11/2026 — nueva revisión y alpha.40

David autoriza resolver los issues nuevos verificables y publicar otra alpha.
#218 sustituye la galería sobre el folio por un Inicio reversible en el lateral;
#219–#222 devuelven el formato a una barra inferior con cuatro grupos. Se conserva
el motor y los permisos. El descarte exige confirmación porque la API pública
no ofrece una retención reversible que permita Deshacer con garantías. Las
capturas públicas, Windows ocasional y los bloqueos externos conservan su alcance.
[Implementación, pruebas y límites](ALPHA40_INTERFACE.md).



## Publicada y verificada automáticamente: 0.1.0-alpha.39

Se publican los mismos bytes de la candidata tras superar todas las pruebas
nativas y de instalación. Se verifican fuente, tag y trece archivos públicos antes
de cerrar #212, #213, #214. [Identidad y límites](LATEST_RELEASE.md).

## 8/11/2026 — alpha.39: miniaturas y proceso de publicación

David autoriza aplicar la aceleración de CI, resolver los issues nuevos y publicar
otra alpha. Se conservan las comprobaciones completas y los bytes del único build;
se reparten trabajos y runtimes, y las capturas esperan a la escena renderizada.
Las recuperaciones se previsualizan sobre copias privadas mediante API pública;
los registros originales conservan sus bytes y disponibilidad. La publicación
verifica trece archivos antes de cerrar el plan explícito y actualizar documentos.
[Alcance y medición pendiente](ALPHA39_PREVIEWS.md).

## 8/11/2026 — alpha.38 publicada y verificada

David autoriza otra release cerrando lo verificable e incluyendo PR #210.
Se completan #207/#209 y se distribuye #208: filas iniciales condicionales,
recuperación directa y retirada nativa de 180 ms al escribir, con Navegación/Revisión
fija. El folio sigue editable; cargar, cancelar o retirar las filas no descarta
copias pendientes. Los resultados obsoletos no sustituyen ediciones ni quitan el
foco a otra pestaña. La compensación observa la solicitud de tamaño de GTK antes
de asignar posiciones; no usa el tamaño del fotograma anterior.

Cierres posteriores a validación nativa, GNOME 49/51, comparación del ejecutable
y verificación de trece assets. Permanecen los bloqueos del motor/portal/tienda y
los aplazamientos de David. Se conserva la entrega Windows ocasional; esta alpha
es Linux. [Fuente, pruebas y límites](ALPHA38_START_VIEW.md).

## 8/11/2026 — Navegación/Revisión fija al pie del panel

David aclara #200 tras observar la regresión #208: las dos pestañas inferiores
deben permanecer fijas; los selectores superiores sí pertenecen al contenido
desplazable. Se mantiene una sola barra en el AdwToolbarView común, accesible
también desde Búsqueda, sin copias en las listas. La prueba geométrica anterior
esperaba erróneamente que se desplazase y se corrige junto con la interfaz.
Integrada en main mediante PR #210 después de superar la batería nativa completa;
no se publica una nueva alpha. #209 registra por separado la retirada animada
hacia arriba de las filas transitorias al empezar a editar.
[Implementación y validación](SIDEBAR208.md).

## 8/11/2026 — alpha.37 publicada y verificada

David autoriza crear los informes de salto de Estructura, foco/continuidad y
nombre de guardado, resolver los bugs recientes y publicar otra alpha.
Se cierran #199–#204 después de la batería nativa completa, GNOME 49/51,
comparación del ejecutable y verificación del tag y trece archivos públicos.

Galería y página comparten desplazamiento; cada panel conserva sus propios
controles y posición. Las restauraciones esperan la asignación de tamaño de GTK
y se cancelan si interviene el usuario. Los reajustes repintan la página sin
mover la galería. Los diálogos propios se integran en su ventana; los portales
siguen siendo nativos y los campos conservan Deshacer/Rehacer.

Para #201 se activa exclusivamente la API v1_5 del crate libadwaita existente,
sin cambiar su versión, Cargo.lock, motor, perfiles ni parches. Esta excepción
concreta queda comprobada en CI. Los nombres sugeridos son una operación de
lectura: no modifican título, selección ni historial. Se conservan Windows,
los bloqueos anteriores y las capturas públicas de David.
[Fuente, ejecución y límites](ALPHA37_INTERFACE.md).

## 8/11/2026 — Windows publicada y comprobada

Publicada `0.1.0-alpha.36-windows.1`: 210 pruebas Rust Windows, 211 Linux y
comprobaciones reales de los paquetes portátiles e instalados en/es, incluido
traslado a ruta Unicode, guardado/reapertura, SVG, PDF, preferencias y ventanas.
Los avisos finales se completan sin alterar el ejecutable; después se repiten
las pruebas de ambos paquetes y se publican esos bytes, con cuatro assets y tag
verificados. Impresión física, accesibilidad y GPU reales quedan sin validación.
[Identidad, ejecuciones y digests](WINDOWS.md).

## 8/11/2026 — primera versión para Windows

David solicita preparar una primera versión experimental del estado actual.
Se mantiene el cliente GTK/libadwaita, WordCraft y los parches fijados.
Las entregas Windows se generan de forma ocasional, sin exigir paridad con cada
alpha Linux. El instalador es por usuario, sin privilegios administrativos;
el ZIP usa las mismas carpetas de configuración y recuperación del usuario.
Publicar solo los bytes comprobados en Windows, registrando las limitaciones
de impresión, accesibilidad y pruebas humanas. [Detalle](WINDOWS.md).

## 8/11/2026 — revisión de interfaz alpha.36

La galería transitoria sustituye el diálogo de plantillas. Fuentes vive dentro
de Propiedades; la búsqueda recibe caracteres pegados sin selector especial.
Estructura conserva anclas visuales y usa menús para mover títulos. El cursor de
arrastre es pasivo y su posición es el único destino válido de la importación.
Publicada y verificada con nueve cierres. #171 permanece abierto: se conserva
`xdg-templates:ro` y se desactiva el selector de otra carpeta en Flatpak mientras
el portal no permita el acceso mínimo requerido. No sustituirlo por acceso de
escritura ni por un permiso amplio. [Fuente, verificación y límites](ALPHA36_INTERFACE.md).


## 2026-11-08: ficha, colores y novedades

David aprueba la descripción y los colores claro `#E4F2E8` y oscuro `#1B3327`,
y autoriza implementar las novedades y su sincronización. Las nuevas capturas
siguen a su cargo y esperan corregir el espacio de la barra de título; se
revisarán una a una. No se inicia una release de binarios por este cambio.
La fuente editorial bilingüe genera AppStream y las notas utilizadas por la
publicación de GitHub. FlatPark debe importar metadatos de la versión exacta
que distribuye, conservando su política y los detalles de instalación.
[Proceso y límites](STORE_METADATA.md).

## 2026-11-07: propuestas aceptadas para alpha.35

David autoriza crear los trece issues #170–#182, implementarlos, cerrar solo los
verificados y publicar automáticamente una alpha. Se mantiene el flujo de estilos
basado en el documento; solo se añade el estilo siguiente. Las plantillas son
archivos normales de la carpeta XDG Templates o de una carpeta elegida mediante
el portal. Se aplazan protección frente a conflictos y selector de símbolos.

Las operaciones estructurales y de fuentes se registran como comandos del motor
mediante su API pública: WordCraft mantiene protección, transacciones, revisión e
historial. No se modifica el motor ni el snapshot de formatos. Los límites de
selección de búsqueda invalidan resultados tras cambios ajenos. Se verifican
las nuevas acciones con pruebas sin GUI, controles GTK y gestos reales, también
en los paquetes GNOME 49/51. [Detalle](ALPHA35_WORKFLOWS.md).


Alpha.35 publicada y verificada el 8/11/2026; los trece informes están cerrados.
Se conservan la prueba de ciclo de vida de los controles, las regresiones nativas
e instaladas y la comparación del ejecutable antes de promoverlo sin recompilar.
La publicación rechaza candidatas superadas por otra fuente en main.
[Identidad y comprobaciones](ALPHA35_WORKFLOWS.md).

Estas decisiones proceden de David y del trabajo ya aceptado. Prevalecen sobre
propuestas históricas; una hoja de ruta no autoriza por sí sola nuevas funciones.

| Tema | Decisión | Consecuencia al modificar código |
| --- | --- | --- |
| Publicación autónoma de arreglos pendientes | David pide resolver los issues restantes y generar la siguiente release sin otro visto bueno el 7/11/2026. | Publicar tras verificar arreglos reales; mantener abiertos los bloqueos de integración o prueba. La revisión actual no aporta cambios de código. [Seis pendientes](REMAINING_AFTER_ALPHA34.md). |
| Diccionarios #84 — alpha.34 publicada | David autoriza preparar una candidata para probar en su sistema el 7/11/2026. | Carpeta mediante el selector/portal, sin ampliar permisos del host; conservar importación, datos incluidos, idioma/documento e historial. Issue abierto hasta su prueba GNOME OS. [Alcance](ISSUE84_DICTIONARIES.md) y [validación de la entrega](ALPHA34_VALIDATION.md). |
| Alpha.33 publicada/verificada | David pide actuar sobre #9/#99/#156 y conserva la autorización de publicación automática tras comprobar la entrega. | Cerrado #156 por dos fallos nativos reproducidos y corregidos; #9 tiene explicación/controles y #99 reproducción pública del motor. Ambos permanecen abiertos. Batería completa, GNOME 49/51 y doce archivos públicos verificados. [Fuentes y límites](ALPHA33_VALIDATION.md). |
| Revisión posterior de alpha.32 | David pide revisar si pueden cerrarse los diez informes restantes. | Cerrado #100 por requisitos verificados en la release existente; no atribuir una causa histórica desconocida ni exigirla como condición de cierre. #152 sigue reproducible en 0.16.0. [Evidencia y nueve pendientes](ISSUE_TRIAGE_AFTER_ALPHA32.md). |
| Alpha.32 publicada/verificada | David autoriza el 7/11/2026 actualizar a la última WordCraft estable, adaptar Leaf, optimizar rendimiento y claridad y publicar otra release. | Publicada con WordCraft 0.16.0, mayor ODT y pegado HTML/RTF, después de la batería nativa y GNOME 49/51. Datos fijados, compatibilidad residual revisada y comparación con el mismo motor; doce archivos públicos comprobados. No se cierran issues adicionales. [Revisión](WORDCRAFT_016_REVIEW.md). |
| Alpha.31 publicada/verificada | David autoriza el 7/11/2026 resolver/cerrar lo verificable y publicar automáticamente sin otro visto bueno. | Publicada el 7/11/2026 tras toda la validación nativa/instalada, con los mismos bytes y doce digests públicos comprobados. Cerrados #135/#149/#150/#151/#155; sustituye las aprobaciones pendientes para esta entrega. [Resultados y límites](ALPHA31_REMAINING.md). |
| Tercera candidata alpha.31 validada | Completados #121/#124/#126/#127 mediante propiedad canónica de leyendas, controles compartidos, navegación tipada y decoración temporal. Batería nativa completa y GNOME 49/51 verificados. | [Fuente, sumas y límites](CAPTION_OWNERSHIP.md). Conservar texto/historial, no inferir propiedad por proximidad sin atribuir a #132 una corrección no identificada; David confirmó el original y se cerró el 7/11/2026. La aprobación previa de aquella candidata quedó sustituida por la autorización posterior de publicación automática tras todas las comprobaciones. |
| Segunda candidata alpha.31 histórica | En aquella entrega se cerraron #134–#136; #135 se reabrió después y volvió a cerrarse en la última pasada. Se aisló la captura de eventos de la regla; conserva #133. Batería nativa completa, GNOME 49/51 y menús realmente traducidos verificados sobre los mismos archivos. | [Fuente, sumas y límites](ALPHA31_UI_VALIDATION.md). La aprobación previa de aquella candidata quedó sustituida por la autorización posterior para la última pasada alpha.31; conservar su evidencia histórica. |
| Primera candidata alpha.31 histórica | Solo cerraba #133; los experimentos de interfaz se retiraron en aquella entrega. | [Identidad histórica](ALPHA31_VALIDATION.md). La segunda candidata la sustituye; no reutilizar su lista de issues como estado actual. |
| Revisión visual #111–#121 | David autoriza incorporar los once issues nuevos a la misma candidata. Revisión reúne correcciones/cambios/comentarios; modos Editar/Sugerir, filas de formato con casillas mixtas y Elementos en cabecera con creación/propiedades compartidas. | Conservar flotación fijada, selección, herencia y API públicas. #121 sustituye nombre/ubicación de Insertar de #109, #116 el icono anterior, #120 las rejillas anteriores; conservar capacidades. |
| Navegación #131 | Una entrada principal Navegación, con Páginas/Estructura dentro del panel; Revisión es la otra entrada principal. Sustituye los tres destinos principales anteriores. | Conservar por pestaña representación, filtro, expansión y scroll independiente; sincronizar destino real sin alterar selección/documento al cambiar de vista. |
| Alpha.30 publicada/verificada | Publicación automática autorizada y completada el 6/11/2026; integra la candidata 29 y siete cierres nuevos. | [CI, fuente, sumas y límites](ALPHA30_VALIDATION.md). Sin recompilar al promover; #121/#124/#126/#127 conservan trabajo. |
| Candidatas y alpha.30 | David autoriza explícitamente publicar esta nueva pasada automáticamente, sin esperar su visto bueno. | Conservar pruebas nativas e instaladas y promover los mismos bytes; `[auto-publish-alpha]` habilita la promoción automática solo después del éxito de una candidata de main. Sin ese marcador se mantiene el proceso de aprobación. |
| Alpha.28 publicada/verificada | David autoriza el 5/11/2026 notas al pie, índice, referencias cruzadas, imágenes flotantes, zoom/navegación y galería pública. | Reutilizar historias, campos y comandos de WordCraft, verificar DOCX/PDF e historial y conservar las guardas ODT. La autorización de galería sustituye el aplazamiento anterior para esta entrega. [CI nativa e instalada completa](https://github.com/frandavid100/leaf/actions/runs/42057994674). |
| Rama de desarrollo | `main` para desarrollo, compilación y PR; ambos historiales integrados. | ApexDesk archivado en `archive/apexdesk-main-2026-11-02`; conservar `wordcraft-saving` y releases/tags existentes. [Revisión](MAIN_MIGRATION.md). |
| Modelo del documento | WordCraft es la fuente de verdad. | Usar Session, layout y comandos públicos; sin editor oculto ni historial paralelo. |
| Frontera con upstream | Optimizar Leaf; proponer en WordCraft los cambios de su motor. | No modificar engine/render ni dependencias para un ajuste del cliente. Conservar las excepciones ODT ya registradas en [patches](../patches/README.md). |
| Actualizaciones de la base | David autoriza el 2/11/2026 actualizar a la última versión estable: WordCraft 0.14.0, commit `5cd53f184aab11def13b7fced18727159b9bc96a`. | Motor y ajustes de interfaz en cambios separados; formatos recopiado y cuatro parches ODT reaplicados. Seguir [UPDATING_WORDCRAFT](../UPDATING_WORDCRAFT.md) y validar antes de publicar. |
| Regla | Escalas transitorias limitadas a la hoja visible; números más legibles y arrastres con imán de 0,5 cm (#118). | Conservar teclado, geometría importada y precisión de entradas numéricas. Márgenes en ambos ejes, sangrías/tabulaciones con su origen correcto, vista previa, cancelación y un paso de Deshacer. |
| Revisión tras alpha.21 | Integrar el PDF específico de impresión de WordCraft 0.14; separar entrada/diálogos y limpiar formato; evitar trabajo de estilos ocultos. | [Investigación y comprobaciones](WORDCRAFT_014_REVIEW.md). Mantener el motor/dependencias y el comportamiento restante; no es autorización para implementar toda la interfaz de WordCraft. |
| Diseño | GTK4/libadwaita y convenciones GNOME. | Conservar controles nativos, orden Estilo → Párrafo → Texto, adaptación ancho/estrecho y menús contextuales contenidos. |
| Ciclo de vida | Propietarios explícitos, callbacks débiles y limpieza al cerrar. | No reintroducir ciclos Rc/widget; comprobar cierre de pestañas/ventanas y trabajadores. |
| Marcadores y comentarios | Marcadores bajo sus títulos en Navegación; comentarios y respuestas en Revisión junto a correcciones y cambios (#117/#119). Creación desde Elementos/contexto (#121). | WordCraft conserva datos e historial. Mantener conectores y autoría; DOCX conserva los hilos y ODT queda protegido mientras su exportador los omita. |
| Ortografía y gramática | Correcciones en el panel Revisión con Aplicar/Ignorar y acciones ampliadas. Preferencias fija nuevos documentos; Propiedades fija herencia y Texto las excepciones. | Invalidar posiciones obsoletas, coalescer el trabajo y cancelarlo al cerrar la pestaña. El modo Editar/Sugerir es independiente. |
| Metadatos | Editar título, asunto, autor, palabras clave y categoría en Propiedades → Información, usando `file.properties`. | No sobrescribir otras propiedades ni la identidad de la sesión. DOCX conserva los cinco campos; bloquear ODT con categoría para impedir su pérdida. Aplicar es un solo paso de deshacer. |
| Alpha.23 / issues | Cerrados #64, #65, #7 y #37 tras validar la alpha instalada. | Guardar como DOCX para los saltos que el exportador ODT no conserva. Propiedades usa el selector inline desde libadwaita 1.7 y mantiene el selector anterior en versiones compatibles más antiguas. |
| Alpha.22 / issues | Publicada con WordCraft 0.14.0, impresión y optimización, siete cierres y validación del Flatpak instalado. | Estilos usa la galería rápida cuando existe. Las sangrías conservan sus controles directos tras evaluar la alternativa agrupada. Mantener abiertos #65 y #69 hasta resolver las limitaciones descritas en [la revisión](FORMAT_POPOVER_REVIEW.md). |
| Alpha.21 / issues | Publicada y verificada con WordCraft 0.14.0; cuatro cierres (#18, #48, #51, #61) y diez abiertos. Reglas transitorias finas, historial AdwSplitButton y Fuente/Tamaño acotados. | [Alcance](ISSUE_SWEEP_ALPHA21.md) y [evidencia](ISSUE_SWEEP_ALPHA21_VALIDATION.json). Mantener guardas ODT y límites de autoría; actualizar la referencia de reutilización solo a fuente/CI validadas. |
| Alpha.20 / issues (histórico) | Publicada; seis cierres y 12 pendientes. Regla y selector GNOME quedaron con revisión visual pendiente, completada en alpha.21. Cuatro guardas ODT adicionales evitan pérdida estructural. | [Alcance](ISSUE_SWEEP_ALPHA20.md) y [evidencia](ISSUE_SWEEP_ALPHA20_VALIDATION.json). No presentar el bloqueo ODT como autoría avanzada completa; el diseño con pin de alpha.20 queda sustituido por #48. |
| Alpha.19 / issues | Publicada y verificada; nueve cierres de los nueve informes autorizados. #35 reproducido en Flatpak GNOME 49 y verificado tras corregir filtro y foco. | [Alcance](ISSUE_SWEEP_ALPHA19.md) y [validación](ISSUE_SWEEP_ALPHA19_VALIDATION.json). Mantener eventos reales, drenaje de entrada antes de aserciones y pruebas instaladas; no afirmar cobertura humana Wayland/IME. |
| Alpha.18 / issues | Publicada y verificada; diez cierres de la pasada inicial y 20 abiertos tras nuevos/reabiertos. | [Alcance](ISSUE_SWEEP_ALPHA18.md) y [validación](ISSUE_SWEEP_ALPHA18_VALIDATION.json). No dar por resueltos requisitos posteriores con pruebas de las especificaciones anteriores. |
| Alpha.17 / issues | Publicada y verificada; 18 informes cerrados y 14 abiertos al finalizar. | [Alcance](ISSUE_SWEEP_ALPHA17.md) y [validación](ISSUE_SWEEP_ALPHA17_VALIDATION.json). Nuevos #40–#43 quedan fuera de esta entrega; en aquella entrega main conservaba el prototipo. |
| Alpha.16 / issues | Publicada; 32 revisados, 14 cerrados y 18 abiertos. #12 conserva trabajo parcial pendiente. | [Evidencia y límites](ISSUE_SWEEP_ALPHA16.md); cerrar solo lo verificado y mantener las tareas restantes. |
| Guardado explícito | Ctrl+S guarda sobre el archivo abierto; Guardar como elige otro destino. | Recuperación sigue predeterminada; fallos conservan los cambios. |
| Controles de formato | Cambios escalares válidos inmediatos; paneles fijados siguen selección. | Validar entrada, evitar ediciones al sincronizar y agrupar deshacer. Excepciones atómicas documentadas en la revisión. |
| Modos Editar/Sugerir | Elegir si las nuevas ediciones se registran como propuestas por documento (#117). | Cambiar de modo no acepta/rechaza ni descarta revisiones existentes; sus acciones siguen disponibles en ambos modos y son independientes de ortografía/comentarios. |
| Identidad | Preferencias configura autor de nuevos comentarios y revisiones. | Valor inicial de cuenta del sistema; no reescribir autorías anteriores ni metadatos. |
| Alpha.15 acumulada | Publicada con refinamiento de paneles y metadatos; CI nativa e instalada y descarga verificadas. | Mantener [evidencia y límites](SIDEBAR_REFINEMENT_VALIDATION.json); las propuestas posteriores no son tareas automáticas. |
| Alpha.14 acumulada | Publicada con anotaciones y revisión lateral tras validación nativa/instalada. | Conservar alcance, evidencia y límites de [la entrega](BOOKMARKS_COMMENTS_VALIDATION.json); nuevas ampliaciones requieren su propia petición. |
| Conflictos con cambios externos | Aplazados para priorizar marcadores/comentarios. | No iniciar esa ampliación ahora. |
| Orca | La primera etapa se conserva; ampliaciones aplazadas. | Mantener las regresiones actuales sin iniciar tablas/imágenes accesibles por cuenta propia. |
| Pruebas con personas | Esperar a que haya usuarios disponibles. | Diferenciar éxito automático de prueba humana; no bloquear trabajo independiente. |
| Capturas | Usar capturas públicas reales proporcionadas por David; la principal ya está incorporada al README. | Conservar procedencia; no sustituirlas por capturas de CI. Las capturas internas son solo validación. |
| Traducciones | Inglés/español disponibles; colaboración por PO. Weblate preparado para `main`; alta/autorización pendientes de David. | No repetir el acceso remoto; conservar mensajes/placeholders y herramientas gettext. |
| Corpus | Comprobaciones extensas manuales cuando se justifiquen. | No imponer el corpus completo por versión; usar regresiones pequeñas para fallos concretos. |
| FlatPark | Publicar el árbol `/app` en `Leaf-<version>-flatpark-x86_64.tar.xz`, sin carpeta contenedora. | Recursos relativos al ejecutable; validar extracción bajo `/app/extra/leaf` con GNOME 51 antes de publicar. Avisar a FlatPark de cambios de desktop/iconos/metainfo y nuevos requisitos de runtime. Confirmar por separado la publicación en su catálogo. |
| Distribución | GitHub Flatpak experimental, GNOME 49, permisos mínimos y portales. | Una nueva release exige fuentes identificables y pruebas instaladas. No iniciar Flathub automáticamente. |
| Licencia y autoría | GPL-3.0-or-later y desarrollo con IA declarado. | No cambiar licencia, atribuciones ni prometer revisión humana inexistente. |

Las leyendas usan propiedad explícita en el documento canónico: marcadores para
imágenes y leyendas, controles OOXML desbloqueados para tablas. Mostrar/ocultar
retiene el texto y el historial nativo; no se atribuyen leyendas por proximidad.
Navegación y decoración derivan de esos datos. #132 se cerró el 7/11/2026 tras
la confirmación de David con el documento original; el disparador y la corrección
concreta siguen sin identificar.
[Implementación y límites](CAPTION_OWNERSHIP.md).

El [historial](history/README.md) conserva el razonamiento y las entregas anteriores.
Las propuestas del [plan](../ROADMAP.md) pueden elegirse en una conversación
posterior; las decisiones de esta página deben seguir visibles al hacerlo.
