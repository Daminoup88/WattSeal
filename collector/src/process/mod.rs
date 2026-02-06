pub mod groups;

use std::{collections::HashMap, thread, time::Duration};

use windows::Win32::{
    Foundation::*,
    System::{
        Diagnostics::ToolHelp::*,
        ProcessStatus::GetProcessMemoryInfo,
        Threading::{GetProcessTimes, OpenProcess, PROCESS_QUERY_INFORMATION},
    },
};

/// Represents CPU and GPU usage data for an application
#[derive(Debug, Clone)]
pub struct AppPowerData {
    pub app_name: String,
    pub app_cpu_usage: f64,
    pub gpu_memory_mb: f64,
    pub process_count: usize,
}

struct ProcessTimes {
    kernel: u64,
    user: u64,
    system_time: u64,
}

struct MemoryInfo {
    working_set: u64,
}

/// Get CPU usage per process using Windows API
pub fn estimate_app_power_consumption() -> Vec<AppPowerData> {
    let mut first_sample = HashMap::new();

    // First sample
    collect_process_times(&mut first_sample);

    // Wait 500ms
    thread::sleep(Duration::from_millis(500));

    // Second sample
    let mut second_sample = HashMap::new();
    collect_process_times(&mut second_sample);

    // Calculate CPU usage percentages
    let mut app_cpu_usage: HashMap<String, f64> = HashMap::new();
    let mut app_gpu_memory: HashMap<String, f64> = HashMap::new();
    let mut app_process_count: HashMap<String, usize> = HashMap::new();

    for (pid, (name, times2)) in second_sample {
        if let Some((_, times1)) = first_sample.get(&pid) {
            let cpu_delta = (times2.kernel + times2.user) - (times1.kernel + times1.user);
            let time_delta = times2.system_time - times1.system_time;

            if time_delta > 0 {
                // CPU usage out of 1
                let cpu_usage = cpu_delta as f64 / time_delta as f64;

                *app_cpu_usage.entry(name.clone()).or_insert(0.0) += cpu_usage;
                *app_process_count.entry(name.clone()).or_insert(0) += 1;
            }
        }

        // Get GPU memory usage
        if let Ok(gpu_mem) = get_process_gpu_memory(pid) {
            *app_gpu_memory.entry(name.clone()).or_insert(0.0) += gpu_mem;
        }
    }

    let mut results: Vec<AppPowerData> = app_cpu_usage
        .into_iter()
        .map(|(app_name, app_cpu_usage)| AppPowerData {
            app_name: app_name.clone(),
            app_cpu_usage,
            gpu_memory_mb: *app_gpu_memory.get(&app_name).unwrap_or(&0.0),
            process_count: *app_process_count.get(&app_name).unwrap_or(&0),
        })
        .collect();

    results.sort_by(|a, b| b.app_cpu_usage.partial_cmp(&a.app_cpu_usage).unwrap());
    results
}

fn collect_process_times(map: &mut HashMap<u32, (String, ProcessTimes)>) {
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return;
        };

        let mut pe32: PROCESSENTRY32 = std::mem::zeroed();
        pe32.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        let system_time = windows::Win32::System::SystemInformation::GetSystemTimeAsFileTime();
        let system_time_u64 = ((system_time.dwHighDateTime as u64) << 32) | system_time.dwLowDateTime as u64;

        if Process32First(snapshot, &mut pe32).is_ok() {
            loop {
                // Convert i8 array to u16 for proper UTF-16 handling
                let name_bytes: Vec<u16> = pe32.szExeFile.iter().map(|&c| c as u16).collect();
                let process_name = String::from_utf16_lossy(&name_bytes).trim_end_matches('\0').to_string();

                if let Ok(times) = get_process_times(pe32.th32ProcessID) {
                    map.insert(
                        pe32.th32ProcessID,
                        (
                            process_name,
                            ProcessTimes {
                                kernel: times.0,
                                user: times.1,
                                system_time: system_time_u64,
                            },
                        ),
                    );
                }

                if Process32Next(snapshot, &mut pe32).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);
    }
}

fn get_process_times(pid: u32) -> Result<(u64, u64), String> {
    unsafe {
        let handle =
            OpenProcess(PROCESS_QUERY_INFORMATION, false, pid).map_err(|_| "Failed to open process".to_string())?;

        let mut creation_time: FILETIME = std::mem::zeroed();
        let mut exit_time: FILETIME = std::mem::zeroed();
        let mut kernel_time: FILETIME = std::mem::zeroed();
        let mut user_time: FILETIME = std::mem::zeroed();

        let result = GetProcessTimes(
            handle,
            &mut creation_time,
            &mut exit_time,
            &mut kernel_time,
            &mut user_time,
        );

        let _ = CloseHandle(handle);

        result.map_err(|_| "Failed to get process times".to_string())?;

        let kernel = ((kernel_time.dwHighDateTime as u64) << 32) | kernel_time.dwLowDateTime as u64;
        let user = ((user_time.dwHighDateTime as u64) << 32) | user_time.dwLowDateTime as u64;

        Ok((kernel, user))
    }
}

/// Get GPU memory usage for a process (in MB)
fn get_process_gpu_memory(pid: u32) -> Result<f64, String> {
    unsafe {
        let handle =
            OpenProcess(PROCESS_QUERY_INFORMATION, false, pid).map_err(|_| "Failed to open process".to_string())?;

        let mut mem_info: windows::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS = std::mem::zeroed();

        let success = GetProcessMemoryInfo(
            handle,
            &mut mem_info,
            std::mem::size_of::<windows::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS>() as u32,
        );

        let _ = CloseHandle(handle);

        if success.is_ok() {
            let gpu_memory_mb = mem_info.WorkingSetSize as f64 / (1024.0 * 1024.0);
            Ok(gpu_memory_mb)
        } else {
            Err("Failed to get process memory info".to_string())
        }
    }
}
