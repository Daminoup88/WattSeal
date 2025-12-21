use windows_cpu::WindowsCPUSensor;

use super::{Sensor, SensorError, SensorType};
use crate::core::types::{CPUData, Event, SensorData};

#[cfg(target_os = "windows")]
mod windows_cpu;

pub enum CPUSensor {
    Windows(WindowsCPUSensor),
}

impl Sensor for CPUSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        match self {
            CPUSensor::Windows(sensor) => sensor.read_full_data(),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum CPUVendor {
    Intel,
    Amd,
    Other,
}

impl CPUVendor {
    pub fn from_str(vendor_str: &str) -> CPUVendor {
        let vendor_lower = vendor_str.to_lowercase();
        if vendor_lower.contains("intel") {
            CPUVendor::Intel
        } else if vendor_lower.contains("amd") {
            CPUVendor::Amd
        } else {
            CPUVendor::Other
        }
    }
}

pub fn get_cpu_list() -> Vec<String> {
    let s = sysinfo::System::new_all();
    s.cpus().iter().map(|cpu| cpu.brand().to_string()).collect()
}

pub fn get_cpu_power_sensor(index: usize) -> Result<SensorType, SensorError> {
    let s = sysinfo::System::new_all();
    let cpu = s.cpus().get(index);
    let vendor_id = match cpu {
        None => return Err(SensorError::NotSupported),
        Some(cpu_info) => cpu_info.vendor_id(),
    };

    #[cfg(target_os = "windows")]
    return Ok(SensorType::CPU(CPUSensor::Windows(WindowsCPUSensor::new(vendor_id))));

    #[cfg(not(target_os = "windows"))]
    return Err(SensorError::NotSupported);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cpu_vendor_from_str() {
        let intel = CPUVendor::from_str("GenuineIntel");
        assert!(matches!(intel, CPUVendor::Intel));

        let amd = CPUVendor::from_str("AuthenticAMD");
        assert!(matches!(amd, CPUVendor::Amd));

        let other = CPUVendor::from_str("SomeOtherVendor");
        assert!(matches!(other, CPUVendor::Other));
    }

    #[test]
    fn test_get_cpu_power_sensor() {
        let sensor_result = get_cpu_power_sensor(0);

        #[cfg(target_os = "windows")]
        {
            assert!(sensor_result.is_ok());
        }
    }
}
