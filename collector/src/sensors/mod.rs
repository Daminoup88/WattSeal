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
    time::{Instant, SystemTime},
};

use battery::{Battery, Manager};
pub use common::{
    AllTimeData, Event, GPUData, GeneralData, ProcessData, SensorData, TotalData,
    types::{BatteryInfo, CpuInfo, DiskInfo, HardwareInfo, InitialInfo, MemoryInfo, ScreenInfo, SystemInfo},
};
pub use cpu::CPUSensor;
pub use disk::DiskSensor;
use display_info::DisplayInfo;
pub use gpu::{GPUSensor, get_gpu_list};
pub use network::NetworkSensor;
pub use process::get_processes;
pub use ram::RamSensor;
use sysinfo::System;

pub enum SensorType {
    CPU(CPUSensor),
    GPU(GPUSensor),
    RAM(RamSensor),
    Disk(DiskSensor),
    Network(NetworkSensor),
    Process,
    Total,
}

impl Sensor for SensorType {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        match self {
            SensorType::CPU(sensor) => sensor.read_full_data(),
            SensorType::GPU(sensor) => sensor.read_full_data(),
            SensorType::RAM(sensor) => sensor.read_full_data(),
            SensorType::Disk(sensor) => sensor.read_full_data(),
            SensorType::Network(sensor) => sensor.read_full_data(),
            SensorType::Process => Err(SensorError::NotSupported),
            SensorType::Total => Err(SensorError::NotSupported),
        }
    }

    fn read_initial_info(&self) -> Result<InitialInfo, SensorError> {
        match self {
            SensorType::CPU(sensor) => sensor.read_initial_info(),
            SensorType::GPU(sensor) => sensor.read_initial_info(),
            SensorType::RAM(sensor) => sensor.read_initial_info(),
            SensorType::Disk(sensor) => sensor.read_initial_info(),
            SensorType::Network(_) => Err(SensorError::NotSupported),
            SensorType::Process => Err(SensorError::NotSupported),
            SensorType::Total => Err(SensorError::NotSupported),
        }
    }

    fn read_name(&self) -> Result<String, SensorError> {
        match self {
            SensorType::CPU(sensor) => sensor.read_name(),
            SensorType::GPU(sensor) => sensor.read_name(),
            SensorType::Disk(sensor) => sensor.read_name(),
            SensorType::Network(sensor) => sensor.read_name(),
            SensorType::RAM(_) => Err(SensorError::NotSupported),
            SensorType::Process => Err(SensorError::NotSupported),
            SensorType::Total => Err(SensorError::NotSupported),
        }
    }
}

pub trait Sensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError>;
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

pub fn create_event_from_sensors(
    sensors: &Vec<SensorType>,
    system: Rc<RefCell<System>>,
    total_output: &mut f64,
) -> Event {
    let time = SystemTime::now();
    let mut data: Vec<SensorData> = Vec::new();

    let (mut cpu_power, mut cpu_usage, mut nb_cpus) = (0.0, 0.0, 0);
    let (mut gpu_power, mut gpu_usage, mut nb_gpus) = (0.0, 0.0, 0);

    let mut total_power = 0.0;
    let mut integrated_gpu_power: Option<f64> = None;
    let mut proc_gpu_usage = HashMap::new();
    for sensor in sensors {
        match sensor {
            SensorType::Process | SensorType::Total => continue,
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
        if let SensorType::Total | SensorType::Process = sensor {
            continue;
        }

        let sensor_data = sensor.read_full_data();
        match sensor_data {
            Ok(mut d) => {
                if let SensorData::CPU(ref mut cpu) = d {
                    if let Some(pp1) = cpu.pp1_power_watts.take() {
                        if pp1 > 0.0 {
                            if let Some(ref mut total) = cpu.total_power_watts {
                                *total -= pp1;
                            }
                            integrated_gpu_power = Some(pp1);
                        }
                    }
                }

                if let Some(power) = d.total_power_watts() {
                    total_power += power;

                    if let SensorData::CPU(cpu) = &d {
                        cpu_power += power;
                        cpu_usage += cpu.usage_percent.unwrap_or(0.0);
                        nb_cpus += 1;
                    }

                    if let SensorData::GPU(gpu) = &d {
                        gpu_power += power;
                        gpu_usage += gpu.usage_percent.unwrap_or(0.0);
                        nb_gpus += 1;
                    }
                }
                data.push(d);
            }
            Err(e) => eprintln!("✗ Error reading sensor data: {:?}", e),
        }
    }

    if let Some(igpu_power) = integrated_gpu_power {
        let merged = data.iter_mut().any(|d| {
            if let SensorData::GPU(gpu) = d {
                if gpu.total_power_watts.is_none() {
                    gpu.total_power_watts = Some(igpu_power);
                    return true;
                }
            }
            false
        });
        if !merged {
            data.push(SensorData::GPU(GPUData {
                total_power_watts: Some(igpu_power),
                usage_percent: None,
                vram_usage_percent: None,
            }));
            nb_gpus += 1;
        }
        gpu_power += igpu_power;
        total_power += igpu_power;
    }

    data.push(SensorData::Total(TotalData {
        total_power_watts: total_power,
        period_type: "second".to_string(),
    }));

    *total_output = total_power;

    cpu_usage /= nb_cpus.max(1) as f64;
    gpu_usage /= nb_gpus.max(1) as f64;

    let timer = Instant::now();
    let top10_process_data: Vec<ProcessData> = get_processes(
        system.clone(),
        cpu_power,
        cpu_usage,
        gpu_power,
        gpu_usage,
        total_power,
        10,
        proc_gpu_usage,
    );
    data.push(SensorData::Process(top10_process_data));
    println!("Process time: {} ms", timer.elapsed().as_millis());

    return Event::new(time, data);
}

pub fn get_hardware_info(sensors: &Vec<SensorType>) -> GeneralData {
    let mut tables: Vec<String> = Vec::new();
    let mut detected_materials: Vec<String> = Vec::new();
    let mut sensors_info: Vec<InitialInfo> = Vec::new();

    for sensor in sensors {
        tables.push(sensor.table_name().to_string());

        if let Ok(name) = sensor.read_name() {
            detected_materials.push(name);
        } else {
            eprintln!("✗ No name available for sensor: {:?}", sensor.table_name());
        }

        if let Ok(info) = sensor.read_initial_info() {
            sensors_info.push(info);
        } else {
            eprintln!("✗ Failed to read initial info for sensor: {:?}", sensor.table_name());
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
        tables: tables.join(","),
        hardware_info_serialized: hardware_info.serialized(),
    };

    return data;
}

fn disk_kind_label(kind: &sysinfo::Disk) -> &'static str {
    if kind.is_removable() {
        "Removable"
    } else {
        match kind.kind() {
            sysinfo::DiskKind::HDD => "HDD",
            sysinfo::DiskKind::SSD => "SSD",
            _ => "Unknown",
        }
    }
}
