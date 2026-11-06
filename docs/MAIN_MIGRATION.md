# Consolidación de Leaf en main — 2 de noviembre de 2026

## Estado previo y conservación

- Main anterior: `58da57efcfd03e7b442b20629703d32a697274f4`, archivado sin cambios
  en [archive/apexdesk-main-2026-11-02](https://github.com/frandavid100/leaf/tree/archive/apexdesk-main-2026-11-02).
- Leaf remoto: `61f3045d63eb5c0f985c4adacc338f51172daa52`, conservado en
  [wordcraft-saving](https://github.com/frandavid100/leaf/tree/wordcraft-saving).
- Ancestro común: `6ab4b958c9ae78b87706fff8d8e629791e2e21de`.
  Main tenía 19 commits exclusivos; Leaf, 473. La integración usa un commit de
  fusión con ambos extremos como padres; no reemplaza ni aplasta sus historiales.
- Checkout previo limpio. Sus 12 commits locales y los 12 remotos divergentes
  tenían el mismo árbol final; se mantiene el checkout original y la rama local
  `archive/local-wordcraft-saving-2026-11-02`. Trabajo aislado en otro worktree.
- No había pull requests abiertas que redirigir.
- Última release: [v0.1.0-alpha.18](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.18),
  fuente/tag `a8207cbdb5b11f050373287a44aecd835aefd50c`. No se crea otra release
  ni se mueven etiquetas por esta migración.

## Revisión de cambios exclusivos

Comparados los 19 commits propios de main y su diff completo desde el ancestro:
solo cambiaban seis archivos. La fusión del resto de Leaf fue automática.

| Archivos de main | Resolución |
| --- | --- |
| README.md | Misma presentación actual de Leaf en ambas ramas; se conservan texto, atribución y captura de David. Enlaces e imágenes relativos resuelven ahora desde main. |
| AGENTS.md y CONTRIBUTING.md | Las versiones de main eran guías de redirección. Se conservan las guías completas de Leaf, actualizadas para desarrollar, compilar y contribuir en main. |
| Tres formularios/configuración de issues | Coinciden con Leaf; se conservan. El enlace activo a TESTING pasa a main. |
| Código, dependencias y binarios antiguos ApexDesk | La evolución de Leaf desde el ancestro común reemplaza el prototipo. El estado exacto anterior sigue accesible en la rama de archivo y en el historial de la fusión. |

Commits exclusivos de main revisados (orden cronológico):

```
c4b0be9 Point Leaf landing page to current development and enable testing issue forms [skip ci]
b131b30 Link first installable Leaf Flatpak prerelease from repository landing page [skip ci]
22e0d91 Point project landing page to columns alpha.2 [skip ci]
b68a406 Link HiDPI alpha.3 from repository landing page [skip ci]
3bc4704 Surface confirmed ODT preservation issue on landing page [skip ci]
bd220a6 Point Leaf landing page to alpha.4 and verified ODT fixes [skip ci]
444f18e Link alpha.5 printing release and validation from Leaf landing page [skip ci]
901ff79 docs: link alpha.6 and verified GTK memory audit
0362c8f docs: link alpha.8 and community translation instructions
ed5867a docs: point landing page to verified alpha.9 with editable page headers and numbering [skip ci]
9f87bef docs: point landing page to verified alpha.10 section options and local spelling review [skip ci]
623b0e5 docs: point landing page to verified alpha.11 and independent interoperability audit [skip ci]
c260bc7 docs: point landing page to alpha.12 and first-stage Orca support [skip ci]
fe04fc4 docs: link alpha.13 host performance release from landing page [skip ci]
952df58 docs: direct human and automated contributors to current Leaf modules and handoff guide [skip ci]
40488be Show current Leaf presentation on the repository landing page
846762c Clarify Leaf's engine capabilities and AI-generated implementation
885c701 Lead with Leaf's goal of advanced word processing in a focused GNOME interface
58da57e Condense Leaf presentation around focused editing, capabilities and transparent development
```

En el lado Leaf se conserva íntegro el árbol de desarrollo actual, salvo los
ajustes de migración: guías y enlaces activos, URL de ayuda/traducciones en About
y AppStream, filtros de rama y guardas de publicación en Actions, y preparación
de Weblate. Se conservan Cargo.toml/Cargo.lock, vendor, parches, pruebas,
catálogos, capturas e informes históricos. La URL de About cambia en una sola
línea, sin alterar mensajes gettext ni su localización en el código.

## Validación

Comprobaciones locales: formato del paquete correcto, cuatro pruebas Python,
587 mensajes gettext con catálogos completos y compilados, enlaces de 14
documentos y YAML de cinco workflows correctos. 123 pruebas Rust superadas.
Compilación GUI release optimizada superada (3 min 26 s); `--version` conserva
alpha.18. Diez advertencias de compilación preexistentes; sin errores.
Commit de integración publicado: `770a43cfa094db05308cef9980c5f0273e68a3e5`.
[CI de la integración en main](https://github.com/frandavid100/leaf/actions/runs/41410124180):
**superada**, concluida a las 08:25:56 UTC del 2 de noviembre de 2026.
Trabajo nativo `125161018239`: compilación y regresiones gráficas completas,
controles/ratón, idiomas/tema oscuro, encabezados, secciones, comentarios,
memoria GTK, teclado/impresión, comparación debug/release y escalado 100/200 %.
La publicación Flatpak y la auditoría de base opcional se omitieron según sus
condiciones. No se publicó otra versión.

Se verificaron los dos padres y la identidad
del árbol local/remoto. Las 17 etiquetas y las 17 releases (incluidos sus assets)
conservan sus identificadores y contenido.
La publicación Flatpak sigue requiriendo `[release-alpha]`. El commit posterior
solo registra esta evidencia en este informe (`[skip ci]`); no modifica código,
configuración, workflows ni catálogos respecto al commit validado.
Las auditorías opcionales conservan sus hashes, fuentes y condiciones históricas.

[Configuración y pasos pendientes de Weblate](WEBLATE_SETUP.md).
