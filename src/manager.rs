//! Estado de la aplicación: servidor adb, dispositivos, sesiones de scrcpy y registro.

use crate::adb::{self, AdbVersion, Device, DeviceDetails, InstallFailure};
use crate::instances::{self, InstancesReport, ScanInput};
use crate::process::{self, Output};
use crate::scrcpy;
use crate::tools::{self, Tool};
use crate::settings::{RecordOptions, ScrcpyOptions, Settings};
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

const POLL_EVERY: Duration = Duration::from_millis(1500);
const DETAILS_EVERY: Duration = Duration::from_secs(30);
const LOG_MAX: usize = 400;

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub running: bool,
    pub starting: bool,
    pub port: u16,
    pub version: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: u64,
    pub time: String,
    pub command: String,
    pub code: Option<i32>,
    pub ms: u128,
    pub ok: bool,
    pub output: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub id: u32,
    pub serial: String,
    pub title: String,
    pub pid: u32,
    pub started: i64,
    pub summary: String,
}

struct SessionHandle {
    info: SessionInfo,
    child: Child,
    stopping: bool,
    lines: Arc<Mutex<VecDeque<String>>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingInfo {
    pub serial: String,
    pub pid: u32,
    pub started: i64,
    pub path: String,
    pub audio: bool,
}

/// Progreso de la descarga de adb o scrcpy, emitido como evento `tool`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolEvent {
    pub tool: String,
    pub label: String,
    /// resolve | download | extract | done | error
    pub phase: String,
    pub received: u64,
    pub total: Option<u64>,
    pub elapsed_ms: u128,
    pub version: Option<String>,
    pub path: Option<String>,
    pub message: Option<String>,
}

/// Progreso de una instalación, emitido como evento `install`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallEvent {
    pub id: u32,
    pub serial: String,
    pub device: String,
    pub path: String,
    pub file: String,
    /// uninstall | copy | install | done | error
    pub phase: String,
    pub sent: u64,
    pub total: u64,
    pub elapsed_ms: u128,
    pub failure: Option<InstallFailure>,
}

struct RecordingHandle {
    info: RecordingInfo,
    child: Child,
    stopping: bool,
    lines: Arc<Mutex<VecDeque<String>>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceView {
    #[serde(flatten)]
    pub device: Device,
    pub details: Option<DeviceDetails>,
    pub session: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub settings: Settings,
    pub adb_path: Option<String>,
    pub scrcpy_path: Option<String>,
    pub server: ServerStatus,
    pub devices: Vec<DeviceView>,
    pub sessions: Vec<SessionInfo>,
    pub recordings: Vec<RecordingInfo>,
}

pub struct Core {
    app: AppHandle,
    settings_path: PathBuf,
    settings: Mutex<Settings>,
    server: Mutex<ServerStatus>,
    devices: Mutex<Vec<Device>>,
    details: Mutex<HashMap<String, (DeviceDetails, Instant)>>,
    details_pending: Mutex<HashSet<String>>,
    sessions: Mutex<HashMap<u32, SessionHandle>>,
    next_session: AtomicU32,
    /// Grabaciones en curso por número de serie.
    recordings: Mutex<HashMap<String, RecordingHandle>>,
    next_install: AtomicU32,
    log: Mutex<VecDeque<LogEntry>>,
    next_log: std::sync::atomic::AtomicU64,
    versions: Mutex<HashMap<String, Option<AdbVersion>>>,
    /// Herramientas que se están descargando (adb, scrcpy).
    tool_installs: Mutex<HashSet<&'static str>>,
}

pub type Shared = Arc<Core>;

fn now_ms() -> i64 {
    chrono::Local::now().timestamp_millis()
}

impl Core {
    pub fn new(app: AppHandle, settings_path: PathBuf) -> Shared {
        let settings = Settings::load(&settings_path);
        let port = settings.server_port;
        Arc::new(Core {
            app,
            settings_path,
            settings: Mutex::new(settings),
            server: Mutex::new(ServerStatus { port, ..Default::default() }),
            devices: Mutex::new(Vec::new()),
            details: Mutex::new(HashMap::new()),
            details_pending: Mutex::new(HashSet::new()),
            sessions: Mutex::new(HashMap::new()),
            next_session: AtomicU32::new(1),
            recordings: Mutex::new(HashMap::new()),
            next_install: AtomicU32::new(1),
            log: Mutex::new(VecDeque::new()),
            next_log: std::sync::atomic::AtomicU64::new(1),
            versions: Mutex::new(HashMap::new()),
            tool_installs: Mutex::new(HashSet::new()),
        })
    }

    // -------------------------------------------------------------------
    // Ajustes y rutas
    // -------------------------------------------------------------------

    pub fn settings(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    pub fn save_settings(&self, new: Settings) -> Result<(), String> {
        new.save(&self.settings_path)?;
        let port_changed = {
            let mut s = self.settings.lock().unwrap();
            let changed = s.server_port != new.server_port;
            *s = new;
            changed
        };
        if port_changed {
            *self.devices.lock().unwrap() = Vec::new();
            let port = self.port();
            *self.server.lock().unwrap() = ServerStatus { port, ..Default::default() };
        }
        self.emit_state();
        Ok(())
    }

    pub fn update_settings(&self, f: impl FnOnce(&mut Settings)) -> Result<(), String> {
        let mut s = self.settings();
        f(&mut s);
        self.save_settings(s)
    }

    pub fn port(&self) -> u16 {
        self.settings.lock().unwrap().server_port
    }

    pub fn adb_path(&self) -> Option<PathBuf> {
        let custom = self.settings.lock().unwrap().adb_path.trim().to_string();
        if !custom.is_empty() {
            let p = PathBuf::from(&custom);
            return p.is_file().then_some(p);
        }
        adb::auto_adb()
    }

    pub fn scrcpy_path(&self) -> Option<PathBuf> {
        let custom = self.settings.lock().unwrap().scrcpy_path.trim().to_string();
        if !custom.is_empty() {
            let p = PathBuf::from(&custom);
            return p.is_file().then_some(p);
        }
        scrcpy::auto_scrcpy()
    }

    fn adb_env(&self) -> Vec<(String, String)> {
        vec![("ANDROID_ADB_SERVER_PORT".into(), self.port().to_string())]
    }

    pub fn version_of(&self, path: &Path) -> Option<AdbVersion> {
        let key = adb::normalize(path);
        if let Some(v) = self.versions.lock().unwrap().get(&key) {
            return v.clone();
        }
        let v = process::run(path, &["version".into()], &[], Duration::from_secs(5), true)
            .ok()
            .and_then(|o| adb::parse_version(&o.stdout_text()));
        self.versions.lock().unwrap().insert(key, v.clone());
        v
    }

    // -------------------------------------------------------------------
    // Eventos hacia la interfaz
    // -------------------------------------------------------------------

    pub fn snapshot(&self) -> Snapshot {
        let devices = self.devices.lock().unwrap().clone();
        let details = self.details.lock().unwrap();
        let sessions: Vec<SessionInfo> = {
            let mut v: Vec<_> = self.sessions.lock().unwrap().values().map(|h| h.info.clone()).collect();
            v.sort_by_key(|s| s.id);
            v
        };
        Snapshot {
            settings: self.settings(),
            adb_path: self.adb_path().map(|p| p.display().to_string()),
            scrcpy_path: self.scrcpy_path().map(|p| p.display().to_string()),
            server: self.server.lock().unwrap().clone(),
            devices: devices
                .into_iter()
                .map(|d| DeviceView {
                    details: details.get(&d.serial).map(|(x, _)| x.clone()),
                    session: sessions.iter().find(|s| s.serial == d.serial).map(|s| s.id),
                    device: d,
                })
                .collect(),
            sessions,
            recordings: self.recordings.lock().unwrap().values().map(|h| h.info.clone()).collect(),
        }
    }

    pub fn emit_state(&self) {
        let _ = self.app.emit("state", self.snapshot());
    }

    pub fn toast(&self, level: &str, text: impl Into<String>) {
        let _ = self.app.emit(
            "toast",
            serde_json::json!({ "level": level, "text": text.into() }),
        );
    }

    /// Aviso con un archivo que la interfaz ofrece mostrar en el Explorador.
    pub fn toast_file(&self, level: &str, text: impl Into<String>, path: &str) {
        let _ = self.app.emit(
            "toast",
            serde_json::json!({ "level": level, "text": text.into(), "path": path }),
        );
    }

    pub fn logs(&self) -> Vec<LogEntry> {
        self.log.lock().unwrap().iter().cloned().collect()
    }

    pub fn clear_logs(&self) {
        self.log.lock().unwrap().clear();
    }

    fn push_log(&self, command: String, out: Option<&Output>, err: Option<&str>) {
        let entry = LogEntry {
            id: self.next_log.fetch_add(1, Ordering::Relaxed),
            time: chrono::Local::now().format("%H:%M:%S").to_string(),
            command,
            code: out.and_then(|o| o.code),
            ms: out.map(|o| o.elapsed.as_millis()).unwrap_or(0),
            ok: out.map(|o| o.ok()).unwrap_or(false),
            output: match (out, err) {
                (Some(o), _) if o.timed_out => format!("Tiempo agotado\n{}", truncate(&o.combined(), 4000)),
                (Some(o), _) => truncate(&o.combined(), 4000),
                (None, Some(e)) => e.to_string(),
                _ => String::new(),
            },
        };
        {
            let mut log = self.log.lock().unwrap();
            log.push_back(entry.clone());
            while log.len() > LOG_MAX {
                log.pop_front();
            }
        }
        let _ = self.app.emit("log", entry);
    }

    // -------------------------------------------------------------------
    // Ejecución de adb
    // -------------------------------------------------------------------

    pub fn run_adb(&self, args: &[String], timeout: Duration, quiet: bool) -> Result<Output, String> {
        let exe = self
            .adb_path()
            .ok_or("No se encontró adb. Indica la ruta en Ajustes.")?;
        let shown = process::display_command(&exe, args);
        match process::run(&exe, args, &self.adb_env(), timeout, true) {
            Ok(out) => {
                if !quiet {
                    self.push_log(shown, Some(&out), None);
                }
                Ok(out)
            }
            Err(e) => {
                self.push_log(shown, None, Some(&e.to_string()));
                Err(format!("No se pudo ejecutar adb: {e}"))
            }
        }
    }

    fn adb_for(&self, serial: &str, rest: &[&str], timeout: Duration) -> Result<Output, String> {
        let mut args = vec!["-s".to_string(), serial.to_string()];
        args.extend(rest.iter().map(|s| s.to_string()));
        self.run_adb(&args, timeout, false)
    }

    // -------------------------------------------------------------------
    // Servidor
    // -------------------------------------------------------------------

    pub fn start_server(&self) -> Result<(), String> {
        let exe = self.adb_path().ok_or("No se encontró adb. Indica la ruta en Ajustes.")?;
        self.server.lock().unwrap().starting = true;
        self.emit_state();
        let args = vec!["start-server".to_string()];
        // Sin capturar: el daemon hereda los pipes y dejaría la lectura colgada.
        let res = process::run(&exe, &args, &self.adb_env(), Duration::from_secs(20), false);
        self.push_log(process::display_command(&exe, &args), res.as_ref().ok(), res.as_ref().err().map(|e| e.to_string()).as_deref());
        self.server.lock().unwrap().starting = false;
        self.poll_once();
        match res {
            Ok(o) if o.ok() => Ok(()),
            Ok(o) => Err(format!("adb start-server terminó con código {:?}", o.code)),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn stop_server(&self) -> Result<(), String> {
        let out = self.run_adb(&["kill-server".into()], Duration::from_secs(10), false)?;
        thread::sleep(Duration::from_millis(300));
        self.poll_once();
        if out.ok() || adb::server_version(self.port()).is_none() {
            Ok(())
        } else {
            Err(out.combined())
        }
    }

    pub fn restart_server(&self) -> Result<(), String> {
        let _ = self.stop_server();
        thread::sleep(Duration::from_millis(400));
        self.start_server()
    }

    // -------------------------------------------------------------------
    // Descarga de herramientas
    // -------------------------------------------------------------------

    /// Descarga adb o scrcpy a la carpeta de la app, informando con eventos `tool`.
    pub fn install_tool(&self, name: &str) -> Result<(), String> {
        let tool = Tool::parse(name)?;
        if !self.tool_installs.lock().unwrap().insert(tool.key()) {
            return Err(format!("Ya se está descargando {}", tool.label()));
        }
        let start = Instant::now();
        let mut ev = ToolEvent {
            tool: tool.key().into(),
            label: tool.label().into(),
            phase: "resolve".into(),
            received: 0,
            total: None,
            elapsed_ms: 0,
            version: None,
            path: None,
            message: None,
        };
        let emit = |ev: &mut ToolEvent, phase: &str| {
            ev.phase = phase.into();
            ev.elapsed_ms = start.elapsed().as_millis();
            let _ = self.app.emit("tool", ev.clone());
        };

        let res = tools::install(tool, |step| match step {
            tools::Step::Resolve => emit(&mut ev, "resolve"),
            tools::Step::Download { received, total } => {
                ev.received = received;
                ev.total = total;
                emit(&mut ev, "download");
            }
            tools::Step::Extract => emit(&mut ev, "extract"),
        });
        self.tool_installs.lock().unwrap().remove(tool.key());

        let command = format!("Descargar {}", tool.label());
        let note = |ok: bool, text: String| Output {
            code: Some(if ok { 0 } else { 1 }),
            stdout: text.into_bytes(),
            stderr: String::new(),
            timed_out: false,
            elapsed: start.elapsed(),
        };
        match res {
            Ok(done) => {
                let path = done.exe.display().to_string();
                self.push_log(
                    command,
                    Some(&note(true, format!("{}\nVersión {} · {:.1} MB\n{path}", done.url, done.version, done.bytes as f64 / 1_048_576.0))),
                    None,
                );
                // Una ruta manual que ya no existe taparía la copia recién descargada.
                self.update_settings(|s| {
                    let custom = match tool {
                        Tool::Adb => &mut s.adb_path,
                        Tool::Scrcpy => &mut s.scrcpy_path,
                    };
                    if !custom.trim().is_empty() && !Path::new(custom.trim()).is_file() {
                        custom.clear();
                    }
                })?;
                ev.version = Some(done.version);
                ev.path = Some(path);
                emit(&mut ev, "done");
                if tool == Tool::Adb && self.settings().auto_start_server && adb::server_version(self.port()).is_none() {
                    if let Err(e) = self.start_server() {
                        self.toast("danger", e);
                    }
                }
                Ok(())
            }
            Err(e) => {
                self.push_log(command, Some(&note(false, e.clone())), None);
                ev.message = Some(e);
                emit(&mut ev, "error");
                Ok(())
            }
        }
    }

    // -------------------------------------------------------------------
    // Sondeo
    // -------------------------------------------------------------------

    pub fn spawn_poller(self: &Shared) {
        let core = self.clone();
        thread::spawn(move || {
            if core.settings().auto_start_server && adb::server_version(core.port()).is_none() {
                if let Err(e) = core.start_server() {
                    core.toast("danger", e);
                }
            }
            loop {
                core.poll_once();
                thread::sleep(POLL_EVERY);
            }
        });
    }

    pub fn poll_once(self: &Core) {
        let port = self.port();
        let version = adb::server_version(port);
        let devices = match version {
            Some(_) => adb::host_query(port, "host:devices-l")
                .map(|t| adb::parse_devices(&t))
                .unwrap_or_default(),
            None => Vec::new(),
        };

        let mut changed = false;
        {
            let mut s = self.server.lock().unwrap();
            let new = ServerStatus { running: version.is_some(), starting: s.starting, port, version };
            if *s != new {
                *s = new;
                changed = true;
            }
        }
        {
            let mut d = self.devices.lock().unwrap();
            if *d != devices {
                *d = devices.clone();
                changed = true;
            }
        }
        // Limpia detalles de dispositivos que ya no están.
        {
            let mut det = self.details.lock().unwrap();
            let before = det.len();
            det.retain(|k, _| devices.iter().any(|d| &d.serial == k && d.state == "device"));
            changed |= det.len() != before;
        }
        if changed {
            self.emit_state();
        }

        // Detalles de los dispositivos en línea (nuevos o caducados).
        for d in devices.iter().filter(|d| d.state == "device") {
            let stale = self
                .details
                .lock()
                .unwrap()
                .get(&d.serial)
                .map(|(_, at)| at.elapsed() > DETAILS_EVERY)
                .unwrap_or(true);
            if stale && self.details_pending.lock().unwrap().insert(d.serial.clone()) {
                let serial = d.serial.clone();
                let app = self.app.clone();
                thread::spawn(move || {
                    use tauri::Manager;
                    if let Some(core) = app.try_state::<Shared>() {
                        core.fetch_details(&serial);
                    }
                });
            }
        }
    }

    pub fn fetch_details(&self, serial: &str) {
        let first = !self.details.lock().unwrap().contains_key(serial);
        let args = vec![
            "-s".to_string(),
            serial.to_string(),
            "shell".to_string(),
            adb::details_script(),
        ];
        // Solo se registra la primera lectura; los refrescos de batería van en silencio.
        if let Ok(out) = self.run_adb(&args, Duration::from_secs(15), !first)
            && !out.timed_out
        {
            let d = adb::parse_details(&out.stdout_text());
            self.details.lock().unwrap().insert(serial.to_string(), (d, Instant::now()));
            self.emit_state();
        }
        self.details_pending.lock().unwrap().remove(serial);
    }

    pub fn refresh_details(&self, serial: &str) {
        self.details.lock().unwrap().remove(serial);
        self.fetch_details(serial);
    }

    fn device_label(&self, serial: &str) -> String {
        let det = self.details.lock().unwrap();
        if let Some((d, _)) = det.get(serial) {
            if !d.name.is_empty() {
                return d.name.clone();
            }
            if !d.model.is_empty() {
                return d.model.clone();
            }
        }
        self.devices
            .lock()
            .unwrap()
            .iter()
            .find(|d| d.serial == serial && !d.model.is_empty())
            .map(|d| d.model.clone())
            .unwrap_or_else(|| serial.to_string())
    }

    fn file_stem_for(&self, serial: &str) -> String {
        let label = self.device_label(serial);
        let safe: String = label
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' })
            .collect();
        format!("{}_{}", safe.trim_matches('_'), chrono::Local::now().format("%Y%m%d_%H%M%S"))
    }

    // -------------------------------------------------------------------
    // Acciones sobre dispositivos
    // -------------------------------------------------------------------

    pub fn screenshot(&self, serial: &str) -> Result<String, String> {
        let out = self.adb_for(serial, &["exec-out", "screencap", "-p"], Duration::from_secs(20))?;
        if !out.ok() || out.stdout.len() < 8 || &out.stdout[1..4] != b"PNG" {
            return Err(first_line(&out.combined(), "No se pudo capturar la pantalla"));
        }
        let dir = self.settings().capture_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!("{}.png", self.file_stem_for(serial)));
        std::fs::write(&path, &out.stdout).map_err(|e| e.to_string())?;
        Ok(path.display().to_string())
    }

    /// Instala un APK informando el progreso con eventos `install`.
    /// `mode`: "" normal · "downgrade" permite bajar de versión · "uninstall:<paquete>" desinstala antes.
    pub fn install_apk(&self, serial: &str, apk: &str, mode: &str) -> Result<(), String> {
        let path = PathBuf::from(apk);
        let total = std::fs::metadata(&path).map_err(|_| "No se encontró el archivo APK".to_string())?.len();
        let start = Instant::now();
        let mut ev = InstallEvent {
            id: self.next_install.fetch_add(1, Ordering::Relaxed),
            serial: serial.to_string(),
            device: self.device_label(serial),
            path: apk.to_string(),
            file: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            phase: "copy".into(),
            sent: 0,
            total,
            elapsed_ms: 0,
            failure: None,
        };
        let emit = |ev: &mut InstallEvent, phase: &str| {
            ev.phase = phase.into();
            ev.elapsed_ms = start.elapsed().as_millis();
            let _ = self.app.emit("install", ev.clone());
        };
        let fail = |ev: &mut InstallEvent, failure: InstallFailure| {
            ev.failure = Some(failure);
            emit(ev, "error");
            Ok(())
        };

        if let Some(pkg) = mode.strip_prefix("uninstall:") {
            emit(&mut ev, "uninstall");
            let out = self.adb_for(serial, &["uninstall", pkg], Duration::from_secs(60))?;
            if !out.combined().contains("Success") {
                return fail(&mut ev, InstallFailure {
                    message: format!("No se pudo desinstalar {pkg}: {}", first_line(&out.combined(), "sin respuesta")),
                    ..Default::default()
                });
            }
        }

        let mut flags = vec!["-r", "-t"];
        if mode == "downgrade" {
            flags.push("-d");
        }

        emit(&mut ev, "copy");
        let streamed = self.stream_install(serial, &path, total, &flags, |sent, copying| {
            ev.sent = sent;
            emit(&mut ev, if copying { "copy" } else { "install" });
        });
        let output = match streamed {
            Ok(out) => out,
            // Sin «cmd package» (Android 6 o anterior) o sin servicio exec: adb clásico.
            Err(None) => {
                ev.sent = total;
                emit(&mut ev, "install");
                let mut args = vec!["-s".to_string(), serial.to_string(), "install".to_string()];
                args.extend(flags.iter().map(|f| f.to_string()));
                args.push(apk.to_string());
                self.run_adb(&args, Duration::from_secs(300), false)?.combined()
            }
            Err(Some(e)) => {
                return fail(&mut ev, InstallFailure {
                    message: format!("Se interrumpió la copia al dispositivo: {e}"),
                    ..Default::default()
                });
            }
        };

        if output.contains("Success") {
            ev.sent = total;
            emit(&mut ev, "done");
        } else {
            let mut f = adb::parse_install_failure(&output);
            if f.code.is_empty() {
                f.message = first_line(&output, "El dispositivo no respondió");
            }
            fail(&mut ev, f)?;
        }
        Ok(())
    }

    /// Envía el APK por el socket de `cmd package install -S` contando los bytes.
    /// Err(None) = el dispositivo no soporta la vía rápida; Err(Some) = falló a medias.
    fn stream_install(
        &self,
        serial: &str,
        path: &Path,
        total: u64,
        flags: &[&str],
        mut progress: impl FnMut(u64, bool),
    ) -> Result<String, Option<String>> {
        use std::io::{Read, Write};
        let started = Instant::now();
        let service = format!("exec:cmd package install -S {total} {}", flags.join(" "));
        let mut sock = adb::device_service(self.port(), serial, &service, Duration::from_secs(300))
            .map_err(|_| None)?;
        let mut file = std::fs::File::open(path).map_err(|e| Some(e.to_string()))?;
        let mut buf = vec![0u8; 256 * 1024];
        let (mut sent, mut last) = (0u64, Instant::now());
        loop {
            let n = file.read(&mut buf).map_err(|e| Some(e.to_string()))?;
            if n == 0 {
                break;
            }
            sock.write_all(&buf[..n]).map_err(|e| Some(e.to_string()))?;
            sent += n as u64;
            if last.elapsed() >= Duration::from_millis(100) {
                progress(sent, true);
                last = Instant::now();
            }
        }
        // Copia terminada: ahora el sistema verifica e instala el paquete.
        progress(total, false);
        let mut out = Vec::new();
        sock.read_to_end(&mut out).map_err(|e| Some(e.to_string()))?;
        let text = String::from_utf8_lossy(&out).trim().to_string();

        let shown = format!("adb -s {serial} install {} {} (streaming)", flags.join(" "), path.display());
        let ok = text.contains("Success");
        self.push_log(
            shown,
            Some(&Output {
                code: Some(if ok { 0 } else { 1 }),
                stdout: text.clone().into_bytes(),
                stderr: String::new(),
                timed_out: false,
                elapsed: started.elapsed(),
            }),
            None,
        );
        if text.contains("cmd: not found") || text.contains("Can't find service") {
            return Err(None);
        }
        Ok(text)
    }

    pub fn reboot(&self, serial: &str, mode: &str) -> Result<(), String> {
        let mut rest = vec!["reboot"];
        if !mode.is_empty() {
            rest.push(mode);
        }
        let out = self.adb_for(serial, &rest, Duration::from_secs(20))?;
        if out.ok() { Ok(()) } else { Err(first_line(&out.combined(), "No se pudo reiniciar")) }
    }

    pub fn shell(&self, serial: &str, command: &str) -> Result<String, String> {
        let out = self.adb_for(serial, &["shell", command], Duration::from_secs(20))?;
        let mut text = out.combined();
        if out.timed_out {
            text.push_str("\n[se detuvo tras 20 s]");
        }
        Ok(text)
    }

    pub fn connect(&self, addr: &str) -> Result<String, String> {
        let addr = normalize_addr(addr, 5555)?;
        let out = self.run_adb(&["connect".into(), addr.clone()], Duration::from_secs(20), false)?;
        let text = out.combined();
        self.poll_once();
        if text.contains("connected to") {
            Ok(format!("Conectado a {addr}"))
        } else {
            Err(first_line(&text, "No se pudo conectar"))
        }
    }

    pub fn disconnect(&self, serial: &str) -> Result<(), String> {
        let out = self.run_adb(&["disconnect".into(), serial.into()], Duration::from_secs(10), false)?;
        self.poll_once();
        if out.ok() { Ok(()) } else { Err(first_line(&out.combined(), "No se pudo desconectar")) }
    }

    pub fn pair(&self, addr: &str, code: &str) -> Result<String, String> {
        let addr = normalize_addr(addr, 0)?;
        let code = code.trim();
        if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
            return Err("El código de emparejamiento tiene 6 dígitos".into());
        }
        let out = self.run_adb(&["pair".into(), addr, code.into()], Duration::from_secs(30), false)?;
        let text = out.combined();
        if text.contains("Successfully paired") {
            Ok("Emparejado. Ahora conéctalo con la dirección de depuración inalámbrica.".into())
        } else {
            Err(first_line(&text, "No se pudo emparejar"))
        }
    }

    /// Pasa un dispositivo USB a depuración por Wi-Fi (tcpip 5555 + connect).
    pub fn enable_wifi(&self, serial: &str) -> Result<String, String> {
        let ip = {
            let out = self.adb_for(serial, &["shell", "ip -f inet addr show wlan0"], Duration::from_secs(10))?;
            adb::parse_wlan_ip(&out.stdout_text())
        }
        .ok_or("El dispositivo no tiene IP en wlan0. Conéctalo a una red Wi-Fi.")?;
        let out = self.adb_for(serial, &["tcpip", "5555"], Duration::from_secs(15))?;
        if !out.ok() {
            return Err(first_line(&out.combined(), "No se pudo activar tcpip"));
        }
        thread::sleep(Duration::from_millis(1800));
        self.connect(&format!("{ip}:5555"))
    }

    // -------------------------------------------------------------------
    // scrcpy
    // -------------------------------------------------------------------

    pub fn start_mirror(self: &Shared, serial: &str, options: ScrcpyOptions) -> Result<String, String> {
        // Si ya hay una sesión para ese dispositivo, solo se trae al frente.
        if let Some(pid) = self
            .sessions
            .lock()
            .unwrap()
            .values()
            .find(|h| h.info.serial == serial)
            .map(|h| h.info.pid)
        {
            scrcpy::focus_window(pid);
            return Ok("La pantalla ya estaba abierta".into());
        }
        let label = self.device_label(serial);
        let title = if label == serial { serial.to_string() } else { format!("{label} · {serial}") };
        let args = scrcpy::build_args(serial, &title, &options);
        let (child, lines) = self.spawn_scrcpy(&args)?;

        let id = self.next_session.fetch_add(1, Ordering::Relaxed);
        let info = SessionInfo {
            id,
            serial: serial.to_string(),
            title,
            pid: child.id(),
            started: now_ms(),
            summary: scrcpy::summary(&options),
        };
        self.sessions.lock().unwrap().insert(
            id,
            SessionHandle { info, child, stopping: false, lines },
        );
        self.emit_state();

        let core = self.clone();
        thread::spawn(move || core.watch_session(id));
        Ok("Abriendo pantalla".into())
    }

    /// Lanza scrcpy con nuestro adb y captura sus últimas líneas de salida.
    fn spawn_scrcpy(&self, args: &[String]) -> Result<(Child, Arc<Mutex<VecDeque<String>>>), String> {
        let exe = self
            .scrcpy_path()
            .ok_or("No se encontró scrcpy. Instálalo o indica la ruta en Ajustes.")?;
        let adb_exe = self.adb_path().ok_or("No se encontró adb.")?;

        let mut cmd = process::command(&exe);
        cmd.args(args)
            // Clave para no chocar con Android Studio: scrcpy usa nuestro adb, no el suyo.
            .env("ADB", &adb_exe)
            .env("ANDROID_ADB_SERVER_PORT", self.port().to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(dir) = exe.parent() {
            cmd.current_dir(dir);
        }
        let shown = format!("ADB=adb {}", process::display_command(&exe, args));
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                self.push_log(shown, None, Some(&e.to_string()));
                return Err(format!("No se pudo iniciar scrcpy: {e}"));
            }
        };
        self.push_log(shown, None, Some("iniciado"));

        let lines = Arc::new(Mutex::new(VecDeque::new()));
        for stream in [
            child.stdout.take().map(|s| Box::new(s) as Box<dyn std::io::Read + Send>),
            child.stderr.take().map(|s| Box::new(s) as Box<dyn std::io::Read + Send>),
        ]
        .into_iter()
        .flatten()
        {
            let lines = lines.clone();
            thread::spawn(move || {
                for line in BufReader::new(stream).lines().map_while(Result::ok) {
                    let mut l = lines.lock().unwrap();
                    l.push_back(line);
                    while l.len() > 40 {
                        l.pop_front();
                    }
                }
            });
        }
        Ok((child, lines))
    }

    fn scrcpy_error(&self, serial: &str, lines: &Mutex<VecDeque<String>>) -> String {
        let lines = lines.lock().unwrap();
        self.push_log(format!("scrcpy · {serial}"), None, Some(&lines.iter().cloned().collect::<Vec<_>>().join("\n")));
        lines
            .iter()
            .rev()
            .find(|l| l.contains("ERROR"))
            .or(lines.back())
            .cloned()
            .unwrap_or_else(|| "scrcpy se cerró inesperadamente".into())
            .replace("ERROR: ", "")
    }

    fn watch_session(&self, id: u32) {
        loop {
            thread::sleep(Duration::from_millis(400));
            let mut sessions = self.sessions.lock().unwrap();
            let Some(h) = sessions.get_mut(&id) else { return };
            match h.child.try_wait() {
                Ok(None) => continue,
                result => {
                    let h = sessions.remove(&id).unwrap();
                    drop(sessions);
                    let failed = !matches!(result, Ok(Some(s)) if s.success());
                    if failed && !h.stopping {
                        self.toast("danger", self.scrcpy_error(&h.info.serial, &h.lines));
                    }
                    self.emit_state();
                    return;
                }
            }
        }
    }

    pub fn stop_mirror(self: &Shared, id: u32) -> Result<(), String> {
        let pid = {
            let mut s = self.sessions.lock().unwrap();
            let h = s.get_mut(&id).ok_or("La sesión ya terminó")?;
            h.stopping = true;
            h.info.pid
        };
        // Cierre ordenado; si no responde, se termina.
        scrcpy::close_window(pid);
        let core = self.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(2500));
            if let Some(h) = core.sessions.lock().unwrap().get_mut(&id) {
                let _ = h.child.kill();
            }
        });
        Ok(())
    }

    // -------------------------------------------------------------------
    // Grabación de video (scrcpy sin ventana)
    // -------------------------------------------------------------------

    pub fn start_recording(self: &Shared, serial: &str, options: RecordOptions) -> Result<String, String> {
        if self.recordings.lock().unwrap().contains_key(serial) {
            return Err("Ya se está grabando este dispositivo".into());
        }
        let dir = self.settings().capture_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!(
            "{}.{}",
            self.file_stem_for(serial),
            scrcpy::record_extension(&options)
        ));
        let args = scrcpy::record_args(serial, &options, &path);
        let (child, lines) = self.spawn_scrcpy(&args)?;

        let info = RecordingInfo {
            serial: serial.to_string(),
            pid: child.id(),
            started: now_ms(),
            path: path.display().to_string(),
            audio: options.audio,
        };
        self.recordings.lock().unwrap().insert(
            serial.to_string(),
            RecordingHandle { info, child, stopping: false, lines },
        );
        self.emit_state();

        let core = self.clone();
        let serial = serial.to_string();
        thread::spawn(move || core.watch_recording(&serial));
        Ok(path.display().to_string())
    }

    fn watch_recording(&self, serial: &str) {
        loop {
            thread::sleep(Duration::from_millis(400));
            let mut recs = self.recordings.lock().unwrap();
            let Some(h) = recs.get_mut(serial) else { return };
            match h.child.try_wait() {
                Ok(None) => continue,
                _ => {
                    let h = recs.remove(serial).unwrap();
                    drop(recs);
                    let path = PathBuf::from(&h.info.path);
                    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    if h.stopping && size > 0 {
                        self.toast_file(
                            "success",
                            format!("Video guardado · {}", path.file_name().unwrap_or_default().to_string_lossy()),
                            &h.info.path,
                        );
                    } else if size > 0 {
                        // Se cortó solo (cable desconectado, pantalla bloqueada…); scrcpy igual cierra el archivo.
                        self.toast_file(
                            "warning",
                            format!("La grabación se detuvo: {}", self.scrcpy_error(&h.info.serial, &h.lines)),
                            &h.info.path,
                        );
                    } else {
                        let _ = std::fs::remove_file(&path);
                        self.toast("danger", format!("No se pudo grabar: {}", self.scrcpy_error(&h.info.serial, &h.lines)));
                    }
                    self.emit_state();
                    return;
                }
            }
        }
    }

    pub fn stop_recording(self: &Shared, serial: &str) -> Result<(), String> {
        let pid = {
            let mut recs = self.recordings.lock().unwrap();
            let h = recs.get_mut(serial).ok_or("La grabación ya terminó")?;
            h.stopping = true;
            h.info.pid
        };
        // Ctrl+C: scrcpy escribe el índice del MP4 antes de salir.
        let sent = scrcpy::interrupt(pid);
        self.push_log(format!("Ctrl+C → scrcpy PID {pid}"), None, Some(if sent { "ok" } else { "falló" }));
        let core = self.clone();
        let serial = serial.to_string();
        thread::spawn(move || {
            // Último recurso si no respondió: el archivo podría quedar sin índice.
            thread::sleep(Duration::from_secs(if sent { 10 } else { 0 }));
            if let Some(h) = core.recordings.lock().unwrap().get_mut(&serial) {
                let _ = h.child.kill();
            }
        });
        Ok(())
    }

    /// Ventanas de scrcpy que no abrió esta instancia (p. ej. de una ejecución anterior).
    pub fn external_mirror(&self, pid: u32, action: &str) -> Result<(), String> {
        if !instances::is_process_named(pid, "scrcpy.exe") {
            return Err("Esa ventana de scrcpy ya se cerró".into());
        }
        match action {
            "focus" => {
                scrcpy::focus_window(pid);
                Ok(())
            }
            "close" => {
                if !scrcpy::close_window(pid) {
                    instances::kill(pid);
                }
                Ok(())
            }
            _ => Err("Acción desconocida".into()),
        }
    }

    pub fn focus_mirror(&self, id: u32) {
        if let Some(h) = self.sessions.lock().unwrap().get(&id) {
            scrcpy::focus_window(h.info.pid);
        }
    }

    // -------------------------------------------------------------------
    // Instancias de adb
    // -------------------------------------------------------------------

    pub fn instances(&self) -> InstancesReport {
        let s = self.settings();
        let client = self.adb_path();
        let scrcpy = self.scrcpy_path();
        let version_of = |p: &Path| self.version_of(p);
        instances::scan(ScanInput {
            port: s.server_port,
            isolated: s.is_isolated(),
            client_path: client.as_deref(),
            scrcpy_path: scrcpy.as_deref(),
            version_of: &version_of,
        })
    }

    pub fn kill_adb_process(&self, pid: u32) -> Result<(), String> {
        let report = self.instances();
        if !report.processes.iter().any(|p| p.pid == pid) {
            return Err("Ese proceso ya no existe o no es adb".into());
        }
        let ok = instances::kill(pid);
        self.push_log(format!("terminar adb.exe PID {pid}"), None, Some(if ok { "ok" } else { "falló" }));
        thread::sleep(Duration::from_millis(300));
        self.poll_once();
        if ok { Ok(()) } else { Err("No se pudo terminar el proceso (¿permisos de administrador?)".into()) }
    }

    /// Al salir: detiene el servidor aislado si así se configuró.
    pub fn shutdown(&self) {
        // Cierra las grabaciones en curso para no dejar videos sin índice.
        let pids: Vec<u32> = self.recordings.lock().unwrap().values().map(|h| h.info.pid).collect();
        for pid in &pids {
            scrcpy::interrupt(*pid);
        }
        let deadline = Instant::now() + Duration::from_secs(6);
        while !pids.is_empty() && Instant::now() < deadline {
            let mut recs = self.recordings.lock().unwrap();
            recs.retain(|_, h| matches!(h.child.try_wait(), Ok(None)));
            if recs.is_empty() {
                break;
            }
            drop(recs);
            thread::sleep(Duration::from_millis(150));
        }

        let s = self.settings();
        if s.is_isolated() && s.stop_isolated_on_exit && adb::server_version(s.server_port).is_some() {
            let _ = self.run_adb(&["kill-server".into()], Duration::from_secs(5), true);
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{t}…")
    }
}

fn first_line(text: &str, fallback: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(|l| l.trim_start_matches("error: ").to_string())
        .unwrap_or_else(|| fallback.to_string())
}

fn normalize_addr(addr: &str, default_port: u16) -> Result<String, String> {
    let a = addr.trim();
    if a.is_empty() {
        return Err("Escribe una dirección IP".into());
    }
    if a.contains(':') {
        Ok(a.to_string())
    } else if default_port > 0 {
        Ok(format!("{a}:{default_port}"))
    } else {
        Err("Incluye el puerto (IP:puerto) que muestra el teléfono".into())
    }
}
