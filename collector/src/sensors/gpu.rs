use std::collections::HashMap;

use common::types::InitialInfo;

use super::{Sensor, SensorError, SensorType};
use crate::database::SensorData;

/// GPU hardware vendor identifier.
#[derive(Copy, Clone, PartialEq, Debug)]
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
    use std::collections::HashSet;

    use windows::Win32::Graphics::Dxgi::*;

    let mut list = Vec::new();
    // Use a HashSet to track the unique identifiers of the GPUs we've seen
    let mut seen_luids: HashSet<u64> = HashSet::new();

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

            if let Ok(desc) = adapter.GetDesc1() {
                let luid_u64 = ((desc.AdapterLuid.HighPart as u64) << 32) | (desc.AdapterLuid.LowPart as u64);

                if !seen_luids.insert(luid_u64) {
                    i += 1;
                    continue;
                }

                let name = String::from_utf16_lossy(
                    &desc
                        .Description
                        .iter()
                        .take_while(|c| **c != 0)
                        .cloned()
                        .collect::<Vec<u16>>(),
                );
                let name = name.trim();

                // Ignore Microsoft Basic Render Driver fallback driver or empty name
                if name.to_ascii_lowercase().contains("microsoft basic render driver") || name.is_empty() {
                    i += 1;
                    continue;
                }

                list.push(name.to_string());
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

/// Platform-specific GPU energy sensor inner variant.
pub enum GPUSensorInner {
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    Nvidia(nvidia_gpu::NvidiaGPUSensor),
    #[cfg(target_os = "windows")]
    Amd(amd_gpu::AmdGPUSensor),
    #[cfg(target_os = "windows")]
    Intel { sensor: intel_gpu::IntelGPUSensor },
}

/// Global GPU energy sensor wrapper.
pub struct GPUSensor {
    name: String,
    inner: GPUSensorInner,
}

impl Sensor for GPUSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        let mut data = match &self.inner {
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            GPUSensorInner::Nvidia(sensor) => sensor.read_full_data()?,
            #[cfg(target_os = "windows")]
            GPUSensorInner::Amd(sensor) => sensor.read_full_data()?,
            #[cfg(target_os = "windows")]
            GPUSensorInner::Intel { sensor } => sensor.read_full_data()?,
            #[cfg(not(any(target_os = "windows", target_os = "linux")))]
            _ => return Err(SensorError::NotSupported),
        };
        if let SensorData::GPU(ref mut gpu) = data {
            gpu.name = Some(self.name());
        }
        Ok(data)
    }

    fn read_initial_info(&self) -> Result<InitialInfo, SensorError> {
        Ok(InitialInfo::Gpus(get_gpu_list()))
    }

    fn read_name(&self) -> Result<String, SensorError> {
        Ok(self.name.clone())
    }
}

/// Creates a GPU energy sensor appropriate for the given vendor.
pub fn get_gpu_energy_sensor(vendor_id: &str, index: u32) -> Result<SensorType, SensorError> {
    let vendor = GPUVendor::from_str(vendor_id);
    let name = format!("{} ({})", vendor_id, index);

    #[cfg(target_os = "windows")]
    {
        let inner = match vendor {
            GPUVendor::Amd => Ok(GPUSensorInner::Amd(amd_gpu::AmdGPUSensor::new(index)?)),
            GPUVendor::Nvidia => Ok(GPUSensorInner::Nvidia(nvidia_gpu::NvidiaGPUSensor::new(index)?)),
            GPUVendor::Intel => Ok(GPUSensorInner::Intel {
                sensor: intel_gpu::IntelGPUSensor::new(index)?,
            }),
            GPUVendor::Other => Err(SensorError::NotSupported),
        }?;
        return Ok(SensorType::GPU(GPUSensor { name, inner }));
    }

    #[cfg(target_os = "linux")]
    {
        let inner = match vendor {
            GPUVendor::Nvidia => nvidia_gpu::NvidiaGPUSensor::new(index).map(GPUSensorInner::Nvidia),
            _ => Err(SensorError::NotSupported),
        }?;
        return Ok(SensorType::GPU(GPUSensor { name, inner }));
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
        match &self.inner {
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            GPUSensorInner::Nvidia(sensor) => sensor.get_processes_gpu_usage(current_timestamp),
            #[cfg(target_os = "windows")]
            GPUSensorInner::Amd(_) | GPUSensorInner::Intel { .. } => Err(SensorError::NotSupported),
            #[cfg(not(any(target_os = "windows", target_os = "linux")))]
            _ => Err(SensorError::NotSupported),
        }
    }

    /// Returns `true` when this GPU is a known integrated GPU model.
    pub fn is_integrated(&self) -> bool {
        match &self.inner {
            #[cfg(target_os = "windows")]
            GPUSensorInner::Intel { .. } => true,
            _ => false,
        }
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }
}

#[cfg(target_os = "windows")]
mod amd_gpu {
    use std::{cell::RefCell, time::Instant};

    use adlx::{Gpu, helper::AdlxHelper, performance_monitoring_services::PerformanceMonitoringServices};
    use common::types::EnergyUj;

    use super::{Sensor, SensorError};
    use crate::database::{GPUData, SensorData};

    pub struct AmdGPUSensor {
        _helper: AdlxHelper,
        perfo: PerformanceMonitoringServices,
        gpu: Gpu,
        last_reading: RefCell<Instant>,
        vram_supported: bool,
        power_supported: bool,
        usage_supported: bool,
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

            let supported_metrics = perfo
                .supported_gpu_metrics(&gpu)
                .map_err(|e| SensorError::ReadError(e.to_string()))?;

            let power_supported = supported_metrics
                .is_supported_gpu_power()
                .map_err(|e| SensorError::ReadError(e.to_string()))?;
            let usage_supported = supported_metrics
                .is_supported_gpu_usage()
                .map_err(|e| SensorError::ReadError(e.to_string()))?;
            let vram_supported = supported_metrics
                .is_supported_gpu_vram()
                .map_err(|e| SensorError::ReadError(e.to_string()))?;

            if !power_supported && !usage_supported {
                return Err(SensorError::ReadError(format!(
                    "⚠ AMD GPU {} power and usage telemetry unavailable at initialization",
                    index
                )));
            }
            if !power_supported {
                common::clog!(
                    "⚠ AMD GPU {} power telemetry unavailable at initialization: energy will be estimated",
                    index
                );
            }
            if !usage_supported {
                common::clog!(
                    "⚠ AMD GPU {} usage telemetry unavailable at initialization: usage will be None",
                    index
                );
            }
            if !vram_supported {
                common::clog!(
                    "⚠ AMD GPU {} VRAM telemetry unavailable at initialization: VRAM usage will be None",
                    index
                );
            }

            Ok(AmdGPUSensor {
                _helper: helper,
                perfo,
                gpu,
                last_reading: RefCell::new(Instant::now()),
                vram_supported,
                power_supported,
                usage_supported,
            })
        }
    }

    impl Sensor for AmdGPUSensor {
        fn read_full_data(&self) -> Result<SensorData, SensorError> {
            let now = Instant::now();
            let duration = now.duration_since(*self.last_reading.borrow()).as_secs_f64().max(0.001);
            let gpu_metrics = self
                .perfo
                .current_gpu_metrics(&self.gpu)
                .map_err(|e| SensorError::ReadError(e.to_string()))?;

            // Read AMD GPU data here
            let total_energy = if self.power_supported {
                let power_w = gpu_metrics
                    .total_board_power()
                    .ok()
                    .or_else(|| gpu_metrics.power().ok());
                power_w.map(|p| EnergyUj::from_joules(p * duration))
            } else {
                None
            };

            let usage = if self.usage_supported {
                gpu_metrics.usage().ok()
            } else {
                None
            };
            let memory = if self.vram_supported {
                Some(gpu_metrics.vram().map_err(|e| SensorError::ReadError(e.to_string()))? as f64)
            } else {
                None
            };

            let data = GPUData {
                total_energy,
                usage_percent: usage,
                vram_usage_percent: memory,
                name: None,
            };

            *self.last_reading.borrow_mut() = now;

            Ok(data.into())
        }
    }
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
mod nvidia_gpu {
    use std::{cell::RefCell, collections::HashMap, time::Instant};

    use common::types::EnergyUj;
    use nvml_wrapper::{Nvml, enum_wrappers::device::Sampling, enums::device::SampleValue};

    use super::{Sensor, SensorError};
    use crate::database::{GPUData, SensorData};

    enum NvidiaReadMode {
        EnergyCounter,
        PowerInstant, // fallback when energy counter isn't available
        PowerSamples, // fallback when neither counter nor instant reading is available
    }

    pub struct NvidiaGPUSensor {
        nvml: Nvml,
        device_index: u32,
        mode: NvidiaReadMode,
        last_timestamp: RefCell<u64>,
        last_energy_mj: RefCell<u64>,
        last_power_instant: RefCell<Option<Instant>>,
        last_sample_timestamp: RefCell<Option<u64>>,
    }

    impl NvidiaGPUSensor {
        pub fn new(index: u32) -> Result<Self, SensorError> {
            let nvml = Nvml::init().map_err(|e| SensorError::ReadError(e.to_string()))?;
            let device_count = nvml.device_count().map_err(|e| SensorError::ReadError(e.to_string()))?;
            if index >= device_count {
                return Err(SensorError::ReadError(format!(
                    "NVIDIA GPU index {} out of range ({} device(s) reported by NVML)",
                    index, device_count
                )));
            }

            let device = nvml
                .device_by_index(index)
                .map_err(|e| SensorError::ReadError(e.to_string()))?;

            device
                .utilization_rates()
                .map_err(|e| SensorError::ReadError(format!("NVIDIA GPU {} utilization unavailable: {}", index, e)))?;

            let mode = if device.total_energy_consumption().is_ok() {
                common::clog!("NVIDIA GPU {}: using energy counter mode", index);
                NvidiaReadMode::EnergyCounter
            } else if device.power_usage().is_ok() {
                common::clog!(
                    "NVIDIA GPU {}: energy counter unavailable, using power instant mode",
                    index
                );
                NvidiaReadMode::PowerInstant
            } else if device.samples(Sampling::Power, None).is_ok() {
                common::clog!(
                    "NVIDIA GPU {}: instant power unavailable, falling back to sampled power (driver-side estimate)",
                    index
                );
                NvidiaReadMode::PowerSamples
            } else {
                return Err(SensorError::ReadError(format!(
                    "⚠ NVIDIA GPU {}: no power telemetry available",
                    index
                )));
            };

            Ok(NvidiaGPUSensor {
                nvml,
                device_index: index,
                mode,
                last_timestamp: RefCell::new(0),
                last_energy_mj: RefCell::new(0),
                last_power_instant: RefCell::new(None),
                last_sample_timestamp: RefCell::new(None),
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
            let device = self
                .nvml
                .device_by_index(self.device_index)
                .map_err(|e| SensorError::ReadError(e.to_string()))?;

            let now = Instant::now();

            let energy = match self.mode {
                NvidiaReadMode::EnergyCounter => {
                    let current_energy_mj = device
                        .total_energy_consumption()
                        .map_err(|e| SensorError::ReadError(e.to_string()))?;
                    let mut last_energy_mj = self
                        .last_energy_mj
                        .try_borrow_mut()
                        .map_err(|_| SensorError::ReadError("Failed to borrow last energy".to_string()))?;
                    let energy_mj = if *last_energy_mj == 0 {
                        0
                    } else {
                        current_energy_mj.saturating_sub(*last_energy_mj)
                    };
                    *last_energy_mj = current_energy_mj;
                    EnergyUj::from_millijoules(energy_mj)
                }
                NvidiaReadMode::PowerInstant => {
                    let mw = device
                        .power_usage()
                        .map_err(|e| SensorError::ReadError(e.to_string()))?;
                    let mut last_instant = self
                        .last_power_instant
                        .try_borrow_mut()
                        .map_err(|_| SensorError::ReadError("Failed to borrow last power instant".to_string()))?;
                    let energy = match *last_instant {
                        None => EnergyUj::from_joules(0.0),
                        Some(last) => {
                            let dt = now.duration_since(last).as_secs_f64();
                            EnergyUj::from_joules(mw as f64 / 1000.0 * dt)
                        }
                    };
                    *last_instant = Some(now);
                    energy
                }
                NvidiaReadMode::PowerSamples => {
                    let mut last_ts = self
                        .last_sample_timestamp
                        .try_borrow_mut()
                        .map_err(|_| SensorError::ReadError("Failed to borrow last sample timestamp".to_string()))?;

                    let samples = device
                        .samples(Sampling::Power, *last_ts)
                        .map_err(|e| SensorError::ReadError(e.to_string()))?;

                    if samples.is_empty() {
                        EnergyUj::from_joules(0.0)
                    } else {
                        let avg_mw: f64 = samples
                            .iter()
                            .filter_map(|s| match s.value {
                                SampleValue::F64(v) => Some(v),
                                SampleValue::U32(v) => Some(v as f64),
                                SampleValue::U64(v) => Some(v as f64),
                                _ => None,
                            })
                            .sum::<f64>()
                            / samples.len().max(1) as f64;

                        *last_ts = samples.last().map(|s| s.timestamp).or(*last_ts);

                        let dt = now
                            .duration_since(self.last_power_instant.borrow().unwrap_or(now))
                            .as_secs_f64();
                        *self.last_power_instant.borrow_mut() = Some(now);

                        EnergyUj::from_joules((avg_mw / 1000.0) * dt)
                    }
                }
            };

            let utilization = device
                .utilization_rates()
                .map_err(|e| SensorError::ReadError(e.to_string()))?;

            Ok(GPUData {
                total_energy: Some(energy),
                usage_percent: Some(utilization.gpu as f64),
                vram_usage_percent: Some(utilization.memory as f64),
                name: None,
            }
            .into())
        }
    }
}

#[cfg(target_os = "windows")]
mod intel_gpu {
    use std::slice;

    use windows::{
        Win32::System::Performance::{
            PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY, PdhAddEnglishCounterW,
            PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW, PdhOpenQueryW,
        },
        core::PCWSTR,
    };

    use super::{Sensor, SensorError};
    use crate::database::{GPUData, SensorData};

    const PDH_MORE_DATA: u32 = 0x800007D2;

    pub struct IntelGPUSensor {
        query: PDH_HQUERY,
        counter: PDH_HCOUNTER,
        initialized: std::cell::Cell<bool>,
    }

    impl IntelGPUSensor {
        pub fn new(_index: u32) -> Result<Self, SensorError> {
            unsafe {
                let mut query = std::mem::MaybeUninit::<PDH_HQUERY>::uninit();
                if PdhOpenQueryW(None, 0, query.as_mut_ptr()) != 0 {
                    return Err(SensorError::ReadError("PdhOpenQuery failed".to_string()));
                }
                let query = query.assume_init();

                let path: Vec<u16> = "\\GPU Engine(*)\\Utilization Percentage\0".encode_utf16().collect();
                let mut counter = std::mem::MaybeUninit::<PDH_HCOUNTER>::uninit();
                if PdhAddEnglishCounterW(query, PCWSTR(path.as_ptr()), 0, counter.as_mut_ptr()) != 0 {
                    let _ = PdhCloseQuery(query);
                    return Err(SensorError::ReadError("PdhAddEnglishCounter failed".to_string()));
                }
                let counter = counter.assume_init();

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
                        total_energy: None,
                        usage_percent: Some(0.0),
                        vram_usage_percent: None,
                        name: None,
                    }
                    .into());
                }
                let mut buf = vec![0u8; size as usize];
                let items = buf.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
                if PdhGetFormattedCounterArrayW(self.counter, PDH_FMT_DOUBLE, &mut size, &mut count, Some(items)) != 0 {
                    return Ok(GPUData {
                        total_energy: None,
                        usage_percent: Some(0.0),
                        vram_usage_percent: None,
                        name: None,
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
                    total_energy: None,
                    usage_percent: Some(max.clamp(0.0, 100.0)),
                    vram_usage_percent: None,
                    name: None,
                }
                .into())
            }
        }
    }
}
