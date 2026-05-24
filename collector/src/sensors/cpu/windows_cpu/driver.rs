use scaphandre_driver_rs::ScaphandreDriver;

/// Safe wrapper around the Scaphandre RAPL driver for MSR access.
pub struct ScaphandreMsrReader {
    driver: ScaphandreDriver,
    cpu_index: u32,
}

impl ScaphandreMsrReader {
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
    pub fn is_installed() -> Result<bool, String> {
        ScaphandreDriver::is_installed().map_err(|e| format!("Failed to query Scaphandre driver status: {e}"))
    }

    /// Installs the driver (requires Administrator privileges).
    pub fn install() -> Result<(), String> {
        // 1072 means the service is marked for deletion.
        match ScaphandreDriver::install() {
            Ok(()) => return Ok(()),
            Err(e) => {
                let message = format!("{e}");
                return Err(format!(
                    "Failed to install Scaphandre driver: {message}. {}",
                    explain_windows_error_code(extract_windows_error_code(&message).unwrap_or(0))
                ));
            }
        }
    }

    /// Uninstalls the driver (requires Administrator privileges).
    pub fn uninstall() -> Result<(), String> {
        match Self::is_installed()? {
            false => return Ok(()),
            true => {}
        }

        let mut driver = match ScaphandreDriver::new() {
            Ok(driver) => driver,
            Err(e) => return Err(format!("Failed to open Scaphandre driver for uninstall: {e}")),
        };

        match driver.uninstall() {
            Ok(()) => Ok(()),
            Err(e) => {
                let message = format!("{e}");
                let code = extract_windows_error_code(&message);
                if code == Some(1072) {
                    // Already marked for deletion: treat as successful uninstall.
                    Ok(())
                } else {
                    Err(format!(
                        "Failed to uninstall Scaphandre driver: {message}. {}",
                        explain_windows_error_code(code.unwrap_or(0))
                    ))
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

fn extract_windows_error_code(message: &str) -> Option<u32> {
    let code_prefix = "(code ";
    if let Some(start) = message.find(code_prefix) {
        let start_index = start + code_prefix.len();
        if let Some(end) = message[start_index..].find(')') {
            let code_str = &message[start_index..start_index + end];
            return code_str.parse::<u32>().ok();
        }
    }
    None
}

fn explain_windows_error_code(code: u32) -> &'static str {
    match code {
        1072 => {
            "Windows reports the service is marked for deletion; close running WattSeal instances (and any tool using the Scaphandre driver), then retry. If it persists, reboot Windows."
        }
        5 => "Administrator privileges are required.",
        _ => "Unknown error code.",
    }
}
