//! Inventario de procesos y binarios adb, y detección de conflictos entre servidores.

use crate::adb::{self, AdbVersion};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbProcess {
    pub pid: u32,
    pub path: String,
    pub version: Option<AdbVersion>,
    pub ports: Vec<u16>,
    pub origin: String,
    pub started: u64,
    /// Es el binario que usa esta app.
    pub is_configured: bool,
    /// server | scrcpy | client
    pub role: String,
    /// Argumentos del proceso (sin el ejecutable), para identificar clientes.
    pub command: String,
}

/// Ventana de scrcpy en ejecución (lanzada por esta app o por fuera).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrcpyProcess {
    pub pid: u32,
    pub serial: String,
    pub started: u64,
}

/// Comandos de adb que terminan en segundos; si siguen vivos, están colgados.
const QUICK_COMMANDS: [&str; 8] = [
    "devices", "start-server", "kill-server", "version", "connect", "disconnect", "get-state", "reconnect",
];

fn classify(args: &[String]) -> &'static str {
    if args.iter().any(|a| a == "fork-server" || a == "server") {
        "server"
    } else if args.iter().any(|a| a.contains("com.genymobile.scrcpy") || a.contains("scrcpy-server")) {
        "scrcpy"
    } else {
        "client"
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbBinary {
    pub path: String,
    pub version: Option<AdbVersion>,
    pub origin: String,
    pub is_configured: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    /// danger | warning | info
    pub level: String,
    pub title: String,
    pub detail: String,
    /// Acción sugerida que la interfaz puede ofrecer: use-adb:<ruta> | kill:<pid> | shared-port
    pub action: Option<String>,
    pub action_label: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstancesReport {
    pub port: u16,
    pub isolated: bool,
    pub server_running: bool,
    pub server_version: Option<u32>,
    pub server_pid: Option<u32>,
    pub server_path: Option<String>,
    pub client_path: Option<String>,
    pub client_version: Option<AdbVersion>,
    pub studio_running: bool,
    pub studio_sdk_adb: Option<String>,
    pub processes: Vec<AdbProcess>,
    pub binaries: Vec<AdbBinary>,
    pub scrcpy: Vec<ScrcpyProcess>,
    pub issues: Vec<Issue>,
}

pub fn origin_of(path: &Path, scrcpy: Option<&Path>) -> String {
    if adb::is_in_sdk(path) {
        return "SDK de Android".into();
    }
    if let Some(s) = scrcpy.and_then(|s| s.parent())
        && path.parent().map(|p| adb::same_path(p, s)).unwrap_or(false)
    {
        return "Incluido en scrcpy".into();
    }
    if path.parent().map(|p| p.join("scrcpy.exe").is_file()).unwrap_or(false) {
        return "Incluido en scrcpy".into();
    }
    let lower = path.display().to_string().to_lowercase();
    for (needle, label) in [
        ("genymotion", "Genymotion"),
        ("bluestacks", "BlueStacks"),
        ("nox", "Nox"),
        ("memu", "MEmu"),
        ("ldplayer", "LDPlayer"),
        ("vysor", "Vysor"),
        ("jetbrains", "JetBrains"),
        ("unity", "Unity"),
    ] {
        if lower.contains(needle) {
            return label.into();
        }
    }
    "Otro".into()
}

/// Puertos TCP en escucha por PID (solo los de procesos que interesan).
fn listening_ports() -> HashMap<u32, Vec<u16>> {
    use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
    let mut map: HashMap<u32, Vec<u16>> = HashMap::new();
    let af = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    if let Ok(sockets) = netstat2::get_sockets_info(af, ProtocolFlags::TCP) {
        for s in sockets {
            if let ProtocolSocketInfo::Tcp(t) = &s.protocol_socket_info
                && t.state == TcpState::Listen
            {
                for pid in &s.associated_pids {
                    let ports = map.entry(*pid).or_default();
                    if !ports.contains(&t.local_port) {
                        ports.push(t.local_port);
                    }
                }
            }
        }
    }
    for v in map.values_mut() {
        v.sort_unstable();
    }
    map
}

pub struct ScanInput<'a> {
    pub port: u16,
    pub isolated: bool,
    pub client_path: Option<&'a Path>,
    pub scrcpy_path: Option<&'a Path>,
    pub version_of: &'a dyn Fn(&Path) -> Option<AdbVersion>,
}

pub fn scan(input: ScanInput) -> InstancesReport {
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_exe(UpdateKind::Always)
            .with_cmd(UpdateKind::Always),
    );
    let ports = listening_ports();

    let mut studio_running = false;
    let mut processes = Vec::new();
    let mut scrcpy = Vec::new();
    for (pid, p) in sys.processes() {
        let name = p.name().to_string_lossy().to_lowercase();
        if name == "studio64.exe" || name == "studio.exe" {
            studio_running = true;
        }
        let args: Vec<String> = p.cmd().iter().skip(1).map(|a| a.to_string_lossy().to_string()).collect();
        // Las grabaciones sin ventana no son pantallas que se puedan traer al frente.
        if name == "scrcpy.exe" && !args.iter().any(|a| a == "--no-window") {
            let serial = args
                .iter()
                .enumerate()
                .find_map(|(i, a)| {
                    a.strip_prefix("--serial=")
                        .map(str::to_string)
                        .or_else(|| (a == "-s" || a == "--serial").then(|| args.get(i + 1).cloned()).flatten())
                })
                .unwrap_or_default();
            scrcpy.push(ScrcpyProcess { pid: pid.as_u32(), serial, started: p.start_time() });
            continue;
        }
        if name == "scrcpy.exe" {
            continue;
        }
        if name != "adb.exe" {
            continue;
        }
        let role = classify(&args);
        let command = args.join(" ");
        let pid = pid.as_u32();
        let path = p.exe().map(Path::to_path_buf);
        let version = path.as_deref().and_then(|x| (input.version_of)(x));
        processes.push(AdbProcess {
            pid,
            path: path.as_ref().map(|x| x.display().to_string()).unwrap_or_default(),
            origin: path
                .as_deref()
                .map(|x| origin_of(x, input.scrcpy_path))
                .unwrap_or_else(|| "Desconocido".into()),
            is_configured: match (path.as_deref(), input.client_path) {
                (Some(a), Some(b)) => adb::same_path(a, b),
                _ => false,
            },
            version,
            ports: ports.get(&pid).cloned().unwrap_or_default(),
            started: p.start_time(),
            role: role.into(),
            command: if command.len() > 300 { format!("{}…", &command[..command.floor_char_boundary(300)]) } else { command },
        });
    }
    processes.sort_by_key(|p| p.pid);
    scrcpy.sort_by_key(|s| s.pid);

    // Binarios conocidos en el sistema.
    let mut bin_paths: Vec<PathBuf> = Vec::new();
    bin_paths.extend(input.client_path.map(Path::to_path_buf));
    bin_paths.extend(adb::sdk_adb_candidates());
    bin_paths.extend(adb::find_in_path("adb.exe"));
    bin_paths.extend(input.scrcpy_path.and_then(crate::scrcpy::bundled_adb));
    bin_paths.extend(processes.iter().filter(|p| !p.path.is_empty()).map(|p| PathBuf::from(&p.path)));
    let binaries: Vec<AdbBinary> = adb::dedup_paths(bin_paths)
        .into_iter()
        .filter(|p| p.is_file())
        .map(|p| AdbBinary {
            version: (input.version_of)(&p),
            origin: origin_of(&p, input.scrcpy_path),
            is_configured: input.client_path.map(|c| adb::same_path(&p, c)).unwrap_or(false),
            path: p.display().to_string(),
        })
        .collect();

    let server_version = adb::server_version(input.port);
    let server_proc = processes.iter().find(|p| p.ports.contains(&input.port));
    let client_version = input.client_path.and_then(|p| (input.version_of)(p));
    let studio_sdk_adb = adb::sdk_adb_candidates().first().map(|p| p.display().to_string());

    let mut report = InstancesReport {
        port: input.port,
        isolated: input.isolated,
        server_running: server_version.is_some(),
        server_version,
        server_pid: server_proc.map(|p| p.pid),
        server_path: server_proc.map(|p| p.path.clone()).filter(|p| !p.is_empty()),
        client_path: input.client_path.map(|p| p.display().to_string()),
        client_version,
        studio_running,
        studio_sdk_adb,
        processes,
        binaries,
        scrcpy,
        issues: Vec::new(),
    };
    report.issues = detect_issues(&report);
    report
}

fn issue(level: &str, title: impl Into<String>, detail: impl Into<String>) -> Issue {
    Issue {
        level: level.into(),
        title: title.into(),
        detail: detail.into(),
        action: None,
        action_label: None,
    }
}

fn detect_issues(r: &InstancesReport) -> Vec<Issue> {
    let mut out = Vec::new();

    let Some(client) = &r.client_path else {
        out.push(issue(
            "danger",
            "No se encontró adb",
            "Instala Android SDK Platform-Tools (desde Android Studio → SDK Manager) o indica la ruta de adb.exe en Ajustes.",
        ));
        return out;
    };

    // 1. Versión de protocolo distinta: la pelea clásica de servidores.
    if let (Some(sv), Some(cv)) = (r.server_version, r.client_version.as_ref())
        && sv != cv.protocol
    {
        let mut i = issue(
            "danger",
            format!("Versión distinta en el puerto {}", r.port),
            format!(
                "El servidor en ejecución es v{sv} y el adb de esta app es v{}. Cada vez que uno de los dos se conecte, reiniciará el servidor del otro y los dispositivos se desconectarán.",
                cv.protocol
            ),
        );
        if let Some(sp) = &r.server_path {
            i.action = Some(format!("use-adb:{sp}"));
            i.action_label = Some("Usar el adb del servidor".into());
        }
        out.push(i);
    } else if let Some(sp) = &r.server_path
        && !adb::same_path(Path::new(sp), Path::new(client))
    {
        out.push(issue(
            "info",
            "El servidor lo inició otro binario",
            format!("{sp} tiene la misma versión de protocolo; se puede compartir sin conflictos."),
        ));
    }

    // 2. El adb configurado no es el del SDK que usa Android Studio.
    if !r.isolated
        && let Some(sdk) = &r.studio_sdk_adb
        && !adb::same_path(Path::new(sdk), Path::new(client))
    {
        let sdk_ver = r.binaries.iter().find(|b| adb::same_path(Path::new(&b.path), Path::new(sdk)));
        let differs = match (sdk_ver.and_then(|b| b.version.as_ref()), r.client_version.as_ref()) {
            (Some(a), Some(b)) => a.protocol != b.protocol,
            _ => true,
        };
        if differs {
            let mut i = issue(
                "warning",
                "No usas el adb de Android Studio",
                "Android Studio usa el adb del SDK. Con un binario de otra versión en el mismo puerto, los servidores se reinician mutuamente.",
            );
            i.action = Some(format!("use-adb:{sdk}"));
            i.action_label = Some("Usar el adb del SDK".into());
            out.push(i);
        }
    }

    // 3. Clientes adb de comandos rápidos que siguen vivos — colgados.
    //    (Los túneles de scrcpy, logcat o shell interactivos son legítimos.)
    let is_quick = |p: &&AdbProcess| {
        p.role == "client"
            && p.command
                .split_whitespace()
                .map(|a| a.trim_matches('"'))
                .any(|a| QUICK_COMMANDS.contains(&a))
    };
    for p in r.processes.iter().filter(|p| p.ports.is_empty()).filter(is_quick) {
        let started = p.started;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if now.saturating_sub(started) > 20 {
            let mut i = issue(
                "warning",
                format!("Cliente adb colgado · PID {}", p.pid),
                format!("«adb {}» lleva más de 20 s sin terminar. Puede bloquear el reinicio del servidor.", p.command),
            );
            i.action = Some(format!("kill:{}", p.pid));
            i.action_label = Some("Terminar proceso".into());
            out.push(i);
        }
    }

    // 4. Varios servidores a la vez.
    let servers: Vec<_> = r.processes.iter().filter(|p| !p.ports.is_empty()).collect();
    if servers.len() > 1 {
        let list = servers
            .iter()
            .map(|p| format!("PID {} en {}", p.pid, p.ports.iter().map(u16::to_string).collect::<Vec<_>>().join(", ")))
            .collect::<Vec<_>>()
            .join(" · ");
        out.push(issue(
            "warning",
            "Hay más de un servidor adb",
            format!("{list}. Un dispositivo USB solo puede ser atendido por un servidor a la vez; el otro lo verá como offline o no lo verá."),
        ));
    }

    // 5. Modo aislado con Android Studio abierto.
    if r.isolated && r.studio_running {
        let mut i = issue(
            "info",
            format!("Modo aislado en el puerto {}", r.port),
            "Android Studio sigue en el 5037. Los dispositivos por Wi-Fi funcionan en ambos; los USB los toma el primer servidor que los abre.",
        );
        i.action = Some("shared-port".into());
        i.action_label = Some("Volver a compartido".into());
        out.push(i);
    }

    // 6. adb incluido en scrcpy con otra versión (se evita pasando ADB=…).
    if let Some(cv) = &r.client_version {
        for b in r.binaries.iter().filter(|b| b.origin == "Incluido en scrcpy") {
            if let Some(bv) = &b.version
                && bv.protocol != cv.protocol
            {
                out.push(issue(
                    "info",
                    format!("scrcpy incluye adb {}", bv.tools),
                    "Esta app lanza scrcpy con la variable ADB apuntando al adb configurado, así que ese binario no se usa ni reinicia el servidor.",
                ));
            }
        }
    }

    out
}

pub fn is_process_named(pid: u32, name: &str) -> bool {
    let mut sys = System::new();
    let spid = sysinfo::Pid::from_u32(pid);
    sys.refresh_processes(ProcessesToUpdate::Some(&[spid]), true);
    sys.process(spid)
        .map(|p| p.name().to_string_lossy().eq_ignore_ascii_case(name))
        .unwrap_or(false)
}

pub fn kill(pid: u32) -> bool {
    let mut sys = System::new();
    let spid = sysinfo::Pid::from_u32(pid);
    sys.refresh_processes(ProcessesToUpdate::Some(&[spid]), true);
    sys.process(spid).map(|p| p.kill()).unwrap_or(false)
}
