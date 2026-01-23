use sysinfo::System;
use windows::{Win32::Graphics::Dxgi::*, core::PCWSTR};

use super::{Sensor, SensorError, SensorType};
use crate::database::{GPUData, SensorData};

#[derive(Copy, Clone, PartialEq)]
pub enum GPUVendor {
    Nvidia,
    Amd,
    Intel,
    Other,
}

impl GPUVendor {
    pub fn from_str(vendor_str: &str) -> GPUVendor {
        let vendor_lower = vendor_str.to_lowercase();
        if vendor_lower.contains("nvidia") {
            GPUVendor::Nvidia
        } else if vendor_lower.contains("amd") {
            GPUVendor::Amd
        } else if vendor_lower.contains("intel") {
            GPUVendor::Intel
        } else {
            GPUVendor::Other
        }
    }
}

pub fn get_gpu_list() -> Vec<String> {
    use windows::{Win32::Graphics::Dxgi::*, core::Result};

    let mut list = Vec::new();

    unsafe {
        let factory: IDXGIFactory1 = match CreateDXGIFactory1() {
            Ok(f) => f,
            Err(_) => return vec![],
        };

        let mut i = 0;
        loop {
            let adapter = match factory.EnumAdapters1(i) {
                Ok(a) => a,
                Err(_) => break,
            };

            let mut desc = DXGI_ADAPTER_DESC1::default();
            if adapter.GetDesc1(&mut desc).is_ok() {
                let name = String::from_utf16_lossy(
                    &desc
                        .Description
                        .iter()
                        .take_while(|c| **c != 0)
                        .cloned()
                        .collect::<Vec<u16>>(),
                );
                list.push(name);
            }
            i += 1;
        }
    }
    list
}

pub enum GPUSensor {
    Nvidia(nvidia_gpu::NvidiaGPUSensor),
    Amd(amd_gpu::AmdGPUSensor),
    Intel(intel_gpu::IntelGPUSensor),
}

impl Sensor for GPUSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        let data = match self {
            GPUSensor::Nvidia(sensor) => sensor.read_full_data()?,
            GPUSensor::Amd(sensor) => sensor.read_full_data()?,
            GPUSensor::Intel(sensor) => sensor.read_full_data()?,
        };
        Ok(data)
    }
}

pub fn get_gpu_power_sensor(vendor_id: &str, index: u32) -> Result<SensorType, SensorError> {
    let vendor = GPUVendor::from_str(vendor_id);
    let sensor = match vendor {
        GPUVendor::Amd => Ok(GPUSensor::Amd(amd_gpu::AmdGPUSensor::new(index)?)),
        GPUVendor::Nvidia => Ok(GPUSensor::Nvidia(nvidia_gpu::NvidiaGPUSensor::new(index)?)),
        GPUVendor::Intel => Ok(GPUSensor::Intel(intel_gpu::IntelGPUSensor::new(index)?)),
        GPUVendor::Other => Err(SensorError::NotSupported),
    };
    match sensor {
        Ok(s) => Ok(SensorType::GPU(s)),
        Err(e) => Err(e),
    }
}

mod amd_gpu {
    use std::ops::Index;

    use adlx::{
        gpu::Gpu, gpu_list::GpuList, gpu_metrics::GpuMetrics, helper::AdlxHelper,
        performance_monitoring_services::PerformanceMonitoringServices, system::System,
    };

    use super::{Sensor, SensorError};
    use crate::database::{GPUData, SensorData};

    pub struct AmdGPUSensor {
        gpu_metrics: GpuMetrics,
    }

    impl AmdGPUSensor {
        pub fn new(index: u32) -> Result<Self, SensorError> {
            let helper = AdlxHelper::new().map_err(|e| SensorError::ReadError(e.to_string()))?;
            let system = helper.system();
            let perfo = system
                .performance_monitoring_services()
                .map_err(|e| SensorError::ReadError(e.to_string()))?;
            let gpu_list = system.gpus().map_err(|e| SensorError::ReadError(e.to_string()))?;

            let gpu = gpu_list.at(index).map_err(|e| SensorError::ReadError(e.to_string()))?;
            let gpu_metrics = perfo
                .current_gpu_metrics(&gpu)
                .map_err(|e| SensorError::ReadError(e.to_string()))?;

            Ok(AmdGPUSensor { gpu_metrics })
        }
    }

    impl Sensor for AmdGPUSensor {
        fn read_full_data(&self) -> Result<SensorData, SensorError> {
            // Read AMD GPU data here
            let power_mw = self
                .gpu_metrics
                .power()
                .map_err(|e| SensorError::ReadError(e.to_string()))?;
            let usage = self
                .gpu_metrics
                .usage()
                .map_err(|e| SensorError::ReadError(e.to_string()))?;
            let memory = self
                .gpu_metrics
                .vram()
                .map_err(|e| SensorError::ReadError(e.to_string()))?;

            let data = GPUData {
                total_power_watts: Some(power_mw as f64 / 1000.0),
                usage_percent: Some(usage as f64),
                vram_usage_percent: Some(memory as f64),
            };

            Ok(data.into())
        }
    }
}

mod nvidia_gpu {
    use nvml_wrapper::Nvml;

    use super::{Sensor, SensorError};
    use crate::database::{GPUData, SensorData};

    pub struct NvidiaGPUSensor {
        nvml: Nvml,
        device_index: u32,
    }

    impl NvidiaGPUSensor {
        pub fn new(index: u32) -> Result<Self, SensorError> {
            let nvml = Nvml::init().map_err(|e| SensorError::ReadError(e.to_string()))?;
            // Validate that the device exists
            let _ = nvml
                .device_by_index(index)
                .map_err(|e| SensorError::ReadError(e.to_string()))?;
            Ok(NvidiaGPUSensor {
                nvml,
                device_index: index,
            })
        }
    }

    impl Sensor for NvidiaGPUSensor {
        fn read_full_data(&self) -> Result<SensorData, SensorError> {
            // Read NVIDIA GPU data here
            let device = self
                .nvml
                .device_by_index(self.device_index)
                .map_err(|e| SensorError::ReadError(e.to_string()))?;
            let power_usage_mw = device
                .power_usage()
                .map_err(|e| SensorError::ReadError(e.to_string()))?;
            let utilization = device
                .utilization_rates()
                .map_err(|e| SensorError::ReadError(e.to_string()))?;

            let data = GPUData {
                total_power_watts: Some(power_usage_mw as f64 / 1000.0),
                usage_percent: Some(utilization.gpu as f64),
                vram_usage_percent: Some(utilization.memory as f64),
            };

            Ok(data.into())
        }
    }
}

mod intel_gpu {
    use std::time::Duration;

    use windows::{
        Win32::System::Performance::{
            PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE, PdhAddCounterW, PdhCloseQuery, PdhCollectQueryData,
            PdhGetFormattedCounterValue, PdhOpenQueryW,
        },
        core::HSTRING,
    };

    use super::{Sensor, SensorError};
    use crate::database::{GPUData, SensorData};

    pub struct IntelGPUSensor {
        adapter_index: u32,
        query: isize,
        counter: isize,
    }

    impl IntelGPUSensor {
        pub fn new(index: u32) -> Result<Self, SensorError> {
            unsafe {
                let mut query: isize = 0;
                PdhOpenQueryW(None, 0, &mut query);

                // Try to find Intel GPU counter
                let counter_path = HSTRING::from(r"\GPU Engine(*engtype_3D)\Utilization Percentage");
                let mut counter: isize = 0;

                PdhAddCounterW(query, &counter_path, 0, &mut counter);

                Ok(IntelGPUSensor {
                    adapter_index: index,
                    query,
                    counter,
                })
            }
        }

        fn read_counter(&self) -> Result<f64, SensorError> {
            unsafe {
                PdhCollectQueryData(self.query);

                let mut value = PDH_FMT_COUNTERVALUE::default();
                PdhGetFormattedCounterValue(self.counter, PDH_FMT_DOUBLE, None, &mut value);

                Ok(value.Anonymous.doubleValue)
            }
        }
    }

    impl Drop for IntelGPUSensor {
        fn drop(&mut self) {
            unsafe {
                let _ = PdhCloseQuery(self.query);
            }
        }
    }

    impl Sensor for IntelGPUSensor {
        fn read_full_data(&self) -> Result<SensorData, SensorError> {
            // First collection initializes the counter
            let _ = self.read_counter();

            std::thread::sleep(Duration::from_millis(100));

            // Second collection gets actual value
            let usage_percent = self.read_counter()?;

            let data = GPUData {
                total_power_watts: None,
                usage_percent: Some(usage_percent.clamp(0.0, 100.0)),
                vram_usage_percent: None,
            };

            Ok(data.into())
        }
    }
}

// mod intel_gpu {
//     use super::{Sensor, SensorError};
//     use crate::database::{GPUData, SensorData};

//     pub struct IntelGPUSensor {
//         index: u32,
//     }

//     impl IntelGPUSensor {
//         pub fn new(index: u32) -> Result<Self, SensorError> {
//             // Initialize Intel GPU sensor here
//             Ok(IntelGPUSensor { index })
//         }
//     }

//     impl Sensor for IntelGPUSensor {
//         fn read_full_data(&self) -> Result<SensorData, SensorError> {
//             // Read Intel GPU data here
//             // Placeholder implementation
//             let data = GPUData {
//                 total_power_watts: None,
//                 usage_percent: None,
//                 vram_usage_percent: None,
//             };

//             Ok(data.into())
//         }
//     }
// }
