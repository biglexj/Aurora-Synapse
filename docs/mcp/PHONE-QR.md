# Conectar Synapse del teléfono con el PC

Implementado el 2026-10-23. Es una conexión de enrutamiento con el PC; la
autorización de Super Galería sigue siendo independiente.

## Uso

1. PC: Ajustes → Conexiones → Mostrar QR de conexión.
2. Teléfono: Ajustes → Conexiones → Escanear QR del PC.
3. Revisar la dirección escaneada y confirmar la conexión.

Ambos dispositivos deben estar en la misma red local. La aplicación configura
la IP del PC sin escribirla. Los envíos posteriores a ese PC usan la ruta
autenticada `/paired/dispatch` y propagan los errores del receptor.

## Contrato

- Invitación `aurora-synapse://pair?v=1&address=...&ticket=...`.
- QR con 256 bits aleatorios, un uso y caducidad de 90 segundos. Generar otro
  o cerrar ajustes invalida el anterior. El QR nunca contiene credenciales MCP.
- `POST /pair/claim` intercambia la invitación por una credencial independiente.
- `/paired/status` y `/paired/dispatch` exigen Bearer. No se aceptan orígenes web
  en estas rutas ni en el intercambio; solo direcciones locales.
- La credencial del teléfono se cifra con Android Keystore, en un espacio
  independiente del utilizado por Super Galería. El PC solo mantiene su hash
  en memoria. Caduca a las 12 horas o al salir del proceso del PC.
- El PC muestra «vinculado» después de recibir una petición autenticada; eso
  no significa que el teléfono esté conectado permanentemente.
- Se mantiene la compatibilidad con las rutas LAN anteriores. Este cambio no
  convierte esas rutas heredadas en rutas autenticadas.
- El servidor MCP permanece en loopback; no se abre a la LAN para emparejar.

El escáner nativo usa [Google Code Scanner](https://developers.google.com/ml-kit/vision/barcode-scanning/code-scanner)
16.1.0. Requiere Google Play Services; puede descargar el módulo en el primer uso.

## Verificación

- `bun run build` y Tauri debug escritorio correctos.
- APK Android aarch64 compilado e instalado sobre el Motorola conectado.
- 31 pruebas Rust correctas, incluidas uso único, vencimiento, Bearer,
  rechazo de QR externo/puerto incorrecto y cuerpos HTTP fragmentados.
- Desde el teléfono, GET al estado del PC por LAN devuelve HTTP 200.
- Estado vinculado sin credencial devuelve HTTP 401; intercambio con Origin
  externo devuelve HTTP 403.
- Se reprodujo un conflicto entre dos instancias de desarrollo: solo la primera
  tenía el receptor LAN y la segunda rechazaba generar el QR. Se activó instancia
  única también en desarrollo. Prueba real posterior: el segundo proceso sale,
  queda una sola ventana/proceso y 49295/49296 pertenecen al mismo PID.
- Vista de 476 × 932: una columna, sin desbordamiento horizontal y un solo
  contenedor desplazable dentro del diálogo. El máximo de escritorio es 1024 px,
  con dos columnas desde 781 px. No hay scroll independiente por columna.
- El usuario confirmó «conectado» tras escanear físicamente desde el Motorola.
- Ajustes posteriores de interfaz: secciones Conexiones/General/Acerca de,
  icono real de Super Galería y vista completa en teléfono. Android aplica
  insets de barras del sistema y recortes al contenedor nativo del WebView.
- El QR sustituye los campos de dirección mientras está visible; incluye Volver
  para recuperar el formulario y cancelar la invitación pendiente.
- Comprobación de la nueva distribución en navegador a 1280 × 720: diálogo de
  1024 px, sin desbordamiento horizontal y sin contenedores desplazables en
  Conexiones y General. La comprobación estrecha anterior corresponde al diseño
  previo; la nueva vista móvil se extiende a toda la superficie del WebView.
- APK con la navegación y los insets instalado en el Motorola. El usuario confirmó
  que la vista móvil y sus márgenes están correctos.
- Ajuste posterior del escritorio solicitado por el usuario: ancho máximo 920 px,
  solo Conexiones/General y Acerca de integrado en General. Comprobado en navegador:
  ancho real 920 px, sección Acerca de visible, sin scroll ni desbordamiento
  horizontal en General a 1280 × 720. La instalación Android conserva el diseño
  anterior de tres secciones hasta una siguiente actualización móvil.
