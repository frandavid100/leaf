# Continuidad y mantenimiento de Leaf

Antes de editar, leer [PROJECT_STATUS](PROJECT_STATUS.md), [ROADMAP](ROADMAP.md),
[DECISIONS](docs/DECISIONS.md) y el [mapa de código](docs/ARCHITECTURE.md).
El historial archivado conserva pruebas anteriores; no es una lista de tareas vigente.

La rama activa es **wordcraft-saving**. Comprobar rama, remoto y cambios locales;
no descartar trabajo preexistente. main conserva el antiguo prototipo ApexDesk.
La [guía para otro chat](docs/CONTINUING.md) explica cómo retomar el proyecto.

## Frontera del cambio

La interfaz es GTK4/libadwaita. WordCraft conserva documento, comandos, historial,
maquetación y renderizado. Usar API públicas; no crear un editor/modelo paralelo.
Las optimizaciones de Leaf no deben modificar engine/render, dependencias ni
parches existentes. Proponer allí un defecto del motor con reproducción.
Una actualización explícitamente autorizada sigue [UPDATING_WORDCRAFT](UPDATING_WORDCRAFT.md).

GTK está organizado en módulos privados bajo src/gnome; el binario es solo la
entrada. Diagnósticos gráficos en src/gnome/diagnostics, pruebas sin GUI en tests.
No añadir nuevos include! para compartir funciones GTK. Conservar callbacks
Weak, limpieza de pestañas, identidad/revisión de resultados async y contratos de foco.
Consultar documentación oficial GNOME al cambiar patrones de interfaz.

## Validación

Seguir los comandos de [CONTRIBUTING](CONTRIBUTING.md): formato del paquete Leaf,
pruebas Python, catálogos gettext y enlaces actuales; para cambios de código,
cargo test --locked y build GUI release. La CI añade regresiones de interfaz.
Regenerar referencias gettext después de mover/formatear código, conservando
mensajes y placeholders. No formatear vendor con los archivos del cliente.

Distribuir solo release optimizada, con panic unwinding conservado. Las auditorías
extensas/performance/corpus son opcionales cuando se justifiquen; no repetirlas
por rutina ni imponer el corpus por versión. Medir antes/después en condiciones
equivalentes y no extrapolar una muestra a toda la app.

## Alcance y cierre

Una propuesta de ROADMAP no autoriza implementarla. Respetar los aplazamientos:
nuevas tareas Orca, validación humana cuando haya usuarios, Weblate y capturas
públicas a cargo de David. No repetir el intento remoto de alta en Weblate ni
sustituir capturas públicas por otras de CI. No iniciar Flathub automáticamente.

Actualizar estado, hoja de ruta y decisiones tras una entrega/cambio importante.
Registrar fuente, validación y límites; los informes detallados van en docs y el
historial conserva evidencia antigua. Documentación sola no exige recompilar ni
publicar paquete. No afirmar que algo está publicado antes de verificarlo.
