//! Localización de scrcpy, armado de argumentos y control de su ventana.

use crate::adb::{dedup_paths, find_in_path};
use crate::settings::{RecordOptions, ScrcpyOptions};
use std::path::{Path, PathBuf};

pub fn candidates() -> Vec<PathBuf> {
    let mut list = find_in_path("scrcpy.exe");
    let env = |k: &str| std::env::var_os(k).map(PathBuf::from);

    if let Some(local) = env("LOCALAPPDATA") {
        let winget = local.join("Microsoft").join("WinGet");
        list.push(winget.join("Links").join("scrcpy.exe"));
        if let Ok(entries) = std::fs::read_dir(winget.join("Packages")) {
            for pkg in entries.flatten() {
                if !pkg.file_name().to_string_lossy().starts_with("Genymobile.scrcpy") {
                    continue;
                }
                list.push(pkg.path().join("scrcpy.exe"));
                if let Ok(sub) = std::fs::read_dir(pkg.path()) {
                    list.extend(sub.flatten().map(|e| e.path().join("scrcpy.exe")));
                }
            }
        }
    }
    if let Some(home) = env("USERPROFILE") {
        list.push(home.join("scoop").join("apps").join("scrcpy").join("current").join("scrcpy.exe"));
    }
    if let Some(pd) = env("ProgramData") {
        list.push(pd.join("chocolatey").join("bin").join("scrcpy.exe"));
    }
    if let Some(pf) = env("ProgramFiles") {
        list.push(pf.join("scrcpy").join("scrcpy.exe"));
    }
    list.extend(crate::tools::scrcpy_exe());

    // Resolver enlaces (WinGet\Links) para llegar a la carpeta real.
    let list = list
        .into_iter()
        .filter(|p| p.is_file())
        .map(|p| std::fs::canonicalize(&p).map(strip_unc).unwrap_or(p))
        .collect();
    dedup_paths(list)
}

fn strip_unc(p: PathBuf) -> PathBuf {
    let s = p.display().to_string();
    match s.strip_prefix(r"\\?\") {
        Some(rest) => PathBuf::from(rest),
        None => p,
    }
}

pub fn auto_scrcpy() -> Option<PathBuf> {
    candidates().into_iter().next()
}

/// adb que viene empaquetado junto a scrcpy (fuente clásica de conflictos).
pub fn bundled_adb(scrcpy: &Path) -> Option<PathBuf> {
    let p = scrcpy.parent()?.join("adb.exe");
    p.is_file().then_some(p)
}

pub fn build_args(serial: &str, title: &str, o: &ScrcpyOptions) -> Vec<String> {
    let mut a = vec![format!("--serial={serial}"), format!("--window-title={title}")];
    if o.max_size > 0 {
        a.push(format!("--max-size={}", o.max_size));
    }
    if o.bit_rate > 0 {
        a.push(format!("--video-bit-rate={}M", o.bit_rate));
    }
    if o.max_fps > 0 {
        a.push(format!("--max-fps={}", o.max_fps));
    }
    let flags = [
        (o.stay_awake, "--stay-awake"),
        (o.turn_screen_off, "--turn-screen-off"),
        (o.show_touches, "--show-touches"),
        (o.always_on_top, "--always-on-top"),
        (o.no_audio, "--no-audio"),
        (o.fullscreen, "--fullscreen"),
    ];
    a.extend(flags.iter().filter(|(on, _)| *on).map(|(_, f)| f.to_string()));
    a
}

/// Grabación en segundo plano: sin ventana, sin control y sin reproducir en el PC.
pub fn record_args(serial: &str, o: &RecordOptions, path: &Path) -> Vec<String> {
    let mut a = vec![
        format!("--serial={serial}"),
        "--no-window".into(),
        "--no-playback".into(),
        "--no-control".into(),
        format!("--record={}", path.display()),
    ];
    if o.max_size > 0 {
        a.push(format!("--max-size={}", o.max_size));
    }
    if o.bit_rate > 0 {
        a.push(format!("--video-bit-rate={}M", o.bit_rate));
    }
    if o.audio {
        // AAC se reproduce en cualquier visor; Opus en MP4 no siempre.
        a.push("--audio-codec=aac".into());
    } else {
        a.push("--no-audio".into());
    }
    a
}

pub fn record_extension(o: &RecordOptions) -> &'static str {
    if o.format == "mkv" { "mkv" } else { "mp4" }
}

/// Resumen corto para mostrar en la lista de sesiones.
pub fn summary(o: &ScrcpyOptions) -> String {
    let mut parts = vec![
        if o.max_size > 0 { format!("{} px", o.max_size) } else { "Original".into() },
        format!("{} Mbps", o.bit_rate),
    ];
    if o.max_fps > 0 {
        parts.push(format!("{} FPS", o.max_fps));
    }
    if o.turn_screen_off {
        parts.push("Pantalla apagada".into());
    }
    parts.join(" · ")
}

// ---------------------------------------------------------------------------
// Ventanas de scrcpy (cierre ordenado y traer al frente)
// ---------------------------------------------------------------------------

#[cfg(windows)]
mod win {
    use windows_sys::Win32::Foundation::{HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsIconic, IsWindowVisible, PostMessageW,
        SW_RESTORE, SetForegroundWindow, ShowWindow, WM_CLOSE,
    };
    use windows_sys::core::BOOL;

    struct Search {
        pid: u32,
        found: Vec<HWND>,
    }

    unsafe extern "system" fn enum_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            let search = &mut *(lparam as *mut Search);
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == search.pid && IsWindowVisible(hwnd) != 0 {
                search.found.push(hwnd);
            }
        }
        1
    }

    fn windows_of(pid: u32) -> Vec<HWND> {
        let mut s = Search { pid, found: Vec::new() };
        unsafe {
            EnumWindows(Some(enum_cb), &mut s as *mut Search as LPARAM);
        }
        s.found
    }

    pub fn close(pid: u32) -> bool {
        let wins = windows_of(pid);
        for w in &wins {
            unsafe {
                PostMessageW(*w, WM_CLOSE, 0, 0);
            }
        }
        !wins.is_empty()
    }

    pub fn focus(pid: u32) -> bool {
        let wins = windows_of(pid);
        if let Some(w) = wins.first() {
            unsafe {
                if IsIconic(*w) != 0 {
                    ShowWindow(*w, SW_RESTORE);
                }
                SetForegroundWindow(*w);
            }
            true
        } else {
            false
        }
    }
}

/// Ctrl+C a un proceso de consola sin ventana (así scrcpy cierra el MP4 correctamente).
#[cfg(windows)]
mod ctrl {
    use std::sync::Mutex;
    use std::time::Duration;
    use windows_sys::Win32::System::Console::{
        AttachConsole, CTRL_C_EVENT, FreeConsole, GenerateConsoleCtrlEvent, SetConsoleCtrlHandler,
    };
    use windows_sys::core::BOOL;

    static LOCK: Mutex<()> = Mutex::new(());

    // Esta app también queda unida a la consola por un instante y recibe el
    // evento; un manejador propio lo absorbe. A diferencia de
    // SetConsoleCtrlHandler(NULL, TRUE), no lo heredan los procesos hijos.
    unsafe extern "system" fn swallow(_ctrl: u32) -> BOOL {
        1
    }

    pub fn interrupt(pid: u32) -> bool {
        let _guard = LOCK.lock().unwrap();
        unsafe {
            FreeConsole();
            if AttachConsole(pid) == 0 {
                return false;
            }
            // Debe registrarse DESPUÉS de AttachConsole: al unirse a una consola
            // se reinician los manejadores y el evento cerraría esta app.
            SetConsoleCtrlHandler(Some(swallow), 1);
            let ok = GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0) != 0;
            // El evento llega en un hilo aparte; se espera a que se absorba
            // antes de soltar la consola.
            std::thread::sleep(Duration::from_millis(300));
            FreeConsole();
            ok
        }
    }
}

#[cfg(windows)]
pub use ctrl::interrupt;
#[cfg(windows)]
pub use win::{close as close_window, focus as focus_window};

#[cfg(not(windows))]
pub fn interrupt(_pid: u32) -> bool {
    false
}
#[cfg(not(windows))]
pub fn close_window(_pid: u32) -> bool {
    false
}
#[cfg(not(windows))]
pub fn focus_window(_pid: u32) -> bool {
    false
}
