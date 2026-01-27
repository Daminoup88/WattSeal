use std::collections::HashMap;

use super::AppPowerData;
use common::ProcessData;

/// Group processes by application name and calculate power consumption
pub fn group_processes_by_app(processes: Vec<AppPowerData>, total_cpu_power_watts: f64) -> Vec<ProcessData> {
    let mut grouped: HashMap<String, (f64, f64, u32)> = HashMap::new();
    let mut total_cpu_percent = 0.0;

    // Calculate total CPU percentage across all processes
    for process in &processes {
        total_cpu_percent += process.cpu_usage_percent;
    }

    // Group by application name
    for process in processes {
        let app_name = normalize_app_name(&process.app_name);
        let entry = grouped.entry(app_name).or_insert((0.0, 0.0, 0));
        entry.0 += process.cpu_usage_percent;
        entry.1 += process.gpu_memory_mb; // Add VRAM per app
        entry.2 += process.process_count as u32;
    }

    // Convert to power consumption
    let mut results: Vec<ProcessData> = grouped
        .into_iter()
        .map(|(app_name, (cpu_percent, vram_mb, count))| {
            let power_watts = if total_cpu_percent > 0.0 {
                (cpu_percent / total_cpu_percent) * total_cpu_power_watts
            } else {
                0.0
            };

            ProcessData {
                app_name,
                vram_usage: vram_mb,
                cpu_usage_watts: power_watts,
                subprocess_count: count,
            }
        })
        .collect();

    results.sort_by(|a, b| b.cpu_usage_watts.partial_cmp(&a.cpu_usage_watts).unwrap());
    results
}

/// Normalize application names by extracting the base name
fn normalize_app_name(name: &str) -> String {
    // Remove .exe extension
    let base_name = name.trim_end_matches(".exe");

    // Split on common delimiters and take the first part
    let parts: Vec<&str> = base_name
        .split(&[' ', '-', '_', '.'][..])
        .filter(|s| !s.is_empty())
        .collect();

    if let Some(first_part) = parts.first() {
        // Capitalize first letter
        let mut chars = first_part.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    } else {
        base_name.to_string()
    }
}
