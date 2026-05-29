pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod network;
pub mod ram;

use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use battery::Manager;
use common::EnergyUJ;
pub use common::{
    AllTimeData, Event, GPUData, GeneralData, SensorData,
    types::{
        BatteryInfo, CpuInfo, DiskInfo, HardwareInfo, InitialInfo, MemoryInfo, ScreenInfo, SensorKind, SystemInfo,
    },
};
pub use cpu::CPUSensor;
pub use disk::DiskSensor;
use display_info::DisplayInfo;
pub use gpu::{GPUSensor, get_gpu_list};
pub use network::NetworkSensor;
pub use ram::RamSensor;
use sysinfo::System;

/// Variant wrapper for all supported sensor.
pub enum SensorType {
    CPU(CPUSensor),
    GPU(GPUSensor),
    RAM(RamSensor),
    Disk(DiskSensor),
    Network(NetworkSensor),
}

impl SensorType {
    /// Returns the sensor kind of this sensor variant.
    pub fn sensor_kind(&self) -> SensorKind {
        match self {
            SensorType::CPU(_) => SensorKind::CPU,
            SensorType::GPU(_) => SensorKind::GPU,
            SensorType::RAM(_) => SensorKind::Ram,
            SensorType::Disk(_) => SensorKind::Disk,
            SensorType::Network(_) => SensorKind::Network,
        }
    }
}

impl Sensor for SensorType {
    fn read_full_data(&self) -> Result<SensorData<EnergyUJ>, SensorError> {
        match self {
            SensorType::CPU(sensor) => sensor.read_full_data(),
            SensorType::GPU(sensor) => sensor.read_full_data(),
            SensorType::RAM(sensor) => sensor.read_full_data(),
            SensorType::Disk(sensor) => sensor.read_full_data(),
            SensorType::Network(sensor) => sensor.read_full_data(),
        }
    }

    fn read_initial_info(&self) -> Result<InitialInfo, SensorError> {
        match self {
            SensorType::CPU(sensor) => sensor.read_initial_info(),
            SensorType::GPU(sensor) => sensor.read_initial_info(),
            SensorType::RAM(sensor) => sensor.read_initial_info(),
            SensorType::Disk(sensor) => sensor.read_initial_info(),
            SensorType::Network(_) => Err(SensorError::NotSupported),
        }
    }

    fn read_name(&self) -> Result<String, SensorError> {
        match self {
            SensorType::CPU(sensor) => sensor.read_name(),
            SensorType::GPU(sensor) => sensor.read_name(),
            SensorType::Disk(sensor) => sensor.read_name(),
            SensorType::Network(sensor) => sensor.read_name(),
            SensorType::RAM(_) => Err(SensorError::NotSupported),
        }
    }
}

/// Common interface for hardware sensors.
pub trait Sensor {
    /// Reads current power, usage, and throughput data.
    fn read_full_data(&self) -> Result<SensorData<EnergyUJ>, SensorError>;
    /// Returns static hardware specs (model, capacity, etc.).
    fn read_initial_info(&self) -> Result<InitialInfo, SensorError> {
        Err(SensorError::NotSupported)
    }
    fn read_name(&self) -> Result<String, SensorError> {
        Err(SensorError::NotSupported)
    }
}

#[derive(Debug)]
pub enum SensorError {
    NotSupported,
    ReadError(String),
}

/// Aggregates readings from all sensors into a single timestamped event.
pub fn create_event_from_sensors(sensors: &Vec<SensorType>, duration: Duration) -> Event<EnergyUJ> {
    let time = SystemTime::now();
    let mut data: Vec<SensorData<EnergyUJ>> = Vec::new();

    let mut integrated_gpu_consumption: Option<EnergyUJ> = None;
    let mut has_pp1_source = false;
    let mut integrated_gpu_indices: Vec<usize> = Vec::new();
    for sensor in sensors {
        let sensor_data = sensor.read_full_data();
        match sensor_data {
            Ok(mut d) => {
                if let SensorData::CPU(ref mut cpu) = d {
                    if let Some(pp1) = cpu.pp1_consumption.take() {
                        has_pp1_source = true;
                        if pp1 == 0 {
                            if let Some(ref mut total) = cpu.total_consumption {
                                *total = *total - pp1;
                            }
                            integrated_gpu_consumption = Some(pp1);
                        }
                    }
                }

                // Track integrated Intel GPUs for estimation fallback.
                if let SensorType::GPU(gpu_sensor) = sensor {
                    if gpu_sensor.is_integrated() {
                        integrated_gpu_indices.push(data.len());
                    }
                }

                data.push(d);
            }
            #[cfg(debug_assertions)]
            Err(e) => eprintln!("✗ Error reading sensor data: {:?}", e),
            #[cfg(not(debug_assertions))]
            Err(_) => {}
        }
    }

    // --- Integrated-GPU power resolution ---
    // Priority 1: Real PP1 reading from MSR (Scaphandre driver).
    if let Some(igpu_consumption) = integrated_gpu_consumption {
        let merged = data.iter_mut().any(|d| {
            if let SensorData::GPU(gpu) = d {
                if gpu.total_consumption.is_none() {
                    gpu.total_consumption = Some(igpu_consumption);
                    return true;
                }
            }
            false
        });
        if !merged {
            data.push(SensorData::GPU(GPUData {
                total_consumption: Some(igpu_consumption),
                usage_percent: None,
                vram_usage_percent: None,
            }));
        }
    }

    // Priority 2: Estimate iGPU power from usage when PP1 is unavailable.
    if !has_pp1_source {
        for &idx in &integrated_gpu_indices {
            if let SensorData::GPU(ref mut gpu) = data[idx] {
                if gpu.total_consumption.is_none() {
                    if let Some(usage) = gpu.usage_percent {
                        let estimated = cpu::estimate_igpu_energy(usage, duration);
                        gpu.total_consumption = Some(estimated);
                    }
                }
            }
        }
    }

    return Event::new(time, data);
}

/// Collects hardware info (names + initial specs) from all sensors.
pub fn get_hardware_info(sensors: &Vec<SensorType>) -> GeneralData {
    let mut sensors_kind: Vec<SensorKind> = Vec::new();
    let mut detected_materials: Vec<String> = Vec::new();
    let mut sensors_info: Vec<InitialInfo> = Vec::new();

    for sensor in sensors {
        sensors_kind.push(sensor.sensor_kind()); // Problem general data with entry
        match sensor.read_name() {
            Ok(name) => detected_materials.push(name),
            Err(SensorError::NotSupported) => {}
            Err(e) => crate::clog!("✗ Failed to read sensor name for {}: {:?}", sensor.table_name(), e),
        }

        match sensor.read_initial_info() {
            Ok(info) => sensors_info.push(info),
            Err(SensorError::NotSupported) => {}
            Err(e) => crate::clog!(
                "✗ Failed to read initial info for sensor {}: {:?}",
                sensor.table_name(),
                e
            ),
        }
    }

    // System information
    let os_name = format!(
        "{} {}",
        System::name().unwrap_or_default(),
        System::os_version().unwrap_or_default()
    );
    let hostname = System::host_name().unwrap_or_default();

    let system_info = SystemInfo {
        os: os_name,
        hostname,
        is_virtual_machine: false,
    };
    sensors_info.push(InitialInfo::System(system_info));

    // Display info
    let display_infos = DisplayInfo::all().unwrap_or_default();
    let mut display_names = Vec::new();
    let mut screen_infos = Vec::new();
    for display_info in display_infos {
        let resolution = format!("{}x{}", display_info.width, display_info.height);
        let friendly_name = display_info.friendly_name.clone();
        display_names.push(friendly_name.clone());
        screen_infos.push(ScreenInfo {
            model: friendly_name,
            resolution: resolution,
            refresh_rate_hz: display_info.frequency as u32,
            is_primary: display_info.is_primary,
        });
    }
    detected_materials.push(format!("Display(s): [{}]", display_names.join(", ")));
    sensors_info.push(InitialInfo::Displays(screen_infos));

    // Battery info
    let battery_info = BatteryInfo {
        present: false,
        name: None,
        design_capacity_wh: None,
        full_charge_capacity_wh: None,
        cycle_count: None,
    };

    let mut battery_names: Vec<String> = Vec::new();
    let manager = Manager::new().unwrap();
    let battery_info = match manager.batteries() {
        Ok(mut batteries) => {
            if let Some(Ok(battery)) = batteries.next() {
                let battery_name = battery.vendor().map(|v| v.to_string());
                if let Some(ref name) = battery_name {
                    battery_names.push(name.clone());
                }
                BatteryInfo {
                    present: true,
                    name: battery_name,
                    design_capacity_wh: Some(battery.energy_full_design().get::<battery::units::energy::watt_hour>()),
                    full_charge_capacity_wh: Some(battery.energy_full().get::<battery::units::energy::watt_hour>()),
                    cycle_count: battery.cycle_count(),
                }
            } else {
                battery_info
            }
        }
        Err(_) => battery_info,
    };
    detected_materials.push(format!("Battery(s): [{}]", battery_names.join(", ")));
    sensors_info.push(InitialInfo::Battery(battery_info));
    let hardware_info: HardwareInfo = sensors_info.into();

    let data = GeneralData {
        sensors: sensors_kind,
        hardware_info: hardware_info,
    };

    return data;
}

pub fn get_process_gpu_usage(sensors: &Vec<SensorType>) -> HashMap<u32, f64> {
    let time = SystemTime::now();
    let mut proc_gpu_usage = HashMap::new();

    for sensor in sensors {
        match sensor {
            SensorType::GPU(gpu_sensor) => {
                if let Ok(gpu_process_usage) = gpu_sensor.get_process_gpu_usage(
                    time.duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                ) {
                    proc_gpu_usage.extend(gpu_process_usage);
                }
            }
            _ => {}
        }
    }

    proc_gpu_usage
}
