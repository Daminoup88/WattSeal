use std::{cell::RefCell, collections::HashMap, hash::Hash, time::Instant};

use sysinfo::Disks;

use crate::{
    database::{DiskData, SensorData},
    sensors::{Sensor, SensorError},
};

pub struct DiskSensor {
    disks: RefCell<Disks>,
}

impl DiskSensor {
    pub fn new() -> Self {
        Self {
            disks: RefCell::new(Disks::new_with_refreshed_list()),
        }
    }
}

impl Sensor for DiskSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        // let mut total_space = 0.0;
        // let mut used_space = 0.0;
        // let mut free_space = 0.0;
        let mut read_speed = 0.0;
        let mut write_speed = 0.0;

        let mut disks = self
            .disks
            .try_borrow_mut()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow disks: {}", e)))?;
        disks.refresh(true);

        for disk in disks.iter() {
            let usage = disk.usage();
            read_speed += usage.read_bytes as f64 / 1_048_576.0; // Convert to MB/s
            write_speed += usage.written_bytes as f64 / 1_048_576.0; // Convert to MB/s
            // total_space += disk.total_space() as f64 / 1_073_741_824.0; // Convert to GB
            // used_space += (disk.total_space() - disk.available_space()) as f64 / 1_073_741_824.0; // Convert to GB
            // free_space += disk.available_space() as f64 / 1_073_741_824.0; // Convert to GB
        }

        Ok(SensorData::Disk(DiskData {
            total_power_watts: None,
            // total_gb: total_space,
            // used_gb: used_space,
            // free_gb: free_space,
            read_usage_mb_s: read_speed,
            write_usage_mb_s: write_speed,
        }))
    }
}
