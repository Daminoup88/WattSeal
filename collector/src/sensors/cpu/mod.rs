use windows_cpu::WindowsCPUSensor;

use super::{Sensor, SensorError};
use crate::core::types::{CPUData, Event};

#[cfg(target_os = "windows")]
mod windows_cpu;

pub fn get_cpu_power_sensor() -> Result<impl Sensor<CPUData>, SensorError> {
    let s = sysinfo::System::new_all();
    let cpu = s.cpus().first();
    let vendor_id = match cpu {
        None => return Err(SensorError::NotSupported),
        Some(cpu_info) => cpu_info.vendor_id(),
    };

    #[cfg(target_os = "windows")]
    return Ok(WindowsCPUSensor::new(vendor_id));

    #[cfg(not(target_os = "windows"))]
    return Err(SensorError::NotSupported);
}

enum CPUVendor {
    Intel,
    Amd,
    Other,
}

impl CPUVendor {
    fn from_str(vendor_str: &str) -> CPUVendor {
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
        let sensor_result = get_cpu_power_sensor();

        #[cfg(target_os = "windows")]
        {
            assert!(sensor_result.is_ok());
            assert_eq!(sensor_result.unwrap().name(), "Windows CPU");
        }
    }
}
