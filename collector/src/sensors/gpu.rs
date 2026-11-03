use std::collections::HashMap;

use common::types::{InitialInfo, SensorData};

use super::{Sensor, SensorError, SensorType};
pub mod estimation;

/// GPU hardware vendor identifier.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GPUVendor {
    Nvidia,
    Amd,
    Intel,
    Other,
}

impl GPUVendor {
    /// Detects the vendor from PCI vendor ID.
    pub fn from_pci_vendor_id(vendor_id: u32) -> GPUVendor {
        match vendor_id {
            0x10DE => GPUVendor::Nvidia,
            0x1002 | 0x1022 => GPUVendor::Amd,
            0x8086 => GPUVendor::Intel,
            _ => GPUVendor::Other,
        }
    }

    /// Detects the vendor from a name string (e.g. "NVIDIA GeForce RTX 3070").
    pub fn from_str(vendor_str: &str) -> GPUVendor {
        let vendor_lower = vendor_str.to_lowercase();
        if vendor_lower.contains("nvidia") {
            GPUVendor::Nvidia
        } else if vendor_lower.contains("amd") || vendor_lower.contains("radeon") {
            GPUVendor::Amd
        } else if vendor_lower.contains("intel") {
            GPUVendor::Intel
        } else {
            GPUVendor::Other
        }
    }
}

/// System adapter identity holding display name and stable hardware identifiers.
#[derive(Clone, Debug, PartialEq)]
pub struct GpuAdapter {
    pub name: String,
    pub vendor: GPUVendor,
    pub vendor_id: u32,
    pub device_id: u32,
    pub luid: Option<u64>,
    pub pci_bus_id: Option<String>,
}

/// Returns the list of detected GPU adapter identities.
#[cfg(target_os = "windows")]
pub fn get_gpu_adapters() -> Vec<GpuAdapter> {
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

                let mut vendor = GPUVendor::from_pci_vendor_id(desc.VendorId);
                if vendor == GPUVendor::Other {
                    vendor = GPUVendor::from_str(name);
                }

                list.push(GpuAdapter {
                    name: name.to_string(),
                    vendor,
                    vendor_id: desc.VendorId,
                    device_id: desc.DeviceId,
                    luid: Some(luid_u64),
                    pci_bus_id: None,
                });
            }
            i += 1;
        }
    }
    list
}

/// Returns the list of GPU adapter identities.
#[cfg(target_os = "linux")]
pub fn get_gpu_adapters() -> Vec<GpuAdapter> {
    if let Ok(gpus) = sysinfo::Gpus::new_with_refreshed_list() {
        if !gpus.is_empty() {
            return gpus
                .iter()
                .map(|gpu| {
                    let name = gpu
                        .model()
                        .filter(|m| !m.is_empty())
                        .or_else(|| gpu.vendor())
                        .unwrap_or("Unknown GPU")
                        .to_string();
                    let vendor = gpu
                        .vendor()
                        .map(GPUVendor::from_str)
                        .unwrap_or_else(|| GPUVendor::from_str(&name));
                    let pci_bus_id = Some(gpu.pci().to_string());
                    GpuAdapter {
                        name,
                        vendor,
                        vendor_id: 0,
                        device_id: 0,
                        luid: None,
                        pci_bus_id,
                    }
                })
                .collect();
        }
    }

    // Fallback to NVML if sysinfo returned empty
    nvml_wrapper::Nvml::init()
        .and_then(|nvml| {
            let count = nvml.device_count()?;
            Ok((0..count)
                .filter_map(|i| nvml.device_by_index(i).ok())
                .filter_map(|d| {
                    let name = d.name().ok()?;
                    let pci_info = d.pci_info().ok();
                    let bus_id = pci_info.as_ref().map(|p| p.bus_id.clone());
                    let pci_device_id = pci_info.as_ref().map(|p| p.pci_device_id).unwrap_or(0);
                    let vendor_id = pci_device_id & 0xFFFF;
                    let device_id = (pci_device_id >> 16) & 0xFFFF;
                    let mut vendor = GPUVendor::from_pci_vendor_id(vendor_id);
                    if vendor == GPUVendor::Other {
                        vendor = GPUVendor::from_str(&name);
                    }
                    Some(GpuAdapter {
                        name,
                        vendor,
                        vendor_id,
                        device_id,
                        luid: None,
                        pci_bus_id: bus_id,
                    })
                })
                .collect())
        })
        .unwrap_or_default()
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_gpu_adapters() -> Vec<GpuAdapter> {
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
        Ok(InitialInfo::Gpus(vec![self.name()]))
    }

    fn read_name(&self) -> Result<String, SensorError> {
        Ok(self.name.clone())
    }
}

/// Creates a GPU energy sensor for the given adapter and display index.
pub fn get_gpu_energy_sensor(adapter: &GpuAdapter, index: u32) -> Result<SensorType, SensorError> {
    let name = format!("{} ({})", adapter.name, index);

    #[cfg(target_os = "windows")]
    {
        let inner = match adapter.vendor {
            GPUVendor::Amd => Ok(GPUSensorInner::Amd(amd_gpu::AmdGPUSensor::new(adapter, index)?)),
            GPUVendor::Nvidia => Ok(GPUSensorInner::Nvidia(nvidia_gpu::NvidiaGPUSensor::new(
                adapter, index,
            )?)),
            GPUVendor::Intel => Ok(GPUSensorInner::Intel {
                sensor: intel_gpu::IntelGPUSensor::new(adapter)?,
            }),
            GPUVendor::Other => Err(SensorError::NotSupported),
        }?;
        return Ok(SensorType::GPU(GPUSensor { name, inner }));
    }

    #[cfg(target_os = "linux")]
    {
        let inner = match adapter.vendor {
            GPUVendor::Nvidia => nvidia_gpu::NvidiaGPUSensor::new(adapter, index).map(GPUSensorInner::Nvidia),
            _ => Err(SensorError::NotSupported),
        }?;
        return Ok(SensorType::GPU(GPUSensor { name, inner }));
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = (adapter, index);
        Err(SensorError::NotSupported)
    }
}

fn names_match(left: &str, right: &str) -> bool {
    let left = left.to_lowercase();
    let right = right.to_lowercase();
    left.contains(&right) || right.contains(&left)
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
            GPUSensorInner::Amd(sensor) => sensor.is_integrated(),
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
    use std::{
        cell::Cell,
        sync::{Arc, Mutex},
        time::Instant,
    };

    use adlx::{Gpu, helper::AdlxHelper, performance_monitoring_services::PerformanceMonitoringServices};
    use common::types::{EnergyUj, GPUData, SensorData};

    use super::{GpuAdapter, Sensor, SensorError};

    static SHARED_ADLX: Mutex<Option<Arc<AdlxHelper>>> = Mutex::new(None);

    fn get_or_init_adlx() -> Result<Arc<AdlxHelper>, String> {
        let mut guard = SHARED_ADLX
            .lock()
            .map_err(|e| format!("ADLX shared-state mutex poisoned: {e}"))?;
        if let Some(helper) = &*guard {
            return Ok(helper.clone());
        }
        match AdlxHelper::new() {
            Ok(helper) => {
                let helper_arc = Arc::new(helper);
                *guard = Some(helper_arc.clone());
                Ok(helper_arc)
            }
            Err(e) => Err(format!("{e:#}")),
        }
    }

    pub fn match_adlx_gpu<'a>(adapter: &GpuAdapter, candidates: &'a [Gpu]) -> Option<&'a Gpu> {
        let dev_id_hex = format!("{:04x}", adapter.device_id);
        for g in candidates {
            if let Ok(adlx_dev_id) = g.device_id() {
                let clean_adlx = adlx_dev_id
                    .trim_start_matches("0x")
                    .trim_start_matches("0X")
                    .to_lowercase();
                if clean_adlx == dev_id_hex && adapter.device_id != 0 {
                    return Some(g);
                }
            }
        }

        for g in candidates {
            if let Ok(name) = g.name() {
                if super::names_match(&adapter.name, &name) {
                    return Some(g);
                }
            }
        }
        None
    }

    pub struct AmdGPUSensor {
        _helper: Arc<AdlxHelper>,
        perfo: PerformanceMonitoringServices,
        gpu: Gpu,
        last_reading: Cell<Instant>,
        vram_supported: bool,
        power_supported: bool,
        usage_supported: bool,
    }

    impl AmdGPUSensor {
        pub fn new(adapter: &GpuAdapter, fallback_index: u32) -> Result<Self, SensorError> {
            let helper = get_or_init_adlx().map_err(SensorError::ReadError)?;
            common::clog!("AMD GPU ('{}'): ADLX version {}", adapter.name, helper.full_version());
            let system = helper.system();
            let perfo = system
                .performance_monitoring_services()
                .map_err(|e| SensorError::ReadError(format!("{e:?}")))?;
            let gpu_list = system.gpus().map_err(|e| SensorError::ReadError(format!("{e:?}")))?;

            let gpu_count = gpu_list.size();
            let candidates = (0..gpu_count).filter_map(|i| gpu_list.at(i).ok()).collect::<Vec<_>>();

            let matched_gpu = match_adlx_gpu(adapter, &candidates).cloned().or_else(|| {
                let fallback = candidates.get(fallback_index as usize).cloned();
                if fallback.is_some() {
                    common::clog!(
                        "AMD GPU '{}': identity matching inconclusive, falling back to ADLX GPU at index {}",
                        adapter.name,
                        fallback_index
                    );
                }
                fallback
            });

            let gpu = match matched_gpu {
                Some(g) => {
                    let matched_name = g.name().unwrap_or("Unknown");
                    let dev_id = g.device_id().unwrap_or("Unknown");
                    let luid_str = adapter
                        .luid
                        .map(|l| format!("0x{:016x}", l))
                        .unwrap_or_else(|| "none".to_string());
                    common::clog!(
                        "✓ AMD adapter '{}' (LUID: {}) mapped to ADLX GPU '{}' (Device ID: {})",
                        adapter.name,
                        luid_str,
                        matched_name,
                        dev_id
                    );
                    g
                }
                None => {
                    return Err(SensorError::ReadError(format!(
                        "AMD GPU '{}' could not be resolved in ADLX ({} GPU(s) reported by ADLX)",
                        adapter.name, gpu_count
                    )));
                }
            };

            let supported_metrics = perfo
                .supported_gpu_metrics(&gpu)
                .map_err(|e| SensorError::ReadError(format!("{e:?}")))?;

            let power_supported = supported_metrics
                .is_supported_gpu_power()
                .inspect_err(|e| {
                    common::clog!(
                        "AMD GPU '{}' power telemetry unavailable at initialization ({e:?})",
                        adapter.name
                    );
                })
                .unwrap_or(false);
            let usage_supported = supported_metrics
                .is_supported_gpu_usage()
                .inspect_err(|e| {
                    common::clog!(
                        "AMD GPU '{}' usage telemetry unavailable at initialization ({e:?})",
                        adapter.name
                    );
                })
                .unwrap_or(false);
            let vram_supported = supported_metrics
                .is_supported_gpu_vram()
                .inspect_err(|e| {
                    common::clog!(
                        "AMD GPU '{}' VRAM telemetry unavailable at initialization ({e:?})",
                        adapter.name
                    );
                })
                .unwrap_or(false);

            if !power_supported && !usage_supported {
                return Err(SensorError::ReadError(format!(
                    "⚠ AMD GPU '{}' power and usage telemetry unavailable at initialization, sensor cannot be created",
                    adapter.name
                )));
            }

            let telemetry_mode = if power_supported {
                "measured power"
            } else if usage_supported {
                "usage-only (energy will be estimated)"
            } else {
                "none"
            };
            common::clog!(
                "AMD GPU '{}' telemetry mode: {telemetry_mode} (vram_supported={vram_supported})",
                adapter.name
            );

            Ok(AmdGPUSensor {
                gpu,
                _helper: helper,
                perfo,
                last_reading: Cell::new(Instant::now()),
                vram_supported,
                power_supported,
                usage_supported,
            })
        }

        pub fn is_integrated(&self) -> bool {
            if let Ok(gpu_type) = self.gpu.type_() {
                if gpu_type == adlx::ffi::ADLX_GPU_TYPE_GPUTYPE_INTEGRATED {
                    return true;
                }
            }
            false
        }
    }

    impl Sensor for AmdGPUSensor {
        fn read_full_data(&self) -> Result<SensorData, SensorError> {
            let now = Instant::now();
            let duration = now.duration_since(self.last_reading.get()).as_secs_f64().max(0.001);
            let gpu_metrics = self
                .perfo
                .current_gpu_metrics(&self.gpu)
                .map_err(|e| SensorError::ReadError(e.to_string()))?;

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

            self.last_reading.set(now);

            Ok(data.into())
        }
    }
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
mod nvidia_gpu {
    use std::{cell::Cell, collections::HashMap, time::Instant};

    use common::types::{EnergyUj, GPUData, SensorData};
    use nvml_wrapper::{Nvml, enum_wrappers::device::Sampling, enums::device::SampleValue};

    use super::{GpuAdapter, Sensor, SensorError};

    #[derive(Clone, Debug, PartialEq)]
    pub struct NvmlCandidate {
        pub index: u32,
        pub name: String,
        pub bus_id: String,
        pub pci_device_id: u32,
    }

    pub fn match_nvml_device(adapter: &GpuAdapter, candidates: &[NvmlCandidate]) -> Option<u32> {
        if let Some(ref bus_id) = adapter.pci_bus_id {
            let bus_id_lower = bus_id.to_lowercase();
            for c in candidates {
                if c.bus_id.to_lowercase() == bus_id_lower {
                    return Some(c.index);
                }
            }
        }

        if adapter.device_id != 0 {
            for c in candidates {
                let nvml_dev_id = (c.pci_device_id >> 16) & 0xFFFF;
                if nvml_dev_id == adapter.device_id {
                    return Some(c.index);
                }
            }
        }

        for c in candidates {
            if super::names_match(&adapter.name, &c.name) {
                return Some(c.index);
            }
        }
        None
    }

    enum NvidiaReadMode {
        EnergyCounter,
        PowerInstant,
        PowerSamples,
        UsageOnly,
    }

    pub struct NvidiaGPUSensor {
        nvml: Nvml,
        device_index: u32,
        mode: NvidiaReadMode,
        last_timestamp: Cell<u64>,
        last_energy_mj: Cell<u64>,
        last_power_instant: Cell<Option<Instant>>,
        last_sample_timestamp: Cell<Option<u64>>,
    }

    impl NvidiaGPUSensor {
        pub fn new(adapter: &GpuAdapter, fallback_index: u32) -> Result<Self, SensorError> {
            let nvml = Nvml::init().map_err(|e| SensorError::ReadError(e.to_string()))?;
            let device_count = nvml.device_count().map_err(|e| SensorError::ReadError(e.to_string()))?;

            let mut candidates = Vec::new();
            for i in 0..device_count {
                if let Ok(device) = nvml.device_by_index(i) {
                    let name = device.name().unwrap_or_default();
                    let pci_info = device.pci_info().ok();
                    let bus_id = pci_info.as_ref().map(|p| p.bus_id.clone()).unwrap_or_default();
                    let pci_device_id = pci_info.as_ref().map(|p| p.pci_device_id).unwrap_or(0);
                    candidates.push(NvmlCandidate {
                        index: i,
                        name,
                        bus_id,
                        pci_device_id,
                    });
                }
            }

            let resolved_index = match_nvml_device(adapter, &candidates).or_else(|| {
                if (fallback_index as usize) < candidates.len() {
                    common::clog!(
                        "NVIDIA GPU '{}': identity matching inconclusive, falling back to NVML device at index {}",
                        adapter.name,
                        fallback_index
                    );
                    Some(fallback_index)
                } else {
                    None
                }
            });

            let device_index = match resolved_index {
                Some(idx) => {
                    let cand = candidates.iter().find(|c| c.index == idx);
                    let cand_name = cand.map(|c| c.name.as_str()).unwrap_or("Unknown");
                    let cand_bus = cand.map(|c| c.bus_id.as_str()).unwrap_or("Unknown");
                    let luid_str = adapter
                        .luid
                        .map(|l| format!("0x{:016x}", l))
                        .unwrap_or_else(|| "none".to_string());
                    common::clog!(
                        "✓ NVIDIA adapter '{}' (LUID: {}) mapped to NVML device {} ('{}', Bus ID: {})",
                        adapter.name,
                        luid_str,
                        idx,
                        cand_name,
                        cand_bus
                    );
                    idx
                }
                None => {
                    return Err(SensorError::ReadError(format!(
                        "NVIDIA GPU '{}' could not be resolved in NVML ({} device(s) reported)",
                        adapter.name, device_count
                    )));
                }
            };

            let device = nvml
                .device_by_index(device_index)
                .map_err(|e| SensorError::ReadError(e.to_string()))?;

            let usage_supported = device.utilization_rates().is_ok();

            let mode = if device.total_energy_consumption().is_ok() {
                common::clog!(
                    "NVIDIA GPU '{}' (device {}): using energy counter mode",
                    adapter.name,
                    device_index
                );
                NvidiaReadMode::EnergyCounter
            } else if device.power_usage().is_ok() {
                common::clog!(
                    "NVIDIA GPU '{}' (device {}): energy counter unavailable, using power instant mode",
                    adapter.name,
                    device_index
                );
                NvidiaReadMode::PowerInstant
            } else if device.samples(Sampling::Power, None).is_ok_and(|s| !s.is_empty()) {
                common::clog!(
                    "NVIDIA GPU '{}' (device {}): instant power unavailable, falling back to sampled power (driver-side estimate)",
                    adapter.name,
                    device_index
                );
                NvidiaReadMode::PowerSamples
            } else if usage_supported {
                common::clog!(
                    "NVIDIA GPU '{}' (device {}): no power telemetry available, will be estimated from usage only",
                    adapter.name,
                    device_index
                );
                NvidiaReadMode::UsageOnly
            } else {
                return Err(SensorError::ReadError(format!(
                    "⚠ NVIDIA GPU '{}' (device {}): no power telemetry available and usage telemetry unavailable, sensor cannot be created",
                    adapter.name, device_index
                )));
            };

            Ok(NvidiaGPUSensor {
                nvml,
                device_index,
                mode,
                last_timestamp: Cell::new(0),
                last_energy_mj: Cell::new(0),
                last_power_instant: Cell::new(None),
                last_sample_timestamp: Cell::new(None),
            })
        }

        pub fn get_processes_gpu_usage(&self, current_timestamp: u64) -> Result<HashMap<u32, f64>, SensorError> {
            let last_ts = self.last_timestamp.get();
            if last_ts == 0 {
                self.last_timestamp.set(current_timestamp);
                return Ok(HashMap::new());
            }
            let device = self
                .nvml
                .device_by_index(self.device_index)
                .map_err(|e| SensorError::ReadError(e.to_string()))?;
            let processes = device.process_utilization_stats(last_ts);
            self.last_timestamp.set(current_timestamp);
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
            // CRITICAL: If device not reachable, return error
            let device = self
                .nvml
                .device_by_index(self.device_index)
                .map_err(|e| SensorError::ReadError(e.to_string()))?;

            let now = Instant::now();

            // NON-CRITICAL: Can still read usage if power fails
            let energy = match self.mode {
                NvidiaReadMode::UsageOnly => None,

                NvidiaReadMode::EnergyCounter => device
                    .total_energy_consumption()
                    .inspect_err(|e| {
                        common::logging::log_component_error("NVML", &format!("Failed to read energy counter: {e}"))
                    })
                    .ok()
                    .map(|current_energy_mj| {
                        let last_mj = self.last_energy_mj.get();
                        let energy_mj = if last_mj == 0 {
                            0
                        } else {
                            current_energy_mj.saturating_sub(last_mj)
                        };
                        self.last_energy_mj.set(current_energy_mj);
                        EnergyUj::from_millijoules(energy_mj)
                    }),

                NvidiaReadMode::PowerInstant => device
                    .power_usage()
                    .inspect_err(|e| {
                        common::logging::log_component_error("NVML", &format!("Failed to read power usage: {e}"))
                    })
                    .ok()
                    .map(|mw| {
                        let last_instant = self.last_power_instant.get();
                        let energy = match last_instant {
                            None => EnergyUj::from_joules(0.0),
                            Some(last) => {
                                let dt = now.duration_since(last).as_secs_f64();
                                EnergyUj::from_joules(mw as f64 / 1000.0 * dt)
                            }
                        };
                        self.last_power_instant.set(Some(now));
                        energy
                    }),

                NvidiaReadMode::PowerSamples => {
                    let last_ts = self.last_sample_timestamp.get();
                    device
                        .samples(Sampling::Power, last_ts)
                        .inspect_err(|e| {
                            common::logging::log_component_error("NVML", &format!("Failed to fetch power samples: {e}"))
                        })
                        .ok()
                        .and_then(|samples| {
                            if samples.is_empty() {
                                Some(EnergyUj::from_joules(0.0))
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

                                self.last_sample_timestamp
                                    .set(samples.last().map(|s| s.timestamp).or(last_ts));

                                let last_instant = self.last_power_instant.get();
                                let dt = now.duration_since(last_instant.unwrap_or(now)).as_secs_f64();
                                self.last_power_instant.set(Some(now));
                                Some(EnergyUj::from_joules((avg_mw / 1000.0) * dt))
                            }
                        })
                }
            };

            // NON-CRITICAL: Evaluate utilization rates with logging
            let utilization = device
                .utilization_rates()
                .inspect_err(|e| {
                    common::logging::log_component_error("NVML", &format!("Failed to read utilization rates: {e}"))
                })
                .ok();

            let usage_percent = utilization.as_ref().map(|u| u.gpu as f64);
            let vram_usage_percent = utilization.as_ref().map(|u| u.memory as f64);

            // CRITICAL ERROR CHECK: Only fail if BOTH energy and usage failed
            if energy.is_none() && usage_percent.is_none() {
                return Err(SensorError::ReadError(format!(
                    "⚠ NVIDIA GPU (device {}): Neither energy nor usage telemetry could be retrieved",
                    self.device_index
                )));
            }

            Ok(GPUData {
                total_energy: energy,
                usage_percent,
                vram_usage_percent,
                name: None,
            }
            .into())
        }
    }
}

#[cfg(target_os = "windows")]
mod intel_gpu {
    use std::{cell::Cell, mem::MaybeUninit, slice};

    use common::types::{GPUData, SensorData};
    use windows::{
        Win32::System::Performance::{
            PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY, PdhAddEnglishCounterW,
            PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW, PdhOpenQueryW,
        },
        core::PCWSTR,
    };

    use super::{GpuAdapter, Sensor, SensorError};

    const PDH_MORE_DATA: u32 = 0x800007D2;

    pub fn instance_matches_luid(instance_name: &str, luid: u64) -> bool {
        let high = (luid >> 32) as u32;
        let low = (luid & 0xFFFF_FFFF) as u32;
        let pattern = format!("luid_0x{:08x}_0x{:08x}", high, low);
        instance_name.to_lowercase().contains(&pattern)
    }

    pub struct IntelGPUSensor {
        query: PDH_HQUERY,
        counter: PDH_HCOUNTER,
        luid: Option<u64>,
        initialized: Cell<bool>,
    }

    impl IntelGPUSensor {
        pub fn new(adapter: &GpuAdapter) -> Result<Self, SensorError> {
            unsafe {
                let mut query = MaybeUninit::<PDH_HQUERY>::uninit();
                if PdhOpenQueryW(None, 0, query.as_mut_ptr()) != 0 {
                    return Err(SensorError::ReadError("PdhOpenQuery failed".to_string()));
                }
                let query = query.assume_init();

                let path: Vec<u16> = "\\GPU Engine(*)\\Utilization Percentage\0".encode_utf16().collect();
                let mut counter = MaybeUninit::<PDH_HCOUNTER>::uninit();
                if PdhAddEnglishCounterW(query, PCWSTR(path.as_ptr()), 0, counter.as_mut_ptr()) != 0 {
                    let _ = PdhCloseQuery(query);
                    return Err(SensorError::ReadError("PdhAddEnglishCounter failed".to_string()));
                }
                let counter = counter.assume_init();

                let luid_str = adapter
                    .luid
                    .map(|l| format!("0x{:016x}", l))
                    .unwrap_or_else(|| "none".to_string());
                common::clog!(
                    "✓ Intel adapter '{}' (LUID: {}) mapped to PDH \\GPU Engine(*) filtered by LUID",
                    adapter.name,
                    luid_str
                );

                Ok(IntelGPUSensor {
                    query,
                    counter,
                    luid: adapter.luid,
                    initialized: Cell::new(false),
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
                let raw_items = slice::from_raw_parts(items, count as usize);
                let valid_items: Vec<_> = raw_items
                    .iter()
                    .filter(|i| i.FmtValue.CStatus == 0)
                    .filter_map(|i| {
                        let v = i.FmtValue.Anonymous.doubleValue;
                        if v.is_finite() && !i.szName.is_null() {
                            let name = String::from_utf16_lossy(i.szName.as_wide());
                            Some((name, v))
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut max = 0.0f64;
                let mut matched_luid = false;
                if let Some(target_luid) = self.luid {
                    for (inst_name, val) in &valid_items {
                        if instance_matches_luid(inst_name, target_luid) {
                            max = max.max(*val);
                            matched_luid = true;
                        }
                    }
                }

                // If LUID didn't match any instance or no LUID provided, fall back to global max across all instances
                if !matched_luid {
                    for (_, val) in &valid_items {
                        max = max.max(*val);
                    }
                }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nvidia_match_by_bus_id() {
        let adapter = GpuAdapter {
            name: "NVIDIA GeForce RTX 4080".to_string(),
            vendor: GPUVendor::Nvidia,
            vendor_id: 0x10DE,
            device_id: 0x2704,
            luid: Some(0x12345),
            pci_bus_id: Some("0000:01:00.0".to_string()),
        };

        let candidates = vec![
            nvidia_gpu::NvmlCandidate {
                index: 0,
                name: "NVIDIA GeForce RTX 4090".to_string(),
                bus_id: "0000:02:00.0".to_string(),
                pci_device_id: 0x268410DE,
            },
            nvidia_gpu::NvmlCandidate {
                index: 1,
                name: "NVIDIA GeForce RTX 4080".to_string(),
                bus_id: "0000:01:00.0".to_string(),
                pci_device_id: 0x270410DE,
            },
        ];

        let matched = nvidia_gpu::match_nvml_device(&adapter, &candidates);
        assert_eq!(matched, Some(1));
    }

    #[test]
    fn test_nvidia_match_by_device_id_in_reversed_order() {
        let adapter = GpuAdapter {
            name: "NVIDIA GeForce RTX 3070".to_string(),
            vendor: GPUVendor::Nvidia,
            vendor_id: 0x10DE,
            device_id: 0x2484,
            luid: Some(0x9999),
            pci_bus_id: None,
        };

        let candidates = vec![
            nvidia_gpu::NvmlCandidate {
                index: 0,
                name: "NVIDIA GeForce RTX 3080".to_string(),
                bus_id: "0000:01:00.0".to_string(),
                pci_device_id: 0x220610DE,
            },
            nvidia_gpu::NvmlCandidate {
                index: 1,
                name: "NVIDIA GeForce RTX 3070".to_string(),
                bus_id: "0000:02:00.0".to_string(),
                pci_device_id: 0x248410DE,
            },
        ];

        let matched = nvidia_gpu::match_nvml_device(&adapter, &candidates);
        assert_eq!(matched, Some(1));
    }

    #[test]
    fn test_nvidia_match_fails_when_unmatched() {
        let adapter = GpuAdapter {
            name: "NVIDIA Unknown Card".to_string(),
            vendor: GPUVendor::Nvidia,
            vendor_id: 0x10DE,
            device_id: 0x9999,
            luid: None,
            pci_bus_id: None,
        };

        let candidates = vec![nvidia_gpu::NvmlCandidate {
            index: 0,
            name: "NVIDIA GeForce RTX 3080".to_string(),
            bus_id: "0000:01:00.0".to_string(),
            pci_device_id: 0x220610DE,
        }];

        let matched = nvidia_gpu::match_nvml_device(&adapter, &candidates);
        assert_eq!(matched, None);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_pdh_instance_luid_filtering() {
        let luid: u64 = 0x00000000_0001b375;
        let matching_sample = "pid_11088_luid_0x00000000_0x0001b375_phys_0_eng_0_engtype_3d";
        let other_luid_sample = "pid_11088_luid_0x00000000_0x0002c486_phys_0_eng_0_engtype_3d";

        assert!(intel_gpu::instance_matches_luid(matching_sample, luid));
        assert!(!intel_gpu::instance_matches_luid(other_luid_sample, luid));
    }
}
