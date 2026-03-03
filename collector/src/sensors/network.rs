use std::{cell::RefCell, fmt::format};

use sysinfo::Networks;

use crate::{
    database::{NetworkData, SensorData},
    sensors::{Sensor, SensorError, network},
};

const NIC_IDLE_W: f64 = 0.2;
const NIC_W_PER_MB_S: f64 = 0.01;
const NIC_MAX_W: f64 = 3.0;

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
        let mut total_power = 0.0;

        for (_, data) in networks.iter() {
            let dl = data.received() as f64 / 1_048_576.0;
            let ul = data.transmitted() as f64 / 1_048_576.0;
            download_speed_mb_s += dl;
            upload_speed_mb_s += ul;

            let throughput = dl + ul;
            let nic_power = NIC_IDLE_W + throughput * NIC_W_PER_MB_S;
            total_power += nic_power;
        }

        Ok(SensorData::Network(NetworkData {
            total_power_watts: Some(total_power.min(NIC_MAX_W)),
            download_speed_mb_s,
            upload_speed_mb_s,
        }))
    }

    fn read_name(&self) -> Result<String, SensorError> {
        let mut networks = self
            .networks
            .try_borrow_mut()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow networks: {}", e)))?;
        networks.refresh(true);
        let names: Vec<String> = networks.iter().map(|(name, _)| name.clone()).collect();

        Ok(format!("Network(s): [{}]", names.join(", ")))
    }
}
