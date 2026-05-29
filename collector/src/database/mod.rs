pub mod process;
mod tables;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub use common::{
    DATABASE_PATH, Database, DatabaseEntry, Event,
    database::types::{
        CPUDataDB, DiskDataDB, GPUDataDB, NetworkDataDB, ProcessDataDB, RamDataDB, SensorDataDB, TotalDataDB,
    },
};
use common::{DataDB, EnergyUJ, EventDB, SensorData};
use sysinfo::System;

use crate::database::process::get_processes;

pub fn consumption_event_to_eventdb(
    sensors_event: &Event<EnergyUJ>,
    system: Rc<RefCell<System>>,
    process_gpu_usage: HashMap<u32, f64>,
) -> EventDB {
    let mut data: Vec<DataDB> = sensors_event
        .data()
        .into_iter()
        .map(|sensor_data| DataDB::from(sensor_data))
        .collect();

    let (mut cpu_power, mut cpu_usage, mut nb_cpus) = (0.0, 0.0, 0);
    let (mut gpu_power, mut gpu_usage, mut nb_gpus) = (0.0, 0.0, 0);

    let mut total_power = 0.0;

    for datadb in &data {
        if let Some(power) = datadb.total_consumption() {
            total_power += power;

            if let DataDB::Sensor(SensorData::CPU(cpu)) = &datadb {
                cpu_power += power;
                cpu_usage += cpu.usage_percent.unwrap_or(0.0);
                nb_cpus += 1;
            }

            if let DataDB::Sensor(SensorData::GPU(gpu)) = &datadb {
                gpu_power += power;
                gpu_usage += gpu.usage_percent.unwrap_or(0.0);
                nb_gpus += 1;
            }
        }
    }

    data.push(DataDB::Total(TotalDataDB {
        total_consumption: total_power,
        period_type: "second".to_string(),
    }));

    cpu_usage /= nb_cpus.max(1) as f64;
    gpu_usage /= nb_gpus.max(1) as f64;
    let top10_process_data: Vec<ProcessDataDB> = get_processes(
        system.clone(),
        cpu_power,
        cpu_usage,
        gpu_power,
        gpu_usage,
        total_power,
        10,
        process_gpu_usage,
    );
    data.push(DataDB::Process(top10_process_data));

    EventDB::new(sensors_event.time(), data)
}
