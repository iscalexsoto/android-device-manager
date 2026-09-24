//! Descarga de adb (Platform-Tools) y scrcpy desde sus fuentes oficiales a una
//! carpeta propia de la app, para equipos que no tienen ninguno de los dos.

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const PLATFORM_TOOLS_URL: &str = "https://dl.google.com/android/repository/platform-tools-latest-windows.zip";
const SCRCPY_RELEASE_URL: &str = "https://api.github.com/repos/Genymobile/scrcpy/releases/latest";
const USER_AGENT: &str = concat!("android-device-manager/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tool {
    Adb,
    Scrcpy,
}

impl Tool {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "adb" => Ok(Tool::Adb),
            "scrcpy" => Ok(Tool::Scrcpy),
            _ => Err(format!("Herramienta desconocida: {s}")),
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Tool::Adb => "adb",
            Tool::Scrcpy => "scrcpy",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Tool::Adb => "adb (Platform-Tools)",
            Tool::Scrcpy => "scrcpy",
        }
    }

    fn folder(self) -> &'static str {
        match self {
            Tool::Adb => "platform-tools",
            Tool::Scrcpy => "scrcpy",
        }
    }

    fn exe(self) -> &'static str {
        match self {
            Tool::Adb => "adb.exe",
            Tool::Scrcpy => "scrcpy.exe",
        }
    }
}

// ---------------------------------------------------------------------------
// Ubicación
// ---------------------------------------------------------------------------

/// %LOCALAPPDATA%\com.soto.androiddevicemanager\tools
pub fn root() -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(local).join("com.soto.androiddevicemanager").join("tools"))
}

fn installed(tool: Tool) -> Option<PathBuf> {
    let p = root()?.join(tool.folder()).join(tool.exe());
    p.is_file().then_some(p)
}

pub fn adb_exe() -> Option<PathBuf> {
    installed(Tool::Adb)
}

pub fn scrcpy_exe() -> Option<PathBuf> {
    installed(Tool::Scrcpy)
}

/// ¿El binario está dentro de la carpeta de herramientas descargadas?
pub fn is_downloaded(path: &Path) -> bool {
    root()
        .map(|r| crate::adb::normalize(path).starts_with(&crate::adb::normalize(&r)))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Instalación
// ---------------------------------------------------------------------------

pub enum Step {
    /// Buscando la versión más reciente.
    Resolve,
    Download { received: u64, total: Option<u64> },
    Extract,
}

pub struct Installed {
    pub exe: PathBuf,
    pub version: String,
    pub url: String,
    pub bytes: u64,
}

struct Source {
    url: String,
    file: String,
    version: String,
    size: Option<u64>,
    sha256: Option<String>,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    size: u64,
    /// "sha256:<hex>" (GitHub lo publica para los archivos de un release).
    digest: Option<String>,
}

fn agent() -> ureq::Agent {
    let tls = ureq::tls::TlsConfig::builder()
        .provider(ureq::tls::TlsProvider::NativeTls)
        .root_certs(ureq::tls::RootCerts::PlatformVerifier)
        .build();
    ureq::Agent::config_builder()
        .tls_config(tls)
        .user_agent(USER_AGENT)
        .timeout_connect(Some(Duration::from_secs(15)))
        .timeout_recv_response(Some(Duration::from_secs(30)))
        .build()
        .into()
}

fn net_err(e: ureq::Error) -> String {
    match e {
        ureq::Error::StatusCode(403 | 429) => "GitHub limitó las consultas desde esta red. Intenta de nuevo en unos minutos.".into(),
        ureq::Error::StatusCode(code) => format!("El servidor respondió con el código {code}"),
        ureq::Error::Io(e) => format!("Error de red: {e}"),
        e => format!("No se pudo conectar: {e}"),
    }
}

fn resolve(agent: &ureq::Agent, tool: Tool) -> Result<Source, String> {
    match tool {
        Tool::Adb => Ok(Source {
            url: PLATFORM_TOOLS_URL.into(),
            file: "platform-tools-latest-windows.zip".into(),
            version: "más reciente".into(),
            size: None,
            sha256: None,
        }),
        Tool::Scrcpy => {
            let release: Release = agent
                .get(SCRCPY_RELEASE_URL)
                .header("Accept", "application/vnd.github+json")
                .call()
                .map_err(net_err)?
                .body_mut()
                .read_json()
                .map_err(|e| format!("Respuesta inesperada de GitHub: {e}"))?;
            let asset = release
                .assets
                .into_iter()
                .find(|a| a.name.starts_with("scrcpy-win64-") && a.name.ends_with(".zip"))
                .ok_or_else(|| format!("El release {} de scrcpy no incluye la versión para Windows de 64 bits", release.tag_name))?;
            Ok(Source {
                url: asset.browser_download_url,
                file: asset.name,
                version: release.tag_name,
                size: Some(asset.size),
                sha256: asset.digest.and_then(|d| d.strip_prefix("sha256:").map(str::to_lowercase)),
            })
        }
    }
}

/// Descarga y descomprime la herramienta en `root()\<carpeta>`, reemplazando una
/// copia anterior. Informa el avance con `step`.
pub fn install(tool: Tool, step: impl FnMut(Step)) -> Result<Installed, String> {
    let root = root().ok_or("No se encontró la carpeta LOCALAPPDATA")?;
    install_into(&root, tool, step)
}

fn install_into(root: &Path, tool: Tool, mut step: impl FnMut(Step)) -> Result<Installed, String> {
    let tmp = root.join(".tmp");
    fs::create_dir_all(&tmp).map_err(|e| format!("No se pudo crear {}: {e}", tmp.display()))?;

    let agent = agent();
    step(Step::Resolve);
    let src = resolve(&agent, tool)?;

    let zip_path = tmp.join(&src.file);
    let res = download(&agent, &src, &zip_path, &mut step).and_then(|bytes| {
        step(Step::Extract);
        let staging = tmp.join(tool.folder());
        extract(&zip_path, &staging)?;
        if !staging.join(tool.exe()).is_file() {
            return Err(format!("El archivo descargado no contiene {}", tool.exe()));
        }
        replace_dir(&staging, &root.join(tool.folder()))?;
        Ok(bytes)
    });
    let _ = fs::remove_dir_all(&tmp);
    let bytes = res?;

    Ok(Installed {
        exe: root.join(tool.folder()).join(tool.exe()),
        version: src.version,
        url: src.url,
        bytes,
    })
}

fn download(agent: &ureq::Agent, src: &Source, dest: &Path, step: &mut impl FnMut(Step)) -> Result<u64, String> {
    let mut resp = agent.get(&src.url).call().map_err(net_err)?;
    let total = resp.body().content_length().or(src.size);
    let mut reader = resp.body_mut().as_reader();
    let mut out = BufWriter::new(File::create(dest).map_err(|e| e.to_string())?);
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut received = 0u64;
    let mut last = Instant::now();
    step(Step::Download { received, total });
    loop {
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(format!("Se interrumpió la descarga: {e}")),
        };
        out.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        hasher.update(&buf[..n]);
        received += n as u64;
        if last.elapsed() >= Duration::from_millis(150) {
            last = Instant::now();
            step(Step::Download { received, total });
        }
    }
    out.flush().map_err(|e| e.to_string())?;
    step(Step::Download { received, total });

    if let Some(t) = total
        && received != t
    {
        return Err(format!("Descarga incompleta: {received} de {t} bytes"));
    }
    if let Some(expected) = &src.sha256 {
        let actual: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if &actual != expected {
            return Err("La suma SHA-256 del archivo descargado no coincide con la publicada. Se descartó.".into());
        }
    }
    Ok(received)
}

/// Descomprime el zip en `dest`, quitando la carpeta raíz común
/// (`platform-tools\`, `scrcpy-win64-v3.x\`).
fn extract(zip_path: &Path, dest: &Path) -> Result<(), String> {
    let bad = |e: zip::result::ZipError| format!("El archivo descargado está dañado: {e}");
    let mut zip = zip::ZipArchive::new(File::open(zip_path).map_err(|e| e.to_string())?).map_err(bad)?;

    let mut names = Vec::with_capacity(zip.len());
    for i in 0..zip.len() {
        // enclosed_name descarta rutas absolutas o con `..`.
        names.push(zip.by_index(i).map_err(bad)?.enclosed_name());
    }
    let first = |p: &PathBuf| p.components().next().map(|c| c.as_os_str().to_owned());
    let top = names.iter().flatten().next().and_then(first);
    let strip = top.filter(|t| {
        let all = names.iter().flatten();
        all.clone().all(|p| first(p).as_ref() == Some(t)) && all.clone().any(|p| p.components().count() > 1)
    });

    let _ = fs::remove_dir_all(dest);
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for (i, name) in names.into_iter().enumerate() {
        let Some(name) = name else { continue };
        let rel = match &strip {
            Some(t) => name.strip_prefix(t).map(Path::to_path_buf).unwrap_or(name),
            None => name,
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        let target = dest.join(&rel);
        let mut entry = zip.by_index(i).map_err(bad)?;
        if entry.is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut f = File::create(&target).map_err(|e| format!("No se pudo escribir {}: {e}", target.display()))?;
        io::copy(&mut entry, &mut f).map_err(|e| format!("No se pudo descomprimir {}: {e}", rel.display()))?;
    }
    Ok(())
}

fn replace_dir(staging: &Path, target: &Path) -> Result<(), String> {
    if target.exists() {
        fs::remove_dir_all(target).map_err(|e| {
            format!(
                "No se pudo reemplazar {}: {e}. Cierra los programas que la estén usando (por ejemplo, un servidor adb) e intenta de nuevo.",
                target.display()
            )
        })?;
    }
    fs::rename(staging, target).map_err(|e| format!("No se pudo mover a {}: {e}", target.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Descarga real; se ejecuta a mano con
    /// `TOOLS_TEST_DIR=<carpeta> cargo test downloads -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn downloads() {
        let dir = PathBuf::from(std::env::var("TOOLS_TEST_DIR").expect("TOOLS_TEST_DIR"));
        for tool in [Tool::Adb, Tool::Scrcpy] {
            let mut last = 0;
            let done = install_into(&dir, tool, |s| {
                if let Step::Download { received, total } = s
                    && received - last > 4 << 20
                {
                    last = received;
                    println!("{} {received}/{total:?}", tool.key());
                }
            })
            .unwrap();
            println!("{} {} {} bytes -> {}", tool.key(), done.version, done.bytes, done.exe.display());
            assert!(done.exe.is_file());
            // Una segunda instalación reemplaza la carpeta anterior.
            install_into(&dir, tool, |_| {}).unwrap();
            assert!(done.exe.is_file());
        }
        assert!(!dir.join(".tmp").exists());
    }
}
