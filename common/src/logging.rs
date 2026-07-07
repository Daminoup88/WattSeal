#[cfg(not(debug_assertions))]
use std::{fs::OpenOptions, io::Write};

/// Log file path used by collector and main in release mode.
#[cfg(not(debug_assertions))]
pub const LOG_FILE: &str = "collector.log";

/// Session marker written once at the start of every init session.
#[cfg(not(debug_assertions))]
pub const SESSION_MARKER: &str = ">>> session start <<<";

/// Append a timestamped line to the log file.
#[cfg(not(debug_assertions))]
pub fn log_to_file(msg: &str) {
    let Ok(mut f) = OpenOptions::new().create(true).append(true).open(LOG_FILE) else {
        return;
    };
    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let _ = writeln!(f, "[{ts}] {msg}");
}

/// Trim `collector.log` so that only the last 2 sessions remain, then append the session marker.
#[cfg(not(debug_assertions))]
pub fn start_log_session() {
    use std::io::BufRead;
    if let Ok(f) = std::fs::File::open(LOG_FILE) {
        let lines: Vec<String> = std::io::BufReader::new(f).lines().flatten().collect();
        let starts: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.contains(SESSION_MARKER))
            .map(|(i, _)| i)
            .collect();
        if starts.len() >= 2 {
            let keep_from = starts[starts.len() - 2];
            let _ = std::fs::write(LOG_FILE, lines[keep_from..].join("\n") + "\n");
        }
    }
    log_to_file(SESSION_MARKER);
}

// Debug stubs to keep API available when building in debug
#[cfg(debug_assertions)]
pub fn log_to_file(_msg: &str) {}

#[cfg(debug_assertions)]
pub fn start_log_session() {}

#[cfg(not(debug_assertions))]
use std::{collections::HashMap, sync::Mutex};
#[cfg(not(debug_assertions))]
static ERROR_COUNTS: Mutex<Option<HashMap<&'static str, u32>>> = Mutex::new(None);
#[cfg(not(debug_assertions))]
static MAX_ERROR_LOGS: u32 = 3;

/// Log a runtime error for the given component, but only for the first `MAX_ERROR_LOGS` occurrences in release mode.
pub fn log_component_error(component: &'static str, msg: &str) {
    #[cfg(debug_assertions)]
    {
        eprintln!("✗ [{component}] {msg}");
    }

    #[cfg(not(debug_assertions))]
    {
        let mut guard = match ERROR_COUNTS.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let map = guard.get_or_insert_with(HashMap::new);
        let count = map.entry(component).or_insert(0);
        if *count < MAX_ERROR_LOGS {
            *count += 1;
            let remaining = MAX_ERROR_LOGS - *count;
            if remaining == 0 {
                crate::clog!("✗ [{component}] {msg} (further errors suppressed)");
            } else {
                crate::clog!("✗ [{component}] {msg} ({remaining} log(s) remaining)");
            }
        }
    }
}
