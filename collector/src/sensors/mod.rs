pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod network;
pub mod ram;
// pub mod process;

use std::{
    cell::RefCell,
    rc::Rc,
    time::{Instant, SystemTime},
};

pub use common::{Event, ProcessData, SensorData, TotalData};
pub use cpu::CPUSensor;
pub use disk::DiskSensor;
pub use gpu::GPUSensor;
pub use network::NetworkSensor;
pub use ram::RamSensor;
use sysinfo::System;

pub use crate::process::get_processes;

pub enum SensorType {
    CPU(CPUSensor),
    GPU(GPUSensor),
    RAM(RamSensor),
    Disk(DiskSensor),
    Network(NetworkSensor),
    Process,
    Total,
}

impl Sensor for SensorType {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        match self {
            SensorType::CPU(sensor) => sensor.read_full_data(),
            SensorType::GPU(sensor) => sensor.read_full_data(),
            SensorType::RAM(sensor) => sensor.read_full_data(),
            SensorType::Disk(sensor) => sensor.read_full_data(),
            SensorType::Network(sensor) => sensor.read_full_data(),
            SensorType::Process => Err(SensorError::NotSupported),
            SensorType::Total => Err(SensorError::NotSupported),
        }
    }
}

pub trait Sensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError>;
}

#[derive(Debug)]
pub enum SensorError {
    NotSupported,
    ReadError(String),
}

pub fn create_event_from_sensors(sensors: &Vec<SensorType>, system: Rc<RefCell<System>>) -> Event {
    let time = SystemTime::now();
    let mut data: Vec<SensorData> = Vec::new();

    let (mut cpu_power, mut cpu_usage, mut nb_cpus) = (0.0, 0.0, 0);
    let (mut gpu_power, mut gpu_usage, mut nb_gpus) = (0.0, 0.0, 0);

    let mut total_power = 0.0;
    for sensor in sensors {
        if let SensorType::Total = sensor {
            continue;
        }
        if let SensorType::Process = sensor {
            continue;
        }
        let sensor_data = sensor.read_full_data();
        match sensor_data {
            Ok(d) => {
                if let Some(power) = d.total_power_watts() {
                    total_power += power;

                    if let SensorData::CPU(cpu) = &d {
                        cpu_power += power;
                        cpu_usage += cpu.usage_percent.unwrap_or(0.0);
                        nb_cpus += 1;
                    }
                    
                    if let SensorData::GPU(gpu) = &d {
                        gpu_power += power;
                        gpu_usage += gpu.usage_percent.unwrap_or(0.0);
                        nb_gpus += 1;
                    }
                }
                data.push(d);
            }
            Err(e) => eprintln!("✗ Error reading sensor data: {:?}", e),
        }
    }
    data.push(SensorData::Total(TotalData {
        total_power_watts: total_power,
        period_type: "second".to_string(),
    }));

    cpu_usage /= nb_cpus.max(1) as f64;
    gpu_usage /= nb_gpus.max(1) as f64;
    let top10_process_data: Vec<ProcessData> = get_processes(
        system.clone(),
        cpu_power,
        cpu_usage,
        gpu_power,
        gpu_usage,
        total_power,
        10,
    );
    data.push(SensorData::Process(top10_process_data));

    return Event::new(time, data);
}
