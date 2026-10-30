use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

pub struct AmdGpuDevice {
    pub name: String,
    pub device_path: PathBuf,
    pub pci_address: String,
}

/// Discovers AMD GPUs using the Linux DRM sysfs interface.
pub fn discover() -> Vec<AmdGpuDevice> {
    let mut devices = Vec::new();
    let mut seen = HashSet::new();
    let gpus = sysinfo::Gpus::new_with_refreshed_list().ok();

    let Ok(entries) = fs::read_dir("/sys/class/drm") else {
        return devices;
    };

    for entry in entries.flatten() {
        let card_name = entry.file_name();
        let card_name = card_name.to_string_lossy();

        // Only inspect primary DRM card nodes.
        let Some(index) = card_name.strip_prefix("card") else {
            continue;
        };

        if index.is_empty() || !index.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }

        let Ok(device_path) = fs::canonicalize(entry.path().join("device")) else {
            continue;
        };

        if read_hex(&device_path.join("vendor")) != Some(0x1002) {
            continue;
        }

        // Only support GPUs using the AMDGPU kernel driver.
        let Ok(driver) = fs::read_link(device_path.join("driver")) else {
            continue;
        };

        if driver.file_name().is_none_or(|name| name != "amdgpu") {
            continue;
        }

        // Avoid registering the same PCI device more than once.
        if !seen.insert(device_path.clone()) {
            continue;
        }

        let Some(pci_address) = device_path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.parse::<sysinfo::PCI>().ok())
            .map(|pci| pci.to_string())
        else {
            continue;
        };

        let name = gpus
            .as_ref()
            .and_then(|gpus| {
                gpus.iter()
                    .find(|gpu| gpu.pci().to_string() == pci_address)
                    .and_then(|gpu| gpu.model())
                    .filter(|model| !model.is_empty())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| fallback_name(&device_path));

        devices.push(AmdGpuDevice {
            name,
            device_path,
            pci_address,
        });
    }

    // Keep GPU indices stable regardless of filesystem enumeration order.
    devices.sort_by(|a, b| a.device_path.cmp(&b.device_path));

    devices
}

fn read_hex(path: &Path) -> Option<u32> {
    let value = fs::read_to_string(path).ok()?;
    u32::from_str_radix(value.trim().trim_start_matches("0x"), 16).ok()
}

fn fallback_name(device_path: &Path) -> String {
    let device_id = read_hex(&device_path.join("device"));

    match device_id {
        Some(id) => format!("AMD Radeon (PCI device {id:04x})"),
        None => "AMD Radeon".to_string(),
    }
}
