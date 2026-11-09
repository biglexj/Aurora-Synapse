# Consolidación del puente MCP

> Antecedente del 2026-10-23. Desde el 2026-10-26, el alcance y las tareas nuevas
> se gestionan en el [proceso de puente universal MCP e interconectividad](../../process/active/2026-10-26_puente-universal-mcp-e-interconectividad/PLAN.md).
> Se conserva este plan para interpretar la evidencia anterior; sus aplazamientos
> no limitan el alcance del nuevo proceso.

Fecha: 2026-10-23. Alcance: completar por etapas las integraciones ya avanzadas.

## Objetivo

Synapse ofrece un único catálogo MCP a los agentes y traduce sus llamadas a las
superficies reales de Luna Fetch, Prisma, Prisma Upscaler y Super Galería. Una herramienta
publicada no implica que la app esté encendida, ni un envío implica ejecución.

## Etapa 1 — fiabilidad de los adaptadores (implementada y verificada localmente)

- Consultar disponibilidad real en `apps_list`, separada del soporte del adaptador.
- Reparar la sonda PowerShell: sesión, errores JSON-RPC y cierre de sesión.
- Validar argumentos y evitar que `dispatch` eluda las validaciones específicas.
- Distinguir `sent`, `accepted` y `completed` en las operaciones.
- Sustituir el falso escalado por apertura de la GUI por ejecución real de la CLI
  instalada. Conservar parámetros y devolver una salida PNG nueva y verificable.
- No sobrescribir archivos ni reintentar un escalado en otro motor tras un error
  de procesamiento. Elegir transporte antes de iniciar el trabajo.
- Probar regresiones, HTTP/stdio y una operación real de escalado local.

## Etapa 2 — completar el recorrido con agentes

- Verificar desde una conversación real de Ely Intelligence el descubrimiento,
  selección y ejecución de herramientas, con el estado de configuración visible.
  Confirmar dónde corre su backend: loopback solo alcanza el mismo equipo;
  un backend remoto necesitará un canal autenticado específico, todavía pendiente.
- Comprobar Luna Fetch abierto: recepción de descarga y archivo final. Su HTTP
  actual confirma recepción, pero no ofrece un identificador de trabajo ni una
  consulta de finalización; no anunciar esa capacidad hasta implementar el contrato.
- Comprobar apertura y control de Prisma con su estado de reproducción posterior.
- Completar NCNN local cuando haya binario y modelos configurados; validar HTTP
  del Upscaler contra un servicio real si se utiliza ese transporte.
- Revisar el transporte MCP antes de ampliar sus consumidores: negociación,
  sesiones, validación de origen y dirección de escucha. No exponerlo a la LAN.

Evidencia de esta entrega: [VERIFICATION.md](VERIFICATION.md). Las pruebas
locales de la etapa 1 no cierran los recorridos de la etapa 2.

## Aplazado

WinTTS, Ely-Tesia, LyraFlow, MouziFlow y nuevas integraciones. No añadir herramientas
para apps sin contrato externo.

## Etapa actual — teléfono e Intelligence local

- Reutilizar HTTP 49288 y descubrimiento UDP 49289 de Super Galería/Prisma.
- Autorizar mediante código temporal introducido por el usuario en Synapse;
  guardar credenciales en Windows Credential Manager y Android Keystore.
- Exponer estado, dispositivos, catálogo paginado, envío y descarga por ID.
  Excluir contenido privado, oculto, papelera y carpetas excluidas.
- Verificar tamaño y SHA-256 en transferencias; no sobrescribir destinos.
- Consumir las 20 herramientas desde el backend local de Intelligence y resolver
  los nombres de función del modelo contra los nombres canónicos del catálogo.
- Cerrar la verificación física después del emparejamiento manual del usuario.
- El agente de interfaz trabaja por separado en el botón MCP y las etiquetas.

## Criterio de cierre

Registrar por separado pruebas automatizadas, llamadas a aplicaciones reales y
recorridos aún pendientes. No marcar el conector universal como completo por
tener servidor MCP, pruebas unitarias o un ACK. No publicar ni instalar releases
como parte de esta etapa.
