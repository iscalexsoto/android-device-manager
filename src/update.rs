//! Actualización de la app desde las releases de GitHub.
//!
//! El workflow de GitHub Actions publica una release al subir un tag `v*`, con el
//! ejecutable portable y el instalador NSIS. Se usa el que corresponde a cómo se
//! está ejecutando la app:
//! - Instalada (hay `uninstall.exe` junto al ejecutable): se descarga el instalador y
//!   se ejecuta en modo pasivo con `/P /UPDATE /R`, que instala encima sin
//!   desinstalar, cierra la app si sigue abierta y la vuelve a abrir al terminar.
//! - Portable: se descarga el ejecutable, el actual se aparta como `.old.exe`
//!   (Windows permite renombrar un exe en ejecución) y el nuevo queda en su lugar.

use crate::tools;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

const API_LATEST: &str = "https://api.github.com/repos/iscalexsoto/android-device-manager/releases/latest";
/// Solo se abren notas de este repositorio.
pub const RELEASES_URL: &str = "https://github.com/iscalexsoto/android-device-manager/releases/";
pub const CURRENT: &str = env!("CARGO_PKG_VERSION");
/// Entre comprobaciones automáticas deben pasar al menos 24 h.
pub const CHECK_INTERVAL: u64 = 24 * 60 * 60;
/// Argumento con el que la versión nueva espera a que la anterior termine de cerrarse.
const WAIT_ARG: &str = "--wait-pid=";

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Installed,
    Portable,
    /// Compilación de depuración (`cargo run`): se avisa, pero no se reemplaza nada.
    Dev,
}

pub fn mode() -> Mode {
    if cfg!(debug_assertions) {
        return Mode::Dev;
    }
    let installed = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join("uninstall.exe").is_file()))
        .unwrap_or(false);
    if installed { Mode::Installed } else { Mode::Portable }
}

/// Versión nueva publicada en GitHub.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub version: String,
    pub notes_url: String,
    pub asset_name: String,
    #[serde(skip)]
    asset_url: String,
    pub size: u64,
    /// SHA-256 publicado por GitHub (hex en minúsculas).
    #[serde(skip)]
    sha256: Option<String>,
    /// Hay SHA-256 con qué verificar la descarga.
    pub verified: bool,
}

/// Estado que ve la interfaz (comando `get_update` y evento `update`).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateState {
    pub current: &'static str,
    pub mode: Mode,
    /// idle | checking | uptodate | available | downloading | ready | error
    pub phase: &'static str,
    pub release: Option<Release>,
    pub received: u64,
    pub total: u64,
    pub message: Option<String>,
    /// Ejecutable nuevo (portable) o instalador descargado (instalada).
    pub file: Option<String>,
}

pub struct Updater {
    app: AppHandle,
    state: Mutex<UpdateState>,
}

impl Updater {
    pub fn new(app: AppHandle) -> Self {
        let state = UpdateState {
            current: CURRENT,
            mode: mode(),
            phase: "idle",
            release: None,
            received: 0,
            total: 0,
            message: None,
            file: None,
        };
        Self { app, state: Mutex::new(state) }
    }

    pub fn state(&self) -> UpdateState {
        self.state.lock().unwrap().clone()
    }

    fn set(&self, f: impl FnOnce(&mut UpdateState)) {
        let snapshot = {
            let mut s = self.state.lock().unwrap();
            f(&mut s);
            s.clone()
        };
        let _ = self.app.emit("update", snapshot);
    }

    /// Consulta la última release. No hace nada si ya hay una operación en curso
    /// o una versión descargada esperando el reinicio.
    pub fn check(&self) -> UpdateState {
        {
            let mut s = self.state.lock().unwrap();
            if matches!(s.phase, "checking" | "downloading" | "ready") {
                return s.clone();
            }
            s.phase = "checking";
            s.message = None;
        }
        self.set(|_| {});
        let mode = self.state().mode;
        match fetch_latest(mode) {
            Ok(Some(rel)) => self.set(|s| {
                s.phase = "available";
                s.total = rel.size;
                s.release = Some(rel);
            }),
            Ok(None) => self.set(|s| {
                s.phase = "uptodate";
                s.release = None;
            }),
            Err(e) => self.set(|s| {
                s.phase = "error";
                s.message = Some(e);
            }),
        }
        self.state()
    }

    /// Descarga la versión disponible y la deja lista para el reinicio.
    pub fn download(&self) -> Result<UpdateState, String> {
        let (rel, mode) = {
            let mut s = self.state.lock().unwrap();
            if s.mode == Mode::Dev {
                return Err("Es una compilación de desarrollo: actualiza con git pull y cargo build.".into());
            }
            let rel = match (&s.release, s.phase) {
                (Some(rel), "available" | "error") => rel.clone(),
                _ => return Err("No hay ninguna actualización pendiente".into()),
            };
            s.phase = "downloading";
            s.received = 0;
            s.total = rel.size;
            s.message = None;
            s.file = None;
            (rel, s.mode)
        };
        self.set(|_| {});
        let progress = |received: u64, total: Option<u64>| {
            self.set(|s| {
                s.received = received;
                if let Some(t) = total {
                    s.total = t;
                }
            })
        };
        let res = match mode {
            Mode::Installed => download_installer(&rel, progress).map_err(|e| (e, None)),
            _ => replace_exe(&rel, progress),
        };
        match res {
            Ok(path) => {
                self.set(|s| {
                    s.phase = "ready";
                    s.file = Some(path.display().to_string());
                });
                Ok(self.state())
            }
            Err((reason, saved)) => {
                // Con el archivo guardado aparte se avisa dónde quedó; si no, se puede reintentar.
                self.set(|s| {
                    s.phase = if saved.is_some() { "error" } else { "available" };
                    s.message = Some(reason.clone());
                    s.file = saved.map(|p| p.display().to_string());
                });
                Err(reason)
            }
        }
    }

    /// Abre la versión nueva (o el instalador). Quien llama debe cerrar la app después.
    pub fn apply(&self) -> Result<(), String> {
        let s = self.state();
        let (Some(file), "ready") = (&s.file, s.phase) else {
            return Err("La actualización todavía no está descargada".into());
        };
        let mut cmd = std::process::Command::new(file);
        match s.mode {
            Mode::Installed => cmd.args(["/P", "/UPDATE", "/R"]),
            _ => cmd.arg(format!("{WAIT_ARG}{}", std::process::id())),
        };
        cmd.spawn().map(|_| ()).map_err(|e| format!("No se pudo iniciar {file}: {e}"))
    }
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Al arrancar: espera a que termine la versión anterior (si esta la reemplazó)
/// y borra lo que dejó la última actualización.
pub fn on_start() {
    if let Some(pid) = std::env::args().find_map(|a| a.strip_prefix(WAIT_ARG).and_then(|p| p.parse::<u32>().ok())) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while crate::instances::is_running(pid) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(200));
        }
    }
    if let Some(old) = std::env::current_exe().ok().and_then(|exe| old_exe_path(&exe)) {
        let _ = fs::remove_file(old);
    }
    let _ = fs::remove_dir_all(installer_dir());
}

/// `1.4.0` > `1.3.1`; los tags pueden llevar `v` delante. Si no se entiende, no es más nueva.
pub fn newer(candidate: &str, current: &str) -> bool {
    match (parse_version(candidate), parse_version(current)) {
        (Some(a), Some(b)) => a > b,
        _ => false,
    }
}

fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let s = s.trim().trim_start_matches(['v', 'V']);
    let s = s.split(['-', '+']).next()?;
    let mut it = s.split('.').map(|p| p.parse::<u64>().ok());
    let major = it.next()??;
    let minor = it.next().unwrap_or(Some(0))?;
    let patch = it.next().unwrap_or(Some(0))?;
    if it.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GhAsset>,
}

#[derive(Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    /// "sha256:<hex>"
    digest: Option<String>,
}

/// Instalador NSIS (`…_x64-setup.exe`) o ejecutable portable (`android-device-manager….exe`).
fn pick_asset(assets: &[GhAsset], mode: Mode) -> Option<&GhAsset> {
    assets.iter().find(|a| {
        let n = a.name.to_ascii_lowercase();
        match mode {
            Mode::Installed => n.ends_with("-setup.exe"),
            _ => n.starts_with("android-device-manager") && n.ends_with(".exe") && !n.contains("setup"),
        }
    })
}

/// `None` si la última release no es más nueva que esta versión.
fn release_for(gh: GhRelease, mode: Mode) -> Result<Option<Release>, String> {
    let version = gh.tag_name.trim_start_matches(['v', 'V']).to_string();
    if !newer(&version, CURRENT) {
        return Ok(None);
    }
    let what = if mode == Mode::Installed { "el instalador" } else { "el ejecutable portable" };
    let asset = pick_asset(&gh.assets, mode).ok_or_else(|| format!("La versión {version} no incluye {what}."))?;
    let sha256 = asset
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
        .filter(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
        .map(str::to_ascii_lowercase);
    Ok(Some(Release {
        version,
        notes_url: gh.html_url,
        asset_name: asset.name.clone(),
        asset_url: asset.browser_download_url.clone(),
        size: asset.size,
        verified: sha256.is_some(),
        sha256,
    }))
}

fn fetch_latest(mode: Mode) -> Result<Option<Release>, String> {
    let gh: GhRelease = tools::agent()
        .get(API_LATEST)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .call()
        .map_err(|e| match e {
            ureq::Error::StatusCode(404) => "Todavía no hay ninguna versión publicada.".into(),
            ureq::Error::HostNotFound | ureq::Error::ConnectionFailed => "Sin conexión con GitHub.".into(),
            e => tools::net_err(e),
        })?
        .body_mut()
        .read_json()
        .map_err(|e| format!("Respuesta inesperada de GitHub: {e}"))?;
    release_for(gh, mode)
}

fn installer_dir() -> PathBuf {
    std::env::temp_dir().join("android-device-manager-update")
}

fn download_installer(rel: &Release, progress: impl FnMut(u64, Option<u64>)) -> Result<PathBuf, String> {
    let dir = installer_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("No se pudo crear {}: {e}", dir.display()))?;
    let dest = dir.join(&rel.asset_name);
    tools::download_to(&tools::agent(), &rel.asset_url, Some(rel.size), rel.sha256.as_deref(), &dest, progress)
        .inspect_err(|_| {
            let _ = fs::remove_file(&dest);
        })?;
    Ok(dest)
}

fn old_exe_path(exe: &Path) -> Option<PathBuf> {
    let stem = exe.file_stem()?.to_str()?;
    Some(exe.with_file_name(format!("{stem}.old.exe")))
}

/// Descarga el ejecutable junto al actual, lo verifica y lo pone en su lugar.
///
/// `Err((motivo, Some(ruta)))`: el nuevo quedó en `ruta` pero no se pudo reemplazar
/// el actual. `Err((motivo, None))`: no quedó nada utilizable.
fn replace_exe(rel: &Release, progress: impl FnMut(u64, Option<u64>)) -> Result<PathBuf, (String, Option<PathBuf>)> {
    let exe = std::env::current_exe().map_err(|e| (format!("No se encontró el ejecutable actual: {e}"), None))?;
    let dir = exe.parent().ok_or_else(|| ("El ejecutable no tiene carpeta.".to_string(), None))?;
    let stem = exe.file_stem().and_then(|s| s.to_str()).unwrap_or("android-device-manager");
    let part = dir.join(format!("{stem}.update.part"));

    if let Err(e) = tools::download_to(&tools::agent(), &rel.asset_url, Some(rel.size), rel.sha256.as_deref(), &part, progress) {
        let _ = fs::remove_file(&part);
        return Err((e, None));
    }

    let keep_aside = |reason: String| {
        let dest = dir.join(format!("{stem}-{}.exe", rel.version));
        match fs::rename(&part, &dest) {
            Ok(()) => (reason, Some(dest)),
            Err(_) => (reason, Some(part.clone())),
        }
    };
    let old = old_exe_path(&exe).ok_or_else(|| ("Nombre de ejecutable no válido.".to_string(), None))?;
    let _ = fs::remove_file(&old);
    if let Err(e) = fs::rename(&exe, &old) {
        return Err(keep_aside(format!("No se pudo apartar el ejecutable actual: {e}")));
    }
    if let Err(e) = fs::rename(&part, &exe) {
        let _ = fs::rename(&old, &exe);
        return Err(keep_aside(format!("No se pudo colocar el ejecutable nuevo: {e}")));
    }
    Ok(exe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions() {
        assert!(newer("v1.4.0", "1.3.1"));
        assert!(newer("1.10.0", "1.9.9"));
        assert!(newer("2", "1.99.99"));
        assert!(!newer("1.3.1", "1.3.1"));
        assert!(!newer("v1.3.0", "1.3.1"));
        assert!(!newer("1.4.0-beta", "1.4.0"));
        assert!(!newer("abc", "1.3.1"));
        assert!(!newer("1.4.0", "x"));
    }

    fn sample(tag: &str) -> GhRelease {
        serde_json::from_str(&format!(
            r#"{{
              "tag_name": "{tag}",
              "html_url": "https://github.com/iscalexsoto/android-device-manager/releases/tag/{tag}",
              "assets": [
                {{"name": "android-device-manager.exe", "browser_download_url": "https://x/portable", "size": 7983616,
                  "digest": "sha256:D59A0902529F270397069400744CB7F4F2BD15495E118AFB523B573DCAC82536"}},
                {{"name": "Android.Device.Manager_99.0.0_x64-setup.exe", "browser_download_url": "https://x/setup", "size": 2083778,
                  "digest": null}}
              ]
            }}"#
        ))
        .unwrap()
    }

    #[test]
    fn picks_asset_by_mode() {
        let rel = release_for(sample("v99.0.0"), Mode::Portable).unwrap().unwrap();
        assert_eq!(rel.version, "99.0.0");
        assert_eq!(rel.asset_url, "https://x/portable");
        assert_eq!(rel.sha256.as_deref(), Some("d59a0902529f270397069400744cb7f4f2bd15495e118afb523b573dcac82536"));
        assert!(rel.verified);

        let rel = release_for(sample("v99.0.0"), Mode::Installed).unwrap().unwrap();
        assert_eq!(rel.asset_url, "https://x/setup");
        assert!(!rel.verified);
    }

    #[test]
    fn same_or_older_is_up_to_date() {
        assert!(release_for(sample(&format!("v{CURRENT}")), Mode::Portable).unwrap().is_none());
        assert!(release_for(sample("v0.0.1"), Mode::Installed).unwrap().is_none());
    }

    #[test]
    fn newer_without_asset_is_an_error() {
        let mut gh = sample("v99.0.0");
        gh.assets.retain(|a| !a.name.ends_with("-setup.exe"));
        assert!(release_for(gh, Mode::Installed).is_err());
    }

    #[test]
    fn old_name() {
        let p = old_exe_path(Path::new(r"C:\Apps\android-device-manager.exe")).unwrap();
        assert_eq!(p, PathBuf::from(r"C:\Apps\android-device-manager.old.exe"));
    }

    /// Contra el repositorio real (red): `cargo test real_release -- --ignored --nocapture`.
    /// Descarga el instalador de la última release y comprueba su SHA-256.
    #[test]
    #[ignore]
    fn real_release() {
        let gh: GhRelease = tools::agent().get(API_LATEST).call().unwrap().body_mut().read_json().unwrap();
        println!("última release: {}", gh.tag_name);
        for mode in [Mode::Portable, Mode::Installed] {
            let a = pick_asset(&gh.assets, mode).expect("asset");
            println!("{mode:?}: {} ({} bytes, digest {:?})", a.name, a.size, a.digest);
        }
        // Se hace pasar por más nueva para obtener el Release con su SHA-256.
        let mut gh = gh;
        gh.tag_name = "v999.0.0".into();
        let rel = release_for(gh, Mode::Installed).unwrap().unwrap();
        assert!(rel.verified, "GitHub publica digest para los assets");
        let path = download_installer(&rel, |_, _| {}).expect("descarga y SHA-256");
        assert_eq!(fs::metadata(&path).unwrap().len(), rel.size);
        let _ = fs::remove_dir_all(installer_dir());
    }
}
