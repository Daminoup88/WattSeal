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

    let factory: IDXGIFactory1 = match unsafe { CreateDXGIFactory1() } {
        Ok(factory) => factory,
        Err(error) => {
            common::clog!("Failed to create DXGI factory: {error:?}");
            return Vec::new();
        }
    };

    let mut adapters = Vec::new();
    let mut seen_luids = HashSet::new();
    let mut index = 0;

    loop {
        let adapter = match unsafe { factory.EnumAdapters1(index) } {
            Ok(adapter) => adapter,
            Err(_) => break,
        };
        index += 1;

        let desc = match unsafe { adapter.GetDesc1() } {
            Ok(desc) => desc,
            Err(_) => continue,
        };

        let luid = ((desc.AdapterLuid.HighPart as u32 as u64) << 32) | u64::from(desc.AdapterLuid.LowPart);

        // Ignore duplicate DXGI adapter identities.
        if !seen_luids.insert(luid) {
            continue;
        }

        let name = String::from_utf16_lossy(&desc.Description);
        let name = name.trim_end_matches('\0').trim();

        // Ignore software renderers and unnamed adapters.
        if name.is_empty()
            || name.to_ascii_lowercase().contains("microsoft basic render driver")
            || desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0
        {
            continue;
        }

        let mut vendor = GPUVendor::from_pci_vendor_id(desc.VendorId);
        if vendor == GPUVendor::Other {
            vendor = GPUVendor::from_str(name);
        }

        // Exclude software and indirect-display adapters.
        match query_d3dkmt_adapter_type(desc.AdapterLuid) {
            Ok(flags) => {
                common::clog!(
                    "GPU adapter '{}' (vendor {:?}, LUID 0x{luid:016x}) D3DKMT flags: 0x{flags:08x}",
                    name,
                    vendor
                );

                if is_software_or_indirect_adapter(flags) {
                    common::clog!(
                        "Skipping software/indirect-display adapter '{}' (LUID 0x{luid:016x})",
                        name
                    );
                    continue;
                }
            }
            Err(status) => {
                common::clog!(
                    "Could not classify GPU adapter '{}' (LUID 0x{luid:016x}): \
             NTSTATUS 0x{status:08x}; keeping adapter",
                    name
                );
            }
        }

        adapters.push(GpuAdapter {
            name: name.to_string(),
            vendor,
            vendor_id: desc.VendorId,
            device_id: desc.DeviceId,
            luid: Some(luid),
            pci_bus_id: None,
        });
    }

    adapters
}

#[cfg(target_os = "windows")]
fn query_d3dkmt_adapter_type(luid: windows::Win32::Foundation::LUID) -> Result<u32, i32> {
    use windows::Wdk::Graphics::Direct3D::{
        D3DKMT_ADAPTERTYPE, D3DKMT_CLOSEADAPTER, D3DKMT_OPENADAPTERFROMLUID, D3DKMT_QUERYADAPTERINFO,
        D3DKMTCloseAdapter, D3DKMTOpenAdapterFromLuid, D3DKMTQueryAdapterInfo, KMTQAITYPE_ADAPTERTYPE,
    };

    unsafe {
        let mut open = D3DKMT_OPENADAPTERFROMLUID {
            AdapterLuid: luid,
            hAdapter: 0,
        };

        let status = D3DKMTOpenAdapterFromLuid(&mut open);
        if status.0 < 0 {
            return Err(status.0);
        }

        let mut adapter_type = D3DKMT_ADAPTERTYPE::default();

        let mut query = D3DKMT_QUERYADAPTERINFO {
            hAdapter: open.hAdapter,
            Type: KMTQAITYPE_ADAPTERTYPE,
            pPrivateDriverData: std::ptr::from_mut(&mut adapter_type).cast(),
            PrivateDriverDataSize: std::mem::size_of::<D3DKMT_ADAPTERTYPE>() as u32,
        };

        let query_status = D3DKMTQueryAdapterInfo(&mut query);

        let result = if query_status.0 < 0 {
            Err(query_status.0)
        } else {
            Ok(adapter_type.Anonymous.Value)
        };

        let close_status = D3DKMTCloseAdapter(&D3DKMT_CLOSEADAPTER {
            hAdapter: open.hAdapter,
        });

        if close_status.0 < 0 {
            common::clog!(
                "Failed to close D3DKMT adapter handle: NTSTATUS 0x{:08x}",
                close_status.0
            );
        }

        result
    }
}

#[cfg(target_os = "windows")]
fn is_software_or_indirect_adapter(flags: u32) -> bool {
    const SOFTWARE_DEVICE: u32 = 1 << 2;
    const INDIRECT_DISPLAY_DEVICE: u32 = 1 << 6;

    flags & (SOFTWARE_DEVICE | INDIRECT_DISPLAY_DEVICE) != 0
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
    #[cfg(any(target_os = "windows", target_os = "linux"))]
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
            #[cfg(any(target_os = "windows", target_os = "linux"))]
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
            GPUVendor::Amd => amd_gpu::AmdGPUSensor::new(adapter).map(GPUSensorInner::Amd),
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
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            GPUSensorInner::Amd(_) => Err(SensorError::NotSupported),
            #[cfg(target_os = "windows")]
            GPUSensorInner::Intel { .. } => Err(SensorError::NotSupported),
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

#[cfg(target_os = "linux")]
mod amd_gpu {
    use std::{
        cell::Cell,
        fs,
        path::{Path, PathBuf},
        time::Instant,
    };

    use common::types::{EnergyUj, GPUData, SensorData};

    use crate::sensors::{GpuAdapter, Sensor, SensorError};

    pub struct AmdGPUSensor {
        device_path: PathBuf,
        power_paths: Vec<PathBuf>,
        energy_paths: Vec<PathBuf>,
        last_energy_reading: Cell<Option<u64>>,
        last_reading: Cell<Option<Instant>>,
    }

    impl AmdGPUSensor {
        pub fn new(adapter: &GpuAdapter) -> Result<Self, SensorError> {
            let pci_address = adapter.pci_bus_id.as_deref().ok_or(SensorError::NotSupported)?;
            let device_path = Path::new("/sys/bus/pci/devices").join(pci_address);
            let driver = fs::read_link(device_path.join("driver")).map_err(|_| SensorError::NotSupported)?;
            if driver.file_name().and_then(|name| name.to_str()) != Some("amdgpu") {
                return Err(SensorError::NotSupported);
            }
            let device_path = fs::canonicalize(device_path).map_err(|_| SensorError::NotSupported)?;
            Self::new_with_paths(device_path, pci_address.to_string())
        }

        fn new_with_paths(device_path: PathBuf, _pci_address: String) -> Result<Self, SensorError> {
            let power_paths = find_power_sensors(&device_path);
            let energy_paths = find_energy_sensors(&device_path);
            let usage_path = device_path.join("gpu_busy_percent");

            if power_paths.is_empty() && energy_paths.is_empty() && !usage_path.exists() {
                return Err(SensorError::NotSupported);
            }

            Ok(Self {
                device_path,
                power_paths,
                energy_paths,
                last_energy_reading: Cell::new(None),
                last_reading: Cell::new(None),
            })
        }
    }

    impl Sensor for AmdGPUSensor {
        fn read_full_data(&self) -> Result<SensorData, SensorError> {
            let now = Instant::now();

            let power_uw = self
                .power_paths
                .iter()
                .find_map(|path| read_number(path).filter(|power| *power >= 0.0));

            let counter_uj = self
                .energy_paths
                .iter()
                .find_map(|path| fs::read_to_string(path).ok()?.trim().parse::<u64>().ok());

            let counter_energy = counter_uj.and_then(|current| {
                let previous = self.last_energy_reading.replace(Some(current))?;
                current.checked_sub(previous).map(EnergyUj::from_u64)
            });

            let power_energy = power_uw.and_then(|power| {
                let previous = self.last_reading.replace(Some(now))?;
                let duration = now.duration_since(previous).as_secs_f64();

                // Microwatts multiplied by seconds give microjoules.
                Some(EnergyUj::from_joules(power * duration / 1_000_000.0))
            });

            let total_energy = if let Some(energy) = counter_energy {
                Some(energy)
            } else {
                if counter_uj.is_none() && power_energy.is_some() {
                    // The power estimate covers this interval. Rebaseline the
                    // counter on recovery to avoid counting it twice.
                    self.last_energy_reading.set(None);
                }

                power_energy
            };

            let usage_percent =
                read_number(&self.device_path.join("gpu_busy_percent")).filter(|value| (0.0..=100.0).contains(value));

            if total_energy.is_none() && power_uw.is_none() && counter_uj.is_none() && usage_percent.is_none() {
                return Err(SensorError::ReadError(
                    "AMD GPU power and utilization telemetry unavailable".to_string(),
                ));
            }

            let vram_used = read_number(&self.device_path.join("mem_info_vram_used"));

            let vram_total = read_number(&self.device_path.join("mem_info_vram_total"));

            let vram_usage_percent = match (vram_used, vram_total) {
                (Some(used), Some(total)) if total > 0.0 => Some((used / total * 100.0).clamp(0.0, 100.0)),
                _ => None,
            };

            Ok(GPUData {
                total_energy,
                usage_percent,
                vram_usage_percent,
                name: None,
            }
            .into())
        }
    }

    fn read_number(path: &Path) -> Option<f64> {
        let value = fs::read_to_string(path).ok()?;
        let number = value.trim().parse::<f64>().ok()?;
        number.is_finite().then_some(number)
    }

    fn find_energy_sensors(device_path: &Path) -> Vec<PathBuf> {
        let Ok(entries) = fs::read_dir(device_path.join("hwmon")) else {
            return Vec::new();
        };

        let mut energy_paths = Vec::new();

        for entry in entries.flatten() {
            let hwmon_path = entry.path();

            if fs::read_to_string(hwmon_path.join("name"))
                .ok()
                .is_none_or(|name| name.trim() != "amdgpu")
            {
                continue;
            }

            let Ok(files) = fs::read_dir(hwmon_path) else {
                continue;
            };

            for file in files.flatten() {
                let filename = file.file_name();
                let filename = filename.to_string_lossy();

                if let Some(index) = filename
                    .strip_prefix("energy")
                    .and_then(|name| name.strip_suffix("_input"))
                {
                    if !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit()) {
                        energy_paths.push(file.path());
                    }
                }
            }
        }

        energy_paths.sort();
        energy_paths
    }

    fn find_power_sensors(device_path: &Path) -> Vec<PathBuf> {
        let Ok(entries) = fs::read_dir(device_path.join("hwmon")) else {
            return Vec::new();
        };

        let mut average_paths = Vec::new();
        let mut input_paths = Vec::new();

        for entry in entries.flatten() {
            let hwmon_path = entry.path();

            if fs::read_to_string(hwmon_path.join("name"))
                .ok()
                .is_none_or(|name| name.trim() != "amdgpu")
            {
                continue;
            }

            let average_path = hwmon_path.join("power1_average");
            if average_path.exists() {
                average_paths.push(average_path);
            }

            let input_path = hwmon_path.join("power1_input");
            if input_path.exists() {
                input_paths.push(input_path);
            }
        }

        average_paths.extend(input_paths);
        average_paths
    }

    #[cfg(test)]
    mod tests {
        use std::{
            fs,
            path::PathBuf,
            sync::atomic::{AtomicU64, Ordering},
        };

        use super::{AmdGPUSensor, find_power_sensors, read_number};
        use crate::{database::SensorData, sensors::Sensor};

        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        fn temp_file() -> PathBuf {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

            std::env::temp_dir().join(format!("wattseal-amdgpu-test-{}-{id}", std::process::id()))
        }

        #[test]
        fn recovers_after_both_energy_sources_fail() {
            let device_path = temp_file();
            let hwmon_path = device_path.join("hwmon/hwmon0");

            fs::create_dir_all(&hwmon_path).unwrap();
            fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();

            let energy_path = hwmon_path.join("energy1_input");
            let power_path = hwmon_path.join("power1_average");

            fs::write(&energy_path, "10000000\n").unwrap();
            fs::write(&power_path, "100000000\n").unwrap();
            fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

            let sensor = AmdGPUSensor::new_with_paths(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

            sensor.read_full_data().unwrap();

            fs::write(&energy_path, "invalid\n").unwrap();
            fs::write(&power_path, "invalid\n").unwrap();

            match sensor.read_full_data().unwrap() {
                SensorData::GPU(data) => assert!(data.total_energy.is_none()),
                _ => panic!("Expected GPU sensor data"),
            }

            assert_eq!(sensor.last_energy_reading.get(), Some(10_000_000));

            fs::write(&power_path, "100000000\n").unwrap();

            sensor
                .last_reading
                .set(Some(std::time::Instant::now() - std::time::Duration::from_secs(2)));

            match sensor.read_full_data().unwrap() {
                SensorData::GPU(data) => {
                    let joules = data.total_energy.expect("Expected power fallback").as_joules();

                    assert!(
                        (200.0..205.0).contains(&joules),
                        "Expected approximately 200 J, got {joules} J"
                    );
                }
                _ => panic!("Expected GPU sensor data"),
            }

            assert_eq!(sensor.last_energy_reading.get(), None);

            fs::write(&energy_path, "13000000\n").unwrap();

            sensor.read_full_data().unwrap();
            assert_eq!(sensor.last_energy_reading.get(), Some(13_000_000));

            fs::write(&energy_path, "15000000\n").unwrap();

            match sensor.read_full_data().unwrap() {
                SensorData::GPU(data) => {
                    let joules = data.total_energy.expect("Expected counter energy").as_joules();

                    assert_eq!(joules, 2.0);
                }
                _ => panic!("Expected GPU sensor data"),
            }

            fs::remove_dir_all(device_path).unwrap();
        }

        #[test]
        fn falls_back_to_power_when_energy_counter_fails() {
            let device_path = temp_file();
            let hwmon_path = device_path.join("hwmon/hwmon0");

            fs::create_dir_all(&hwmon_path).unwrap();
            fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();

            let energy_path = hwmon_path.join("energy1_input");
            fs::write(&energy_path, "10000000\n").unwrap();
            fs::write(hwmon_path.join("power1_average"), "100000000\n").unwrap();
            fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

            let sensor = AmdGPUSensor::new_with_paths(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

            sensor.read_full_data().unwrap();

            fs::write(&energy_path, "invalid\n").unwrap();

            sensor
                .last_reading
                .set(Some(std::time::Instant::now() - std::time::Duration::from_secs(2)));

            match sensor.read_full_data().unwrap() {
                SensorData::GPU(data) => {
                    let joules = data.total_energy.expect("Expected power fallback").as_joules();
                    assert!(
                        (200.0..205.0).contains(&joules),
                        "Expected approximately 200 J from power, got {joules} J"
                    );
                }
                _ => panic!("Expected GPU sensor data"),
            }

            assert_eq!(sensor.last_energy_reading.get(), None);

            fs::write(&energy_path, "13000000\n").unwrap();

            // The first recovered reading establishes a new counter baseline.
            sensor.read_full_data().unwrap();
            assert_eq!(sensor.last_energy_reading.get(), Some(13_000_000));

            fs::write(&energy_path, "15000000\n").unwrap();

            match sensor.read_full_data().unwrap() {
                SensorData::GPU(data) => {
                    let energy = data.total_energy.expect("Expected recovered counter energy");
                    assert_eq!(energy.as_joules(), 2.0);
                }
                _ => panic!("Expected GPU sensor data"),
            }

            fs::remove_dir_all(device_path).unwrap();
        }

        #[test]
        fn handles_energy_counter_reset_with_power_available() {
            let device_path = temp_file();
            let hwmon_path = device_path.join("hwmon/hwmon0");

            fs::create_dir_all(&hwmon_path).unwrap();
            fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();

            let energy_path = hwmon_path.join("energy1_input");
            fs::write(&energy_path, "10000000\n").unwrap();
            fs::write(hwmon_path.join("power1_average"), "100000000\n").unwrap();
            fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

            let sensor = AmdGPUSensor::new_with_paths(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

            sensor.read_full_data().unwrap();

            fs::write(&energy_path, "1000000\n").unwrap();

            sensor
                .last_reading
                .set(Some(std::time::Instant::now() - std::time::Duration::from_secs(2)));

            match sensor.read_full_data().unwrap() {
                SensorData::GPU(data) => {
                    let joules = data.total_energy.expect("Expected power fallback").as_joules();
                    assert!(
                        (200.0..205.0).contains(&joules),
                        "Expected approximately 200 J, got {joules} J"
                    );
                }
                _ => panic!("Expected GPU sensor data"),
            }

            assert_eq!(sensor.last_energy_reading.get(), Some(1_000_000));

            fs::write(&energy_path, "3000000\n").unwrap();

            match sensor.read_full_data().unwrap() {
                SensorData::GPU(data) => {
                    let energy = data.total_energy.expect("Expected counter energy");
                    assert_eq!(energy.as_joules(), 2.0);
                }
                _ => panic!("Expected GPU sensor data"),
            }

            fs::remove_dir_all(device_path).unwrap();
        }

        #[test]
        fn handles_cumulative_energy_counter_reset() {
            let device_path = temp_file();
            let hwmon_path = device_path.join("hwmon/hwmon0");

            fs::create_dir_all(&hwmon_path).unwrap();
            fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();

            let energy_path = hwmon_path.join("energy1_input");
            fs::write(&energy_path, "10000000\n").unwrap();
            fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

            let sensor = AmdGPUSensor::new_with_paths(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

            sensor.read_full_data().unwrap();

            fs::write(&energy_path, "1000000\n").unwrap();

            match sensor.read_full_data().unwrap() {
                SensorData::GPU(data) => assert!(data.total_energy.is_none()),
                _ => panic!("Expected GPU sensor data"),
            }

            fs::write(&energy_path, "3000000\n").unwrap();

            match sensor.read_full_data().unwrap() {
                SensorData::GPU(data) => {
                    let energy = data.total_energy.expect("Expected energy after counter reset");
                    assert_eq!(energy.as_joules(), 2.0);
                }
                _ => panic!("Expected GPU sensor data"),
            }

            fs::remove_dir_all(device_path).unwrap();
        }

        #[test]
        fn uses_cumulative_energy_counter_before_power_estimate() {
            let device_path = temp_file();
            let hwmon_path = device_path.join("hwmon/hwmon0");

            fs::create_dir_all(&hwmon_path).unwrap();
            fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();

            let energy_path = hwmon_path.join("energy1_input");
            fs::write(&energy_path, "10000000\n").unwrap();
            fs::write(hwmon_path.join("power1_average"), "100000000\n").unwrap();
            fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

            let sensor = AmdGPUSensor::new_with_paths(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

            assert_eq!(sensor.energy_paths, vec![energy_path.clone()]);

            match sensor.read_full_data().unwrap() {
                SensorData::GPU(data) => assert!(data.total_energy.is_none()),
                _ => panic!("Expected GPU sensor data"),
            }

            fs::write(&energy_path, "13000000\n").unwrap();

            sensor
                .last_reading
                .set(Some(std::time::Instant::now() - std::time::Duration::from_secs(2)));

            match sensor.read_full_data().unwrap() {
                SensorData::GPU(data) => {
                    let energy = data.total_energy.expect("Expected counter energy");
                    assert_eq!(energy.as_joules(), 3.0);
                }
                _ => panic!("Expected GPU sensor data"),
            }

            fs::remove_dir_all(device_path).unwrap();
        }

        #[test]
        fn first_power_sample_initializes_energy_measurement() {
            let device_path = temp_file();
            let hwmon_path = device_path.join("hwmon/hwmon0");

            fs::create_dir_all(&hwmon_path).unwrap();
            fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();
            fs::write(hwmon_path.join("power1_average"), "100000000\n").unwrap();
            fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

            let sensor = AmdGPUSensor::new_with_paths(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

            assert!(sensor.last_reading.get().is_none());

            let first = sensor.read_full_data().unwrap();

            match first {
                SensorData::GPU(data) => assert!(data.total_energy.is_none()),
                _ => panic!("Expected GPU sensor data"),
            }

            assert!(sensor.last_reading.get().is_some());

            std::thread::sleep(std::time::Duration::from_millis(20));

            let second = sensor.read_full_data().unwrap();

            match second {
                SensorData::GPU(data) => assert!(data.total_energy.is_some()),
                _ => panic!("Expected GPU sensor data"),
            }

            fs::remove_dir_all(device_path).unwrap();
        }

        #[test]
        fn preserves_energy_interval_after_power_read_failure() {
            let device_path = temp_file();
            let hwmon_path = device_path.join("hwmon/hwmon0");

            fs::create_dir_all(&hwmon_path).unwrap();
            fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();

            let power_path = hwmon_path.join("power1_average");
            fs::write(&power_path, "100000000\n").unwrap();
            fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

            let sensor = AmdGPUSensor::new_with_paths(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

            sensor.read_full_data().unwrap();
            let previous = sensor.last_reading.get().unwrap();

            fs::write(&power_path, "invalid\n").unwrap();

            let failed_power_sample = sensor.read_full_data().unwrap();

            match failed_power_sample {
                SensorData::GPU(data) => assert!(data.total_energy.is_none()),
                _ => panic!("Expected GPU sensor data"),
            }

            assert_eq!(sensor.last_reading.get(), Some(previous));

            sensor
                .last_reading
                .set(Some(std::time::Instant::now() - std::time::Duration::from_secs(2)));

            fs::write(&power_path, "100000000\n").unwrap();

            let recovered_sample = sensor.read_full_data().unwrap();

            match recovered_sample {
                SensorData::GPU(data) => {
                    let energy = data.total_energy.expect("Expected energy after recovery");
                    let joules = energy.as_joules();

                    assert!(
                        (200.0..205.0).contains(&joules),
                        "Expected approximately 200 J, got {joules} J"
                    );
                    assert!(sensor.last_reading.get().unwrap() > previous);
                }
                _ => panic!("Expected GPU sensor data"),
            }

            fs::remove_dir_all(device_path).unwrap();
        }

        #[test]
        fn falls_back_to_power_input_when_average_is_invalid() {
            let device_path = temp_file();
            let hwmon_path = device_path.join("hwmon/hwmon0");

            fs::create_dir_all(&hwmon_path).unwrap();
            fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();
            fs::write(hwmon_path.join("power1_average"), "invalid\n").unwrap();
            fs::write(hwmon_path.join("power1_input"), "80000000\n").unwrap();
            fs::write(device_path.join("gpu_busy_percent"), "25\n").unwrap();

            let sensor = AmdGPUSensor::new_with_paths(device_path.clone(), "ffff:ff:ff.f".to_string()).unwrap();

            assert_eq!(sensor.power_paths.len(), 2);

            let first = sensor.read_full_data().unwrap();
            match first {
                SensorData::GPU(data) => assert!(data.total_energy.is_none()),
                _ => panic!("Expected GPU sensor data"),
            }

            sensor
                .last_reading
                .set(Some(std::time::Instant::now() - std::time::Duration::from_secs(2)));

            let second = sensor.read_full_data().unwrap();
            match second {
                SensorData::GPU(data) => {
                    let energy = data.total_energy.expect("Expected fallback energy");
                    let joules = energy.as_joules();

                    assert!(
                        (160.0..165.0).contains(&joules),
                        "Expected approximately 160 J, got {joules} J"
                    );
                }
                _ => panic!("Expected GPU sensor data"),
            }

            fs::remove_dir_all(device_path).unwrap();
        }

        #[test]
        fn selects_power_sensor_with_fallback() {
            let device_path = temp_file();
            let hwmon_path = device_path.join("hwmon/hwmon0");

            fs::create_dir_all(&hwmon_path).unwrap();
            fs::write(hwmon_path.join("name"), "amdgpu\n").unwrap();

            let average = hwmon_path.join("power1_average");
            let input = hwmon_path.join("power1_input");

            assert!(find_power_sensors(&device_path).is_empty());

            fs::write(&input, "100000000\n").unwrap();
            assert_eq!(find_power_sensors(&device_path), vec![input.clone()]);

            fs::write(&average, "90000000\n").unwrap();
            assert_eq!(find_power_sensors(&device_path), vec![average, input]);

            fs::remove_dir_all(device_path).unwrap();
        }

        #[test]
        fn prefers_average_power_across_hwmon_directories() {
            let device_path = temp_file();
            let first_hwmon = device_path.join("hwmon/hwmon0");
            let second_hwmon = device_path.join("hwmon/hwmon1");

            fs::create_dir_all(&first_hwmon).unwrap();
            fs::create_dir_all(&second_hwmon).unwrap();
            fs::write(first_hwmon.join("name"), "amdgpu\n").unwrap();
            fs::write(second_hwmon.join("name"), "amdgpu\n").unwrap();

            let input = first_hwmon.join("power1_input");
            let average = second_hwmon.join("power1_average");
            fs::write(&input, "100000000\n").unwrap();
            fs::write(&average, "90000000\n").unwrap();

            let paths = find_power_sensors(&device_path);
            assert_eq!(paths, vec![average, input]);

            fs::remove_dir_all(device_path).unwrap();
        }

        #[test]
        fn reads_valid_sysfs_number() {
            let path = temp_file();
            fs::write(&path, "12345\n").unwrap();

            assert_eq!(read_number(&path), Some(12345.0));

            fs::remove_file(path).unwrap();
        }

        #[test]
        fn rejects_invalid_sysfs_number() {
            let path = temp_file();
            fs::write(&path, "not-a-number\\n").unwrap();

            assert_eq!(read_number(&path), None);

            fs::remove_file(path).unwrap();
        }

        #[test]
        fn handles_missing_sysfs_file() {
            let path = temp_file();

            assert_eq!(read_number(&path), None);
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

    pub fn max_utilization_for_luid(items: &[(String, f64)], luid: u64) -> Option<f64> {
        items
            .iter()
            .filter(|(instance_name, _)| instance_matches_luid(instance_name, luid))
            .map(|(_, value)| *value)
            .reduce(f64::max)
    }

    pub struct IntelGPUSensor {
        query: PDH_HQUERY,
        counter: PDH_HCOUNTER,
        luid: u64,
        initialized: Cell<bool>,
    }

    impl IntelGPUSensor {
        pub fn new(adapter: &GpuAdapter) -> Result<Self, SensorError> {
            let luid = adapter.luid.ok_or_else(|| {
                SensorError::ReadError(format!(
                    "Intel GPU '{}' has no DXGI LUID; skipping sensor",
                    adapter.name
                ))
            })?;

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

                common::clog!(
                    "✓ Intel adapter '{}' (LUID: 0x{:016x}) mapped to PDH \\GPU Engine(*) filtered by LUID",
                    adapter.name,
                    luid
                );

                Ok(IntelGPUSensor {
                    query,
                    counter,
                    luid,
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
                        usage_percent: None,
                        vram_usage_percent: None,
                        name: None,
                    }
                    .into());
                }
                let item_size = std::mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>();
                let item_capacity = (size as usize).div_ceil(item_size);
                let mut buf = Vec::<MaybeUninit<PDH_FMT_COUNTERVALUE_ITEM_W>>::with_capacity(item_capacity);
                buf.resize_with(item_capacity, MaybeUninit::uninit);
                let items = buf.as_mut_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>();
                if PdhGetFormattedCounterArrayW(self.counter, PDH_FMT_DOUBLE, &mut size, &mut count, Some(items)) != 0 {
                    return Ok(GPUData {
                        total_energy: None,
                        usage_percent: None,
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

                let usage_percent =
                    max_utilization_for_luid(&valid_items, self.luid).map(|value| value.clamp(0.0, 100.0));

                Ok(GPUData {
                    total_energy: None,
                    usage_percent,
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

    #[cfg(target_os = "windows")]
    #[test]
    fn pdh_utilization_requires_the_known_luid_and_aggregates_its_engines() {
        let target = "pid_1_luid_0x00000000_0x0000f9ef_phys_0_eng_0_engtype_3d".to_string();
        let target_other_engine = "pid_2_luid_0x00000000_0x0000f9ef_phys_0_eng_1_engtype_copy".to_string();
        let other = "pid_1_luid_0x00000000_0x0001d05b_phys_0_eng_0_engtype_3d".to_string();
        let items = vec![(target, 23.0), (target_other_engine, 41.0), (other, 97.0)];
        let target_luid = 0xf9ef;
        let absent_luid = 0x1234;

        assert_eq!(intel_gpu::max_utilization_for_luid(&items, target_luid), Some(41.0));
        assert_eq!(intel_gpu::max_utilization_for_luid(&items, absent_luid), None);
        assert_eq!(intel_gpu::max_utilization_for_luid(&[], target_luid), None);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn intel_sensor_requires_a_luid_before_opening_pdh() {
        let adapter = GpuAdapter {
            name: "Intel GPU".to_string(),
            vendor: GPUVendor::Intel,
            vendor_id: 0x8086,
            device_id: 0x1234,
            luid: None,
            pci_bus_id: None,
        };

        assert!(intel_gpu::IntelGPUSensor::new(&adapter).is_err());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn filters_software_and_indirect_adapters() {
        // Software device.
        assert!(is_software_or_indirect_adapter(1 << 2));

        // Indirect display device.
        assert!(is_software_or_indirect_adapter(1 << 6));

        // Both flags.
        assert!(is_software_or_indirect_adapter((1 << 2) | (1 << 6)));

        // Physical adapter.
        assert!(!is_software_or_indirect_adapter(0));

        // Paravirtualized alone should not be excluded.
        assert!(!is_software_or_indirect_adapter(1 << 7));
    }
}
