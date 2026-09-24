#![windows_subsystem = "windows"]

mod adb;
mod manager;
mod instances;
mod process;
mod scrcpy;
mod settings;

use crate::manager::{Core, LogEntry, Shared, Snapshot};
use crate::instances::InstancesReport;
use crate::settings::{RecordOptions, ScrcpyOptions, Settings};
use std::time::Duration;
use tauri::{AppHandle, Manager, RunEvent, State};
use tauri_plugin_dialog::DialogExt;

type R<T> = Result<T, String>;

/// Ejecuta trabajo bloqueante (adb, diálogos) fuera del hilo principal.
async fn blocking<T: Send + 'static>(f: impl FnOnce() -> R<T> + Send + 'static) -> R<T> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_state(core: State<Shared>) -> Snapshot {
    core.snapshot()
}

#[tauri::command]
fn get_logs(core: State<Shared>) -> Vec<LogEntry> {
    core.logs()
}

#[tauri::command]
fn clear_logs(core: State<Shared>) {
    core.clear_logs()
}

#[tauri::command]
fn save_settings(core: State<Shared>, settings: Settings) -> R<()> {
    core.save_settings(settings)
}

#[tauri::command]
async fn set_port(core: State<'_, Shared>, port: u16) -> R<()> {
    let c = core.inner().clone();
    blocking(move || {
        if port < 1024 {
            return Err("Usa un puerto entre 1024 y 65535".into());
        }
        c.update_settings(|s| s.server_port = port)?;
        c.poll_once();
        Ok(())
    })
    .await
}

#[tauri::command]
async fn use_adb(core: State<'_, Shared>, path: String) -> R<()> {
    let c = core.inner().clone();
    blocking(move || {
        if !std::path::Path::new(&path).is_file() {
            return Err("Esa ruta no existe".into());
        }
        c.update_settings(|s| s.adb_path = path)
    })
    .await
}

#[tauri::command]
async fn server_action(core: State<'_, Shared>, action: String) -> R<()> {
    let c = core.inner().clone();
    blocking(move || match action.as_str() {
        "start" => c.start_server(),
        "stop" => c.stop_server(),
        "restart" => c.restart_server(),
        _ => Err("Acción desconocida".into()),
    })
    .await
}

#[tauri::command]
async fn get_instances(core: State<'_, Shared>) -> R<InstancesReport> {
    let c = core.inner().clone();
    blocking(move || Ok(c.instances())).await
}

#[tauri::command]
async fn kill_process(core: State<'_, Shared>, pid: u32) -> R<()> {
    let c = core.inner().clone();
    blocking(move || c.kill_adb_process(pid)).await
}

#[tauri::command]
async fn refresh_device(core: State<'_, Shared>, serial: String) -> R<()> {
    let c = core.inner().clone();
    blocking(move || {
        c.refresh_details(&serial);
        Ok(())
    })
    .await
}

#[tauri::command]
async fn screenshot(core: State<'_, Shared>, serial: String) -> R<String> {
    let c = core.inner().clone();
    blocking(move || c.screenshot(&serial)).await
}

/// Elige un APK y lo instala; el progreso llega por eventos `install`.
#[tauri::command]
async fn install_apk(app: AppHandle, core: State<'_, Shared>, serial: String) -> R<bool> {
    let c = core.inner().clone();
    blocking(move || {
        let picked = app
            .dialog()
            .file()
            .set_title("Instalar APK")
            .add_filter("Paquete Android", &["apk"])
            .blocking_pick_file();
        let Some(path) = picked.and_then(|p| p.as_path().map(|p| p.display().to_string())) else {
            return Ok(false);
        };
        c.install_apk(&serial, &path, "")?;
        Ok(true)
    })
    .await
}

/// Reintento desde la tarjeta de error: "downgrade" o "uninstall:<paquete>".
#[tauri::command]
async fn install_apk_path(core: State<'_, Shared>, serial: String, path: String, mode: String) -> R<()> {
    let c = core.inner().clone();
    blocking(move || c.install_apk(&serial, &path, &mode)).await
}

#[tauri::command]
async fn reboot(core: State<'_, Shared>, serial: String, mode: String) -> R<()> {
    let c = core.inner().clone();
    blocking(move || c.reboot(&serial, &mode)).await
}

#[tauri::command]
async fn shell(core: State<'_, Shared>, serial: String, command: String) -> R<String> {
    let c = core.inner().clone();
    blocking(move || c.shell(&serial, &command)).await
}

#[tauri::command]
async fn connect(core: State<'_, Shared>, addr: String) -> R<String> {
    let c = core.inner().clone();
    blocking(move || c.connect(&addr)).await
}

#[tauri::command]
async fn disconnect(core: State<'_, Shared>, serial: String) -> R<()> {
    let c = core.inner().clone();
    blocking(move || c.disconnect(&serial)).await
}

#[tauri::command]
async fn pair(core: State<'_, Shared>, addr: String, code: String) -> R<String> {
    let c = core.inner().clone();
    blocking(move || c.pair(&addr, &code)).await
}

#[tauri::command]
async fn enable_wifi(core: State<'_, Shared>, serial: String) -> R<String> {
    let c = core.inner().clone();
    blocking(move || c.enable_wifi(&serial)).await
}

#[tauri::command]
async fn start_mirror(core: State<'_, Shared>, serial: String, options: ScrcpyOptions) -> R<String> {
    let c = core.inner().clone();
    blocking(move || c.start_mirror(&serial, options)).await
}

#[tauri::command]
fn stop_mirror(core: State<Shared>, id: u32) -> R<()> {
    core.inner().stop_mirror(id)
}

#[tauri::command]
fn focus_mirror(core: State<Shared>, id: u32) {
    core.focus_mirror(id)
}

#[tauri::command]
async fn start_recording(core: State<'_, Shared>, serial: String, options: RecordOptions) -> R<String> {
    let c = core.inner().clone();
    blocking(move || c.start_recording(&serial, options)).await
}

#[tauri::command]
async fn stop_recording(core: State<'_, Shared>, serial: String) -> R<()> {
    let c = core.inner().clone();
    blocking(move || c.stop_recording(&serial)).await
}

#[tauri::command]
async fn external_mirror(core: State<'_, Shared>, pid: u32, action: String) -> R<()> {
    let c = core.inner().clone();
    blocking(move || c.external_mirror(pid, &action)).await
}

#[tauri::command]
async fn pick_path(app: AppHandle, kind: String) -> R<Option<String>> {
    blocking(move || {
        let dlg = app.dialog().file();
        let picked = match kind.as_str() {
            "folder" => dlg.set_title("Elegir carpeta").blocking_pick_folder(),
            "adb" => dlg.set_title("Elegir adb.exe").add_filter("adb", &["exe"]).blocking_pick_file(),
            _ => dlg.set_title("Elegir scrcpy.exe").add_filter("scrcpy", &["exe"]).blocking_pick_file(),
        };
        Ok(picked.and_then(|p| p.as_path().map(|p| p.display().to_string())))
    })
    .await
}

#[tauri::command]
fn detect_paths() -> serde_json::Value {
    serde_json::json!({
        "adb": adb::auto_adb().map(|p| p.display().to_string()),
        "scrcpy": scrcpy::auto_scrcpy().map(|p| p.display().to_string()),
    })
}

#[tauri::command]
fn open_path(path: String) -> R<()> {
    let p = std::path::PathBuf::from(&path);
    let mut cmd = std::process::Command::new("explorer.exe");
    if p.is_file() {
        cmd.arg(format!("/select,{}", p.display()));
    } else {
        std::fs::create_dir_all(&p).map_err(|e| e.to_string())?;
        cmd.arg(&p);
    }
    cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
fn capture_dir(core: State<Shared>) -> String {
    core.settings().capture_dir().display().to_string()
}

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            let core = Core::new(app.handle().clone(), dir.join("settings.json"));
            app.manage(core.clone());
            core.spawn_poller();

            // La interfaz muestra la ventana al terminar de pintar; esto es solo un respaldo.
            if let Some(w) = app.get_webview_window("main") {
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_secs(3));
                    let _ = w.show();
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            get_logs,
            clear_logs,
            save_settings,
            set_port,
            use_adb,
            server_action,
            get_instances,
            kill_process,
            refresh_device,
            screenshot,
            install_apk,
            install_apk_path,
            reboot,
            shell,
            connect,
            disconnect,
            pair,
            enable_wifi,
            start_mirror,
            stop_mirror,
            focus_mirror,
            external_mirror,
            start_recording,
            stop_recording,
            pick_path,
            detect_paths,
            open_path,
            capture_dir,
        ])
        .build(tauri::generate_context!())
        .expect("no se pudo iniciar la aplicación");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            handle.state::<Shared>().shutdown();
        }
    });
}
