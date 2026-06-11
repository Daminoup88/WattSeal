use std::{cell::RefCell, rc::Rc, time::Instant};

use common::{
    EnergyUj,
    types::{InitialInfo, MemoryInfo},
};

use crate::{
    database::{RamData, SensorData},
    sensors::{Sensor, SensorError, System},
};

/// RAM usage sensor backed by sysinfo.
pub struct RamSensor {
    system: Rc<RefCell<System>>,
    last_reading: RefCell<Instant>,
}

impl RamSensor {
    /// Creates a sensor sharing the given `System` handle.
    pub fn new(system: Rc<RefCell<System>>) -> Self {
        Self {
            system,
            last_reading: RefCell::new(Instant::now()),
        }
    }
}

impl Sensor for RamSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        let now = Instant::now();
        let duration = now.duration_since(*self.last_reading.borrow()).as_secs_f64().max(0.001);

        let mut system = self
            .system
            .try_borrow_mut()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow system: {}", e)))?;
        system.refresh_memory();

        let total_memory = system.total_memory() as f64 / 1024.0; // Convert to GB
        let used_memory = system.used_memory() as f64 / 1024.0; // Convert to GB
        let usage_percent = if total_memory > 0.0 {
            (used_memory / total_memory) * 100.0
        } else {
            0.0
        };

        let energy_j = 5.0 * duration;

        *self.last_reading.borrow_mut() = now;

        Ok(SensorData::Ram(RamData {
            total_energy: Some(EnergyUj::from_joules(energy_j)),
            usage_percent: Some(usage_percent),
        }))
    }

    fn read_initial_info(&self) -> Result<InitialInfo, SensorError> {
        let sys = self
            .system
            .try_borrow()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow system: {}", e)))?;
        let total_ram = sys.total_memory();
        let total_swap = sys.total_swap();

        let memory_info = MemoryInfo {
            total_ram_bytes: total_ram,
            total_swap_bytes: total_swap,
        };

        Ok(InitialInfo::Memory(memory_info))
    }
}
