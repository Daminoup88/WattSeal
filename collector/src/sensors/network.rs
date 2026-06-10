use std::{cell::RefCell, time::Instant};

use common::{EnergyUj, types::Byte};
use sysinfo::Networks;

use crate::{
    database::{NetworkData, SensorData},
    sensors::{Sensor, SensorError},
};

const NIC_IDLE_W: f64 = 0.2;
const NIC_W_PER_MB: f64 = 0.01;
const NIC_MAX_W: f64 = 3.0;

/// Network interface sensor that estimates energy consumption from throughput.
pub struct NetworkSensor {
    networks: RefCell<Networks>,
    last_reading: RefCell<Instant>,
}

impl NetworkSensor {
    /// Creates a sensor with an empty network list.
    pub fn new() -> Self {
        Self {
            networks: RefCell::new(Networks::new()),
            last_reading: RefCell::new(Instant::now()),
        }
    }
}

impl Sensor for NetworkSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        let now = Instant::now();
        let duration = now.duration_since(*self.last_reading.borrow()).as_secs_f64().max(0.001);

        let mut networks = self
            .networks
            .try_borrow_mut()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow networks: {}", e)))?;
        networks.refresh(true);

        let mut downloaded_bytes = 0;
        let mut uploaded_bytes = 0;
        let mut total_energy_j = 0.0;

        for (_, data) in networks.iter() {
            let dl_b = data.received();
            let ul_b = data.transmitted();
            downloaded_bytes += dl_b;
            uploaded_bytes += ul_b;
            let dl = dl_b as f64 / 1_048_576.0;
            let ul = ul_b as f64 / 1_048_576.0;

            let throughput = dl + ul;
            let nic_power = (NIC_IDLE_W + throughput * NIC_W_PER_MB).min(NIC_MAX_W);
            total_energy_j += nic_power * duration;
        }

        *self.last_reading.borrow_mut() = now;

        Ok(SensorData::Network(NetworkData {
            total_energy: Some(EnergyUj::from_joules(total_energy_j)),
            downloaded_bytes: Byte::from(downloaded_bytes),
            uploaded_bytes: Byte::from(uploaded_bytes),
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
