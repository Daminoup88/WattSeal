use super::{Sensor, SensorError};
use crate::core::types::{Event, GPUData};
use sysinfo::{System};

pub fn get_gpu_power_sensor() -> Result<impl Sensor<GPUData>, SensorError> {
    Ok(nvidia_gpu::NvidiaGPUSensor::new()?)
}

enum GPUVendor {
    Nvidia,
    Amd,
    Other,
}

impl GPUVendor {
    fn from_str(vendor_str: &str) -> GPUVendor {
        let vendor_lower = vendor_str.to_lowercase();
        if vendor_lower.contains("nvidia") {
            GPUVendor::Nvidia
        } else if vendor_lower.contains("amd") {
            GPUVendor::Amd
        } else {
            GPUVendor::Other
        }
    }
}

mod nvidia_gpu {
    use nvml_wrapper::Nvml;
    use super::{Sensor, SensorError};
    use crate::core::types::{Event, GPUData};

    pub struct NvidiaGPUSensor {
        nvml: Nvml,
    }

    impl NvidiaGPUSensor {
        pub fn new() -> Result<Self, SensorError> {
            let nvml = Nvml::init().map_err(|e| SensorError::ReadError(e.to_string()))?;
            Ok(NvidiaGPUSensor { nvml })
        }
    }

    impl Sensor<GPUData> for NvidiaGPUSensor {
        fn new(_param: &str) -> Self {
            NvidiaGPUSensor::new().unwrap()
        }

        fn name(&self) -> &'static str {
            "Nvidia GPU"
        }

        fn read_full_data(&self) -> Result<Event<GPUData>, SensorError> {
            let device = self.nvml.device_by_index(0).map_err(|e| SensorError::ReadError(e.to_string()))?;
            let power_usage_mw = device.power_usage().map_err(|e| SensorError::ReadError(e.to_string()))?;
            let utilization = device.utilization_rates().map_err(|e| SensorError::ReadError(e.to_string()))?;
            let data = GPUData {
                total_power_watts: power_usage_mw as f64 / 1000.0,
                usage_percent: utilization.gpu as f64,
                memory_usage_percent: utilization.memory as f64,
            };

            Ok(Event::new(data))
        }
    }
}