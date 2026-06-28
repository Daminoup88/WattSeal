use std::{cell::RefCell, collections::HashMap, rc::Rc};

use common::types::{Byte, EnergyUj, MeasuredProcessData};
use sysinfo::System;

use crate::sensors::ProcessData;

/// Collects the measured processes data without estimating energy consumption.
pub fn get_measured_processes(
    system: Rc<RefCell<System>>,
    proc_gpu_usage: HashMap<u32, f64>,
) -> Vec<MeasuredProcessData> {
    let mut sys = match system.try_borrow_mut() {
        Ok(sys) => sys,
        Err(e) => {
            crate::clog!("✗ Failed to borrow system for measured process collection: {}", e);
            return Vec::new();
        }
    };
    let nb_cores = sys.cpus().len();
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        sysinfo::ProcessRefreshKind::everything()
            .without_environ()
            .without_cwd()
            .without_root()
            .without_tasks()
            .without_user(),
    );

    let mut measured_processes: Vec<MeasuredProcessData> = Vec::new();
    let processes = sys.processes();
    let total_memory = sys.total_memory();

    for (pid, proc_) in processes {
        let name = proc_.name().to_str().unwrap_or("Other").to_string().to_lowercase();
        let exe = proc_.exe().and_then(|path| path.to_str().map(|s| s.to_string()));
        let process_cpu_usage = proc_.cpu_usage() as f64 / nb_cores as f64;
        let process_gpu_usage = proc_gpu_usage.get(&pid.as_u32());
        let mem = proc_.memory() as f64 / total_memory as f64 * 100.0;
        let disk = proc_.disk_usage();

        measured_processes.push(MeasuredProcessData {
            pid: Some(pid.as_u32()),
            app_name: name,
            process_exe_path: exe,
            process_cpu_usage,
            process_gpu_usage: process_gpu_usage.copied(),
            process_mem_usage: mem,
            read_bytes: Byte::from(disk.read_bytes),
            written_bytes: Byte::from(disk.written_bytes),
        });
    }
    measured_processes
}

/// Groups measured processes by app name and estimates per-app energy consumption.
pub fn compute_processes_energy(
    measured_processes: Vec<MeasuredProcessData>,
    cpu_energy: EnergyUj,
    cpu_usage: f64,
    gpu_energy: EnergyUj,
    gpu_usage: f64,
    total_energy: EnergyUj,
) -> Vec<ProcessData> {
    let mut grouped: HashMap<String, ProcessData> = HashMap::new();

    let cpu_energy_f = cpu_energy.as_f64();
    let gpu_energy_f = gpu_energy.as_f64();
    let total_energy_f = total_energy.as_f64();
    let total_compute_energy_f = cpu_energy_f + gpu_energy_f;

    for measured_process in &measured_processes {
        let process_cpu_energy = if cpu_usage > 0.0 {
            (measured_process.process_cpu_usage / cpu_usage) * cpu_energy_f
        } else {
            0.0
        }
        .min(cpu_energy_f);

        let process_gpu_energy = match measured_process.process_gpu_usage {
            Some(gpu_usage_pct) if gpu_usage > 0.0 => (gpu_usage_pct / gpu_usage) * gpu_energy_f,
            _ => 0.0,
        }
        .min(gpu_energy_f);

        // Estimate process energy based on the ponderation of CPU and GPU usage compared to the TOTAL
        let process_energy_f = if total_compute_energy_f > 0.0 {
            ((process_cpu_energy + process_gpu_energy) / total_compute_energy_f) * total_energy_f
        } else {
            0.0
        }
        .min(total_energy_f);

        let process_energy = EnergyUj::from_f64(process_energy_f);

        // Group by application name
        let entry = grouped
            .entry(measured_process.app_name.clone())
            .or_insert_with(ProcessData::default);
        if entry.measured.app_name.is_empty() {
            entry.measured.app_name = measured_process.app_name.clone();
        }
        entry.process_energy += process_energy;
        entry.measured.process_cpu_usage += measured_process.process_cpu_usage;
        if let Some(gpu_usage) = measured_process.process_gpu_usage {
            match entry.measured.process_gpu_usage {
                Some(current_gpu_usage) => entry.measured.process_gpu_usage = Some(current_gpu_usage + gpu_usage),
                None => entry.measured.process_gpu_usage = Some(gpu_usage),
            }
        }
        entry.measured.process_mem_usage += measured_process.process_mem_usage;
        entry.measured.read_bytes += measured_process.read_bytes;
        entry.measured.written_bytes += measured_process.written_bytes;
        entry.subprocess_count += 1;
        if entry.measured.process_exe_path.is_none() && measured_process.process_exe_path.is_some() {
            entry.measured.process_exe_path = measured_process.process_exe_path.clone();
        }
    }

    grouped.into_iter().map(|(_, data)| data).collect()
}

/// Sorts the processes by estimated energy consumption in descending order.
pub fn sort_processes_by_energy(mut processes: Vec<ProcessData>) -> Vec<ProcessData> {
    processes.sort_by(|a, b| {
        b.process_energy
            .partial_cmp(&a.process_energy)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    processes
}
