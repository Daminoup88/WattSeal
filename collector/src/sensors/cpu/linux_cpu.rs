use std::cell::RefCell;

use common::types::EnergyUj;

use super::{Sensor, SensorError};
use crate::database::{CPUData, SensorData};

/// Linux CPU energy consumption sensor using Intel RAPL (Running Average Power Limit)
/// via sysfs at `/sys/class/powercap/intel-rapl:0/`.
pub struct LinuxCPUSensor {
    rapl_path: String,
    max_energy_uj: u64,
    last_reading: RefCell<Option<u64>>,
}

impl LinuxCPUSensor {
    /// Creates a RAPL sensor, failing if the sysfs endpoint is not readable.
    pub fn new() -> Result<Self, SensorError> {
        let rapl_path = "/sys/class/powercap/intel-rapl:0".to_string();
        let max_path = format!("{}/max_energy_range_uj", rapl_path);
        let max_energy_uj = std::fs::read_to_string(&max_path)
            .map_err(|e| SensorError::ReadError(format!("Failed to read RAPL max range: {}", e)))?
            .trim()
            .parse::<u64>()
            .map_err(|e| SensorError::ReadError(format!("Failed to parse RAPL max value: {}", e)))?;

        Ok(LinuxCPUSensor {
            rapl_path,
            max_energy_uj,
            last_reading: RefCell::new(None),
        })
    }

    /// Reads the cumulative energy counter in microjoules.
    fn read_energy_uj(&self) -> Result<u64, SensorError> {
        let path = format!("{}/energy_uj", self.rapl_path);
        std::fs::read_to_string(&path)
            .map_err(|e| SensorError::ReadError(format!("Failed to read RAPL: {}", e)))?
            .trim()
            .parse::<u64>()
            .map_err(|e| SensorError::ReadError(format!("Failed to parse RAPL value: {}", e)))
    }
}

impl Sensor for LinuxCPUSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        let current_uj = self.read_energy_uj()?;
        let last = *self.last_reading.borrow();

        let delta_uj = match last {
            Some(last_uj) => {
                let delta = if current_uj >= last_uj {
                    current_uj - last_uj
                } else {
                    self.max_energy_uj.saturating_sub(last_uj) + current_uj
                };
                Some(EnergyUj::from_u64(delta))
            }
            None => None,
        };

        *self.last_reading.borrow_mut() = Some(current_uj);

        Ok(SensorData::CPU(CPUData {
            total_energy: delta_uj,
            pp0_energy: None,
            pp1_energy: None,
            dram_energy: None,
            usage_percent: None,
        }))
    }
}
