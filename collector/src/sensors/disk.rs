use std::{cell::RefCell, time::Instant};

use common::{
    EnergyUj,
    types::{Byte, DiskInfo, InitialInfo},
};
use sysinfo::Disks;

use crate::{
    database::{DiskData, SensorData},
    sensors::{Sensor, SensorError},
};

const SSD_IDLE_W: f64 = 0.05;
const HDD_IDLE_W: f64 = 3.0;
const UNKNOWN_IDLE_W: f64 = 0.3;

const SSD_W_PER_MB: f64 = 0.015;
const HDD_W_PER_MB: f64 = 0.035;
const UNKNOWN_W_PER_MB: f64 = 0.02;

/// Disk I/O sensor that estimates power from throughput.
pub struct DiskSensor {
    disks: RefCell<Disks>,
    last_reading: RefCell<Instant>,
}

impl DiskSensor {
    /// Creates a sensor with a refreshed list of disks.
    pub fn new() -> Self {
        Self {
            disks: RefCell::new(Disks::new_with_refreshed_list()),
            last_reading: RefCell::new(Instant::now()),
        }
    }
}

impl Sensor for DiskSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        let now = Instant::now();
        let duration = now.duration_since(*self.last_reading.borrow()).as_secs_f64().max(0.001);

        let mut read_bytes = 0;
        let mut written_bytes = 0;
        let mut total_energy_j = 0.0;

        let mut disks = self
            .disks
            .try_borrow_mut()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow disks: {}", e)))?;
        disks.refresh(true);

        for disk in disks.iter() {
            let usage = disk.usage();
            let rd_b = usage.read_bytes;
            let wr_b = usage.written_bytes;
            read_bytes += rd_b;
            written_bytes += wr_b;
            let rd_mb = rd_b as f64 / 1_048_576.0;
            let wr_mb = wr_b as f64 / 1_048_576.0;

            let throughput = rd_mb + wr_mb;
            let (idle, per_mb) = match disk.kind() {
                sysinfo::DiskKind::SSD => (SSD_IDLE_W, SSD_W_PER_MB),
                sysinfo::DiskKind::HDD => (HDD_IDLE_W, HDD_W_PER_MB),
                _ => (UNKNOWN_IDLE_W, UNKNOWN_W_PER_MB),
            };
            let power = idle + throughput * per_mb;
            total_energy_j += power * duration;
        }

        *self.last_reading.borrow_mut() = now;

        Ok(SensorData::Disk(DiskData {
            total_energy: Some(EnergyUj::from_joules(total_energy_j)),
            read_bytes: Byte::from(read_bytes),
            written_bytes: Byte::from(written_bytes),
        }))
    }

    fn read_initial_info(&self) -> Result<InitialInfo, SensorError> {
        let disks = self
            .disks
            .try_borrow()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow disks: {}", e)))?;

        let mut disk_infos = Vec::new();
        for disk in disks.list() {
            let name = disk.name().to_string_lossy().to_string();
            let mount = disk.mount_point().display().to_string();
            let kind = disk_kind_label(disk);
            let total = disk.total_space();
            let avail = disk.available_space();
            let used = total - avail;
            let fs = disk.file_system().to_string_lossy().to_string();

            disk_infos.push(DiskInfo {
                name: name.clone(),
                mount_point: mount.clone(),
                file_system: fs,
                disk_type: kind.to_string(),
                total_bytes: total,
                used_bytes: used,
            });
        }
        Ok(InitialInfo::Disks(disk_infos))
    }

    fn read_name(&self) -> Result<String, SensorError> {
        let disks = self
            .disks
            .try_borrow()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow disks: {}", e)))?;

        let names: Vec<String> = disks
            .list()
            .iter()
            .map(|disk| disk.name().to_string_lossy().to_string())
            .collect();

        Ok(format!("Disk(s): [{}]", names.join(", ")))
    }
}

/// Returns a string label for the disk type.
fn disk_kind_label(kind: &sysinfo::Disk) -> &'static str {
    if kind.is_removable() {
        "Removable"
    } else {
        match kind.kind() {
            sysinfo::DiskKind::HDD => "HDD",
            sysinfo::DiskKind::SSD => "SSD",
            _ => "Unknown",
        }
    }
}
