use std::{thread, time::Duration};

use scaphandre_driver_rs::ScaphandreDriver;

/// Safe wrapper around the Scaphandre RAPL driver for MSR access.
pub struct ScaphandreMsrReader {
    driver: ScaphandreDriver,
    cpu_index: u32,
}

impl ScaphandreMsrReader {
    fn has_windows_error_code(message: &str, code: u32) -> bool {
        let code_fragment = format!("(code {code})");
        message.contains(&code_fragment)
    }

    /// Opens the Scaphandre driver device for MSR access.
    pub fn new() -> Result<Self, String> {
        let driver = ScaphandreDriver::new().map_err(|e| format!("Failed to open Scaphandre driver: {e}"))?;
        Ok(Self { driver, cpu_index: 0 })
    }

    /// Reads a Model-Specific Register by address.
    pub fn read_msr(&self, msr: u32) -> Result<u64, String> {
        self.driver
            .read_msr(msr, self.cpu_index)
            .map_err(|e| format!("Failed to read MSR {msr:#x}: {e}"))
    }

    /// Returns whether the driver is installed on the system.
    pub fn is_installed() -> bool {
        match ScaphandreDriver::is_installed() {
            Ok(installed) => installed,
            Err(e) => {
                eprintln!("Warning: failed to query Scaphandre driver status: {e}");
                false
            }
        }
    }

    /// Installs the driver (requires Administrator privileges).
    pub fn install() -> Result<(), String> {
        let mut last_error = String::new();

        // 1072 means the service is marked for deletion. This can be transient
        // after uninstall or while another process still releases service handles.
        for attempt in 0..3 {
            match ScaphandreDriver::install() {
                Ok(()) => return Ok(()),
                Err(e) => {
                    let message = format!("{e}");
                    if Self::has_windows_error_code(&message, 1072) && attempt < 2 {
                        thread::sleep(Duration::from_millis(500));
                        last_error = message;
                        continue;
                    }
                    return Err(format!("Failed to install Scaphandre driver: {message}"));
                }
            }
        }

        Err(format!(
            "Failed to install Scaphandre driver: {last_error}. Service is marked for deletion (code 1072); close apps using the driver and retry, or reboot Windows."
        ))
    }

    /// Uninstalls the driver (requires Administrator privileges).
    pub fn uninstall() -> Result<(), String> {
        match ScaphandreDriver::is_installed() {
            Ok(false) => return Ok(()),
            Ok(true) => {}
            Err(e) => return Err(format!("Failed to query Scaphandre driver status: {e}")),
        }

        let mut driver = match ScaphandreDriver::new() {
            Ok(driver) => driver,
            Err(e) => return Err(format!("Failed to open Scaphandre driver for uninstall: {e}")),
        };

        match driver.uninstall() {
            Ok(()) => Ok(()),
            Err(e) => {
                let message = format!("{e}");
                if Self::has_windows_error_code(&message, 1072) {
                    // Already marked for deletion: treat as successful uninstall.
                    Ok(())
                } else {
                    Err(format!("Failed to uninstall Scaphandre driver: {message}"))
                }
            }
        }
    }
}

impl Drop for ScaphandreMsrReader {
    fn drop(&mut self) {
        let _ = self.driver.close();
    }
}
