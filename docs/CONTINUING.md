# Continuar Leaf desde otra conversación

El proyecto está en GitHub y no depende de conservar este chat ni sus archivos
temporales. La rama correcta es **wordcraft-saving**; main contiene el prototipo
anterior y una portada que enlaza al desarrollo actual.

## Mensaje que puedes pegar en un chat nuevo

> Continuamos Leaf: https://github.com/frandavid100/leaf, rama wordcraft-saving.
> Lee AGENTS.md, PROJECT_STATUS.md, ROADMAP.md, docs/DECISIONS.md y
> docs/ARCHITECTURE.md. Comprueba el estado remoto antes de editar y continúa
> desde el trabajo vigente. Las propuestas de la hoja de ruta no son autorización
> general. Modificamos Leaf; cualquier cambio del motor se propone en WordCraft.

Añade después la tarea que quieras realizar. No hace falta pegar toda la
conversación ni pedir al asistente que reconstruya las decisiones a partir de ella.

## Lectura mínima

1. [AGENTS](../AGENTS.md): reglas de trabajo y comprobaciones.
2. [Estado actual](../PROJECT_STATUS.md): versión publicada y trabajo en curso.
3. [Decisiones](DECISIONS.md) y [hoja de ruta](../ROADMAP.md): alcance, propuestas y aplazamientos.
4. [Arquitectura](ARCHITECTURE.md): archivos y flujo de edición.
5. El informe de la función afectada, solo si hace falta. El [historial](history/README.md)
   conserva revisiones/hashes antiguos; no se debe tratar como lista de tareas actual.

## Preparar un checkout

```sh
git clone --branch wordcraft-saving https://github.com/frandavid100/leaf.git
cd leaf
git status --short --branch
git log -5 --oneline
```

Si ya hay checkout, comprobar la rama, cambios locales y el remoto antes de
sincronizar; no usar reset --hard ni descartar trabajo preexistente.
Los requisitos y órdenes de prueba están en [CONTRIBUTING](../CONTRIBUTING.md).

## Al terminar una tarea

Actualizar el estado y el plan vigentes, dejar fuente/commit y validación, y
explicar cualquier limitación. Guardar el detalle de auditorías en su informe,
en vez de volver a convertir PROJECT_STATUS en una transcripción del chat.
Una propuesta descartada o aplazada debe quedar en DECISIONS o ROADMAP, no solo
en un mensaje de conversación. No prometer archivos publicados hasta verificarlos.
