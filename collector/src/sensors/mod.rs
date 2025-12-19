use crate::core::types::{Event, SensorData};

pub mod cpu;
pub mod gpu;

pub trait Sensor {
    fn read_full_data(&self) -> Result<impl Into<SensorData>, SensorError>;
}

#[derive(Debug)]
pub enum SensorError {
    NotSupported,
    ReadError(String),
}
