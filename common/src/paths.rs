//! Single source of truth for where WattSeal stores its files.
use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

const DB_FILE: &str = "power_monitoring.db";
const LOG_FILE: &str = "collector.log";
// XDG convention is lowercase; Windows/macOS use the product name.
const APP_DIR: &str = if cfg!(target_os = "linux") {
    "wattseal"
} else {
    "WattSeal"
};

struct Locations {
    data: PathBuf,
    logs: PathBuf,
    custom: bool,
}

impl Locations {
    fn os_default() -> Self {
        let data = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir).join(APP_DIR);
        // `state_dir()` is Some only on Linux, elsewhere logs live with the data.
        let logs = dirs::state_dir()
            .map(|d| d.join(APP_DIR))
            .unwrap_or_else(|| data.clone());
        Self {
            data,
            logs,
            custom: false,
        }
    }
}

static LOCATIONS: OnceLock<Locations> = OnceLock::new();

fn locations() -> &'static Locations {
    LOCATIONS.get_or_init(Locations::os_default)
}

/// Call once at startup, right after CLI parsing. A custom dir disables legacy migration.
pub fn init(custom_dir: Option<&PathBuf>) {
    let chosen = match custom_dir {
        Some(dir) => Locations {
            logs: dir.to_path_buf(),
            data: dir.to_path_buf(),
            custom: true,
        },
        None => Locations::os_default(),
    };
    let _ = LOCATIONS.set(chosen);
    let _ = fs::create_dir_all(data_dir());
    let _ = fs::create_dir_all(log_dir());
    if !locations().custom {
        migrate_legacy_files();
    }
}

pub fn data_dir() -> &'static Path {
    &locations().data
}
pub fn log_dir() -> &'static Path {
    &locations().logs
}
pub fn database_path() -> PathBuf {
    data_dir().join(DB_FILE)
}
pub fn log_file() -> PathBuf {
    log_dir().join(LOG_FILE)
}

/// `Some` only when `--data-dir` was given (so the UI child can be started with the same one).
pub fn custom_data_dir() -> Option<&'static Path> {
    locations().custom.then_some(data_dir())
}

/// Moves files that older versions kept next to the executable (their working directory).
fn migrate_legacy_files() {
    let Some(old_dir) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(Path::to_path_buf))
    else {
        return;
    };
    let (data, logs) = (data_dir(), log_dir());
    if old_dir == data || data.join(DB_FILE).exists() {
        return; // nothing legacy, or already migrated / target already contains db
    }

    let old_db = old_dir.join(DB_FILE);
    if old_db.exists() {
        move_file(&old_db, &data.join(DB_FILE));
    }

    // SQLite auxiliary files (and old lock file) from the previous folder can be deleted
    let _ = fs::remove_file(old_dir.join(format!("{DB_FILE}-wal")));
    let _ = fs::remove_file(old_dir.join(format!("{DB_FILE}-shm")));
    let _ = fs::remove_file(old_dir.join(format!("{DB_FILE}.collector.lock")));

    // collector.log is migrated to logs directory and removed from old directory
    let old_log = old_dir.join(LOG_FILE);
    if old_log.exists() {
        move_file(&old_log, &logs.join(LOG_FILE));
    }
}

fn move_file(from: &Path, to: &Path) {
    if !from.exists() {
        return;
    }

    if fs::rename(from, to).is_err() {
        if fs::copy(from, to).is_ok() {
            let _ = fs::remove_file(from);
        }
    } else {
        // If rename succeeded, 'from' is already moved and no longer at old path
        return;
    }
}
