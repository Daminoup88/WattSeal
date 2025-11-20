use crate::core::types::{Event};

pub mod cpu;
pub mod gpu;

pub trait Sensor<T> {
    fn new(param: &str) -> Self;
    fn name(&self) -> &'static str;
    fn read_full_data(&self) -> Result<Event<T>, SensorError>;
}

#[derive(Debug)]
pub enum SensorError {
    NotSupported,
    ReadError(String),
}