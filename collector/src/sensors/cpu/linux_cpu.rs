use std::cell::RefCell;

use common::{CPUData, EnergyUJ, SensorData};

use super::{Sensor, SensorError};

/// Linux CPU power sensor using Intel RAPL (Running Average Power Limit)
/// via sysfs at `/sys/class/powercap/intel-rapl:0/`.
pub struct LinuxCPUSensor {
    rapl_path: String,
    last_reading: RefCell<Option<u64>>,
}

impl LinuxCPUSensor {
    /// Creates a RAPL sensor, failing if the sysfs endpoint is not readable.
    pub fn new() -> Result<Self, SensorError> {
        let rapl_path = "/sys/class/powercap/intel-rapl:0".to_string();
        let energy_path = format!("{}/energy_uj", rapl_path);
        std::fs::read_to_string(&energy_path)
            .map_err(|e| SensorError::ReadError(format!("RAPL not accessible: {}", e)))?;
        Ok(LinuxCPUSensor {
            rapl_path,
            last_reading: RefCell::new(None),
        })
    }

    /// Reads the cumulative energy counter in microjoules.
    fn read_energy_uj(&self) -> Result<EnergyUJ, SensorError> {
        let path = format!("{}/energy_uj", self.rapl_path);
        std::fs::read_to_string(&path)
            .map_err(|e| SensorError::ReadError(format!("Failed to read RAPL: {}", e)))?
            .trim()
            .parse::<u64>()
            .map_err(|e| SensorError::ReadError(format!("Failed to parse RAPL value: {}", e)))
    }
}

impl Sensor for LinuxCPUSensor {
    fn read_full_data(&self) -> Result<SensorData<EnergyUJ>, SensorError> {
        let current_uj = self.read_energy_uj()?;

        let last = self.last_reading.borrow();

        let energy_uj = match *last {
            Some(last_uj) => {
                let delta = if current_uj >= last_uj {
                    current_uj - last_uj
                } else {
                    current_uj // counter wrapped around
                };
                Some(delta)
            }
            None => None,
        };
        *self.last_reading.borrow_mut() = Some(current_uj);

        Ok(SensorData::CPU(CPUData {
            total_consumption: energy_uj,
            pp0_consumption: None,
            pp1_consumption: None,
            dram_consumption: None,
            usage_percent: None,
        }))
    }
}
