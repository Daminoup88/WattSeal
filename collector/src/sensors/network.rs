use std::{cell::RefCell, time::Instant};

use common::{EnergyUJ, NetworkData, SensorData};
use sysinfo::Networks;

use crate::sensors::{Sensor, SensorError};

const NIC_IDLE_W: f64 = 0.2;
const NIC_W_PER_MB_S: f64 = 0.01;
const NIC_MAX_W: f64 = 3.0;

/// Network interface sensor that estimates power from throughput.
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
    fn read_full_data(&self) -> Result<SensorData<EnergyUJ>, SensorError> {
        let now = Instant::now();
        let duration = now.duration_since(*self.last_reading.borrow()).as_secs_f64().max(0.001);

        let mut networks = self
            .networks
            .try_borrow_mut()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow networks: {}", e)))?;
        networks.refresh(true);

        let mut download_speed_mb_s = 0.0;
        let mut upload_speed_mb_s = 0.0;
        let mut total_energy_uj = 0;

        for (_, data) in networks.iter() {
            let dl = data.received() as f64 / 1_048_576.0;
            let ul = data.transmitted() as f64 / 1_048_576.0;
            download_speed_mb_s += dl;
            upload_speed_mb_s += ul;

            let throughput = dl + ul;
            let nic_power = (NIC_IDLE_W + throughput * NIC_W_PER_MB_S).min(NIC_MAX_W);
            total_energy_uj += (nic_power * duration * 1_000_000.0) as u64;
        }
        *self.last_reading.borrow_mut() = now;

        Ok(SensorData::Network(NetworkData {
            total_consumption: Some(total_energy_uj),
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
