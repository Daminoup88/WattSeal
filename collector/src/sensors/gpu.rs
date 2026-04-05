use std::collections::HashMap;

use common::types::InitialInfo;

use super::{Sensor, SensorError, SensorType};
use crate::database::SensorData;

/// GPU hardware vendor identifier.
#[derive(Copy, Clone, PartialEq)]
pub enum GPUVendor {
    Nvidia,
    Amd,
    Intel,
    Other,
}

impl GPUVendor {
    /// Detects the vendor from a name string (e.g. "NVIDIA GeForce RTX 3070").
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

/// Returns the list of GPU adapter names detected on this system.
#[cfg(target_os = "windows")]
pub fn get_gpu_list() -> Vec<String> {
    use windows::Win32::Graphics::Dxgi::*;

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

/// Returns the list of NVIDIA GPU names via NVML.
#[cfg(target_os = "linux")]
pub fn get_gpu_list() -> Vec<String> {
    nvml_wrapper::Nvml::init()
        .and_then(|nvml| {
            let count = nvml.device_count()?;
            Ok((0..count)
                .filter_map(|i| nvml.device_by_index(i).ok())
                .filter_map(|d| d.name().ok())
                .collect())
        })
        .unwrap_or_default()
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_gpu_list() -> Vec<String> {
    Vec::new()
}

/// Platform-specific GPU power sensor.
pub enum GPUSensor {
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    Nvidia(nvidia_gpu::NvidiaGPUSensor),
    #[cfg(target_os = "windows")]
    Amd(amd_gpu::AmdGPUSensor),
    #[cfg(target_os = "windows")]
    Intel { sensor: intel_gpu::IntelGPUSensor },
}

impl Sensor for GPUSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        match self {
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            GPUSensor::Nvidia(sensor) => sensor.read_full_data(),
            #[cfg(target_os = "windows")]
            GPUSensor::Amd(sensor) => sensor.read_full_data(),
            #[cfg(target_os = "windows")]
            GPUSensor::Intel { sensor, .. } => sensor.read_full_data(),
            #[cfg(not(any(target_os = "windows", target_os = "linux")))]
            _ => Err(SensorError::NotSupported),
        }
    }

    fn read_initial_info(&self) -> Result<InitialInfo, SensorError> {
        Ok(InitialInfo::Gpus(get_gpu_list()))
    }

    fn read_name(&self) -> Result<String, SensorError> {
        Ok(format!("Gpu(s): [{}]", get_gpu_list().join(", ")))
    }
}

/// Creates a GPU power sensor appropriate for the given vendor.
pub fn get_gpu_power_sensor(vendor_id: &str, index: u32) -> Result<SensorType, SensorError> {
    let vendor = GPUVendor::from_str(vendor_id);

    #[cfg(target_os = "windows")]
    {
        let sensor = match vendor {
            GPUVendor::Amd => Ok(GPUSensor::Amd(amd_gpu::AmdGPUSensor::new(index)?)),
            GPUVendor::Nvidia => Ok(GPUSensor::Nvidia(nvidia_gpu::NvidiaGPUSensor::new(index)?)),
            GPUVendor::Intel => Ok(GPUSensor::Intel {
                sensor: intel_gpu::IntelGPUSensor::new(index)?,
            }),
            GPUVendor::Other => Err(SensorError::NotSupported),
        };
        return sensor.map(SensorType::GPU);
    }

    #[cfg(target_os = "linux")]
    {
        return match vendor {
            GPUVendor::Nvidia => nvidia_gpu::NvidiaGPUSensor::new(index).map(|s| SensorType::GPU(GPUSensor::Nvidia(s))),
            _ => Err(SensorError::NotSupported),
        };
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = (vendor, index);
        Err(SensorError::NotSupported)
    }
}

impl GPUSensor {
    /// Returns per-process GPU utilization percentages.
    pub fn get_process_gpu_usage(&self, current_timestamp: u64) -> Result<HashMap<u32, f64>, SensorError> {
        match self {
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            GPUSensor::Nvidia(sensor) => sensor.get_processes_gpu_usage(current_timestamp),
            #[cfg(target_os = "windows")]
            GPUSensor::Amd(_) | GPUSensor::Intel { .. } => Err(SensorError::NotSupported),
            #[cfg(not(any(target_os = "windows", target_os = "linux")))]
            _ => Err(SensorError::NotSupported),
        }
    }

    /// Returns `true` when this GPU is a known integrated GPU model.
    pub fn is_integrated(&self) -> bool {
        match self {
            #[cfg(target_os = "windows")]
            GPUSensor::Intel { .. } => true,
            _ => false,
        }
    }
}

#[cfg(target_os = "windows")]
mod amd_gpu {
    use adlx::{gpu_metrics::GpuMetrics, helper::AdlxHelper};

    use super::{Sensor, SensorError};
    use crate::database::{GPUData, SensorData};

    pub struct AmdGPUSensor {
        _helper: AdlxHelper,
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

            Ok(AmdGPUSensor {
                _helper: helper,
                gpu_metrics,
            })
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

#[cfg(any(target_os = "windows", target_os = "linux"))]
mod nvidia_gpu {
    use std::{cell::RefCell, collections::HashMap};

    use nvml_wrapper::Nvml;

    use super::{Sensor, SensorError};
    use crate::database::{GPUData, SensorData};

    pub struct NvidiaGPUSensor {
        nvml: Nvml,
        device_index: u32,
        last_timestamp: RefCell<u64>,
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
                last_timestamp: RefCell::new(0),
            })
        }

        pub fn get_processes_gpu_usage(&self, current_timestamp: u64) -> Result<HashMap<u32, f64>, SensorError> {
            let mut last_timestamp = self
                .last_timestamp
                .try_borrow_mut()
                .map_err(|_| SensorError::ReadError("Failed to borrow last_timestamp".to_string()))?;
            if *last_timestamp == 0 {
                *last_timestamp = current_timestamp;
                return Ok(HashMap::new());
            }
            let device = self
                .nvml
                .device_by_index(self.device_index)
                .map_err(|e| SensorError::ReadError(e.to_string()))?;
            let processes = device.process_utilization_stats(*last_timestamp);
            *last_timestamp = current_timestamp;
            let mut usage_map = HashMap::new();
            match processes {
                Ok(procs) => {
                    for proc in procs {
                        usage_map.insert(proc.pid, proc.sm_util as f64);
                    }
                    Ok(usage_map)
                }
                Err(e) => Err(SensorError::ReadError(format!(
                    "Failed to get process utilization stats: {}",
                    e
                ))),
            }
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

#[cfg(target_os = "windows")]
mod intel_gpu {
    use std::slice;

    use windows::{
        Win32::System::Performance::{
            PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData,
            PdhGetFormattedCounterArrayW, PdhOpenQueryW,
        },
        core::PCWSTR,
    };

    use super::{Sensor, SensorError};
    use crate::database::{GPUData, SensorData};

    const PDH_MORE_DATA: u32 = 0x800007D2;

    pub struct IntelGPUSensor {
        query: isize,
        counter: isize,
        initialized: std::cell::Cell<bool>,
    }

    impl IntelGPUSensor {
        pub fn new(_index: u32) -> Result<Self, SensorError> {
            unsafe {
                let mut query: isize = 0;
                if PdhOpenQueryW(None, 0, &mut query) != 0 {
                    return Err(SensorError::ReadError("PdhOpenQuery failed".to_string()));
                }
                let path: Vec<u16> = "\\GPU Engine(*)\\Utilization Percentage\0".encode_utf16().collect();
                let mut counter: isize = 0;
                if PdhAddEnglishCounterW(query, PCWSTR(path.as_ptr()), 0, &mut counter) != 0 {
                    let _ = PdhCloseQuery(query);
                    return Err(SensorError::ReadError("PdhAddEnglishCounter failed".to_string()));
                }
                Ok(IntelGPUSensor {
                    query,
                    counter,
                    initialized: std::cell::Cell::new(false),
                })
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
            unsafe {
                PdhCollectQueryData(self.query);
                if !self.initialized.get() {
                    self.initialized.set(true);
                    PdhCollectQueryData(self.query);
                }
                let (mut size, mut count) = (0u32, 0u32);
                if PdhGetFormattedCounterArrayW(self.counter, PDH_FMT_DOUBLE, &mut size, &mut count, None)
                    != PDH_MORE_DATA
                {
                    return Ok(GPUData {
                        total_power_watts: None,
                        usage_percent: Some(0.0),
                        vram_usage_percent: None,
                    }
                    .into());
                }
                let mut buf = vec![0u8; size as usize];
                let items = buf.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
                if PdhGetFormattedCounterArrayW(self.counter, PDH_FMT_DOUBLE, &mut size, &mut count, Some(items)) != 0 {
                    return Ok(GPUData {
                        total_power_watts: None,
                        usage_percent: Some(0.0),
                        vram_usage_percent: None,
                    }
                    .into());
                }
                let max = slice::from_raw_parts(items, count as usize)
                    .iter()
                    .filter(|i| i.FmtValue.CStatus == 0)
                    .filter_map(|i| {
                        let v = i.FmtValue.Anonymous.doubleValue;
                        v.is_finite().then_some(v)
                    })
                    .fold(0.0f64, f64::max);
                Ok(GPUData {
                    total_power_watts: None,
                    usage_percent: Some(max.clamp(0.0, 100.0)),
                    vram_usage_percent: None,
                }
                .into())
            }
        }
    }
}
