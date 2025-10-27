mod cpu;
mod rapl;

pub trait Sensor {
    fn name(&self) -> &'static str;
    fn read_power_watts(&self) -> Result<f64, SensorError>;
    fn read_usage_percent(&self) -> Result<f64, SensorError>;
}

#[derive(Debug)]
pub enum SensorError {
    ReadError(String),
    Unsupported(String),
}