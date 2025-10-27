use super::Sensor;
use crate::core::types::OS;

mod windows_cpu;

fn get_cpu_power_sensor(os: OS, vendor_id: &str) -> Result<Box<dyn Sensor>, String> {
    match os {
        OS::Windows => {
            let cpu_sensor = windows_cpu::WindowsCPU::new(vendor_id)?;
            Ok(Box::new(cpu_sensor))
        }
        _ => Err("Unsupported OS for CPU power sensor".to_string()),
    }
}

enum CPUVendor {
    Intel,
    Amd,
    Other,
}

struct CPU {
    vendor: CPUVendor,
    energy_unit: f64,
}

/*
    The CPU struct should dispatch the work to the right module depending on the OS.
    Each module implements the Sensor trait and is accountable for backup estimation methods.
 */

impl Sensor for CPU {
    fn name(&self) -> &'static str {
        "CPU"
    }

    fn read_power_watts(&self) -> Result<f64, super::SensorError> {
        // Placeholder implementation
        Ok(0.0)
    }

    fn read_usage_percent(&self) -> Result<f64, super::SensorError> {
        // Placeholder implementation
        Ok(0.0)
    }
}

impl CPU {
    fn new(os: OS, vendor_id: &str) -> Result<CPU, String> {
        let vendor_str = vendor_id.to_lowercase();
        let vendor = if vendor_str.contains("intel") { 
            CPUVendor::Intel
        } else if vendor_str.contains("amd") {
            CPUVendor::Amd
        } else {
            CPUVendor::Other
        };

        Ok(CPU {
            vendor,
            energy_unit: 0.0,
        })
    }
}