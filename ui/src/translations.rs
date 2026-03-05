use common::{CPUData, DatabaseEntry, DiskData, GPUData, MetricType, NetworkData, ProcessData, RamData, TotalData};

use crate::types::{AppLanguage, TimeRange};

// Window title

pub fn app_name(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "WattSeal",
        AppLanguage::French => "WattSeal",
    }
}

// Page titles

pub fn page_dashboard(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Dashboard",
        AppLanguage::French => "Tableau de bord",
    }
}

pub fn page_info(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Info",
        AppLanguage::French => "Infos",
    }
}

// Settings page

pub fn settings_title(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Settings",
        AppLanguage::French => "Paramètres",
    }
}

pub fn settings_general(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "General",
        AppLanguage::French => "Général",
    }
}

pub fn settings_theme(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Theme",
        AppLanguage::French => "Thème",
    }
}

pub fn settings_language(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Language",
        AppLanguage::French => "Langue",
    }
}

pub fn modal_close(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Close",
        AppLanguage::French => "Fermer",
    }
}

// Dashboard

pub fn current_power_consumption(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Current power consumption",
        AppLanguage::French => "Consommation actuelle",
    }
}

pub fn all_time(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "All Time",
        AppLanguage::French => "Total",
    }
}

pub fn emissions(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Emissions",
        AppLanguage::French => "Émissions",
    }
}

pub fn zero_carbon_intensity_warning(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "\u{26a0} Choose a real carbon intensity in the settings! \u{26a0}",
        AppLanguage::French => "\u{26a0} Choisissez une intensité carbone réelle dans les paramètres ! \u{26a0}",
    }
}

// Info page

pub fn cpu(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "CPU",
        AppLanguage::French => "CPU",
    }
}

pub fn processor_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Processor Information",
        AppLanguage::French => "Informations processeur",
    }
}

pub fn model(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Model",
        AppLanguage::French => "Modèle",
    }
}

pub fn cores(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Cores",
        AppLanguage::French => "Cœurs",
    }
}

pub fn cores_and_threads(language: AppLanguage, physical: u16, logical: u16) -> String {
    match language {
        AppLanguage::English => format!("{} cores / {} threads", physical, logical),
        AppLanguage::French => format!("{} cœurs / {} threads", physical, logical),
    }
}

pub fn gpu(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "GPU",
        AppLanguage::French => "GPU",
    }
}

pub fn graphics_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Graphics Information",
        AppLanguage::French => "Informations graphiques",
    }
}

pub fn graphics_processor_n(language: AppLanguage, n: usize) -> String {
    match language {
        AppLanguage::English => format!("Graphics Processor {}", n),
        AppLanguage::French => format!("Processeur graphique {}", n),
    }
}

pub fn memory(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Memory",
        AppLanguage::French => "Mémoire",
    }
}

pub fn ram_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "RAM Information",
        AppLanguage::French => "Informations RAM",
    }
}

pub fn total_memory(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Total Memory",
        AppLanguage::French => "Mémoire totale",
    }
}

pub fn swap(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Swap",
        AppLanguage::French => "Swap",
    }
}

pub fn system(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "System",
        AppLanguage::French => "Système",
    }
}

pub fn os_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "OS Information",
        AppLanguage::French => "Informations OS",
    }
}

pub fn operating_system(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Operating System",
        AppLanguage::French => "Système d'exploitation",
    }
}

pub fn hostname(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Hostname",
        AppLanguage::French => "Nom d'hôte",
    }
}

pub fn storage(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Storage",
        AppLanguage::French => "Stockage",
    }
}

pub fn disk_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Disk Information",
        AppLanguage::French => "Informations disque",
    }
}

pub fn disk(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Disk",
        AppLanguage::French => "Disque",
    }
}

pub fn disk_n(language: AppLanguage, n: usize) -> String {
    match language {
        AppLanguage::English => format!("Disk {}", n),
        AppLanguage::French => format!("Disque {}", n),
    }
}

pub fn space(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Space",
        AppLanguage::French => "Espace",
    }
}

pub fn network(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Network",
        AppLanguage::French => "Réseau",
    }
}

pub fn battery(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Battery",
        AppLanguage::French => "Batterie",
    }
}

pub fn battery_status(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Battery Status",
        AppLanguage::French => "État de la batterie",
    }
}

pub fn process(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Processes",
        AppLanguage::French => "Processus",
    }
}

pub fn name(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Name",
        AppLanguage::French => "Nom",
    }
}

pub fn capacity(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Capacity",
        AppLanguage::French => "Capacité",
    }
}

pub fn capacity_wh_cycles(language: AppLanguage, cap_wh: f32, cycles: u32) -> String {
    match language {
        AppLanguage::English => format!("{:.1} Wh ({} cycles)", cap_wh, cycles),
        AppLanguage::French => format!("{:.1} Wh ({} cycles)", cap_wh, cycles),
    }
}

pub fn capacity_wh_only(language: AppLanguage, cap_wh: f32) -> String {
    match language {
        AppLanguage::English => format!("{:.1} Wh", cap_wh),
        AppLanguage::French => format!("{:.1} Wh", cap_wh),
    }
}

pub fn na_with_cycles(language: AppLanguage, cycles: u32) -> String {
    match language {
        AppLanguage::English => format!("N/A ({} cycles)", cycles),
        AppLanguage::French => format!("N/A ({} cycles)", cycles),
    }
}

pub fn display(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Display",
        AppLanguage::French => "Écran",
    }
}

pub fn screen_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Screen Information",
        AppLanguage::French => "Informations écran",
    }
}

pub fn mode(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Mode",
        AppLanguage::French => "Mode",
    }
}

pub fn primary_display(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Primary Display",
        AppLanguage::French => "Écran principal",
    }
}

pub fn secondary_display(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Secondary Display",
        AppLanguage::French => "Écran secondaire",
    }
}

// General

pub fn na(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "N/A",
        AppLanguage::French => "N/A",
    }
}

pub fn no_data_available(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "No data available",
        AppLanguage::French => "Aucune donnée disponible",
    }
}

// Charts

pub fn power_label(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Power:",
        AppLanguage::French => "Puissance :",
    }
}

pub fn energy_label(language: AppLanguage) -> &'static str {
    energy(language)
}

/// Returns "Power:" or "Energy:" label depending on energy mode.
pub fn power_or_energy_label(language: AppLanguage, energy_mode: bool) -> &'static str {
    if energy_mode {
        energy_label(language)
    } else {
        power_label(language)
    }
}

pub fn tooltip_value(language: AppLanguage, value_text: &str) -> String {
    match language {
        AppLanguage::English => format!("Value: {}", value_text),
        AppLanguage::French => format!("Valeur : {}", value_text),
    }
}

pub fn tooltip_time(language: AppLanguage, time_text: &str) -> String {
    match language {
        AppLanguage::English => format!("Time: {}", time_text),
        AppLanguage::French => format!("Heure : {}", time_text),
    }
}

// Process list

pub fn application(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Application",
        AppLanguage::French => "Application",
    }
}

pub fn power(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Power",
        AppLanguage::French => "Puissance",
    }
}

pub fn energy(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Energy",
        AppLanguage::French => "Énergie",
    }
}

/// Returns "Power" or "Energy" depending on whether energy mode is active.
pub fn power_or_energy(language: AppLanguage, energy_mode: bool) -> &'static str {
    if energy_mode { energy(language) } else { power(language) }
}

/// Returns the label for the power/energy column header with unit.
pub fn power_or_energy_with_unit(language: AppLanguage, energy_mode: bool) -> String {
    if energy_mode {
        format!("{} (Wh)", energy(language))
    } else {
        format!("{} (W)", power(language))
    }
}

pub fn ram(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "RAM",
        AppLanguage::French => "RAM",
    }
}

pub fn disk_read(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Disk read",
        AppLanguage::French => "Lecture disque",
    }
}

pub fn disk_write(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Disk write",
        AppLanguage::French => "Écriture disque",
    }
}

// Time ranges

pub fn last_minute(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Minute",
        AppLanguage::French => "Dernière minute",
    }
}

pub fn last_hour(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Hour",
        AppLanguage::French => "Dernière heure",
    }
}

pub fn last_24_hours(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last 24 Hours",
        AppLanguage::French => "Dernières 24 heures",
    }
}

pub fn last_week(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Week",
        AppLanguage::French => "Dernière semaine",
    }
}

pub fn last_month(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Month",
        AppLanguage::French => "Dernier mois",
    }
}

pub fn last_year(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Year",
        AppLanguage::French => "Dernière année",
    }
}

pub fn time_range_name(language: AppLanguage, range: &TimeRange) -> &'static str {
    match range {
        TimeRange::LastMinute => last_minute(language),
        TimeRange::LastHour => last_hour(language),
        TimeRange::Last24Hours => last_24_hours(language),
        TimeRange::LastWeek => last_week(language),
        TimeRange::LastMonth => last_month(language),
        TimeRange::LastYear => last_year(language),
    }
}

// Metrics

pub fn metric_power(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Power",
        AppLanguage::French => "Puissance",
    }
}

pub fn metric_energy(language: AppLanguage) -> &'static str {
    energy(language)
}

pub fn metric_usage(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Usage",
        AppLanguage::French => "Utilisation",
    }
}

pub fn metric_speed(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Speed",
        AppLanguage::French => "Vitesse",
    }
}

pub fn metric_type_name(language: AppLanguage, metric: MetricType) -> &'static str {
    match metric {
        MetricType::Power => metric_power(language),
        MetricType::Usage => metric_usage(language),
        MetricType::Speed => metric_speed(language),
    }
}

// Labels

pub fn label_usage(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Usage",
        AppLanguage::French => "Utilisation",
    }
}

pub fn label_read(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Read",
        AppLanguage::French => "Lecture",
    }
}

pub fn label_write(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Write",
        AppLanguage::French => "Écriture",
    }
}

pub fn label_download(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Download",
        AppLanguage::French => "Téléchargement",
    }
}

pub fn label_upload(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Upload",
        AppLanguage::French => "Envoi",
    }
}

pub fn translate_label(language: AppLanguage, english_label: &str) -> &'static str {
    match english_label {
        "Power" => metric_power(language),
        "Usage" => label_usage(language),
        "Speed" => metric_speed(language),
        "Read" => label_read(language),
        "Write" => label_write(language),
        "Download" => label_download(language),
        "Upload" => label_upload(language),
        _ => match language {
            AppLanguage::English => "Unknown",
            AppLanguage::French => "Inconnu",
        },
    }
}

pub fn sensor_name<'a>(language: AppLanguage, english_name: &'a str) -> &'a str {
    match (language, english_name) {
        (_, "CPU") => "CPU",
        (_, "GPU") => "GPU",
        (_, "RAM") => "RAM",
        (AppLanguage::French, "Disk") => "Disque",
        (AppLanguage::French, "Network") => "Réseau",
        (AppLanguage::French, "Processes") => "Processus",
        _ => english_name,
    }
}

pub fn chart_legend(language: AppLanguage, metric_label: &str) -> String {
    let metric = translate_label(language, metric_label);
    metric.to_string()
}

// Carbon intensity / setup

pub fn settings_electricity_cost(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Electricity Cost",
        AppLanguage::French => "Coût de l'électricité",
    }
}

pub fn kwh_cost_placeholder(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "e.g. 0.20",
        AppLanguage::French => "ex. 0.20",
    }
}

pub fn kwh_cost_invalid(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Please enter a positive number ($/kWh)",
        AppLanguage::French => "Entrez un nombre positif ($/kWh)",
    }
}

pub fn setup_choose_electricity(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Select your electricity price",
        AppLanguage::French => "Sélectionnez votre tarif d'électricité",
    }
}

pub fn custom_kwh_cost_placeholder(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "e.g. 0.25",
        AppLanguage::French => "ex. 0.25",
    }
}

pub fn electricity_bill(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Est. Bill",
        AppLanguage::French => "Facture est.",
    }
}

pub fn settings_carbon_intensity(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Carbon Intensity",
        AppLanguage::French => "Intensité carbone",
    }
}

pub fn setup_welcome_title(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Welcome to WattSeal",
        AppLanguage::French => "Bienvenue sur WattSeal",
    }
}

pub fn setup_choose_language(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Choose your language",
        AppLanguage::French => "Choisissez votre langue",
    }
}

pub fn setup_choose_carbon(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Select the carbon intensity of your electricity grid",
        AppLanguage::French => "Sélectionnez l'intensité carbone de votre réseau électrique",
    }
}

pub fn setup_confirm(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Confirm",
        AppLanguage::French => "Confirmer",
    }
}

pub fn custom_carbon_placeholder(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "e.g. 300",
        AppLanguage::French => "ex. 300",
    }
}

pub fn custom_carbon_invalid(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Please enter a positive number (g CO₂/kWh)",
        AppLanguage::French => "Entrez un nombre positif (g CO₂/kWh)",
    }
}

// Info modal

pub fn info_modal_current_power(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Current power",
        AppLanguage::French => "Puissance actuelle",
    }
}

pub fn info_modal_all_time_power(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "All-time energy",
        AppLanguage::French => "Énergie totale",
    }
}

pub fn info_modal_top_consumer(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Highest consumer",
        AppLanguage::French => "Plus gros consommateur",
    }
}

pub fn info_modal_top_process(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Top process",
        AppLanguage::French => "Processus principal",
    }
}

pub fn info_modal_coming_soon(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Coming soon",
        AppLanguage::French => "Bientôt disponible",
    }
}

pub fn info_modal_title(language: AppLanguage, key: &str) -> String {
    if key == CPUData::table_name_static() {
        return cpu(language).to_string();
    } else if key == GPUData::table_name_static() {
        return gpu(language).to_string();
    } else if key == RamData::table_name_static() {
        return ram(language).to_string();
    } else if key == DiskData::table_name_static() {
        return disk(language).to_string();
    } else if key == NetworkData::table_name_static() {
        return network(language).to_string();
    } else if key == TotalData::table_name_static() {
        return all_time(language).to_string();
    } else if key == ProcessData::table_name_static() {
        return process(language).to_string();
    } else {
        return match key {
            "system" => system(language).to_string(),
            "battery" => battery(language).to_string(),
            "display" => display(language).to_string(),
            "carbon_emissions" => match language {
                AppLanguage::English => "Carbon Emissions".to_string(),
                AppLanguage::French => "Émissions carbone".to_string(),
            },
            _ => match language {
                AppLanguage::English => "Info".to_string(),
                AppLanguage::French => "Info".to_string(),
            },
        };
    }
}

pub fn info_modal_description(language: AppLanguage, key: &str) -> &'static str {
    if key == CPUData::table_name_static() {
        return match language {
            AppLanguage::English => {
                "The CPU (Central Processing Unit) is the brain of your computer. \
                 It executes all instructions and computations.\n\n\
                 Main power consumers:\n\
                 \u{2022} Higher clock speeds increase power draw\n\
                 \u{2022} More active cores = more consumption\n\
                 \u{2022} Intensive tasks (compilation, encoding) spike usage\n\
                 \u{2022} Higher voltages (overclocking) raise consumption"
            }
            AppLanguage::French => {
                "Le CPU (processeur central) est le cerveau de votre ordinateur. \
                 Il exécute toutes les instructions et calculs.\n\n\
                 Principaux consommateurs d'énergie :\n\
                 \u{2022} Des fréquences plus élevées augmentent la consommation\n\
                 \u{2022} Plus de cœurs actifs = plus de consommation\n\
                 \u{2022} Les tâches intensives (compilation, encodage) augmentent la charge\n\
                 \u{2022} Des tensions plus élevées (overclocking) augmentent la consommation"
            }
        };
    } else if key == GPUData::table_name_static() {
        return match language {
            AppLanguage::English => {
                "The GPU (Graphics Processing Unit) handles graphics rendering and \
                 parallel computations.\n\n\
                 Main power consumers:\n\
                 \u{2022} 3D rendering and gaming\n\
                 \u{2022} Video encoding / decoding\n\
                 \u{2022} AI and machine learning workloads\n\
                 \u{2022} High VRAM usage and memory bandwidth"
            }
            AppLanguage::French => {
                "Le GPU (processeur graphique) gère le rendu graphique et les calculs \
                 parallèles.\n\n\
                 Principaux consommateurs d'énergie :\n\
                 \u{2022} Rendu 3D et jeux vidéo\n\
                 \u{2022} Encodage / décodage vidéo\n\
                 \u{2022} Charges IA et apprentissage automatique\n\
                 \u{2022} Utilisation élevée de la VRAM"
            }
        };
    } else if key == RamData::table_name_static() {
        return match language {
            AppLanguage::English => {
                "RAM (Random Access Memory) provides fast temporary storage for running \
                 programs and active data.\n\n\
                 Main power consumers:\n\
                 \u{2022} Higher memory frequencies (MHz)\n\
                 \u{2022} More active memory modules\n\
                 \u{2022} Frequent read/write operations\n\
                 \u{2022} Always draws power while the system is on"
            }
            AppLanguage::French => {
                "La RAM (mémoire vive) fournit un stockage temporaire rapide pour les \
                 programmes en cours et les données actives.\n\n\
                 Principaux consommateurs d'énergie :\n\
                 \u{2022} Fréquences mémoire plus élevées (MHz)\n\
                 \u{2022} Plus de modules mémoire actifs\n\
                 \u{2022} Opérations de lecture/écriture fréquentes\n\
                 \u{2022} Consomme toujours de l'énergie tant que le système est allumé"
            }
        };
    } else if key == DiskData::table_name_static() {
        return match language {
            AppLanguage::English => {
                "Storage drives (SSD / HDD) provide permanent data storage for your \
                 files and system.\n\n\
                 Main power consumers:\n\
                 \u{2022} Sustained read/write operations\n\
                 \u{2022} Spinning platters (HDD)\n\
                 \u{2022} NAND write operations (SSD)\n\
                 \u{2022} Drive seeking and indexing"
            }
            AppLanguage::French => {
                "Les disques de stockage (SSD / HDD) fournissent un stockage permanent \
                 pour vos fichiers et votre système.\n\n\
                 Principaux consommateurs d'énergie :\n\
                 \u{2022} Opérations de lecture/écriture soutenues\n\
                 \u{2022} Plateaux en rotation (HDD)\n\
                 \u{2022} Opérations d'écriture NAND (SSD)\n\
                 \u{2022} Recherche et indexation sur le disque"
            }
        };
    } else if key == NetworkData::table_name_static() {
        return match language {
            AppLanguage::English => {
                "Network interfaces handle data transmission between your computer \
                 and other devices or the internet.\n\n\
                 Main power consumers:\n\
                 \u{2022} High data throughput\n\
                 \u{2022} Wi-Fi radio transmission\n\
                 \u{2022} Active network connections\n\
                 \u{2022} Bluetooth and wireless peripherals"
            }
            AppLanguage::French => {
                "Les interfaces réseau gèrent la transmission de données entre votre \
                 ordinateur et d'autres appareils ou internet.\n\n\
                 Principaux consommateurs d'énergie :\n\
                 \u{2022} Débit de données élevé\n\
                 \u{2022} Transmission radio Wi-Fi\n\
                 \u{2022} Connexions réseau actives\n\
                 \u{2022} Bluetooth et périphériques sans fil"
            }
        };
    } else if key == ProcessData::table_name_static() {
        return match language {
            AppLanguage::English => {
                "Shows which applications consume the most power on your system.\n\n\
                 Power is estimated based on CPU, GPU, and disk usage of each process. \
                 Background processes and services also contribute to total consumption."
            }
            AppLanguage::French => {
                "Montre quelles applications consomment le plus d'énergie sur votre \
                 système.\n\n\
                 La puissance est estimée à partir de l'utilisation CPU, GPU et disque \
                 de chaque processus. Les processus en arrière-plan et les services \
                 contribuent aussi à la consommation totale."
            }
        };
    } else if key == TotalData::table_name_static() {
        return match language {
            AppLanguage::English => {
                "Shows the total power consumption of your entire system.\n\n\
                 This is the sum of all hardware components (CPU, GPU, RAM, Disk, \
                 Network). Understanding which component consumes the most helps \
                 optimize energy usage."
            }
            AppLanguage::French => {
                "Affiche la consommation totale de votre système.\n\n\
                 C'est la somme de tous les composants (CPU, GPU, RAM, Disque, \
                 Réseau). Comprendre quel composant consomme le plus aide à \
                 optimiser la consommation d'énergie."
            }
        };
    } else if key == "carbon_emissions" {
        return match language {
            AppLanguage::English => {
                "These emissions reflect only the CO₂ generated by the electricity \
                 your PC consumes while running.\n\n\
                 To estimate your full carbon footprint, you should also add:\n\
                 \u{2022} Manufacturing emissions (~300–500 kg CO₂eq one-time)\n\
                 \u{2022} Transport emissions (~20–50 kg CO₂eq one-time)\n\
                 \u{2022} Network infrastructure emissions (~30–70 kg CO₂/year)\n\n\
                 The estimates below use ~400 kg CO₂eq for manufacturing + transport \
                 and ~50 kg CO₂/year for network usage."
            }
            AppLanguage::French => {
                "Ces émissions ne reflètent que le CO₂ généré par l'électricité \
                 consommée par votre PC en fonctionnement.\n\n\
                 Pour estimer votre empreinte carbone complète, vous devriez aussi \
                 inclure :\n\
                 \u{2022} Émissions de fabrication (~300–500 kg CO₂eq une fois)\n\
                 \u{2022} Émissions de transport (~20–50 kg CO₂eq une fois)\n\
                 \u{2022} Émissions liées aux réseaux (~30–70 kg CO₂/an)\n\n\
                 Les estimations ci-dessous utilisent ~400 kg CO₂eq pour fabrication \
                 + transport et ~50 kg CO₂/an pour l'utilisation réseau."
            }
        };
    } else {
        return match key {
            "system" => match language {
                AppLanguage::English => {
                    "Your operating system manages all hardware resources and running \
                     software.\n\n\
                     Impact on power:\n\
                     \u{2022} Background services and scheduled tasks\n\
                     \u{2022} System indexing and updates\n\
                     \u{2022} Power plan settings affect all components"
                }
                AppLanguage::French => {
                    "Votre système d'exploitation gère toutes les ressources matérielles \
                     et les logiciels en cours.\n\n\
                     Impact sur la consommation :\n\
                     \u{2022} Services en arrière-plan et tâches planifiées\n\
                     \u{2022} Indexation et mises à jour du système\n\
                     \u{2022} Les paramètres du plan d'alimentation affectent tous les \
                     composants"
                }
            },
            "battery" => match language {
                AppLanguage::English => {
                    "The battery stores energy for portable use and affects how power is \
                     managed.\n\n\
                     Key facts:\n\
                     \u{2022} Cycle count reflects battery health and aging\n\
                     \u{2022} Design capacity decreases over time\n\
                     \u{2022} Running on battery often triggers power-saving modes\n\
                     \u{2022} Fast charging generates more heat and uses more energy"
                }
                AppLanguage::French => {
                    "La batterie stocke l'énergie pour une utilisation portable et \
                     influence la gestion de l'alimentation.\n\n\
                     Points clés :\n\
                     \u{2022} Le nombre de cycles reflète l'état et le vieillissement de \
                     la batterie\n\
                     \u{2022} La capacité diminue avec le temps\n\
                     \u{2022} L'utilisation sur batterie active souvent des modes \
                     d'économie\n\
                     \u{2022} La charge rapide génère plus de chaleur et consomme plus"
                }
            },
            "display" => match language {
                AppLanguage::English => {
                    "Displays are a major power consumer, especially at high brightness.\n\n\
                     Main power consumers:\n\
                     \u{2022} Screen brightness (biggest factor)\n\
                     \u{2022} Higher refresh rates (Hz)\n\
                     \u{2022} Higher resolutions\n\
                     \u{2022} HDR and wide color gamut"
                }
                AppLanguage::French => {
                    "Les écrans sont un gros consommateur d'énergie, surtout à haute \
                     luminosité.\n\n\
                     Principaux consommateurs d'énergie :\n\
                     \u{2022} Luminosité de l'écran (facteur principal)\n\
                     \u{2022} Taux de rafraîchissement élevés (Hz)\n\
                     \u{2022} Résolutions plus élevées\n\
                     \u{2022} HDR et gamme de couleurs étendue"
                }
            },
            "storage" => match language {
                AppLanguage::English => {
                    "Storage drives (SSD / HDD) provide permanent data storage for your \
                     files and system.\n\n\
                     Main power consumers:\n\
                     \u{2022} Sustained read/write operations\n\
                     \u{2022} Spinning platters (HDD)\n\
                     \u{2022} NAND write operations (SSD)\n\
                     \u{2022} Drive seeking and indexing"
                }
                AppLanguage::French => {
                    "Les disques de stockage (SSD / HDD) fournissent un stockage \
                     permanent pour vos fichiers et votre système.\n\n\
                     Principaux consommateurs d'énergie :\n\
                     \u{2022} Opérations de lecture/écriture soutenues\n\
                     \u{2022} Plateaux en rotation (HDD)\n\
                     \u{2022} Opérations d'écriture NAND (SSD)\n\
                     \u{2022} Recherche et indexation sur le disque"
                }
            },
            _ => match language {
                AppLanguage::English => "No additional information available for this component.",
                AppLanguage::French => "Aucune information supplémentaire disponible pour ce composant.",
            },
        };
    }
}

/// Label for measured (app-tracked) CO₂ emissions row in the carbon info modal.
pub fn carbon_info_measured(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Measured by app",
        AppLanguage::French => "Mesuré par l'application",
    }
}

/// Label for estimated base (manufacturing + transport) CO₂ row.
pub fn carbon_info_base(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Estimated base (manufacturing + transport)",
        AppLanguage::French => "Base estimée (fabrication + transport)",
    }
}

/// Label for estimated annual network CO₂ row.
pub fn carbon_info_annual(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Estimated annual (network usage)",
        AppLanguage::French => "Annuel estimé (utilisation réseau)",
    }
}

/// Label for estimated all-time CO₂ total row.
pub fn carbon_info_all_time(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Estimated all-time total",
        AppLanguage::French => "Total estimé (toutes sources)",
    }
}

// Pick lists

#[derive(Debug, Clone, PartialEq)]
pub struct TranslatedTimeRange {
    pub range: TimeRange,
    language: AppLanguage,
}

impl TranslatedTimeRange {
    pub fn new(range: TimeRange, language: AppLanguage) -> Self {
        Self { range, language }
    }

    pub fn options_total(language: AppLanguage) -> Vec<Self> {
        TimeRange::all_total()
            .iter()
            .map(|r| Self::new(r.clone(), language))
            .collect()
    }

    pub fn options_component(language: AppLanguage) -> Vec<Self> {
        TimeRange::all_component()
            .iter()
            .map(|r| Self::new(r.clone(), language))
            .collect()
    }

    pub fn options(language: AppLanguage) -> Vec<Self> {
        Self::options_component(language)
    }
}

impl std::fmt::Display for TranslatedTimeRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", time_range_name(self.language, &self.range))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranslatedMetricType {
    pub metric: MetricType,
    language: AppLanguage,
}

impl TranslatedMetricType {
    pub fn new(metric: MetricType, language: AppLanguage) -> Self {
        Self { metric, language }
    }
}

impl std::fmt::Display for TranslatedMetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", metric_type_name(self.language, self.metric))
    }
}

// Close dialog

pub fn close_dialog_title(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Close WattSeal",
        AppLanguage::French => "Fermer WattSeal",
    }
}

pub fn close_dialog_description(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Do you want to close only the UI, or also stop the background collector?",
        AppLanguage::French => "Voulez-vous fermer uniquement l'interface, ou aussi arrêter le collecteur ?",
    }
}

pub fn close_ui_only(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Close UI only",
        AppLanguage::French => "Fermer l'interface",
    }
}

pub fn close_everything(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Close everything",
        AppLanguage::French => "Tout fermer",
    }
}
