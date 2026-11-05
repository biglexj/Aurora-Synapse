---
trigger: always_on
---

# Instrucciones del proyecto — Aurora Synapse

## Documentación Core — entrada compartida

Consultar primero [Documentación Core](../../../Docs/README.md) y seleccionar solo los módulos pertinentes. Los estándares compartidos se mantienen allí; este archivo conserva el alcance y los contratos específicos del proyecto.

- [Calidad y entrega proporcionales](../../../Docs/global/quality/quality-baseline.md): build para ajustes visuales pequeños; pruebas pertinentes para elementos, lógica o páginas nuevos y cambios grandes; revisión acotada para documentación.
- [Procesos según alcance](../../../Docs/global/documentation/process-workflow.md): no añadir fases, archivos ni nuevas aprobaciones por rutina a ajustes pequeños ya autorizados.
- [Diseño común](../../../Docs/global/design/design_system.md): contenido preciso, espacio útil, mínimo scroll innecesario, fondo global y presets compartidos, conservando la identidad del producto.
- [Diagnósticos reutilizables](../../../Docs/troubleshooting/README.md) y [promoción a Core](../../../Docs/global/documentation/core-promotion-workflow.md): documentar en Core los fallos confirmados o aprendizajes generalizables y enlazarlos desde el proyecto; mantener locales los detalles exclusivos.

En estos criterios comunes, aplicar las bases vigentes enlazadas en lugar de copias históricas de este archivo. Respetar la dirección actual de Biglex y los contratos específicos del producto; ante una contradicción, comprobar la implementación vigente y resolverla explícitamente. Las skills generales importadas son apoyo técnico, no una autoridad para redefinir estas reglas.

Entrada única de instrucciones: `.agents/rules/base.md`.

## Módulos de Core aplicables a este proyecto

- **Protocolo y arquitectura**: [Aurora Synapse en Core](../../../Docs/features/aurora-synapse/README.md) · [contratos de payload](../../../Docs/features/aurora-synapse/payload-contracts.md) · [registro de puertos y red](../../../Docs/features/aurora-synapse/port-and-network-registry.md) · [despacho de instancia única](../../../Docs/features/aurora-synapse/single-instance-dispatch.md) · [especificación de URI](../../../Docs/features/aurora-synapse/uri-specification.md) · [puente MCP](../../../Docs/features/aurora-synapse/bridge-as-mcp-server.md) · [entrada para agentes y CLI](../../../Docs/features/aurora-synapse/agent-and-cli-gateway.md).
- **Comandos**: [comandos mediante Mise](../../../Docs/global/architecture/mise-command-standard.md) · [scripts y alias](../../../Docs/global/architecture/command-scripts-standard.md) · [organización de carpetas](../../../Docs/global/architecture/project-structure.md).
- **Interfaz**: [sistema de diseño](../../../Docs/global/design/design_system.md) · [textos de interfaz](../../../Docs/global/design/interface-writing.md) · [estándares de escritorio](../../../Docs/types/desktop/README.md) · [base de React](../../../Docs/stacks/react/react-baseline.md) · [instalador de Windows](../../../Docs/platforms/windows/installer.md) · [bloqueo de instancia única](../../../Docs/platforms/windows/single_instance_lock.md).
- **Agentes**: [modelos de IA](../../../Docs/features/ai-models/README.md) · [estilo de comunicación (Scientific Elegant)](../../../Docs/global/agents/communication-style.md).
- **Documentación**: [reglas de trabajo del agente](../../../Docs/global/documentation/agent-working-rules.md) · [gobernanza de la documentación](../../../Docs/global/documentation/documentation-governance.md).
- **Publicación**: [base de lanzamientos](../../../Docs/global/releases/release-baseline.md) · [protocolo de lanzamiento y actualización](../../../Docs/global/releases/protocolo_lanzamiento_actualizacion.md) · [mensaje de lanzamiento web](../../../Docs/global/releases/web-release-message-standard.md).
- **Soporte**: [soporte oficial y diálogo «Acerca de»](../../../Docs/features/official-support/official-support-baseline.md).

Los enlaces a Core sustituyen a las copias anteriores de estas reglas: no se reescriben aquí estilos de comunicación, enlaces de donación y «Acerca de», normas de ROADMAP, notas de lanzamiento, normas de `.agents/rules/`, commits de resguardo, flujo de desarrollo, catálogo genérico de modelos de IA ni política de ramas.

## Contratos locales de Aurora Synapse

- **Identidad**: orquestador global y puente universal multiplataforma del ecosistema biglexj. Recibe enlaces, medios y archivos desde cualquier dispositivo y los enruta de forma determinista a las aplicaciones satélite.
- **Stack**: Tauri v2 + Rust; React 19 + TypeScript + Vite + Bun; CSS propio con paleta tonal, micro-animaciones y contenedores elevados, conforme al [sistema de diseño de Core](../../../Docs/global/design/design_system.md).
- **Interfaz**: rejilla adaptativa de 2 a 6 columnas (cambio en 680, 960, 1280 y 1560 px), bandeja del sistema, persistencia de ventana e inicio automático con Windows.
- **Puertos y servicios**: `49295` servidor HTTP LAN, `49289` beacon UDP de descubrimiento, `49296` puente MCP en loopback. `49286` sigue reservado y sin implementar en este repositorio.
- **Detalle operativo**: [docs/architecture.md](../../docs/architecture.md) (arquitectura local, mapa del repositorio y configuración). Documentación funcional en [docs/mcp/](../../docs/mcp/) y procesos en [process/](../../process/).
- **Comandos**: `mise dev | build | check | test | release | publish`. `release` prepara artefactos y no publica; `publish` es explícito y exige autorización.
- **Licencia y autoría**: GNU GPL v2.0 (`GPL-2.0`), autor biglexj (2026). Los enlaces oficiales de donación, GitHub y el contenido del diálogo «Acerca de» son los de [la base de soporte oficial](../../../Docs/features/official-support/official-support-baseline.md); no se redefinen aquí.
