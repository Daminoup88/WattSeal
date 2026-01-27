pub mod cpu;
pub mod gpu;

use std::time::SystemTime;

pub use common::{Event, ProcessData, SensorData, TotalData};
pub use cpu::CPUSensor;
pub use gpu::GPUSensor;

pub use crate::process::{estimate_app_power_consumption, groups::group_processes_by_app};

pub enum SensorType {
    CPU(CPUSensor),
    GPU(GPUSensor),
    Total,
}

impl Sensor for SensorType {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        match self {
            SensorType::CPU(sensor) => sensor.read_full_data(),
            SensorType::GPU(sensor) => sensor.read_full_data(),
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

pub fn create_event_from_sensors(sensors: &Vec<SensorType>) -> Event {
    let time = SystemTime::now();
    let mut data: Vec<SensorData> = Vec::new();

    let mut top10_process_data: Vec<ProcessData> = Vec::new();
    let mut total_power = 0.0;
    for sensor in sensors {
        if let SensorType::Total = sensor {
            continue;
        }
        let sensor_data = sensor.read_full_data();
        match sensor_data {
            Ok(d) => {
                if let Some(power) = d.total_power_watts() {
                    total_power += power;

                    if let SensorType::CPU(_) = sensor {
                        let cpu_power = power;

                        // Get process data
                        let processes = estimate_app_power_consumption();
                        let list_of_apps = group_processes_by_app(processes, cpu_power);
                        top10_process_data = list_of_apps.into_iter().take(10).collect();
                    }
                }
                data.push(d);
            }
            Err(e) => eprintln!("✗ Error reading sensor data: {:?}", e),
        }
    }
    data.push(SensorData::Total(TotalData {
        total_power_watts: total_power,
    }));

    data.push(SensorData::Process(top10_process_data));

    return Event::new(time, data);
}
