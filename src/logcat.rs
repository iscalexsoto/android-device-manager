//! Logcat: argumentos, validación de filtros y lectura del formato `threadtime`.

use crate::settings::LogcatOptions;
use serde::Serialize;
use std::collections::HashSet;

/// Líneas previas que se piden con el historial "recent".
pub const RECENT_LINES: u32 = 500;

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    /// HH:MM:SS.mmm
    pub time: String,
    pub pid: Option<u32>,
    pub tid: Option<u32>,
    /// V D I W E F, o vacío si la línea no tiene el formato `threadtime`.
    pub level: String,
    pub tag: String,
    pub message: String,
    /// Línea tal cual la imprimió logcat; es lo que se exporta.
    #[serde(skip)]
    pub raw: String,
}

/// Lee una línea de `logcat -v threadtime`:
/// `MM-DD HH:MM:SS.mmm  PID  TID L Tag     : mensaje`
pub fn parse_line(raw: &str) -> LogLine {
    let raw = raw.trim_end_matches(['\r', '\n']);
    parse_threadtime(raw).unwrap_or_else(|| LogLine {
        time: String::new(),
        pid: None,
        tid: None,
        level: String::new(),
        tag: String::new(),
        message: raw.to_string(),
        raw: raw.to_string(),
    })
}

fn parse_threadtime(raw: &str) -> Option<LogLine> {
    let mut rest = raw;
    let date = next_token(&mut rest)?;
    let time = next_token(&mut rest)?;
    let pid: u32 = next_token(&mut rest)?.parse().ok()?;
    let tid: u32 = next_token(&mut rest)?.parse().ok()?;
    let level = next_token(&mut rest)?;
    let date_ok = date.len() == 5 && date.as_bytes()[2] == b'-';
    let level_ok = level.len() == 1 && "VDIWEFA".contains(level);
    if !date_ok || !time.contains(':') || !level_ok {
        return None;
    }
    let rest = rest.trim_start();
    let (tag, message) = match rest.find(": ") {
        Some(i) => (rest[..i].trim_end(), &rest[i + 2..]),
        None => match rest.strip_suffix(':') {
            Some(tag) => (tag.trim_end(), ""),
            None => ("", rest),
        },
    };
    Some(LogLine {
        time: time.to_string(),
        pid: Some(pid),
        tid: Some(tid),
        level: if level == "A" { "F".into() } else { level.to_string() },
        tag: tag.to_string(),
        message: message.to_string(),
        raw: raw.to_string(),
    })
}

fn next_token<'a>(s: &mut &'a str) -> Option<&'a str> {
    let t = s.trim_start();
    if t.is_empty() {
        return None;
    }
    let end = t.find(char::is_whitespace).unwrap_or(t.len());
    let (token, rest) = t.split_at(end);
    *s = rest;
    Some(token)
}

/// Orden de severidad; las líneas sin nivel cuentan como verbose.
pub fn level_rank(level: &str) -> u8 {
    match level {
        "D" => 1,
        "I" => 2,
        "W" => 3,
        "E" => 4,
        "F" | "A" => 5,
        _ => 0,
    }
}

/// Texto en el que busca el filtro de vista. La interfaz arma exactamente el
/// mismo (`searchText` en app.js) para que lo exportado coincida con lo visible.
pub fn search_text(line: &LogLine) -> String {
    let num = |n: Option<u32>| n.map(|n| n.to_string()).unwrap_or_default();
    format!(
        "{} {} {} {} {}: {}",
        line.time,
        num(line.pid),
        num(line.tid),
        line.level,
        line.tag,
        line.message
    )
    .to_lowercase()
}

/// Filtro de vista: nivel mínimo y texto (sin distinguir mayúsculas).
/// `query` debe venir ya en minúsculas.
pub fn matches(line: &LogLine, query: &str, min_rank: u8) -> bool {
    level_rank(&line.level) >= min_rank && (query.is_empty() || search_text(line).contains(query))
}

/// Valida filterspecs como los del CLI (`ActivityManager:I MiTag:D *:S`).
pub fn validate_spec(spec: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for token in spec.split_whitespace() {
        let Some((tag, level)) = token.rsplit_once(':') else {
            return Err(format!("«{token}» no es un filtro válido. Usa Tag:Nivel, por ejemplo ActivityManager:I"));
        };
        let level = level.to_ascii_uppercase();
        if level.len() != 1 || !"VDIWEFS".contains(&level) {
            return Err(format!("Nivel desconocido en «{token}». Usa V, D, I, W, E, F o S"));
        }
        let bad = |c: char| c.is_control() || c.is_whitespace() || "'\"`$;&|<>\\(){}".contains(c);
        if tag.is_empty() || tag.chars().any(bad) {
            return Err(format!("El tag de «{token}» no es válido"));
        }
        out.push(format!("{tag}:{level}"));
    }
    Ok(out)
}

pub fn valid_package(package: &str) -> bool {
    !package.is_empty()
        && package.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        && !package.starts_with('.')
}

/// Argumentos de adb para la captura (sin el filtro por app, que se aplica aquí).
pub fn build_args(serial: &str, options: &LogcatOptions) -> Result<Vec<String>, String> {
    let mut args: Vec<String> = ["-s", serial, "logcat", "-v", "threadtime"].map(String::from).to_vec();
    match options.buffer.as_str() {
        "all" => args.extend(["-b".into(), "all".into()]),
        "crash" => args.extend(["-b".into(), "crash".into()]),
        _ => {}
    }
    match options.history.as_str() {
        "all" => {}
        // -T imprime las últimas N líneas y sigue; con 1 casi no hay historial.
        "none" => args.extend(["-T".into(), "1".into()]),
        _ => args.extend(["-T".into(), RECENT_LINES.to_string()]),
    }
    args.extend(validate_spec(&options.spec)?);
    Ok(args)
}

/// PIDs de un paquete (y de sus procesos `paquete:algo`) a partir de `ps`.
/// Sirve tanto para toybox (`ps -A -o PID,NAME`) como para el `ps` antiguo,
/// donde el PID es la primera columna numérica y el nombre la última.
pub fn parse_pids(ps_output: &str, package: &str) -> HashSet<u32> {
    ps_output
        .lines()
        .filter_map(|line| {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let name = *tokens.last()?;
            if !is_package_process(name, package) {
                return None;
            }
            tokens.iter().find_map(|t| t.parse::<u32>().ok())
        })
        .collect()
}

fn is_package_process(name: &str, package: &str) -> bool {
    name == package || name.strip_prefix(package).is_some_and(|r| r.starts_with(':'))
}

/// ActivityManager anuncia cada proceso nuevo:
/// `Start proc 12345:com.example.app/u0a123 for activity {…}`.
/// Así se sigue a la app cuando se reinicia, sin esperar al siguiente `ps`.
pub fn started_pid(message: &str, package: &str) -> Option<u32> {
    let rest = &message[message.find("Start proc ")? + "Start proc ".len()..];
    let (pid, rest) = rest.split_once(':')?;
    let name = rest.split(['/', ' ']).next()?;
    is_package_process(name, package).then(|| pid.trim().parse().ok()).flatten()
}

/// Paquetes de `pm list packages`, ordenados.
pub fn parse_packages(text: &str) -> Vec<String> {
    let mut v: Vec<String> = text
        .lines()
        .filter_map(|l| l.trim().strip_prefix("package:"))
        .map(str::to_string)
        .filter(|p| valid_package(p))
        .collect();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(spec: &str, buffer: &str, history: &str) -> LogcatOptions {
        LogcatOptions { spec: spec.into(), buffer: buffer.into(), history: history.into(), ..Default::default() }
    }

    #[test]
    fn parses_threadtime() {
        let l = parse_line("10-18 12:34:56.789  1234  1250 D ActivityManager: Start proc 99:com.x/u0a1\r\n");
        assert_eq!(l.time, "12:34:56.789");
        assert_eq!((l.pid, l.tid), (Some(1234), Some(1250)));
        assert_eq!(l.level, "D");
        assert_eq!(l.tag, "ActivityManager");
        assert_eq!(l.message, "Start proc 99:com.x/u0a1");
        assert!(!l.raw.ends_with('\n'));

        let l = parse_line("10-18 12:34:56.789   612   612 I Zygote  : Process 99 exited: a: b");
        assert_eq!(l.tag, "Zygote");
        assert_eq!(l.message, "Process 99 exited: a: b");

        let l = parse_line("10-18 12:34:56.789   612   613 E MyTag:");
        assert_eq!((l.tag.as_str(), l.message.as_str()), ("MyTag", ""));

        let l = parse_line("10-18 12:34:56.789   612   613 A DEBUG   : abort");
        assert_eq!(l.level, "F");
    }

    #[test]
    fn keeps_unformatted_lines() {
        let l = parse_line("--------- beginning of main");
        assert_eq!(l.level, "");
        assert_eq!(l.pid, None);
        assert_eq!(l.message, "--------- beginning of main");
    }

    #[test]
    fn filters_by_level_and_text() {
        let w = parse_line("10-18 12:34:56.789  1234  1250 W Net: Timeout talking to Server");
        assert!(matches(&w, "", level_rank("W")));
        assert!(!matches(&w, "", level_rank("E")));
        assert!(matches(&w, "server", 0));
        assert!(matches(&w, "1234", 0));
        assert!(!matches(&w, "client", 0));
    }

    #[test]
    fn validates_specs() {
        assert_eq!(validate_spec("  ActivityManager:i  MyTag:D *:s ").unwrap(), ["ActivityManager:I", "MyTag:D", "*:S"]);
        assert!(validate_spec("").unwrap().is_empty());
        assert!(validate_spec("MyTag").is_err());
        assert!(validate_spec("MyTag:X").is_err());
        assert!(validate_spec(":D").is_err());
        assert!(validate_spec("a;rm:D").is_err());
    }

    #[test]
    fn builds_args() {
        let a = build_args("R58", &opts("MyTag:D *:S", "crash", "none")).unwrap();
        assert_eq!(a, ["-s", "R58", "logcat", "-v", "threadtime", "-b", "crash", "-T", "1", "MyTag:D", "*:S"]);
        let a = build_args("R58", &opts("", "default", "recent")).unwrap();
        assert_eq!(a, ["-s", "R58", "logcat", "-v", "threadtime", "-T", "500"]);
        let a = build_args("R58", &opts("", "all", "all")).unwrap();
        assert_eq!(a, ["-s", "R58", "logcat", "-v", "threadtime", "-b", "all"]);
        assert!(build_args("R58", &opts("bad", "", "")).is_err());
    }

    #[test]
    fn finds_package_pids() {
        let toybox = "  PID NAME\n    1 init\n 4321 com.example.app\n 4400 com.example.app:remote\n 4500 com.example.application\n";
        let pids = parse_pids(toybox, "com.example.app");
        assert_eq!(pids, HashSet::from([4321, 4400]));

        let old = "USER     PID   PPID  VSIZE  RSS     WCHAN    PC         NAME\nu0_a12    2001  180   1000   200   ffffffff 00000000 S com.example.app\n";
        assert_eq!(parse_pids(old, "com.example.app"), HashSet::from([2001]));
        assert!(parse_pids(toybox, "com.other").is_empty());
    }

    #[test]
    fn detects_process_start() {
        let m = "Start proc 12345:com.example.app/u0a123 for activity {com.example.app/.Main}";
        assert_eq!(started_pid(m, "com.example.app"), Some(12345));
        assert_eq!(started_pid("Start proc 77:com.example.app:sync/u0a123 for service", "com.example.app"), Some(77));
        assert_eq!(started_pid(m, "com.example"), None);
        assert_eq!(started_pid("Displayed com.example.app/.Main", "com.example.app"), None);
    }

    #[test]
    fn validates_packages() {
        assert!(valid_package("com.example.app_2"));
        assert!(!valid_package(""));
        assert!(!valid_package("com.x; reboot"));
        assert_eq!(parse_packages("package:com.b\npackage:com.a\r\ngarbage\n"), ["com.a", "com.b"]);
    }
}
