use crate::core::types::{Event, SensorData};

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
