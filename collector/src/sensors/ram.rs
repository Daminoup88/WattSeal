use std::{cell::RefCell, rc::Rc};

use crate::{
    database::{RamData, SensorData},
    sensors::{Sensor, SensorError, System},
};

pub struct RamSensor {
    system: Rc<RefCell<System>>,
}

impl RamSensor {
    pub fn new(system: Rc<RefCell<System>>) -> Self {
        Self { system }
    }
}

impl Sensor for RamSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
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

        Ok(SensorData::Ram(RamData {
            total_power_watts: Some(5.0),
            usage_percent: Some(usage_percent),
        }))
    }
}
