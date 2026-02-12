use std::cell::RefCell;

use sysinfo::Networks;

use crate::{
    database::{NetworkData, SensorData},
    sensors::{Sensor, SensorError, network},
};

pub struct NetworkSensor {
    networks: RefCell<Networks>,
}

impl NetworkSensor {
    pub fn new() -> Self {
        Self {
            networks: RefCell::new(Networks::new()),
        }
    }
}

impl Sensor for NetworkSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        let mut networks = self
            .networks
            .try_borrow_mut()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow networks: {}", e)))?;
        networks.refresh(true);

        let mut download_speed_mb_s = 0.0;
        let mut upload_speed_mb_s = 0.0;

        for (_, data) in networks.iter() {
            download_speed_mb_s += data.received() as f64 / 1_048_576.0; // Convert to MB/s
            upload_speed_mb_s += data.transmitted() as f64 / 1_048_576.0; // Convert to MB/s
        }

        Ok(SensorData::Network(NetworkData {
            total_power_watts: None,
            download_speed_mb_s,
            upload_speed_mb_s,
        }))
    }
}
