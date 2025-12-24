use std::time::SystemTime;

use crate::database::{Event, SensorData};

pub mod cpu;
pub mod gpu;

pub use cpu::CPUSensor;
pub use gpu::GPUSensor;

pub enum SensorType {
    CPU(CPUSensor),
    GPU(GPUSensor),
}

impl Sensor for SensorType {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        match self {
            SensorType::CPU(sensor) => sensor.read_full_data(),
            SensorType::GPU(sensor) => sensor.read_full_data(),
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
    let mut data = Vec::new();
    for sensor in sensors {
        let sensor_data = sensor.read_full_data();
        match sensor_data {
            Ok(d) => {
                println!("{}", d);
                data.push(d);
            }
            Err(e) => eprintln!("✗ Error reading sensor data: {:?}", e),
        }
    }
    Event::new(time, data)
}
