//! Ejecución de procesos externos sin consola visible y con tiempo límite.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Evita que cada invocación de adb abra una ventana de consola.
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub struct Output {
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: String,
    pub timed_out: bool,
    pub elapsed: Duration,
}

impl Output {
    pub fn ok(&self) -> bool {
        self.code == Some(0) && !self.timed_out
    }

    pub fn stdout_text(&self) -> String {
        String::from_utf8_lossy(&self.stdout).replace("\r\n", "\n")
    }

    /// Texto combinado útil para mostrar al usuario.
    pub fn combined(&self) -> String {
        let mut s = self.stdout_text();
        let err = self.stderr.trim();
        if !err.is_empty() {
            if !s.is_empty() && !s.ends_with('\n') {
                s.push('\n');
            }
            s.push_str(err);
        }
        s.trim_end().to_string()
    }
}

pub fn command(exe: &Path) -> Command {
    let mut cmd = Command::new(exe);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// Ejecuta y espera. Los lectores de stdout/stderr van en hilos propios y se
/// abandonan tras el cierre del proceso si algún hijo (p. ej. el daemon que
/// levanta `adb start-server`) heredó los pipes y los mantiene abiertos.
pub fn run(
    exe: &Path,
    args: &[String],
    envs: &[(String, String)],
    timeout: Duration,
    capture: bool,
) -> std::io::Result<Output> {
    let start = Instant::now();
    let mut cmd = command(exe);
    cmd.args(args).stdin(Stdio::null());
    for (k, v) in envs {
        cmd.env(k, v);
    }
    if capture {
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    } else {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }
    let mut child = cmd.spawn()?;

    let out_rx = child.stdout.take().map(spawn_reader);
    let err_rx = child.stderr.take().map(spawn_reader);

    let mut timed_out = false;
    let status = loop {
        match child.try_wait()? {
            Some(s) => break Some(s),
            None => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    timed_out = true;
                    break child.wait().ok();
                }
                thread::sleep(Duration::from_millis(15));
            }
        }
    };

    let grace = Duration::from_millis(800);
    let stdout = out_rx
        .and_then(|rx| rx.recv_timeout(grace).ok())
        .unwrap_or_default();
    let stderr = err_rx
        .and_then(|rx| rx.recv_timeout(grace).ok())
        .map(|b| String::from_utf8_lossy(&b).replace("\r\n", "\n"))
        .unwrap_or_default();

    Ok(Output {
        code: status.and_then(|s| s.code()),
        stdout,
        stderr,
        timed_out,
        elapsed: start.elapsed(),
    })
}

fn spawn_reader<R: Read + Send + 'static>(mut r: R) -> mpsc::Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = r.read_to_end(&mut buf);
        let _ = tx.send(buf);
    });
    rx
}

/// Da formato de línea de comandos para el registro.
pub fn display_command(exe: &Path, args: &[String]) -> String {
    let name = exe
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| exe.display().to_string());
    let mut s = name;
    for a in args {
        s.push(' ');
        if a.contains(' ') || a.is_empty() {
            s.push('"');
            s.push_str(a);
            s.push('"');
        } else {
            s.push_str(a);
        }
    }
    s
}
