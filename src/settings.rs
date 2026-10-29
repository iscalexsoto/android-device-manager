//! Preferencias persistidas en %APPDATA%\com.soto.androiddevicemanager\settings.json

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const DEFAULT_ADB_PORT: u16 = 5037;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ScrcpyOptions {
    /// 0 = resolución original.
    pub max_size: u32,
    /// Mbps.
    pub bit_rate: u32,
    /// 0 = sin límite.
    pub max_fps: u32,
    pub stay_awake: bool,
    pub turn_screen_off: bool,
    pub show_touches: bool,
    pub always_on_top: bool,
    pub no_audio: bool,
    pub fullscreen: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RecordOptions {
    /// Audio interno del teléfono (Android 11+).
    pub audio: bool,
    /// mp4 | mkv
    pub format: String,
    /// 0 = resolución original.
    pub max_size: u32,
    /// Mbps.
    pub bit_rate: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct LogcatOptions {
    /// Paquete de la app a seguir; vacío = todo el dispositivo.
    pub package: String,
    /// Filterspecs como en el CLI, p. ej. "ActivityManager:I MiTag:D *:S".
    pub spec: String,
    /// default (main, system, crash) | all | crash
    pub buffer: String,
    /// recent (últimas líneas) | all (todo el búfer) | none (solo lo nuevo)
    pub history: String,
    /// Nivel mínimo que se muestra: V D I W E F. Es filtro de vista, no reinicia la captura.
    pub min_level: String,
}

impl Default for LogcatOptions {
    fn default() -> Self {
        Self {
            package: String::new(),
            spec: String::new(),
            buffer: "default".into(),
            history: "recent".into(),
            min_level: "V".into(),
        }
    }
}

impl Default for RecordOptions {
    fn default() -> Self {
        Self { audio: true, format: "mp4".into(), max_size: 0, bit_rate: 8 }
    }
}

impl Default for ScrcpyOptions {
    fn default() -> Self {
        Self {
            max_size: 1920,
            bit_rate: 8,
            max_fps: 60,
            stay_awake: true,
            turn_screen_off: false,
            show_touches: false,
            always_on_top: false,
            no_audio: true,
            fullscreen: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// Vacío = detección automática (SDK de Android, el mismo que usa Android Studio).
    pub adb_path: String,
    /// Vacío = detección automática.
    pub scrcpy_path: String,
    /// 5037 = compartido con Android Studio; cualquier otro = servidor aislado.
    pub server_port: u16,
    pub auto_start_server: bool,
    pub stop_isolated_on_exit: bool,
    /// Capturas de pantalla. Vacío = Imágenes\Android Device Manager.
    pub capture_dir: String,
    /// Grabaciones de video. Vacío = Videos\Android Device Manager.
    pub video_dir: String,
    /// system | light | dark
    pub theme: String,
    pub scrcpy: ScrcpyOptions,
    pub record: RecordOptions,
    pub logcat: LogcatOptions,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            adb_path: String::new(),
            scrcpy_path: String::new(),
            server_port: DEFAULT_ADB_PORT,
            auto_start_server: true,
            stop_isolated_on_exit: true,
            capture_dir: String::new(),
            video_dir: String::new(),
            theme: "system".into(),
            scrcpy: ScrcpyOptions::default(),
            record: RecordOptions::default(),
            logcat: LogcatOptions::default(),
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    pub fn is_isolated(&self) -> bool {
        self.server_port != DEFAULT_ADB_PORT
    }

    pub fn capture_dir(&self) -> PathBuf {
        dir_or_default(&self.capture_dir, "Pictures")
    }

    pub fn video_dir(&self) -> PathBuf {
        dir_or_default(&self.video_dir, "Videos")
    }
}

/// Carpeta elegida o, si está vacía, `%USERPROFILE%\<library>\Android Device Manager`.
fn dir_or_default(custom: &str, library: &str) -> PathBuf {
    if !custom.trim().is_empty() {
        return PathBuf::from(custom.trim());
    }
    let base = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(library).join("Android Device Manager")
}
