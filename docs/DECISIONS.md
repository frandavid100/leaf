# Decisiones vigentes

Estas decisiones proceden de David y del trabajo ya aceptado. Prevalecen sobre
propuestas históricas; una hoja de ruta no autoriza por sí sola nuevas funciones.

| Tema | Decisión | Consecuencia al modificar código |
| --- | --- | --- |
| Rama de desarrollo | `main` para desarrollo, compilación y PR; ambos historiales integrados. | ApexDesk archivado en `archive/apexdesk-main-2026-11-02`; conservar `wordcraft-saving` y releases/tags existentes. [Revisión](MAIN_MIGRATION.md). |
| Modelo del documento | WordCraft es la fuente de verdad. | Usar Session, layout y comandos públicos; sin editor oculto ni historial paralelo. |
| Frontera con upstream | Optimizar Leaf; proponer en WordCraft los cambios de su motor. | No modificar engine/render ni dependencias para un ajuste del cliente. Conservar las excepciones ODT ya registradas en [patches](../patches/README.md). |
| Actualizaciones de la base | Mantener dependencias fijadas y modificaciones separadas. | Seguir [UPDATING_WORDCRAFT](../UPDATING_WORDCRAFT.md) cuando se autorice una actualización. |
| Diseño | GTK4/libadwaita y convenciones GNOME. | Conservar controles nativos, orden Estilo → Párrafo → Texto, adaptación ancho/estrecho y menús contextuales contenidos. |
| Ciclo de vida | Propietarios explícitos, callbacks débiles y limpieza al cerrar. | No reintroducir ciclos Rc/widget; comprobar cierre de pestañas/ventanas y trabajadores. |
| Marcadores y comentarios | Creación desde Insertar/contexto; marcadores bajo sus títulos en Navegación y tercera pestaña solo para Comentarios, con una burbuja por hilo. | WordCraft conserva datos e historial. DOCX conserva los hilos; bloquear ODT con comentarios mientras el exportador los omita. |
| Ortografía y gramática | Cuarta pestaña con lista sin instrucciones iniciales; clic solo selecciona y muestra el texto. Sin entrada en el menú de documento; correcciones en el contexto del texto. Preferencias solo fija idioma/revisión de documentos nuevos; Propiedades fija la herencia del documento y Texto permite excepciones o restaurarla. | Revisión automática cuando lo indiquen las propiedades efectivas; sin interruptor global independiente. Invalidar posiciones obsoletas y coalescer el trabajo. |
| Metadatos | Editar título, asunto, autor, palabras clave y categoría en Propiedades → Información, usando `file.properties`. | No sobrescribir otras propiedades ni la identidad de la sesión. DOCX conserva los cinco campos; bloquear ODT con categoría para impedir su pérdida. Aplicar es un solo paso de deshacer. |
| Alpha.20 / issues | Publicada; seis cierres y 12 pendientes. Regla y selector GNOME implementados, con revisión visual instalada aún pendiente. Cuatro guardas ODT adicionales evitan pérdida estructural. | [Alcance](ISSUE_SWEEP_ALPHA20.md) y [evidencia](ISSUE_SWEEP_ALPHA20_VALIDATION.json). No cerrar #18/#48 sin completar su revisión visual ni presentar el bloqueo ODT como autoría avanzada completa. |
| Alpha.19 / issues | Publicada y verificada; nueve cierres de los nueve informes autorizados. #35 reproducido en Flatpak GNOME 49 y verificado tras corregir filtro y foco. | [Alcance](ISSUE_SWEEP_ALPHA19.md) y [validación](ISSUE_SWEEP_ALPHA19_VALIDATION.json). Mantener eventos reales, drenaje de entrada antes de aserciones y pruebas instaladas; no afirmar cobertura humana Wayland/IME. |
| Alpha.18 / issues | Publicada y verificada; diez cierres de la pasada inicial y 20 abiertos tras nuevos/reabiertos. | [Alcance](ISSUE_SWEEP_ALPHA18.md) y [validación](ISSUE_SWEEP_ALPHA18_VALIDATION.json). No dar por resueltos requisitos posteriores con pruebas de las especificaciones anteriores. |
| Alpha.17 / issues | Publicada y verificada; 18 informes cerrados y 14 abiertos al finalizar. | [Alcance](ISSUE_SWEEP_ALPHA17.md) y [validación](ISSUE_SWEEP_ALPHA17_VALIDATION.json). Nuevos #40–#43 quedan fuera de esta entrega; en aquella entrega main conservaba el prototipo. |
| Alpha.16 / issues | Publicada; 32 revisados, 14 cerrados y 18 abiertos. #12 conserva trabajo parcial pendiente. | [Evidencia y límites](ISSUE_SWEEP_ALPHA16.md); cerrar solo lo verificado y mantener las tareas restantes. |
| Guardado explícito | Ctrl+S guarda sobre el archivo abierto; Guardar como elige otro destino. | Recuperación sigue predeterminada; fallos conservan los cambios. |
| Controles de formato | Cambios escalares válidos inmediatos; paneles fijados siguen selección. | Validar entrada, evitar ediciones al sincronizar y agrupar deshacer. Excepciones atómicas documentadas en la revisión. |
| Modo Revisión | Edición con seguimiento de cambios por documento. | No aceptar/rechazar cambios al cambiar el modo; independiente de ortografía. |
| Identidad | Preferencias configura autor de nuevos comentarios y revisiones. | Valor inicial de cuenta del sistema; no reescribir autorías anteriores ni metadatos. |
| Alpha.15 acumulada | Publicada con refinamiento de paneles y metadatos; CI nativa e instalada y descarga verificadas. | Mantener [evidencia y límites](SIDEBAR_REFINEMENT_VALIDATION.json); las propuestas posteriores no son tareas automáticas. |
| Alpha.14 acumulada | Publicada con anotaciones y revisión lateral tras validación nativa/instalada. | Conservar alcance, evidencia y límites de [la entrega](BOOKMARKS_COMMENTS_VALIDATION.json); nuevas ampliaciones requieren su propia petición. |
| Conflictos con cambios externos | Aplazados para priorizar marcadores/comentarios. | No iniciar esa ampliación ahora. |
| Orca | La primera etapa se conserva; ampliaciones aplazadas. | Mantener las regresiones actuales sin iniciar tablas/imágenes accesibles por cuenta propia. |
| Pruebas con personas | Esperar a que haya usuarios disponibles. | Diferenciar éxito automático de prueba humana; no bloquear trabajo independiente. |
| Capturas | Usar capturas públicas reales proporcionadas por David; la principal ya está incorporada al README. | Conservar procedencia; no sustituirlas por capturas de CI. Las capturas internas son solo validación. |
| Traducciones | Inglés/español disponibles; colaboración por PO. Weblate preparado para `main`; alta/autorización pendientes de David. | No repetir el acceso remoto; conservar mensajes/placeholders y herramientas gettext. |
| Corpus | Comprobaciones extensas manuales cuando se justifiquen. | No imponer el corpus completo por versión; usar regresiones pequeñas para fallos concretos. |
| Distribución | GitHub Flatpak experimental, GNOME 49, permisos mínimos y portales. | Una nueva release exige fuentes identificables y pruebas instaladas. No iniciar Flathub automáticamente. |
| Licencia y autoría | GPL-3.0-or-later y desarrollo con IA declarado. | No cambiar licencia, atribuciones ni prometer revisión humana inexistente. |

El [historial](history/README.md) conserva el razonamiento y las entregas anteriores.
Las propuestas del [plan](../ROADMAP.md) pueden elegirse en una conversación
posterior; las decisiones de esta página deben seguir visibles al hacerlo.
