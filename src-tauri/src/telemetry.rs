use serde::Serialize;
use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

#[derive(Serialize)]
pub struct TelemetryEvent<'a> {
    pub ts: u64,
    pub kind: &'a str,
    pub data: serde_json::Value,
}

fn telemetry_path() -> PathBuf {
    let home = if cfg!(windows) {
        std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into())
    } else {
        std::env::var("HOME").unwrap_or_else(|_| ".".into())
    };
    PathBuf::from(home).join(".warp-terminal").join("telemetry.log")
}

pub fn record(kind: &str, data: serde_json::Value) {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let ev = TelemetryEvent { ts, kind, data };
    if let Ok(line) = serde_json::to_string(&ev) {
        if let Some(parent) = telemetry_path().parent() { let _ = fs::create_dir_all(parent); }
        let _ = fs::OpenOptions::new().create(true).append(true).open(telemetry_path()).and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{}", line)
        });
    }
}

pub fn install_panic_hook() {
    let path = telemetry_path();
    std::panic::set_hook(Box::new(move |info| {
        let msg = info.to_string();
        let data = serde_json::json!({"panic": msg});
        if let Some(parent) = path.parent() { let _ = fs::create_dir_all(parent); };
        let _ = fs::OpenOptions::new().create(true).append(true).open(&path).and_then(|mut f| {
            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let line = serde_json::json!({"ts": ts, "kind":"panic","data":data}).to_string();
            use std::io::Write; writeln!(f, "{}", line)
        });
    }));
}
