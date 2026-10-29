use std::{
    cell::Cell,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use common::types::EnergyUj;

use crate::{
    database::{GPUData, SensorData},
    sensors::{Sensor, SensorError},
};

pub struct AmdGPUSensor {
    device_path: PathBuf,
    power_path: Option<PathBuf>,
    last_reading: Cell<Option<Instant>>,
}

impl AmdGPUSensor {
    pub fn new(device_path: PathBuf) -> Result<Self, SensorError> {
        let power_path = find_power_sensor(&device_path);
        let usage_path = device_path.join("gpu_busy_percent");

        if power_path.is_none() && !usage_path.exists() {
            return Err(SensorError::NotSupported);
        }

        Ok(Self {
            device_path,
            power_path,
            last_reading: Cell::new(None),
        })
    }
}

impl Sensor for AmdGPUSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        let now = Instant::now();
        let previous = self.last_reading.replace(Some(now));
        let duration = previous
            .map(|last| now.duration_since(last).as_secs_f64())
            .unwrap_or(0.0);

        let power_uw = self.power_path.as_ref().and_then(|path| read_number(path));

        let total_energy = power_uw.map(|power| {
            // Microwatts multiplied by seconds give microjoules.
            EnergyUj::from_joules(power * duration / 1_000_000.0)
        });

        let usage_percent =
            read_number(&self.device_path.join("gpu_busy_percent")).filter(|value| (0.0..=100.0).contains(value));

        if total_energy.is_none() && usage_percent.is_none() {
            return Err(SensorError::ReadError(
                "AMD GPU power and utilization telemetry unavailable".to_string(),
            ));
        }

        let vram_used = read_number(&self.device_path.join("mem_info_vram_used"));

        let vram_total = read_number(&self.device_path.join("mem_info_vram_total"));

        let vram_usage_percent = match (vram_used, vram_total) {
            (Some(used), Some(total)) if total > 0.0 => Some((used / total * 100.0).clamp(0.0, 100.0)),
            _ => None,
        };

        Ok(GPUData {
            total_energy,
            usage_percent,
            vram_usage_percent,
            name: None,
        }
        .into())
    }
}

fn read_number(path: &Path) -> Option<f64> {
    let value = fs::read_to_string(path).ok()?;
    let number = value.trim().parse::<f64>().ok()?;
    number.is_finite().then_some(number)
}

fn find_power_sensor(device_path: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(device_path.join("hwmon")).ok()?;

    for entry in entries.flatten() {
        let hwmon_path = entry.path();

        if fs::read_to_string(hwmon_path.join("name"))
            .ok()
            .is_none_or(|name| name.trim() != "amdgpu")
        {
            continue;
        }

        let power_path = hwmon_path.join("power1_average");

        if power_path.exists() {
            return Some(power_path);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::read_number;

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    fn temp_file() -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        std::env::temp_dir().join(format!("wattseal-amdgpu-test-{}-{id}", std::process::id()))
    }

    #[test]
    fn reads_valid_sysfs_number() {
        let path = temp_file();
        fs::write(&path, "12345\n").unwrap();

        assert_eq!(read_number(&path), Some(12345.0));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_invalid_sysfs_number() {
        let path = temp_file();
        fs::write(&path, "not-a-number\\n").unwrap();

        assert_eq!(read_number(&path), None);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn handles_missing_sysfs_file() {
        let path = temp_file();

        assert_eq!(read_number(&path), None);
    }
}
