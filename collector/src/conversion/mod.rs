use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};

use common::{
    CPUData, ConsumptionMetric, ConsumptionUnit, DataDB, DiskData, EnergyUnit, Event, EventDB, GPUData, NetworkData,
    PowerUnit, ProcessDataDB, RamData, SensorData, TotalDataDB,
};
use sysinfo::System;

use crate::database::process::get_processes;

/// Trait for types that hold consumption metrics and can convert them to a target unit.
///
/// The `duration` parameter is required for conversions between energy and power units
/// (e.g. µJ -> W), and ignored for same-dimension conversions (e.g. µJ -> Wh).
pub trait ConsumptionConvertible {
    /// The output type after conversion.
    type ConsumptionOutput;

    /// Converts consumption values to watts (W).
    fn to_watts(&self, duration: Duration) -> Self::ConsumptionOutput;

    /// Converts consumption values to microjoules (µJ).
    fn to_uj(&self, duration: Duration) -> Self::ConsumptionOutput;

    /// Converts consumption values to watt-hours (Wh).
    fn to_wh(&self, duration: Duration) -> Self::ConsumptionOutput;
}

fn watts_to_uj(watts: f64, duration: Duration) -> f64 {
    watts * duration.as_secs_f64() * 1_000_000.0
}

fn wh_to_uj(wh: f64) -> f64 {
    wh * 3_600_000_000.0
}

fn watts_to_wh(watts: f64, duration: Duration) -> f64 {
    watts * duration.as_secs_f64() / 3600.0
}

fn uj_to_wh(uj: f64) -> f64 {
    uj / 3_600_000_000.0
}

fn uj_to_watts(uj: f64, duration: Duration) -> f64 {
    let j = uj / 1_000_000.0;
    let secs = duration.as_secs_f64().max(0.001);
    j / secs
}

fn wh_to_watts(wh: f64, duration: Duration) -> f64 {
    let h = duration.as_secs_f64().max(0.001) / 3600.0;
    wh / h
}

impl ConsumptionConvertible for ConsumptionMetric {
    type ConsumptionOutput = f64;

    fn to_wh(&self, duration: Duration) -> Self::ConsumptionOutput {
        match self.unit {
            ConsumptionUnit::Energy(EnergyUnit::UJoul) => uj_to_wh(self.value),
            ConsumptionUnit::Energy(EnergyUnit::WattHour) => self.value,
            ConsumptionUnit::Power(PowerUnit::Watt) => watts_to_wh(self.value, duration),
        }
    }

    fn to_watts(&self, duration: Duration) -> Self::ConsumptionOutput {
        match self.unit {
            ConsumptionUnit::Energy(EnergyUnit::UJoul) => uj_to_watts(self.value, duration),
            ConsumptionUnit::Energy(EnergyUnit::WattHour) => wh_to_watts(self.value, duration),
            ConsumptionUnit::Power(PowerUnit::Watt) => self.value,
        }
    }

    fn to_uj(&self, duration: Duration) -> Self::ConsumptionOutput {
        match self.unit {
            ConsumptionUnit::Energy(EnergyUnit::UJoul) => self.value,
            ConsumptionUnit::Energy(EnergyUnit::WattHour) => wh_to_uj(self.value),
            ConsumptionUnit::Power(PowerUnit::Watt) => watts_to_uj(self.value, duration),
        }
    }
}

impl ConsumptionConvertible for CPUData<ConsumptionMetric> {
    type ConsumptionOutput = CPUData<f64>;

    fn to_wh(&self, duration: Duration) -> Self::ConsumptionOutput {
        CPUData {
            total_consumption: self.total_consumption.map(|t| t.to_wh(duration)),
            pp0_consumption: self.pp0_consumption.map(|pp0| pp0.to_wh(duration)),
            pp1_consumption: self.pp1_consumption.map(|pp1| pp1.to_wh(duration)),
            dram_consumption: self.dram_consumption.map(|dram| dram.to_wh(duration)),
            usage_percent: self.usage_percent,
        }
    }

    fn to_watts(&self, duration: Duration) -> Self::ConsumptionOutput {
        CPUData {
            total_consumption: self.total_consumption.map(|t| t.to_watts(duration)),
            pp0_consumption: self.pp0_consumption.map(|pp0| pp0.to_watts(duration)),
            pp1_consumption: self.pp1_consumption.map(|pp1| pp1.to_watts(duration)),
            dram_consumption: self.dram_consumption.map(|dram| dram.to_watts(duration)),
            usage_percent: self.usage_percent,
        }
    }

    fn to_uj(&self, duration: Duration) -> Self::ConsumptionOutput {
        CPUData {
            total_consumption: self.total_consumption.map(|t| t.to_uj(duration)),
            pp0_consumption: self.pp0_consumption.map(|pp0| pp0.to_uj(duration)),
            pp1_consumption: self.pp1_consumption.map(|pp1| pp1.to_uj(duration)),
            dram_consumption: self.dram_consumption.map(|dram| dram.to_uj(duration)),
            usage_percent: self.usage_percent,
        }
    }
}

impl ConsumptionConvertible for GPUData<ConsumptionMetric> {
    type ConsumptionOutput = GPUData<f64>;

    fn to_wh(&self, duration: Duration) -> Self::ConsumptionOutput {
        GPUData {
            total_consumption: self.total_consumption.map(|t| t.to_wh(duration)),
            usage_percent: self.usage_percent,
            vram_usage_percent: self.vram_usage_percent,
        }
    }

    fn to_watts(&self, duration: Duration) -> Self::ConsumptionOutput {
        GPUData {
            total_consumption: self.total_consumption.map(|t| t.to_watts(duration)),
            usage_percent: self.usage_percent,
            vram_usage_percent: self.vram_usage_percent,
        }
    }

    fn to_uj(&self, duration: Duration) -> Self::ConsumptionOutput {
        GPUData {
            total_consumption: self.total_consumption.map(|t| t.to_uj(duration)),
            usage_percent: self.usage_percent,
            vram_usage_percent: self.vram_usage_percent,
        }
    }
}

impl ConsumptionConvertible for RamData<ConsumptionMetric> {
    type ConsumptionOutput = RamData<f64>;

    fn to_wh(&self, duration: Duration) -> Self::ConsumptionOutput {
        RamData {
            total_consumption: self.total_consumption.map(|t| t.to_wh(duration)),
            usage_percent: self.usage_percent,
        }
    }

    fn to_watts(&self, duration: Duration) -> Self::ConsumptionOutput {
        RamData {
            total_consumption: self.total_consumption.map(|t| t.to_watts(duration)),
            usage_percent: self.usage_percent,
        }
    }

    fn to_uj(&self, duration: Duration) -> Self::ConsumptionOutput {
        RamData {
            total_consumption: self.total_consumption.map(|t| t.to_uj(duration)),
            usage_percent: self.usage_percent,
        }
    }
}

impl ConsumptionConvertible for DiskData<ConsumptionMetric> {
    type ConsumptionOutput = DiskData<f64>;

    fn to_wh(&self, duration: Duration) -> Self::ConsumptionOutput {
        DiskData {
            total_consumption: self.total_consumption.map(|t| t.to_wh(duration)),
            read_usage_mb_s: self.read_usage_mb_s,
            write_usage_mb_s: self.write_usage_mb_s,
        }
    }

    fn to_watts(&self, duration: Duration) -> Self::ConsumptionOutput {
        DiskData {
            total_consumption: self.total_consumption.map(|t| t.to_watts(duration)),
            read_usage_mb_s: self.read_usage_mb_s,
            write_usage_mb_s: self.write_usage_mb_s,
        }
    }

    fn to_uj(&self, duration: Duration) -> Self::ConsumptionOutput {
        DiskData {
            total_consumption: self.total_consumption.map(|t| t.to_uj(duration)),
            read_usage_mb_s: self.read_usage_mb_s,
            write_usage_mb_s: self.write_usage_mb_s,
        }
    }
}

impl ConsumptionConvertible for NetworkData<ConsumptionMetric> {
    type ConsumptionOutput = NetworkData<f64>;

    fn to_wh(&self, duration: Duration) -> Self::ConsumptionOutput {
        NetworkData {
            total_consumption: self.total_consumption.map(|t| t.to_wh(duration)),
            download_speed_mb_s: self.download_speed_mb_s,
            upload_speed_mb_s: self.upload_speed_mb_s,
        }
    }

    fn to_watts(&self, duration: Duration) -> Self::ConsumptionOutput {
        NetworkData {
            total_consumption: self.total_consumption.map(|t| t.to_watts(duration)),
            download_speed_mb_s: self.download_speed_mb_s,
            upload_speed_mb_s: self.upload_speed_mb_s,
        }
    }

    fn to_uj(&self, duration: Duration) -> Self::ConsumptionOutput {
        NetworkData {
            total_consumption: self.total_consumption.map(|t| t.to_uj(duration)),
            download_speed_mb_s: self.download_speed_mb_s,
            upload_speed_mb_s: self.upload_speed_mb_s,
        }
    }
}

impl ConsumptionConvertible for SensorData<ConsumptionMetric> {
    type ConsumptionOutput = SensorData<f64>;

    fn to_wh(&self, duration: Duration) -> Self::ConsumptionOutput {
        match self {
            SensorData::CPU(cpudata) => SensorData::CPU(cpudata.to_wh(duration)),
            SensorData::GPU(gpudata) => SensorData::GPU(gpudata.to_wh(duration)),
            SensorData::Ram(ramdata) => SensorData::Ram(ramdata.to_wh(duration)),
            SensorData::Disk(diskdata) => SensorData::Disk(diskdata.to_wh(duration)),
            SensorData::Network(networkdata) => SensorData::Network(networkdata.to_wh(duration)),
        }
    }

    fn to_watts(&self, duration: Duration) -> Self::ConsumptionOutput {
        match self {
            SensorData::CPU(cpudata) => SensorData::CPU(cpudata.to_watts(duration)),
            SensorData::GPU(gpudata) => SensorData::GPU(gpudata.to_watts(duration)),
            SensorData::Ram(ramdata) => SensorData::Ram(ramdata.to_watts(duration)),
            SensorData::Disk(diskdata) => SensorData::Disk(diskdata.to_watts(duration)),
            SensorData::Network(networkdata) => SensorData::Network(networkdata.to_watts(duration)),
        }
    }

    fn to_uj(&self, duration: Duration) -> Self::ConsumptionOutput {
        match self {
            SensorData::CPU(cpudata) => SensorData::CPU(cpudata.to_uj(duration)),
            SensorData::GPU(gpudata) => SensorData::GPU(gpudata.to_uj(duration)),
            SensorData::Ram(ramdata) => SensorData::Ram(ramdata.to_uj(duration)),
            SensorData::Disk(diskdata) => SensorData::Disk(diskdata.to_uj(duration)),
            SensorData::Network(networkdata) => SensorData::Network(networkdata.to_uj(duration)),
        }
    }
}

fn consumption_sensor_data_to_datadb(sensor_data: &SensorData<ConsumptionMetric>, duration: Duration) -> DataDB {
    DataDB::Sensor(sensor_data.to_watts(duration))
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
