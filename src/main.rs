use std::{thread, time::Duration, time::Instant};
use sysinfo::System;
use win_ring0::WinRing0;

mod core;
mod sensors;

pub fn main() {
    check_permissions();
    let sensor = sensors::cpu::get_cpu_power_sensor().unwrap();
}

fn check_permissions() {
    #[cfg(target_os = "windows")]
    {
        if !is_admin::is_admin() {
            eprintln!("This program requires Administrator privileges on Windows.");
            eprintln!("Please run this program as Administrator.");
            std::process::exit(1);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if !is_root() {
            eprintln!("This program requires root privileges on Linux.");
            eprintln!(
                "Please run with: sudo {}",
                std::env::current_exe().unwrap().display()
            );
            std::process::exit(1);
        }
    }
}

// pub fn main() {
//     // Check if we have the required privileges
    // check_permissions();

    // let mut r0: Box<WinRing0> = Box::from(WinRing0::new());

//     println!("Installing ring0 driver");
//     match r0.install() {
//         Ok(()) => {
//             println!("Driver installed");
//         }
//         Err(err) => {
//             println!("Error: {}", err);
//         }
//     }

//     println!("Opening ring0 driver");
//     match r0.open() {
//         Ok(()) => {
//             println!("Driver opened");
//         }
//         Err(err) => {
//             println!("Error: {}", err);
//         }
//     }

//     println!("Trying to get tjMax value, should work on most Intel CPUs");
//     // MSR_TEMPERATURE_TARGET
//     let msr = 0x1a2;
//     match r0.readMsr(msr) {
//         Ok(out) => {
//             let _edx = ((out >> 32) & 0xffffffff) as u32;
//             let eax = (out & 0xffffffff) as u32;
//             let tj_max = (eax >> 16) & 0xff;
//             println!("MSR Value: {}", tj_max);
//         }
//         Err(err) => {
//             println!("Error reading MSR: {}", err);
//         }
//     }

//     let mut energy_unit: f64 = 0.0;
//     // MSR_RAPL_POWER_UNIT
//     let msr = 0x606;
//     match r0.readMsr(msr) {
//         Ok(out) => {
//             // Split the 64 bit value into the original by offsetting and masking
//             // EDX: high order 32 bits
//             let _edx = ((out >> 32) & 0xffffffff) as u32;
//             // EAX: low order 32 bits
//             let eax = (out & 0xffffffff) as u32;
//             // power_unit = 1/2^PU where PU is bits 3:0 of EAX
//             let power_unit = 1.0 / f64::from(1 << (eax & 0xf));
//             // energy_unit = 1/2^EU where EU is bits 12:8 of EAX
//             energy_unit = 1.0 / f64::from(1 << ((eax >> 8) & 0x1f)) / 3600.0;
//             // time_unit = 1/2^TU where TU is bits 19:16 of EAX
//             let time_unit = 1.0 / f64::from(1 << ((eax >> 16) & 0xf));

//             println!("Raw Power Unit: {:b}", eax & 0xf);
//             println!("Raw Energy Unit: {:b}", (eax >> 8) & 0x1f);
//             println!("Raw Time Unit: {:b}", (eax >> 16) & 0xf);

//             println!("Power Unit: {}mW", power_unit);
//             println!("Energy Unit: {}Wh", energy_unit);
//             println!("Time Unit: {}s", time_unit);
//         }
//         Err(err) => {
//             println!("Error reading MSR: {}", err);
//         }
//     }

//     // AMD ENERGY_PWR_UNIT_MSR
//     let msr = 0xC0010299;
//     match r0.readMsr(msr) {
//         Ok(out) => {
//             let _edx = ((out >> 32) & 0xffffffff) as u32;
//             let eax = (out & 0xffffffff) as u32;
//             let energy_unit = 1.0 / f64::from(1 << (eax & 0xf));

//             println!("AMD Raw Energy Unit: {:b}", eax & 0xf);
//             println!("AMD Energy Unit: {}µJ", energy_unit);
//         }
//         Err(err) => {
//             println!("Error reading MSR: {}", err);
//         }
//     }

//     // AMD ENERGY_CORE_MSR
//     let msr = 0xC001029A;
//     match r0.readMsr(msr) {
//         Ok(out) => {
//             let edx = ((out >> 32) & 0xffffffff) as u32;
//             let eax = (out & 0xffffffff) as u32;
//             let energy = ((edx as u64) << 32) | (eax as u64);
//             println!("AMD Core Energy: {}", energy);
//         }
//         Err(err) => {
//             println!("Error reading MSR: {}", err);
//         }
//     }

//     // MSR_PKG_ENERGY_STATUS
//     let msr = 0x611;
//     match r0.readMsr(msr) {
//         Ok(energy) => {
//             println!("Package Energy: {} Wh", (energy as f64) * energy_unit);
//         }
//         Err(err) => {
//             println!("Error reading MSR: {}", err);
//         }
//     }

//     // MSR_DRAM_ENERGY_STATUS
//     let msr = 0x619;
//     match r0.readMsr(msr) {
//         Ok(energy) => {
//             println!("DRAM Energy: {} Wh", (energy as f64) * energy_unit);
//         }
//         Err(err) => {
//             println!("Error reading MSR: {}", err);
//         }
//     }

//     // MSR_PP0_ENERGY_STATUS
//     let msr = 0x639;
//     match r0.readMsr(msr) {
//         Ok(energy) => {
//             println!("PP0 Energy: {} Wh", (energy as f64) * energy_unit);
//         }
//         Err(err) => {
//             println!("Error reading MSR: {}", err);
//         }
//     }

//     // MSR_PP1_ENERGY_STATUS
//     let msr = 0x641;
//     match r0.readMsr(msr) {
//         Ok(energy) => {
//             println!("PP1 Energy: {} Wh", (energy as f64) * energy_unit);
//         }
//         Err(err) => {
//             println!("Error reading MSR: {}", err);
//         }
//     }

//     // Mesure de la puissance sur une période
//     println!("\n=== Mesure de la puissance ===");
    
//     // Première lecture des énergies
//     let start_time = Instant::now();
    
//     let pkg_energy_start = r0.readMsr(0x611).ok();
//     let dram_energy_start = r0.readMsr(0x619).ok();
//     let pp0_energy_start = r0.readMsr(0x639).ok();
//     let pp1_energy_start = r0.readMsr(0x641).ok();
    
//     // Attendre une seconde
//     println!("Mesure en cours (1 seconde)...");
//     thread::sleep(Duration::from_secs(1));
    
//     // Deuxième lecture des énergies
//     let elapsed = start_time.elapsed();
//     let elapsed_ms = elapsed.as_millis() as f64;
//     let elapsed_s = elapsed.as_secs_f64();
    
//     let pkg_energy_end = r0.readMsr(0x611).ok();
//     let dram_energy_end = r0.readMsr(0x619).ok();
//     let pp0_energy_end = r0.readMsr(0x639).ok();
//     let pp1_energy_end = r0.readMsr(0x641).ok();
    
//     // Calcul et affichage des puissances
//     println!("\nHeure (Temps écoulé): {:.3} ms", elapsed_ms);
//     println!("Temps écoulé: {:.3} s", elapsed_s);
//     println!();
    
//     if let (Some(start), Some(end)) = (dram_energy_start, dram_energy_end) {
//         let energy_diff = (end.wrapping_sub(start)) as f64 * energy_unit; // en Wh
//         let power_w = energy_diff / elapsed_s * 3600.0; // convertir Wh en W
//         let power_mw = power_w * 1000.0; // convertir W en mW
//         println!("RAPL_Package0_DRAM:");
//         println!("  Alimentation: {:.3} mW", power_mw);
//         println!("  Énergie consommée: {:.6} Wh", energy_diff);
//     }
    
//     if let (Some(start), Some(end)) = (pkg_energy_start, pkg_energy_end) {
//         let energy_diff = (end.wrapping_sub(start)) as f64 * energy_unit; // en Wh
//         let power_w = energy_diff / elapsed_s * 3600.0; // convertir Wh en W
//         let power_mw = power_w * 1000.0; // convertir W en mW
//         println!("\nRAPL_Package0_PKG:");
//         println!("  Alimentation: {:.3} mW", power_mw);
//         println!("  Énergie consommée: {:.6} Wh", energy_diff);
//     }
    
//     if let (Some(start), Some(end)) = (pp0_energy_start, pp0_energy_end) {
//         let energy_diff = (end.wrapping_sub(start)) as f64 * energy_unit; // en Wh
//         let power_w = energy_diff / elapsed_s * 3600.0; // convertir Wh en W
//         let power_mw = power_w * 1000.0; // convertir W en mW
//         println!("\nRAPL_Package0_PP0:");
//         println!("  Alimentation: {:.3} mW", power_mw);
//         println!("  Énergie consommée: {:.6} Wh", energy_diff);
//     }
    
//     if let (Some(start), Some(end)) = (pp1_energy_start, pp1_energy_end) {
//         let energy_diff = (end.wrapping_sub(start)) as f64 * energy_unit; // en Wh
//         let power_w = energy_diff / elapsed_s * 3600.0; // convertir Wh en W
//         let power_mw = power_w * 1000.0; // convertir W en mW
//         println!("\nRAPL_Package0_PP1:");
//         println!("  Alimentation: {:.3} mW", power_mw);
//         println!("  Énergie consommée: {:.6} Wh", energy_diff);
//     }

//     uninstall_driver(r0);
// }

// fn uninstall_driver(mut r0: Box<WinRing0>) {
//     println!("Closing ring0 driver");
//     match r0.close() {
//         Ok(()) => {
//             println!("Driver closed");
//         }
//         Err(err) => {
//             println!("Error: {}", err);
//         }
//     }

//     println!("Uninstall ring0 driver");
//     match r0.uninstall() {
//         Ok(()) => {
//             println!("Driver uninstalled");
//         }
//         Err(err) => {
//             println!("Error: {}", err);
//         }
//     }
// }
