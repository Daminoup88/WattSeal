// ## Architecture des données

// Événements (timestamp, valeur):
//     - POWER :
//         - Intel RAPL (PKG, PP0, PP1, DRAM)
//         - AMD RAPL
//         - NVSMI
//         - RAM (estimation)
//         - Disques, périphériques (estimation)
//         - Autres
//         - TOTAL
//     - UTILISATION :
//         - CPU
//         - GPU (NVSMI)
//         - RAM

// Configuration

pub struct Event<T>{
    timestamp: u64,
    value: T,
}

pub enum OS {
    Windows,
    Linux,
    MacOS,
}