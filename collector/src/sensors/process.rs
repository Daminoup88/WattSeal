use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    thread,
    time::{Duration, Instant},
};

use sysinfo::{Pid, Process, System};

use crate::sensors::{ProcessData, cpu, process};

pub fn get_processes(
    system: Rc<RefCell<System>>,
    cpu_power: f64,
    cpu_usage: f64,
    gpu_power: f64,
    gpu_usage: f64,
    total_power: f64,
    number_processes: usize,
    proc_gpu_usage: HashMap<u32, f64>,
) -> Vec<ProcessData> {
    let mut sys = match system.try_borrow_mut() {
        Ok(sys) => sys,
        Err(e) => {
            eprintln!("Failed to borrow system for process collection: {}", e);
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

    let mut processes = extract_and_group_processes(
        sys.processes(),
        sys.total_memory(),
        cpu_power,
        cpu_usage,
        nb_cores,
        gpu_power,
        gpu_usage,
        total_power,
        proc_gpu_usage,
    );
    processes.sort_by(|a, b| {
        b.process_usage_watt
            .partial_cmp(&a.process_usage_watt)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let top_apps = processes.into_iter().take(number_processes).collect();

    return top_apps;
}

fn extract_and_group_processes(
    processes: &HashMap<Pid, Process>,
    total_memory: u64,
    cpu_power: f64,
    cpu_usage: f64,
    nb_cores: usize,
    gpu_power: f64,
    gpu_usage: f64,
    total_power: f64,
    proc_gpu_usage: HashMap<u32, f64>,
) -> Vec<ProcessData> {
    let mut grouped: HashMap<String, ProcessData> = HashMap::new();

    for (pid, proc_) in processes {
        let name = proc_.name().to_str().unwrap_or("Other").to_string().to_lowercase();
        let exe = proc_
            .exe()
            .and_then(|path| path.to_str().and_then(|str| Some(str.to_string())));
        // CPU usage percentage for this process as it would appear in a task manager (normalized by number of cores)
        let process_cpu_usage = proc_.cpu_usage() as f64 / nb_cores as f64;
        let process_gpu_usage = proc_gpu_usage.get(&pid.as_u32());
        let mem = proc_.memory() as f64 / total_memory as f64 * 100.0;
        let disk = proc_.disk_usage();

        let process_cpu_power = if cpu_usage > 0.0 {
            process_cpu_usage / cpu_usage * cpu_power
        } else {
            0.0
        }
        .min(cpu_power);
        let process_gpu_power = match process_gpu_usage {
            Some(process_gpu_usage) if gpu_usage > 0.0 => process_gpu_usage / gpu_usage * gpu_power,
            _ => 0.0,
        }
        .min(gpu_power);

        // Estimate process power based on the ponderation of CPU and GPU usage compared to the TOTAL
        let total_compute_power = cpu_power + gpu_power;
        let process_power = if total_compute_power > 0.0 {
            (process_cpu_power + process_gpu_power) / total_compute_power * total_power
        } else {
            0.0
        }
        .min(total_power);

        // Group by application name
        let entry = grouped.entry(name.clone()).or_insert(ProcessData::default());
        if entry.app_name.is_empty() {
            entry.app_name = name;
        }
        entry.process_usage_watt += process_power;
        entry.process_cpu_usage += process_cpu_usage;
        if let Some(gpu_usage) = process_gpu_usage {
            match entry.process_gpu_usage {
                Some(current_gpu_usage) => entry.process_gpu_usage = Some(current_gpu_usage + gpu_usage),
                None => entry.process_gpu_usage = Some(*gpu_usage),
            }
        }
        entry.process_mem_usage += mem;
        entry.read_bytes_per_sec += disk.read_bytes as f64;
        entry.written_bytes_per_sec += disk.written_bytes as f64;
        entry.subprocess_count += 1;
        if entry.process_exe_path.is_none() && exe.is_some() {
            entry.process_exe_path = exe;
        }
    }

    return grouped.into_iter().map(|(_, data)| data).collect();
}
