//! Cross-vendor GPU power estimation, used as a fallback when a vendor's
//! telemetry API reports usage (%) but not power/energy — e.g. partial ADLX
//! support on some AMD SKUs, or NVML power telemetry missing on some OEM
//! boards (see collector logs).
//!
//! Power is modeled as an idle floor plus a convex, non-linear function of
//! utilization, following the utilization-to-power relationship established
//! for GPUs by:
//! S. Hong and H. Kim, "An Integrated GPU Power and Performance Model,"
//! in Proc. 37th Int'l Symposium on Computer Architecture (ISCA), 2010,
//! pp. 280-289. doi: https://doi.org/10.1145/1815961.1815998
//! (GPU power grows non-linearly with core utilization due to clock/voltage
//! gating at low load — the same shape used for CPU TDP estimation, see
//! `cpu::estimation`, applies here with GPU-specific idle/curve constants.)

use std::time::Duration;

use common::types::EnergyUj;

/// Typical board power budget (TDP, in watts) for common discrete GPU models.
static GPU_TDP_TABLE: &[(&str, f64)] = &[
    // Mobile GPUs (laptops)
    ("RTX 4090 Laptop", 120.0),
    ("RTX 4080 Laptop", 110.0),
    ("RTX 4070 Laptop", 115.0),
    ("RTX 4060 Laptop", 115.0),
    ("RTX 3080 Laptop", 115.0),
    ("RTX 3070 Laptop", 115.0),
    ("RTX 3060 Laptop", 80.0),
    ("RX 7900M", 180.0),
    ("RX 7800M", 180.0),
    ("RX 6800M", 145.0),
    ("RX 6700M", 135.0),
    ("Laptop", 75.0),
    // NVIDIA
    ("RTX 4090", 450.0),
    ("RTX 4080", 320.0),
    ("RTX 4070", 200.0),
    ("RTX 4060", 115.0),
    ("RTX 3090", 350.0),
    ("RTX 3080", 320.0),
    ("RTX 3070", 220.0),
    ("RTX 3060", 170.0),
    ("GTX 1660", 120.0),
    ("T1000", 50.0),
    // AMD
    ("RX 7900 XT", 300.0),
    ("RX 7800 XT", 263.0),
    ("RX 7700 XT", 245.0),
    ("RX 7700", 200.0),
    ("RX 6950 XT", 335.0),
    ("RX 6800", 250.0),
    ("RX 6700", 230.0),
    ("RX 9060 XT", 132.0),
    ("RX 9060", 132.0),
    ("RX 9070 XT", 304.0),
    ("RX 9070", 220.0),
    // Intel
    ("Arc A770", 225.0),
    ("Arc A750", 225.0),
    ("Arc A380", 75.0),
    ("Intel", 15.0), // generic fallback for Intel integrated GPUs
];

const DEFAULT_DGPU_TDP: f64 = 150.0;
/// Idle power floor as a fraction of TDP (fans/VRAM/clock still active at 0% load).
const IDLE_FRACTION: f64 = 0.1;
/// Curve exponent from the Hong & Kim non-linear utilization-power fit.
const CURVE_EXPONENT: f64 = 1.6;

/// Looks up the TDP for a GPU model name, falling back to a default.
pub fn lookup_tdp(gpu_name: &str) -> f64 {
    let name_lower = gpu_name.to_lowercase();
    GPU_TDP_TABLE
        .iter()
        .find(|(pattern, _)| name_lower.contains(&pattern.to_lowercase()))
        .map(|(_, tdp)| *tdp)
        .unwrap_or(DEFAULT_DGPU_TDP)
}

fn estimate_power(tdp: f64, usage_percent: f64) -> f64 {
    let usage_frac = (usage_percent / 100.0).clamp(0.0, 1.0);
    let idle = tdp * IDLE_FRACTION;
    idle + (tdp - idle) * usage_frac.powf(CURVE_EXPONENT)
}

/// Estimates energy (µJ) consumed over `duration` from usage (%) and GPU model name.
pub fn estimate_energy(gpu_name: &str, usage_percent: f64, duration: Duration) -> EnergyUj {
    let tdp = lookup_tdp(gpu_name);
    let energy_joules = estimate_power(tdp, usage_percent) * duration.as_secs_f64();
    EnergyUj::from_joules(energy_joules)
}
