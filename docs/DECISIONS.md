# Decisiones vigentes

Estas decisiones proceden de David y del trabajo ya aceptado. Prevalecen sobre
propuestas históricas; una hoja de ruta no autoriza por sí sola nuevas funciones.

| Tema | Decisión | Consecuencia al modificar código |
| --- | --- | --- |
| Modelo del documento | WordCraft es la fuente de verdad. | Usar Session, layout y comandos públicos; sin editor oculto ni historial paralelo. |
| Frontera con upstream | Optimizar Leaf; proponer en WordCraft los cambios de su motor. | No modificar engine/render ni dependencias para un ajuste del cliente. Conservar las excepciones ODT ya registradas en [patches](../patches/README.md). |
| Actualizaciones de la base | Mantener dependencias fijadas y modificaciones separadas. | Seguir [UPDATING_WORDCRAFT](../UPDATING_WORDCRAFT.md) cuando se autorice una actualización. |
| Diseño | GTK4/libadwaita y convenciones GNOME. | Conservar controles nativos, orden Estilo → Párrafo → Texto, adaptación ancho/estrecho y menús contextuales contenidos. |
| Ciclo de vida | Propietarios explícitos, callbacks débiles y limpieza al cerrar. | No reintroducir ciclos Rc/widget; comprobar cierre de pestañas/ventanas y trabajadores. |
| Orca | La primera etapa se conserva; ampliaciones aplazadas. | Mantener las regresiones actuales sin iniciar tablas/imágenes accesibles por cuenta propia. |
| Pruebas con personas | Esperar a que haya usuarios disponibles. | Diferenciar éxito automático de prueba humana; no bloquear trabajo independiente. |
| Capturas | David proporcionará las públicas. | No sustituirlas por capturas de CI. Las capturas internas son solo validación. |
| Traducciones | Inglés/español disponibles; colaboración por PO. Weblate pendiente de David. | No repetir el acceso remoto; conservar mensajes/placeholders y herramientas gettext. |
| Corpus | Comprobaciones extensas manuales cuando se justifiquen. | No imponer el corpus completo por versión; usar regresiones pequeñas para fallos concretos. |
| Distribución | GitHub Flatpak experimental, GNOME 49, permisos mínimos y portales. | Una nueva release exige fuentes identificables y pruebas instaladas. No iniciar Flathub automáticamente. |
| Licencia y autoría | GPL-3.0-or-later y desarrollo con IA declarado. | No cambiar licencia, atribuciones ni prometer revisión humana inexistente. |

El [historial](history/README.md) conserva el razonamiento y las entregas anteriores.
Las propuestas del [plan](../ROADMAP.md) pueden elegirse en una conversación
posterior; las decisiones de esta página deben seguir visibles al hacerlo.
