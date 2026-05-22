use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};

use common::{
    CPUData, ConsumptionMetric, DataDB, DiskData, EnergyUnit, Event, EventDB, GPUData, NetworkData, PowerUnit,
    ProcessDataDB, RamData, SensorData, TotalDataDB,
};
use sysinfo::System;

use crate::database::process::get_processes;

fn uj_to_watts(uj: f64, duration: Duration) -> f64 {
    let joules = uj / 1_000_000.0;
    let secs = duration.as_secs_f64();
    joules / secs
}

fn wh_to_watts(watt_hours: f64, duration: Duration) -> f64 {
    let hours = duration.as_secs_f64() / 3600.0;
    watt_hours / hours
}

fn consumption_metric_to_watts(c: ConsumptionMetric, duration: Duration) -> f64 {
    match c.unit {
        common::ConsumptionUnit::Energy(EnergyUnit::UJoul) => uj_to_watts(c.value, duration),
        common::ConsumptionUnit::Energy(EnergyUnit::WattHour) => wh_to_watts(c.value, duration),
        common::ConsumptionUnit::Power(PowerUnit::Watt) => c.value,
    }
}

fn consumption_sensor_data_to_datadb(sensor_data: &SensorData<ConsumptionMetric>, duration: Duration) -> DataDB {
    match sensor_data {
        SensorData::CPU(cpudata) => DataDB::Sensor(SensorData::CPU(CPUData {
            total_consumption: cpudata
                .total_consumption
                .map(|t| consumption_metric_to_watts(t, duration)),
            pp0_consumption: cpudata
                .pp0_consumption
                .map(|pp0| consumption_metric_to_watts(pp0, duration)),
            pp1_consumption: cpudata
                .pp1_consumption
                .map(|pp1| consumption_metric_to_watts(pp1, duration)),
            dram_consumption: cpudata
                .dram_consumption
                .map(|dram| consumption_metric_to_watts(dram, duration)),
            usage_percent: cpudata.usage_percent,
        })),
        SensorData::GPU(gpudata) => DataDB::Sensor(SensorData::GPU(GPUData {
            total_consumption: gpudata
                .total_consumption
                .map(|t| consumption_metric_to_watts(t, duration)),
            usage_percent: gpudata.usage_percent,
            vram_usage_percent: gpudata.vram_usage_percent,
        })),
        SensorData::Ram(ramdata) => DataDB::Sensor(SensorData::Ram(RamData {
            total_consumption: ramdata
                .total_consumption
                .map(|t| consumption_metric_to_watts(t, duration)),
            usage_percent: ramdata.usage_percent,
        })),
        SensorData::Disk(diskdata) => DataDB::Sensor(SensorData::Disk(DiskData {
            total_consumption: diskdata
                .total_consumption
                .map(|t| consumption_metric_to_watts(t, duration)),
            read_usage_mb_s: diskdata.read_usage_mb_s,
            write_usage_mb_s: diskdata.write_usage_mb_s,
        })),
        SensorData::Network(networkdata) => DataDB::Sensor(SensorData::Network(NetworkData {
            total_consumption: networkdata
                .total_consumption
                .map(|t| consumption_metric_to_watts(t, duration)),
            download_speed_mb_s: networkdata.download_speed_mb_s,
            upload_speed_mb_s: networkdata.upload_speed_mb_s,
        })),
    }
}

pub fn consumption_event_to_eventdb(
    sensors_event: &Event<ConsumptionMetric>,
    duration: Duration,
    system: Rc<RefCell<System>>,
    process_gpu_usage: HashMap<u32, f64>,
) -> EventDB {
    let mut data: Vec<DataDB> = sensors_event
        .data()
        .into_iter()
        .map(|sensor_data| consumption_sensor_data_to_datadb(sensor_data, duration))
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
