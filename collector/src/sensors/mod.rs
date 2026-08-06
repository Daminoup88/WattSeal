pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod network;
pub mod process;
pub mod ram;

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{Duration, SystemTime},
};

use battery::Manager;
use common::types::EnergyUj;
pub use common::{
    AllTimeData, ComputedSensorData, Event, GPUData, GeneralData, ProcessData, SensorData, TotalData,
    types::{BatteryInfo, CpuInfo, DiskInfo, HardwareInfo, InitialInfo, MemoryInfo, ScreenInfo, SystemInfo},
};
pub use cpu::CPUSensor;
pub use disk::DiskSensor;
use display_info::DisplayInfo;
pub use gpu::{GPUSensor, get_gpu_list};
pub use network::NetworkSensor;
pub use ram::RamSensor;
use sysinfo::System;

use crate::sensors::process::{compute_processes_energy, get_measured_processes, sort_processes_by_energy};

/// Variant wrapper for all supported sensor types.
pub enum SensorType {
    CPU(CPUSensor),
    GPU(GPUSensor),
    RAM(RamSensor),
    Disk(DiskSensor),
    Network(NetworkSensor),
}

impl Sensor for SensorType {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
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
    /// Reads energy consumption since last call, usage, and throughput data.
    fn read_full_data(&self) -> Result<SensorData, SensorError>;
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
pub fn create_event_from_sensors(
    sensors: &Vec<SensorType>,
    system: Rc<RefCell<System>>,
    since_last_update: Duration,
) -> Event {
    let time = SystemTime::now();
    let mut data: Vec<SensorData> = Vec::new();

    let mut integrated_gpu_energy: Option<EnergyUj> = None;
    let mut dram_energy: Option<EnergyUj> = None;
    let mut has_pp1_source = false;
    let mut integrated_gpu_indice: Option<usize> = None;
    let mut ram_indice: Option<usize> = None;
    let mut gpu_indices: Vec<(usize, String)> = Vec::new();
    for sensor in sensors {
        let sensor_data = sensor.read_full_data();
        match sensor_data {
            Ok(mut d) => {
                if let SensorData::CPU(ref mut cpu) = d {
                    if let Some(pp1) = cpu.pp1_energy.take() {
                        has_pp1_source = true;
                        if pp1 > 0.0 {
                            if let Some(ref mut total) = cpu.total_energy {
                                *total -= pp1;
                            }
                            integrated_gpu_energy = Some(pp1);
                        }
                    }
                    if let Some(dram) = cpu.dram_energy.take() {
                        if dram > 0.0 {
                            dram_energy = Some(dram);
                        }
                    }
                }

                // Track integrated Intel GPUs for estimation fallback.
                if let SensorType::GPU(gpu_sensor) = sensor {
                    if gpu_sensor.is_integrated() {
                        integrated_gpu_indice = Some(data.len());
                    }
                    gpu_indices.push((data.len(), gpu_sensor.name()));
                }

                if let SensorData::Ram(_) = d {
                    ram_indice = Some(data.len());
                }

                data.push(d);
            }
            Err(e) => common::logging::log_component_error(
                sensor.table_name(),
                &format!("Failed to read sensor data: {:?}", e),
            ),
        }
    }

    // Add DRAM energy (from CPU MSR) to the RAM sensor reading.
    if let Some(dram) = dram_energy {
        if let Some(idx) = ram_indice {
            if let Some(SensorData::Ram(ram)) = data.get_mut(idx) {
                *ram.total_energy.get_or_insert(EnergyUj::from_u64(0)) += dram;
            }
        }
    }

    // --- Integrated-GPU energy resolution ---
    // Priority 1: Real PP1 reading from MSR (WinRing0).
    if let Some(igpu_energy) = integrated_gpu_energy {
        let mut merged = false;
        // Try to merge into the tracked integrated GPU first
        if let Some(idx) = integrated_gpu_indice {
            if let Some(SensorData::GPU(gpu)) = data.get_mut(idx) {
                if gpu.total_energy.is_none() {
                    gpu.total_energy = Some(igpu_energy);
                    merged = true;
                }
            }
        }
        // Fallback to any GPU without energy reading
        if !merged {
            merged = data.iter_mut().any(|d| {
                if let SensorData::GPU(gpu) = d {
                    if gpu.total_energy.is_none() {
                        gpu.total_energy = Some(igpu_energy);
                        return true;
                    }
                }
                false
            });
        }
        if !merged {
            let igpu_name = sensors.iter().find_map(|s| {
                if let SensorType::GPU(gpu_sensor) = s {
                    if gpu_sensor.is_integrated() {
                        return Some(gpu_sensor.name());
                    }
                }
                None
            });
            data.push(SensorData::GPU(GPUData {
                total_energy: Some(igpu_energy),
                usage_percent: None,
                vram_usage_percent: None,
                name: igpu_name,
            }));
        }
    }

    // Priority 2: Estimate all GPU energy from usage when unavailable including integrated GPUs.
    for (idx, name) in &gpu_indices {
        // Skip the integrated GPU if it has a PP1 source (already accounted for).
        if let Some(igpu_idx) = integrated_gpu_indice {
            if *idx == igpu_idx && has_pp1_source {
                continue;
            }
        }
        if let Some(SensorData::GPU(gpu)) = data.get_mut(*idx) {
            if gpu.total_energy.is_none() {
                if let Some(usage) = gpu.usage_percent {
                    gpu.total_energy = Some(gpu::estimation::estimate_energy(name, usage, since_last_update));
                }
            }
        }
    }

    let proc_gpu_usage = get_process_gpu_usage(sensors);
    let measured_processes = get_measured_processes(system, proc_gpu_usage);
    data.push(SensorData::Process(measured_processes));

    return Event::new(time, data);
}

pub fn to_computed_event(sensors_event: &Event) -> Event<ComputedSensorData> {
    let mut data: Vec<ComputedSensorData> = Vec::new();
    let (mut cpu_energy, mut cpu_usage, mut nb_cpus) = (EnergyUj::from_u64(0), 0.0, 0);
    let (mut gpu_energy, mut gpu_usage, mut nb_gpus) = (EnergyUj::from_u64(0), 0.0, 0);

    let mut total_energy = EnergyUj::from_u64(0);
    let mut process_index = None;

    for sensor_data in sensors_event.data().iter() {
        if let Some(energy) = sensor_data.total_energy() {
            total_energy += energy;

            if let SensorData::CPU(cpu) = &sensor_data {
                cpu_energy += energy;
                cpu_usage += cpu.usage_percent.unwrap_or(0.0);
                nb_cpus += 1;
            }

            if let SensorData::GPU(gpu) = &sensor_data {
                gpu_energy += energy;
                gpu_usage += gpu.usage_percent.unwrap_or(0.0);
                nb_gpus += 1;
            }
        }
        if let SensorData::Process(_) = sensor_data {
            process_index = Some(data.len());
            continue;
        }
        data.push(sensor_data.clone().into());
    }

    data.push(ComputedSensorData::Total(TotalData { total_energy }));

    cpu_usage /= nb_cpus.max(1) as f64;
    gpu_usage /= nb_gpus.max(1) as f64;
    let computed_process_data = compute_processes_energy(
        process_index
            .and_then(|idx| {
                if let SensorData::Process(measured_processes) = &sensors_event.data()[idx] {
                    Some(measured_processes.to_vec())
                } else {
                    None
                }
            })
            .unwrap_or_default(),
        cpu_energy,
        cpu_usage,
        gpu_energy,
        gpu_usage,
        total_energy,
    );
    let top10_processes: Vec<ProcessData> = sort_processes_by_energy(computed_process_data)
        .into_iter()
        .take(10)
        .collect();
    data.push(ComputedSensorData::Process(top10_processes));

    Event::new(sensors_event.time(), data)
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

/// Collects hardware info (names + initial specs) from all sensors.
pub fn get_hardware_info(sensors: &Vec<SensorType>, table_names: &Vec<String>) -> GeneralData {
    let mut sensors_info: Vec<InitialInfo> = Vec::new();

    for sensor in sensors {
        match sensor.read_name() {
            Ok(name) => crate::clog!("✓ Added sensor {}: {}", sensor.table_name(), name),
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
    let display_info = DisplayInfo::all().unwrap_or_default();
    let mut display_names = Vec::new();
    let mut screen_info = Vec::new();
    for display_info in display_info {
        let resolution = format!("{}x{}", display_info.width, display_info.height);
        let friendly_name = display_info.friendly_name.clone();
        display_names.push(friendly_name.clone());
        screen_info.push(ScreenInfo {
            model: friendly_name,
            resolution: resolution,
            refresh_rate_hz: display_info.frequency as u32,
            is_primary: display_info.is_primary,
        });
    }
    crate::clog!(
        "✓ Added {} display(s): [{}]",
        screen_info.len(),
        display_names.join(", ")
    );
    sensors_info.push(InitialInfo::Displays(screen_info));

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
        Err(e) => {
            crate::clog!("✗ Failed to read battery info: {:?}", e);
            battery_info
        }
    };
    crate::clog!(
        "✓ Added battery info: present={}, name(s)=[{}], design_capacity={:?}Wh, full_charge_capacity={:?}Wh, cycle_count={:?}",
        battery_info.present,
        battery_names.join(", "),
        battery_info.design_capacity_wh,
        battery_info.full_charge_capacity_wh,
        battery_info.cycle_count
    );
    sensors_info.push(InitialInfo::Battery(battery_info));
    let hardware_info: HardwareInfo = sensors_info.into();

    let data = GeneralData {
        tables: table_names.join(","),
        hardware_info: hardware_info,
    };

    return data;
}
