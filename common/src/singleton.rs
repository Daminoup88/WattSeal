use std::fs::{File, OpenOptions};

use fs2::FileExt;

/// Holds an exclusive file lock next to the database.
///
/// While alive, no other collector process can lock the same file, which
/// prevents multiple collectors from writing to the same database
/// simultaneously.  Different working directories (different databases)
/// produce different lock files and can coexist.
///
/// When the guard is dropped or the process exits, the OS releases the
/// lock automatically.
pub struct SingletonGuard {
    _file: File,
}

impl SingletonGuard {
    /// Try to acquire the singleton lock for the given database path.
    ///
    /// Creates a `<db_path>.collector.lock` file next to the database and
    /// attempts an exclusive (non-blocking) lock on it.
    ///
    /// Returns `Ok(guard)` if this is the only collector for this database,
    /// or an error message if another collector already holds the lock.
    pub fn acquire(db_path: &str) -> Result<Self, String> {
        let lock_path = format!("{db_path}.collector.lock");

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|e| format!("Failed to create lock file '{lock_path}': {e}"))?;

        file.try_lock_exclusive().map_err(|_| {
            "Another collector is already running for this database. \
             Exiting to prevent database corruption."
                .to_string()
        })?;

        Ok(SingletonGuard { _file: file })
    }
}
