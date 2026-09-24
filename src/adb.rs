//! Descubrimiento del binario adb, protocolo del servidor y parseo de salidas.

use serde::Serialize;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Localización de binarios
// ---------------------------------------------------------------------------

/// Directorios de SDK conocidos, en el orden en que Android Studio los usaría.
pub fn sdk_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for var in ["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
        if let Some(v) = std::env::var_os(var) {
            dirs.push(PathBuf::from(v));
        }
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        dirs.push(PathBuf::from(local).join("Android").join("Sdk"));
    }
    dedup_paths(dirs)
}

/// Carpetas del PATH que contienen el ejecutable indicado.
pub fn find_in_path(exe: &str) -> Vec<PathBuf> {
    let Some(path) = std::env::var_os("PATH") else {
        return vec![];
    };
    std::env::split_paths(&path)
        .map(|d| d.join(exe))
        .filter(|p| p.is_file())
        .collect()
}

pub fn sdk_adb_candidates() -> Vec<PathBuf> {
    sdk_dirs()
        .into_iter()
        .map(|d| d.join("platform-tools").join("adb.exe"))
        .filter(|p| p.is_file())
        .collect()
}

/// adb preferido: el del SDK (compartido con Android Studio), luego el del PATH y
/// por último el que haya descargado esta app.
pub fn auto_adb() -> Option<PathBuf> {
    sdk_adb_candidates()
        .into_iter()
        .chain(find_in_path("adb.exe"))
        .chain(crate::tools::adb_exe())
        .next()
}

pub fn is_in_sdk(path: &Path) -> bool {
    let p = normalize(path);
    sdk_dirs().iter().any(|d| p.starts_with(&normalize(d)))
}

pub fn normalize(p: &Path) -> String {
    let s = std::fs::canonicalize(p)
        .unwrap_or_else(|_| p.to_path_buf())
        .display()
        .to_string();
    s.trim_start_matches(r"\\?\").to_lowercase()
}

pub fn same_path(a: &Path, b: &Path) -> bool {
    normalize(a) == normalize(b)
}

pub fn dedup_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = std::collections::HashSet::new();
    paths
        .into_iter()
        .filter(|p| seen.insert(normalize(p)))
        .collect()
}

// ---------------------------------------------------------------------------
// Versión del cliente
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbVersion {
    /// Versión de protocolo (p. ej. 41 de "1.0.41"). Cliente y servidor deben coincidir.
    pub protocol: u32,
    /// Versión de platform-tools (p. ej. "35.0.2").
    pub tools: String,
}

pub fn parse_version(text: &str) -> Option<AdbVersion> {
    let mut protocol = None;
    let mut tools = String::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Android Debug Bridge version ") {
            protocol = rest.trim().rsplit('.').next().and_then(|v| v.parse().ok());
        } else if let Some(rest) = line.strip_prefix("Version ") {
            tools = rest.split('-').next().unwrap_or(rest).trim().to_string();
        }
    }
    protocol.map(|protocol| AdbVersion { protocol, tools })
}

// ---------------------------------------------------------------------------
// Protocolo del servidor (smart sockets) — no arranca el servidor por accidente
// ---------------------------------------------------------------------------

fn read_exact_str(s: &mut TcpStream, n: usize) -> std::io::Result<String> {
    let mut buf = vec![0u8; n];
    s.read_exact(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

fn read_len_prefixed(s: &mut TcpStream) -> std::io::Result<String> {
    let len = read_exact_str(s, 4)?;
    let len = usize::from_str_radix(&len, 16)
        .map_err(|_| std::io::Error::other("longitud inválida"))?;
    read_exact_str(s, len)
}

/// Envía una petición y espera OKAY; con FAIL devuelve el mensaje del servidor.
fn request(s: &mut TcpStream, service: &str) -> std::io::Result<()> {
    s.write_all(format!("{:04x}{}", service.len(), service).as_bytes())?;
    let status = read_exact_str(s, 4)?;
    match status.as_str() {
        "OKAY" => Ok(()),
        "FAIL" => Err(std::io::Error::other(
            read_len_prefixed(s).unwrap_or_else(|_| "FAIL".into()),
        )),
        other => Err(std::io::Error::other(format!("respuesta inesperada {other}"))),
    }
}

fn connect(port: u16) -> std::io::Result<TcpStream> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(400))
}

pub fn host_query(port: u16, service: &str) -> std::io::Result<String> {
    let mut s = connect(port)?;
    s.set_read_timeout(Some(Duration::from_secs(3)))?;
    s.set_write_timeout(Some(Duration::from_secs(3)))?;
    request(&mut s, service)?;
    read_len_prefixed(&mut s)
}

/// Abre un servicio en el dispositivo (p. ej. `exec:cmd package install -S …`) y
/// devuelve el socket en crudo: lo escrito llega al stdin del comando y lo leído
/// es su salida. Es lo mismo que hace `adb install` por dentro.
pub fn device_service(port: u16, serial: &str, service: &str, timeout: Duration) -> std::io::Result<TcpStream> {
    let mut s = connect(port)?;
    s.set_read_timeout(Some(timeout))?;
    s.set_write_timeout(Some(timeout))?;
    request(&mut s, &format!("host:transport:{serial}"))?;
    request(&mut s, service)?;
    Ok(s)
}

// ---------------------------------------------------------------------------
// Errores de instalación
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InstallFailure {
    /// Código de Android, p. ej. INSTALL_FAILED_UPDATE_INCOMPATIBLE.
    pub code: String,
    /// Explicación para el usuario.
    pub message: String,
    pub package: Option<String>,
    /// Solución que la interfaz puede ofrecer: uninstall | downgrade
    pub fix: Option<String>,
}

pub fn parse_install_failure(output: &str) -> InstallFailure {
    let code = output
        .split(|c: char| !(c.is_ascii_uppercase() || c == '_'))
        .find(|t| t.starts_with("INSTALL_") && t.len() > 10)
        .unwrap_or_default()
        .to_string();
    let package = output
        .split_once("Package ")
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .map(|p| p.trim_end_matches([':', ';', ',', ']']).to_string())
        .filter(|p| p.contains('.'));
    let (message, fix) = match code.as_str() {
        "INSTALL_FAILED_UPDATE_INCOMPATIBLE" | "INSTALL_FAILED_SHARED_USER_INCOMPATIBLE" => (
            "Ya hay una versión instalada firmada con otra clave (p. ej. debug contra release). Hay que desinstalarla primero; se pierden sus datos.",
            Some("uninstall"),
        ),
        "INSTALL_FAILED_VERSION_DOWNGRADE" => (
            "El dispositivo tiene una versión más nueva de esta app.",
            Some("downgrade"),
        ),
        "INSTALL_FAILED_INSUFFICIENT_STORAGE" => ("No hay espacio suficiente en el dispositivo.", None),
        "INSTALL_FAILED_USER_RESTRICTED" | "INSTALL_FAILED_ABORTED" => (
            "La instalación se canceló en el teléfono. Acepta el aviso en pantalla o activa «Instalar vía USB» en Opciones de desarrollador.",
            None,
        ),
        "INSTALL_FAILED_OLDER_SDK" => ("La app requiere una versión de Android más nueva que la del dispositivo.", None),
        "INSTALL_FAILED_NO_MATCHING_ABIS" => ("La app no incluye librerías para la arquitectura de este dispositivo.", None),
        "INSTALL_FAILED_DUPLICATE_PERMISSION" => ("Otra app instalada ya define uno de los permisos de esta.", None),
        "INSTALL_FAILED_INVALID_APK" | "INSTALL_PARSE_FAILED_NOT_APK" | "INSTALL_PARSE_FAILED_NO_CERTIFICATES"
        | "INSTALL_PARSE_FAILED_UNEXPECTED_EXCEPTION" => ("El archivo no es un APK válido o no está firmado.", None),
        "INSTALL_FAILED_VERIFICATION_FAILURE" => ("Play Protect u otra verificación rechazó la app en el teléfono.", None),
        _ => ("El dispositivo rechazó la instalación.", None),
    };
    InstallFailure {
        code,
        message: message.into(),
        package,
        fix: fix.map(str::to_string),
    }
}

/// Versión de protocolo del servidor que escucha en `port`, o None si no hay servidor.
pub fn server_version(port: u16) -> Option<u32> {
    host_query(port, "host:version")
        .ok()
        .and_then(|v| u32::from_str_radix(v.trim(), 16).ok())
}

// ---------------------------------------------------------------------------
// Dispositivos
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub serial: String,
    /// device | unauthorized | offline | recovery | sideload | bootloader | ...
    pub state: String,
    /// usb | wifi | emulator
    pub transport: String,
    pub model: String,
    pub product: String,
    pub device: String,
    pub transport_id: String,
}

pub fn parse_devices(text: &str) -> Vec<Device> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("List of devices") || line.starts_with('*') {
            continue;
        }
        let mut tokens = line.split_whitespace();
        let Some(serial) = tokens.next() else { continue };
        const KEYS: [&str; 5] = ["usb", "product", "model", "device", "transport_id"];
        let mut state_parts = Vec::new();
        let mut kv = HashMap::new();
        let mut in_kv = false;
        for t in tokens {
            if let Some((k, v)) = t.split_once(':')
                && (in_kv || KEYS.contains(&k))
            {
                in_kv = true;
                kv.insert(k.to_string(), v.to_string());
            } else if !in_kv {
                state_parts.push(t);
            }
        }
        let transport = if serial.starts_with("emulator-") {
            "emulator"
        } else if serial.contains(':') || serial.contains("._adb-tls-connect.") {
            "wifi"
        } else {
            "usb"
        };
        out.push(Device {
            serial: serial.to_string(),
            state: state_parts.join(" "),
            transport: transport.into(),
            model: kv.get("model").cloned().unwrap_or_default().replace('_', " "),
            product: kv.get("product").cloned().unwrap_or_default(),
            device: kv.get("device").cloned().unwrap_or_default(),
            transport_id: kv.get("transport_id").cloned().unwrap_or_default(),
        });
    }
    out.sort_by(|a, b| a.serial.cmp(&b.serial));
    out
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDetails {
    pub manufacturer: String,
    pub brand: String,
    pub model: String,
    pub name: String,
    pub android: String,
    pub sdk: String,
    pub abi: String,
    pub resolution: String,
    pub density: String,
    pub battery: Option<u32>,
    pub charging: bool,
    pub ip: String,
    pub fingerprint: String,
}

pub const DETAILS_SEP: &str = "__ADM_SEP__";

/// Un solo `adb shell` con todo lo necesario, separado por marcas.
pub fn details_script() -> String {
    format!(
        "getprop; echo {s}; wm size; wm density; echo {s}; dumpsys battery; echo {s}; ip -f inet addr show wlan0",
        s = DETAILS_SEP
    )
}

pub fn parse_details(text: &str) -> DeviceDetails {
    let parts: Vec<&str> = text.split(DETAILS_SEP).collect();
    let props = parse_getprop(parts.first().copied().unwrap_or_default());
    let get = |k: &str| props.get(k).cloned().unwrap_or_default();

    let mut d = DeviceDetails {
        manufacturer: get("ro.product.manufacturer"),
        brand: get("ro.product.brand"),
        model: get("ro.product.model"),
        name: {
            let n = get("ro.product.marketname");
            if n.is_empty() { get("ro.config.marketing_name") } else { n }
        },
        android: get("ro.build.version.release"),
        sdk: get("ro.build.version.sdk"),
        abi: get("ro.product.cpu.abi"),
        fingerprint: get("ro.build.fingerprint"),
        ..Default::default()
    };

    if let Some(wm) = parts.get(1) {
        let mut physical = String::new();
        let mut over = String::new();
        let mut dens_phys = String::new();
        let mut dens_over = String::new();
        for line in wm.lines() {
            let line = line.trim();
            if let Some(v) = line.strip_prefix("Physical size:") {
                physical = v.trim().replace('x', "×");
            } else if let Some(v) = line.strip_prefix("Override size:") {
                over = v.trim().replace('x', "×");
            } else if let Some(v) = line.strip_prefix("Physical density:") {
                dens_phys = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("Override density:") {
                dens_over = v.trim().to_string();
            }
        }
        d.resolution = if over.is_empty() { physical } else { over };
        d.density = if dens_over.is_empty() { dens_phys } else { dens_over };
    }

    if let Some(bat) = parts.get(2) {
        let (level, charging) = parse_battery(bat);
        d.battery = level;
        d.charging = charging;
    }

    if let Some(ip) = parts.get(3) {
        d.ip = parse_wlan_ip(ip).unwrap_or_default();
    }
    d
}

pub fn parse_getprop(text: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix('[')
            && let Some((k, v)) = rest.split_once("]: [")
        {
            m.insert(k.to_string(), v.trim_end_matches(']').to_string());
        }
    }
    m
}

pub fn parse_battery(text: &str) -> (Option<u32>, bool) {
    let mut level = None;
    let mut charging = false;
    for line in text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("level:") {
            level = v.trim().parse().ok();
        } else if let Some(v) = line.strip_prefix("status:") {
            // 2 = cargando, 5 = llena
            charging = matches!(v.trim(), "2" | "5");
        }
    }
    (level, charging)
}

pub fn parse_wlan_ip(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find_map(|l| l.strip_prefix("inet "))
        .and_then(|rest| rest.split('/').next())
        .map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_devices_l() {
        let text = "List of devices attached\n\
            R58N12345AB            device usb:1-1 product:a51nsxx model:SM_A515F device:a51 transport_id:3\n\
            emulator-5554          offline transport_id:1\n\
            192.168.1.20:5555      unauthorized transport_id:4\n\
            ZY22 no permissions (missing udev rules?); see [http://developer.android.com/tools/device.html] usb:1-2 transport_id:5\n";
        let d = parse_devices(text);
        assert_eq!(d.len(), 4);
        let s = d.iter().find(|x| x.serial == "R58N12345AB").unwrap();
        assert_eq!(s.state, "device");
        assert_eq!(s.model, "SM A515F");
        assert_eq!(s.transport, "usb");
        assert_eq!(d.iter().find(|x| x.serial == "emulator-5554").unwrap().transport, "emulator");
        let w = d.iter().find(|x| x.serial.starts_with("192")).unwrap();
        assert_eq!((w.state.as_str(), w.transport.as_str()), ("unauthorized", "wifi"));
    }

    #[test]
    fn parses_install_failures() {
        let f = parse_install_failure(
            "Failure [INSTALL_FAILED_UPDATE_INCOMPATIBLE: Package com.retailone.app signatures do not match newer version; ignoring!]",
        );
        assert_eq!(f.code, "INSTALL_FAILED_UPDATE_INCOMPATIBLE");
        assert_eq!(f.package.as_deref(), Some("com.retailone.app"));
        assert_eq!(f.fix.as_deref(), Some("uninstall"));

        let f = parse_install_failure("Failure [INSTALL_FAILED_VERSION_DOWNGRADE: Downgrade detected: Update version code 3 is older than current 5]");
        assert_eq!(f.fix.as_deref(), Some("downgrade"));
        assert_eq!(f.package, None);

        assert_eq!(parse_install_failure("adb: failed to install x.apk").code, "");
    }

    #[test]
    fn parses_version() {
        let v = parse_version(
            "Android Debug Bridge version 1.0.41\nVersion 35.0.2-12147458\nInstalled as C:\\x\\adb.exe\n",
        )
        .unwrap();
        assert_eq!(v.protocol, 41);
        assert_eq!(v.tools, "35.0.2");
    }

    #[test]
    fn parses_details() {
        let text = format!(
            "[ro.product.model]: [Pixel 7]\n[ro.build.version.release]: [14]\n{s}\nPhysical size: 1080x2400\nPhysical density: 420\n{s}\n  AC powered: false\n  status: 2\n  level: 87\n{s}\n    inet 192.168.1.23/24 brd 192.168.1.255 scope global wlan0\n",
            s = DETAILS_SEP
        );
        let d = parse_details(&text);
        assert_eq!(d.model, "Pixel 7");
        assert_eq!(d.resolution, "1080×2400");
        assert_eq!(d.density, "420");
        assert_eq!(d.battery, Some(87));
        assert!(d.charging);
        assert_eq!(d.ip, "192.168.1.23");
    }
}
