# Android Device Manager

Aplicación de escritorio (Rust + Tauri 2) para administrar los dispositivos Android
conectados por adb, verlos en el escritorio con scrcpy y controlar el servidor adb
sin pisar el que usa Android Studio. La interfaz sigue el sistema de diseño
Retail One (tokens, tipografía Figtree / JetBrains Mono, barra de 36 + 48 px y rail derecho).

## Requisitos

- Windows 10/11 con WebView2 (viene con Edge).
- Rust con el toolchain MSVC (`rust-toolchain.toml` lo fija) y VS Build Tools.
- Android SDK Platform-Tools (el mismo `adb` que usa Android Studio).
- scrcpy para ver la pantalla: `winget install Genymobile.scrcpy`

## Compilar y ejecutar

```bash
cargo run
```

```bash
cargo build --release
```

El ejecutable queda en `target/release/android-device-manager.exe` con la interfaz
embebida. Para generar un instalador NSIS: `cargo install tauri-cli` y `cargo tauri build`.

## Qué hace

| Vista | Contenido |
|---|---|
| Dispositivo | Estado, Android/SDK, pantalla, batería, IP; opciones de scrcpy; consola `adb shell` |
| Barra superior | Ver pantalla, captura (PNG), grabar video, instalar APK, depuración por Wi-Fi, reiniciar (normal/recovery/bootloader), conectar y emparejar por IP |
| Pantallas | Sesiones de scrcpy abiertas (propias y externas) y grabaciones en curso: traer al frente, cerrar, detener |
| Servidor ADB | Estado, iniciar/reiniciar/detener, modo compartido o aislado, diagnóstico de conflictos, procesos `adb.exe` con puerto y versión, binarios adb del equipo |
| Registro | Cada comando de adb/scrcpy ejecutado, con código de salida, duración y salida |
| Ajustes | Rutas de adb, scrcpy y capturas; arranque automático del servidor; tema |

## Cómo evita chocar con Android Studio

1. **Mismo binario, mismo servidor.** Por defecto usa el `adb.exe` del SDK
   (`ANDROID_HOME`, `ANDROID_SDK_ROOT` o `%LOCALAPPDATA%\Android\Sdk`), el mismo que
   Android Studio, en el puerto 5037. Los dos comparten un servidor y ninguno lo reinicia.
2. **scrcpy no usa su adb incluido.** Se lanza con `ADB=<adb configurado>` y
   `ANDROID_ADB_SERVER_PORT=<puerto>`. El adb que trae scrcpy suele ser de otra versión
   y es la causa típica del “adb server version doesn't match this client; killing…”.
3. **El sondeo no arranca servidores.** La lista de dispositivos se lee hablando el
   protocolo del servidor por TCP (`host:version`, `host:devices-l`), así que detener el
   servidor desde la app no hace que ella misma lo vuelva a levantar.
4. **Diagnóstico.** Detecta versión de protocolo distinta entre servidor y cliente,
   varios servidores a la vez, procesos adb colgados sin puerto y adb ajenos al SDK,
   y ofrece la corrección (usar otro binario, terminar el proceso, volver a compartido).
5. **Modo aislado (opcional).** Servidor propio en otro puerto (p. ej. 5038). Útil para
   dispositivos por Wi-Fi; un dispositivo USB solo puede atenderlo un servidor a la vez.
   Al cerrar la app se detiene ese servidor si así se configura.

## Instalación de APK

El APK se envía directamente al servidor adb por su protocolo
(`host:transport:<serie>` + `exec:cmd package install -S <tamaño> -r -t`), igual que hace
`adb install` por dentro, pero contando los bytes escritos. Así la interfaz muestra:

1. Copiando al dispositivo, con porcentaje y MB.
2. Verificando e instalando (el sistema procesa el paquete; puede tardar).
3. Instalado, o el error traducido con su código de Android.

Los errores comunes traen solución: firma distinta a la instalada → «Desinstalar y
reintentar» (con confirmación, borra los datos de la app); versión más nueva instalada →
«Instalar de todos modos» (`-d`). En Android 6 o anterior (sin `cmd package`) se usa
`adb install` clásico, sin porcentaje. `-t` permite instalar APK de prueba generados por
Android Studio.

## Grabación de video

«Grabar» lanza scrcpy en segundo plano (`--no-window --no-playback --no-control --record`),
con la resolución, el bitrate, el formato (MP4 o MKV) y el audio que se elijan en el popover.
El audio usa AAC, que se reproduce en cualquier visor, y requiere Android 11 o superior.

Para detener, la app envía Ctrl+C a la consola oculta de scrcpy (`AttachConsole` +
`GenerateConsoleCtrlEvent`), de modo que scrcpy escriba el índice del MP4 antes de salir;
matar el proceso dejaría un MP4 ilegible. El manejador que protege a la propia app se
registra después de `AttachConsole`, porque unirse a una consola reinicia los manejadores.
Si scrcpy no responde en 10 s, se termina el proceso. Al cerrar la app se detienen las
grabaciones en curso de la misma forma.

## Estructura

```
src/
  main.rs        comandos Tauri expuestos a la interfaz
  manager.rs     estado: servidor, dispositivos, sesiones scrcpy, registro, sondeo
  adb.rs         localización de adb, protocolo del servidor, parseo de salidas
  scrcpy.rs      localización de scrcpy, argumentos, cierre/foco de su ventana
  instances.rs   procesos adb, puertos en escucha, binarios y conflictos
  process.rs     ejecución sin consola y con tiempo límite
  settings.rs    preferencias (%APPDATA%\com.soto.androiddevicemanager\settings.json)
ui/              index.html, styles.css, app.js, logo.svg
```

Pruebas de los parsers: `cargo test`.
