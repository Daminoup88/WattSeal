//! Keeps one overlay process at a time.

use std::{
    fs::{File, OpenOptions},
    time::{Duration, Instant},
};

use fs2::FileExt;

use crate::config::OverlayConfig;

/// How often a newcomer retries while a previous instance lets go.
const RETRY: Duration = Duration::from_millis(100);

/// Holds the lock that marks this process as *the* overlay.
///
/// The OS releases the lock when the file is dropped or the process ends, so a
/// crash cannot leave an overlay that can never be started again.
pub struct InstanceGuard {
    _file: File,
}

/// What happened when this process asked to be the overlay.
pub enum Claim {
    /// This process holds the lock and is the only overlay.
    Held(InstanceGuard),
    /// Another overlay is running; this process has nothing to do.
    Taken,
    /// The lock could not be used at all.
    Unavailable,
}

/// How long to give a previous instance to release the lock.
///
/// The overlay that is being hidden only finds out on its next poll — the
/// interval the config asks for — so a hide-then-show can arrive before the old
/// process has let go. Waiting one poll interval covers that handover. A genuine
/// duplicate only pays this wait before leaving, and it draws nothing while it
/// waits.
pub fn handover_wait() -> Duration {
    let refresh = OverlayConfig::load().map_or(1, |config| config.refresh_secs.clamp(1, 5));

    Duration::from_secs(u64::from(refresh) + 1)
}

impl InstanceGuard {
    /// Tries to become the overlay, waiting up to `wait` for a previous instance
    /// to let go of the lock.
    ///
    /// The tray, the dashboard footer and `--overlay` each start this process on
    /// their own, and no two of them can see the others' handle. Without the
    /// lock they stack two widgets on the same spot, both of them writing the
    /// same config file.
    pub fn claim(wait: Duration) -> Claim {
        let path = OverlayConfig::lock_path();
        let deadline = Instant::now() + wait;

        loop {
            let file = match OpenOptions::new().write(true).create(true).truncate(false).open(&path) {
                Ok(file) => file,
                Err(err) => {
                    // Not knowing whether we are alone is no reason to show
                    // nothing: two widgets read worse than one, but no widget at
                    // all is a broken feature.
                    log::warn!("overlay: single-instance lock unavailable: {err}");
                    return Claim::Unavailable;
                }
            };

            match file.try_lock_exclusive() {
                Ok(()) => return Claim::Held(Self { _file: file }),
                Err(_) if Instant::now() < deadline => std::thread::sleep(RETRY),
                Err(_) => return Claim::Taken,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_instance_can_hold_the_lock() {
        let Claim::Held(guard) = InstanceGuard::claim(Duration::ZERO) else {
            // Nothing to assert where the filesystem has no locking at all.
            return;
        };

        assert!(matches!(InstanceGuard::claim(Duration::ZERO), Claim::Taken));

        // The lock is handed over, rather than needing the holder to exit.
        drop(guard);
        assert!(matches!(InstanceGuard::claim(Duration::ZERO), Claim::Held(_)));
    }
}
