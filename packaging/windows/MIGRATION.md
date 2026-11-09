# Migración local de Aurora Synapse a Inno Setup

El instalador Windows usa la plantilla compartida con subtítulo propio.
La identidad Inno es com.biglexj.aurorasynapse.inno y permanece estable.
La primera instalación no desinstala automáticamente el NSIS anterior.
Si coexisten, desinstalar la edición anterior desde Configuración de Windows,
comprobando su ruta antes de continuar. No borrar los datos de la aplicación.
WebView2 Runtime debe estar disponible en el equipo.

El empaquetador scripts/packaging/Build-Installer.ps1 admite SkipAppBuild para
reutilizar el ejecutable actual. El pipeline oficial invoca este empaquetador.
La compilación local no valida Android, red, enlaces ni servicios MCP.
