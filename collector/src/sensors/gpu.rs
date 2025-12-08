use sysinfo::System;
use windows::{Win32::Graphics::Dxgi::*, core::PCWSTR};

use super::{Sensor, SensorError};
use crate::core::types::{Event, GPUData};

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

    pub fn differs(&self, other: GPUVendor) -> bool {
        match (self, other) {
            (GPUVendor::Nvidia, GPUVendor::Nvidia) => false,
            (GPUVendor::Amd, GPUVendor::Amd) => false,
            (GPUVendor::Other, GPUVendor::Other) => false,
            _ => true,
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

pub fn get_gpu_power_sensor(vendor_id: &str, index: u32) -> Result<Box<dyn Sensor<GPUData>>, SensorError> {
    let vendor = GPUVendor::from_str(vendor_id);
    match vendor {
        GPUVendor::Amd => Ok(Box::new(amd_gpu::AmdGPUSensor::new(index)?)),
        GPUVendor::Nvidia => Ok(Box::new(nvidia_gpu::NvidiaGPUSensor::new(index)?)),
        GPUVendor::Intel => Ok(Box::new(intel_gpu::IntelGPUSensor::new(index)?)),
        GPUVendor::Other => Err(SensorError::NotSupported),
    }
}

mod amd_gpu {
    use std::ops::Index;

    use adlx::{
        gpu::Gpu, gpu_list::GpuList, gpu_metrics::GpuMetrics, helper::AdlxHelper,
        performance_monitoring_services::PerformanceMonitoringServices, system::System,
    };

    use super::{Sensor, SensorError};
    use crate::core::types::{Event, GPUData};

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

    impl Sensor<GPUData> for AmdGPUSensor {
        fn read_full_data(&self) -> Result<Event<GPUData>, SensorError> {
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

            Ok(Event::new(data))
        }
    }
}

mod nvidia_gpu {
    use nvml_wrapper::Nvml;

    use super::{Sensor, SensorError};
    use crate::core::types::{Event, GPUData};

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

    impl Sensor<GPUData> for NvidiaGPUSensor {
        fn read_full_data(&self) -> Result<Event<GPUData>, SensorError> {
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

            Ok(Event::new(data))
        }
    }
}

mod intel_gpu {
    use super::{Sensor, SensorError};
    use crate::core::types::{Event, GPUData};

    pub struct IntelGPUSensor {
        index: u32,
    }

    impl IntelGPUSensor {
        pub fn new(index: u32) -> Result<Self, SensorError> {
            // Initialize Intel GPU sensor here
            Ok(IntelGPUSensor { index })
        }
    }

    impl Sensor<GPUData> for IntelGPUSensor {
        fn read_full_data(&self) -> Result<Event<GPUData>, SensorError> {
            // Read Intel GPU data here
            // Placeholder implementation
            let data = GPUData {
                total_power_watts: None,
                usage_percent: None,
                vram_usage_percent: None,
            };

            Ok(Event::new(data))
        }
    }
}
