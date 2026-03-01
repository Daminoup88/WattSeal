use common::{CPUData, DatabaseEntry, DiskData, GPUData, MetricType, NetworkData, RamData, SensorData, TotalData};

use crate::types::{AppLanguage, TimeRange};

// Window title

pub fn window_title(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Energy Monitor",
        AppLanguage::French => "Moniteur d'Énergie",
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

pub fn page_optimization(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Optimization",
        AppLanguage::French => "Optimisation",
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

pub fn settings_close(language: AppLanguage) -> &'static str {
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

pub fn time_range_name(language: AppLanguage, range: &TimeRange) -> &'static str {
    match range {
        TimeRange::LastMinute => last_minute(language),
        TimeRange::LastHour => last_hour(language),
        TimeRange::Last24Hours => last_24_hours(language),
    }
}

// Metrics

pub fn metric_power(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Power",
        AppLanguage::French => "Puissance",
    }
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

// pub fn sensor_name<'a>(language: AppLanguage, table_name: &'a str) -> &'a str {
//     if table_name == CPUData::table_name_static() {
//         cpu(language)
//     } else if table_name == GPUData::table_name_static() {
//         gpu(language)
//     } else if table_name == RamData::table_name_static() {
//         ram(language)
//     } else if table_name == DiskData::table_name_static() {
//         disk(language)
//     } else if table_name == NetworkData::table_name_static() {
//         network(language)
//     } else if table_name == TotalData::table_name_static() {
//         all_time(language)
//     } else {
//         match language {
//             AppLanguage::English => "Unknown",
//             AppLanguage::French => "Inconnu",
//         }
//     }
// }

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

// pub fn chart_legend(language: AppLanguage, component: &str, metric_label: &str) -> String {
//     let component = sensor_name(language, component);
//     let _ = translate_label(language, metric_label);
//     component.to_string()
// }

pub fn chart_legend(language: AppLanguage, metric_label: &str) -> String {
    let metric = translate_label(language, metric_label);
    metric.to_string()
}

pub fn optimization_content(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Optimization Page Content",
        AppLanguage::French => "Contenu de la page d'optimisation",
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

    pub fn options(language: AppLanguage) -> Vec<Self> {
        vec![
            Self::new(TimeRange::LastMinute, language),
            Self::new(TimeRange::LastHour, language),
            Self::new(TimeRange::Last24Hours, language),
        ]
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
