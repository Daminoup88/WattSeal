use std::{
    cell::{Cell, RefCell},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use common::types::EnergyUj;
use sysinfo::Gpus;

use crate::{
    database::{GPUData, SensorData},
    sensors::{Sensor, SensorError},
};

pub struct AmdGPUSensor {
    device_path: PathBuf,
    pci_address: String,
    power_paths: Vec<PathBuf>,
    energy_paths: Vec<PathBuf>,
    last_energy_reading: Cell<Option<u64>>,
    last_reading: Cell<Option<Instant>>,
    gpus: RefCell<Option<Gpus>>,
}

impl AmdGPUSensor {
    pub fn new(device_path: PathBuf, pci_address: String) -> Result<Self, SensorError> {
        let power_paths = find_power_sensors(&device_path);
        let energy_paths = find_energy_sensors(&device_path);
        let usage_path = device_path.join("gpu_busy_percent");

        if power_paths.is_empty() && energy_paths.is_empty() && !usage_path.exists() {
            return Err(SensorError::NotSupported);
        }

        Ok(Self {
            device_path,
            pci_address,
            power_paths,
            energy_paths,
            last_energy_reading: Cell::new(None),
            last_reading: Cell::new(None),
            gpus: RefCell::new(Gpus::new_with_refreshed_list().ok()),
        })
    }
}

impl Sensor for AmdGPUSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        let now = Instant::now();

        let power_uw = self
            .power_paths
            .iter()
            .find_map(|path| read_number(path).filter(|power| *power >= 0.0));

        let counter_uj = self
            .energy_paths
            .iter()
            .find_map(|path| fs::read_to_string(path).ok()?.trim().parse::<u64>().ok());

        let counter_energy = counter_uj.and_then(|current| {
            let previous = self.last_energy_reading.replace(Some(current))?;
            current.checked_sub(previous).map(EnergyUj::from_u64)
        });

        let power_energy = power_uw.and_then(|power| {
            let previous = self.last_reading.replace(Some(now))?;
            let duration = now.duration_since(previous).as_secs_f64();

            // Microwatts multiplied by seconds give microjoules.
            Some(EnergyUj::from_joules(power * duration / 1_000_000.0))
        });

        let total_energy = if let Some(energy) = counter_energy {
            Some(energy)
        } else {
            if counter_uj.is_none() && power_energy.is_some() {
                // The power estimate covers this interval. Rebaseline the
                // counter on recovery to avoid counting it twice.
                self.last_energy_reading.set(None);
            }

            power_energy
        };

        let sysinfo_gpu = self.gpus.borrow_mut().as_mut().and_then(|gpus| {
            gpus.refresh(true);

            gpus.iter()
                .find(|gpu| gpu.pci().to_string() == self.pci_address)
                .map(|gpu| {
                    let usage = gpu.usage().map(f64::from);

                    let vram = match (gpu.used_memory(), gpu.total_memory()) {
                        (Some(used), Some(total)) if total > 0 => {
                            Some((used as f64 / total as f64 * 100.0).clamp(0.0, 100.0))
                        }
                        _ => None,
                    };

                    (usage, vram)
                })
        });

        let (sysinfo_usage, sysinfo_vram) = sysinfo_gpu.unwrap_or((None, None));

        let usage_percent = sysinfo_usage.filter(|value| (0.0..=100.0).contains(value)).or_else(|| {
            read_number(&self.device_path.join("gpu_busy_percent")).filter(|value| (0.0..=100.0).contains(value))
        });

        if total_energy.is_none() && power_uw.is_none() && counter_uj.is_none() && usage_percent.is_none() {
            return Err(SensorError::ReadError(
                "AMD GPU power and utilization telemetry unavailable".to_string(),
            ));
        }

        let vram_used = read_number(&self.device_path.join("mem_info_vram_used"));

        let vram_total = read_number(&self.device_path.join("mem_info_vram_total"));

        let vram_usage_percent = sysinfo_vram.or_else(|| match (vram_used, vram_total) {
            (Some(used), Some(total)) if total > 0.0 => Some((used / total * 100.0).clamp(0.0, 100.0)),
            _ => None,
        });

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

fn find_energy_sensors(device_path: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(device_path.join("hwmon")) else {
        return Vec::new();
    };

    let mut energy_paths = Vec::new();

    for entry in entries.flatten() {
        let hwmon_path = entry.path();

        if fs::read_to_string(hwmon_path.join("name"))
            .ok()
            .is_none_or(|name| name.trim() != "amdgpu")
        {
            continue;
        }

        let Ok(files) = fs::read_dir(hwmon_path) else {
            continue;
        };

        for file in files.flatten() {
            let filename = file.file_name();
            let filename = filename.to_string_lossy();

            if let Some(index) = filename
                .strip_prefix("energy")
                .and_then(|name| name.strip_suffix("_input"))
            {
                if !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit()) {
                    energy_paths.push(file.path());
                }
            }
        }
    }

    energy_paths.sort();
    energy_paths
}

fn find_power_sensors(device_path: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(device_path.join("hwmon")) else {
        return Vec::new();
    };

    let mut power_paths = Vec::new();

    for entry in entries.flatten() {
        let hwmon_path = entry.path();

        if fs::read_to_string(hwmon_path.join("name"))
            .ok()
            .is_none_or(|name| name.trim() != "amdgpu")
        {
            continue;
        }

        for filename in ["power1_average", "power1_input"] {
            let power_path = hwmon_path.join(filename);

            if power_path.exists() {
                power_paths.push(power_path);
            }
        }
    }

    power_paths
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::{AmdGPUSensor, find_power_sensors, read_number};
    use crate::{database::SensorData, sensors::Sensor};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    fn temp_file() -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        std::env::temp_dir().join(format!("wattseal-amdgpu-test-{}-{id}", std::process::id()))
    }

    #[test]
    fn recovers_after_both_energy_sources_fail() {
        let device_path = temp_file();
        let hwmon_path = device_path.join("hwmon/hwmon0");

        fs::create_dir_all(&hwmon_path).unwrap();
        fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();

        let energy_path = hwmon_path.join("energy1_input");
        let power_path = hwmon_path.join("power1_average");

        fs::write(&energy_path, "10000000\n").unwrap();
        fs::write(&power_path, "100000000\n").unwrap();
        fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

        let sensor = AmdGPUSensor::new(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

        sensor.read_full_data().unwrap();

        fs::write(&energy_path, "invalid\n").unwrap();
        fs::write(&power_path, "invalid\n").unwrap();

        match sensor.read_full_data().unwrap() {
            SensorData::GPU(data) => assert!(data.total_energy.is_none()),
            _ => panic!("Expected GPU sensor data"),
        }

        assert_eq!(sensor.last_energy_reading.get(), Some(10_000_000));

        fs::write(&power_path, "100000000\n").unwrap();

        sensor
            .last_reading
            .set(Some(std::time::Instant::now() - std::time::Duration::from_secs(2)));

        match sensor.read_full_data().unwrap() {
            SensorData::GPU(data) => {
                let joules = data.total_energy.expect("Expected power fallback").as_joules();

                assert!(
                    (200.0..205.0).contains(&joules),
                    "Expected approximately 200 J, got {joules} J"
                );
            }
            _ => panic!("Expected GPU sensor data"),
        }

        assert_eq!(sensor.last_energy_reading.get(), None);

        fs::write(&energy_path, "13000000\n").unwrap();

        sensor.read_full_data().unwrap();
        assert_eq!(sensor.last_energy_reading.get(), Some(13_000_000));

        fs::write(&energy_path, "15000000\n").unwrap();

        match sensor.read_full_data().unwrap() {
            SensorData::GPU(data) => {
                let joules = data.total_energy.expect("Expected counter energy").as_joules();

                assert_eq!(joules, 2.0);
            }
            _ => panic!("Expected GPU sensor data"),
        }

        fs::remove_dir_all(device_path).unwrap();
    }

    #[test]
    fn falls_back_to_power_when_energy_counter_fails() {
        let device_path = temp_file();
        let hwmon_path = device_path.join("hwmon/hwmon0");

        fs::create_dir_all(&hwmon_path).unwrap();
        fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();

        let energy_path = hwmon_path.join("energy1_input");
        fs::write(&energy_path, "10000000\n").unwrap();
        fs::write(hwmon_path.join("power1_average"), "100000000\n").unwrap();
        fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

        let sensor = AmdGPUSensor::new(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

        sensor.read_full_data().unwrap();

        fs::write(&energy_path, "invalid\n").unwrap();

        sensor
            .last_reading
            .set(Some(std::time::Instant::now() - std::time::Duration::from_secs(2)));

        match sensor.read_full_data().unwrap() {
            SensorData::GPU(data) => {
                let joules = data.total_energy.expect("Expected power fallback").as_joules();
                assert!(
                    (200.0..205.0).contains(&joules),
                    "Expected approximately 200 J from power, got {joules} J"
                );
            }
            _ => panic!("Expected GPU sensor data"),
        }

        assert_eq!(sensor.last_energy_reading.get(), None);

        fs::write(&energy_path, "13000000\n").unwrap();

        // The first recovered reading establishes a new counter baseline.
        sensor.read_full_data().unwrap();
        assert_eq!(sensor.last_energy_reading.get(), Some(13_000_000));

        fs::write(&energy_path, "15000000\n").unwrap();

        match sensor.read_full_data().unwrap() {
            SensorData::GPU(data) => {
                let energy = data.total_energy.expect("Expected recovered counter energy");
                assert_eq!(energy.as_joules(), 2.0);
            }
            _ => panic!("Expected GPU sensor data"),
        }

        fs::remove_dir_all(device_path).unwrap();
    }

    #[test]
    fn handles_energy_counter_reset_with_power_available() {
        let device_path = temp_file();
        let hwmon_path = device_path.join("hwmon/hwmon0");

        fs::create_dir_all(&hwmon_path).unwrap();
        fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();

        let energy_path = hwmon_path.join("energy1_input");
        fs::write(&energy_path, "10000000\n").unwrap();
        fs::write(hwmon_path.join("power1_average"), "100000000\n").unwrap();
        fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

        let sensor = AmdGPUSensor::new(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

        sensor.read_full_data().unwrap();

        fs::write(&energy_path, "1000000\n").unwrap();

        sensor
            .last_reading
            .set(Some(std::time::Instant::now() - std::time::Duration::from_secs(2)));

        match sensor.read_full_data().unwrap() {
            SensorData::GPU(data) => {
                let joules = data.total_energy.expect("Expected power fallback").as_joules();
                assert!(
                    (200.0..205.0).contains(&joules),
                    "Expected approximately 200 J, got {joules} J"
                );
            }
            _ => panic!("Expected GPU sensor data"),
        }

        assert_eq!(sensor.last_energy_reading.get(), Some(1_000_000));

        fs::write(&energy_path, "3000000\n").unwrap();

        match sensor.read_full_data().unwrap() {
            SensorData::GPU(data) => {
                let energy = data.total_energy.expect("Expected counter energy");
                assert_eq!(energy.as_joules(), 2.0);
            }
            _ => panic!("Expected GPU sensor data"),
        }

        fs::remove_dir_all(device_path).unwrap();
    }

    #[test]
    fn handles_cumulative_energy_counter_reset() {
        let device_path = temp_file();
        let hwmon_path = device_path.join("hwmon/hwmon0");

        fs::create_dir_all(&hwmon_path).unwrap();
        fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();

        let energy_path = hwmon_path.join("energy1_input");
        fs::write(&energy_path, "10000000\n").unwrap();
        fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

        let sensor = AmdGPUSensor::new(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

        sensor.read_full_data().unwrap();

        fs::write(&energy_path, "1000000\n").unwrap();

        match sensor.read_full_data().unwrap() {
            SensorData::GPU(data) => assert!(data.total_energy.is_none()),
            _ => panic!("Expected GPU sensor data"),
        }

        fs::write(&energy_path, "3000000\n").unwrap();

        match sensor.read_full_data().unwrap() {
            SensorData::GPU(data) => {
                let energy = data.total_energy.expect("Expected energy after counter reset");
                assert_eq!(energy.as_joules(), 2.0);
            }
            _ => panic!("Expected GPU sensor data"),
        }

        fs::remove_dir_all(device_path).unwrap();
    }

    #[test]
    fn uses_cumulative_energy_counter_before_power_estimate() {
        let device_path = temp_file();
        let hwmon_path = device_path.join("hwmon/hwmon0");

        fs::create_dir_all(&hwmon_path).unwrap();
        fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();

        let energy_path = hwmon_path.join("energy1_input");
        fs::write(&energy_path, "10000000\n").unwrap();
        fs::write(hwmon_path.join("power1_average"), "100000000\n").unwrap();
        fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

        let sensor = AmdGPUSensor::new(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

        assert_eq!(sensor.energy_paths, vec![energy_path.clone()]);

        match sensor.read_full_data().unwrap() {
            SensorData::GPU(data) => assert!(data.total_energy.is_none()),
            _ => panic!("Expected GPU sensor data"),
        }

        fs::write(&energy_path, "13000000\n").unwrap();

        sensor
            .last_reading
            .set(Some(std::time::Instant::now() - std::time::Duration::from_secs(2)));

        match sensor.read_full_data().unwrap() {
            SensorData::GPU(data) => {
                let energy = data.total_energy.expect("Expected counter energy");
                assert_eq!(energy.as_joules(), 3.0);
            }
            _ => panic!("Expected GPU sensor data"),
        }

        fs::remove_dir_all(device_path).unwrap();
    }

    #[test]
    fn first_power_sample_initializes_energy_measurement() {
        let device_path = temp_file();
        let hwmon_path = device_path.join("hwmon/hwmon0");

        fs::create_dir_all(&hwmon_path).unwrap();
        fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();
        fs::write(hwmon_path.join("power1_average"), "100000000\n").unwrap();
        fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

        let sensor = AmdGPUSensor::new(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

        assert!(sensor.last_reading.get().is_none());

        let first = sensor.read_full_data().unwrap();

        match first {
            SensorData::GPU(data) => assert!(data.total_energy.is_none()),
            _ => panic!("Expected GPU sensor data"),
        }

        assert!(sensor.last_reading.get().is_some());

        std::thread::sleep(std::time::Duration::from_millis(20));

        let second = sensor.read_full_data().unwrap();

        match second {
            SensorData::GPU(data) => assert!(data.total_energy.is_some()),
            _ => panic!("Expected GPU sensor data"),
        }

        fs::remove_dir_all(device_path).unwrap();
    }

    #[test]
    fn preserves_energy_interval_after_power_read_failure() {
        let device_path = temp_file();
        let hwmon_path = device_path.join("hwmon/hwmon0");

        fs::create_dir_all(&hwmon_path).unwrap();
        fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();

        let power_path = hwmon_path.join("power1_average");
        fs::write(&power_path, "100000000\n").unwrap();
        fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

        let sensor = AmdGPUSensor::new(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

        sensor.read_full_data().unwrap();
        let previous = sensor.last_reading.get().unwrap();

        fs::write(&power_path, "invalid\n").unwrap();

        let failed_power_sample = sensor.read_full_data().unwrap();

        match failed_power_sample {
            SensorData::GPU(data) => assert!(data.total_energy.is_none()),
            _ => panic!("Expected GPU sensor data"),
        }

        assert_eq!(sensor.last_reading.get(), Some(previous));

        sensor
            .last_reading
            .set(Some(std::time::Instant::now() - std::time::Duration::from_secs(2)));

        fs::write(&power_path, "100000000\n").unwrap();

        let recovered_sample = sensor.read_full_data().unwrap();

        match recovered_sample {
            SensorData::GPU(data) => {
                let energy = data.total_energy.expect("Expected energy after recovery");
                let joules = energy.as_joules();

                assert!(
                    (200.0..205.0).contains(&joules),
                    "Expected approximately 200 J, got {joules} J"
                );
                assert!(sensor.last_reading.get().unwrap() > previous);
            }
            _ => panic!("Expected GPU sensor data"),
        }

        fs::remove_dir_all(device_path).unwrap();
    }

    #[test]
    fn falls_back_to_power_input_when_average_is_invalid() {
        let device_path = temp_file();
        let hwmon_path = device_path.join("hwmon/hwmon0");

        fs::create_dir_all(&hwmon_path).unwrap();
        fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();
        fs::write(hwmon_path.join("power1_average"), "invalid\n").unwrap();
        fs::write(hwmon_path.join("power1_input"), "80000000\n").unwrap();
        fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

        let sensor = AmdGPUSensor::new(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

        assert_eq!(sensor.power_paths.len(), 2);

        let first = sensor.read_full_data().unwrap();
        match first {
            SensorData::GPU(data) => assert!(data.total_energy.is_none()),
            _ => panic!("Expected GPU sensor data"),
        }

        sensor
            .last_reading
            .set(Some(std::time::Instant::now() - std::time::Duration::from_secs(2)));

        let second = sensor.read_full_data().unwrap();
        match second {
            SensorData::GPU(data) => {
                let energy = data.total_energy.expect("Expected fallback energy");
                let joules = energy.as_joules();

                assert!(
                    (160.0..165.0).contains(&joules),
                    "Expected approximately 160 J, got {joules} J"
                );
            }
            _ => panic!("Expected GPU sensor data"),
        }

        fs::remove_dir_all(device_path).unwrap();
    }

    #[test]
    fn selects_power_sensor_with_fallback() {
        let device_path = temp_file();
        let hwmon_path = device_path.join("hwmon/hwmon0");

        fs::create_dir_all(&hwmon_path).unwrap();
        fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();

        let average = hwmon_path.join("power1_average");
        let input = hwmon_path.join("power1_input");

        assert!(find_power_sensors(&device_path).is_empty());

        fs::write(&input, "100000000\n").unwrap();
        assert_eq!(find_power_sensors(&device_path), vec![input.clone()]);

        fs::write(&average, "90000000\n").unwrap();
        assert_eq!(find_power_sensors(&device_path), vec![average, input]);

        fs::remove_dir_all(device_path).unwrap();
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
