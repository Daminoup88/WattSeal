use crate::core::types::Event;

pub mod cpu;
pub mod gpu;

pub trait Sensor<T> {
    fn read_full_data(&self) -> Result<T, SensorError>;
}

#[derive(Debug)]
pub enum SensorError {
    NotSupported,
    ReadError(String),
}
