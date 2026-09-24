<div align="center">

<img src="icons/128x128.png" width="96" height="96" alt="Android Device Manager">

# Android Device Manager

Administra tus dispositivos Android por adb, mira su pantalla con scrcpy y controla el
servidor adb **sin pelearte con Android Studio**.

![Windows 10/11](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D4?logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-2024-CE422B?logo=rust&logoColor=white)
![Tauri 2](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)
![Versión](https://img.shields.io/badge/versi%C3%B3n-1.0.0-3DDC84?logo=android&logoColor=white)
![Licencia MIT](https://img.shields.io/badge/licencia-MIT-yellow)

</div>

<!-- Agrega una captura de la app aquí, por ejemplo: ![Captura](docs/screenshot.png) -->

## Características

- 📱 **Panel de dispositivos**: estado, versión de Android/SDK, pantalla, batería, IP y una consola `adb shell`.
- 🖥️ **Espejo de pantalla** con scrcpy y sus opciones más útiles a mano.
- 📸 **Capturas y video**: PNG al instante y grabación en MP4/MKV, con audio en Android 11+.
- 📦 **Instalación de APK con progreso real**, errores explicados y un botón para corregirlos.
- 📶 **Depuración por Wi-Fi**: conectar y emparejar por IP.
- 🔁 **Reiniciar** en modo normal, recovery o bootloader.
- 🛠️ **Control del servidor adb**: iniciar, reiniciar o detener; modo compartido o aislado; diagnóstico de conflictos.
- 🧾 **Registro** de cada comando de adb/scrcpy con código de salida, duración y salida.

## Requisitos

| Qué | Para qué |
|---|---|
| Windows 10/11 con WebView2 (incluido con Edge) | Ejecutar la app |
| [Android SDK Platform-Tools](https://developer.android.com/tools/releases/platform-tools) | `adb`, el mismo que usa Android Studio |
| [scrcpy](https://github.com/Genymobile/scrcpy) (`winget install Genymobile.scrcpy`) | Ver y grabar la pantalla |
| Rust 1.85+ con toolchain MSVC y VS Build Tools | Solo para compilar |

## Compilar y ejecutar

```bash
git clone https://github.com/iscalexsoto/android-device-manager.git
cd android-device-manager
cargo run
```

`rust-toolchain.toml` fija el toolchain MSVC estable, así que `rustup` lo instala si hace falta.

Versión optimizada:

```bash
cargo build --release
```

El ejecutable queda en `target/release/android-device-manager.exe` con la interfaz embebida.

Instalador NSIS:

```bash
cargo install tauri-cli
cargo tauri build
```

Pruebas:

```bash
cargo test
```

## Vistas

| Vista | Contenido |
|---|---|
| **Dispositivo** | Estado, Android/SDK, pantalla, batería, IP; opciones de scrcpy; consola `adb shell` |
| **Barra superior** | Ver pantalla, captura (PNG), grabar video, instalar APK, depuración por Wi-Fi, reiniciar (normal/recovery/bootloader), conectar y emparejar por IP |
| **Pantallas** | Sesiones de scrcpy abiertas (propias y externas) y grabaciones en curso: traer al frente, cerrar, detener |
| **Servidor ADB** | Estado, iniciar/reiniciar/detener, modo compartido o aislado, diagnóstico de conflictos, procesos `adb.exe` con puerto y versión, binarios adb del equipo |
| **Registro** | Cada comando de adb/scrcpy ejecutado, con código de salida, duración y salida |
| **Ajustes** | Rutas de adb, scrcpy y capturas; arranque automático del servidor; tema |

La interfaz sigue el sistema de diseño Retail One: tipografía Figtree / JetBrains Mono,
barra superior de 36 + 48 px y rail derecho.

## Cómo evita chocar con Android Studio

Si alguna vez viste `adb server version doesn't match this client; killing...`, esta app
está pensada para evitarlo:

1. **Mismo binario, mismo servidor.** Por defecto usa el `adb.exe` del SDK
   (`ANDROID_HOME`, `ANDROID_SDK_ROOT` o `%LOCALAPPDATA%\Android\Sdk`), el mismo que
   Android Studio, en el puerto 5037. Los dos comparten un servidor y ninguno lo reinicia.
2. **scrcpy no usa su adb incluido.** Se lanza con `ADB=<adb configurado>` y
   `ANDROID_ADB_SERVER_PORT=<puerto>`. El adb que trae scrcpy suele ser de otra versión
   y es la causa típica del error de arriba.
3. **El sondeo no arranca servidores.** La lista de dispositivos se lee hablando el
   protocolo del servidor por TCP (`host:version`, `host:devices-l`), así que detener el
   servidor desde la app no hace que ella misma lo vuelva a levantar.
4. **Diagnóstico.** Detecta versión de protocolo distinta entre servidor y cliente,
   varios servidores a la vez, procesos adb colgados sin puerto y adb ajenos al SDK,
   y ofrece la corrección (usar otro binario, terminar el proceso, volver a compartido).
5. **Modo aislado (opcional).** Servidor propio en otro puerto (p. ej. 5038). Útil para
   dispositivos por Wi-Fi; un dispositivo USB solo puede atenderlo un servidor a la vez.
   Al cerrar la app se detiene ese servidor si así se configura.

## Detalles técnicos

<details>
<summary><strong>Instalación de APK con progreso</strong></summary>

<br>

El APK se envía directamente al servidor adb por su protocolo
(`host:transport:<serie>` + `exec:cmd package install -S <tamaño> -r -t`), igual que hace
`adb install` por dentro, pero contando los bytes escritos. Así la interfaz muestra:

1. Copiando al dispositivo, con porcentaje y MB.
2. Verificando e instalando (el sistema procesa el paquete; puede tardar).
3. Instalado, o el error traducido con su código de Android.

Los errores comunes traen solución:

- Firma distinta a la instalada → «Desinstalar y reintentar» (con confirmación, borra los datos de la app).
- Versión más nueva instalada → «Instalar de todos modos» (`-d`).

En Android 6 o anterior (sin `cmd package`) se usa `adb install` clásico, sin porcentaje.
`-t` permite instalar APK de prueba generados por Android Studio.

</details>

<details>
<summary><strong>Grabación de video</strong></summary>

<br>

«Grabar» lanza scrcpy en segundo plano (`--no-window --no-playback --no-control --record`),
con la resolución, el bitrate, el formato (MP4 o MKV) y el audio que se elijan en el popover.
El audio usa AAC, que se reproduce en cualquier visor, y requiere Android 11 o superior.

Para detener, la app envía Ctrl+C a la consola oculta de scrcpy (`AttachConsole` +
`GenerateConsoleCtrlEvent`), de modo que scrcpy escriba el índice del MP4 antes de salir;
matar el proceso dejaría un MP4 ilegible. El manejador que protege a la propia app se
registra después de `AttachConsole`, porque unirse a una consola reinicia los manejadores.
Si scrcpy no responde en 10 s, se termina el proceso. Al cerrar la app se detienen las
grabaciones en curso de la misma forma.

</details>

<details>
<summary><strong>Estructura del proyecto</strong></summary>

<br>

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
capabilities/    permisos de la ventana (Tauri 2)
icons/           iconos de la app y del instalador
```

</details>

## Créditos

- [scrcpy](https://github.com/Genymobile/scrcpy) de Genymobile, para el espejo y la grabación de pantalla.
- [Tauri](https://tauri.app), para la app de escritorio.

## Licencia

[MIT](LICENSE) © 2026 iscalexsoto
