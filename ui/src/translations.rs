use common::{CPUData, DatabaseEntry, DiskData, GPUData, MetricKind, NetworkData, ProcessData, RamData, TotalData};

use crate::{
    themes::AppTheme,
    types::{AppLanguage, CarbonIntensity, ElectricityCost, TimeRange},
};

// Window title

pub fn app_name(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::French | AppLanguage::Romanian => "WattSeal",
    }
}

// Page titles

pub fn page_dashboard(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Dashboard",
        AppLanguage::French => "Tableau de bord",
        AppLanguage::Romanian => "Tablou de bord",
    }
}

pub fn page_info(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::Romanian => "Info",
        AppLanguage::French => "Infos",
    }
}

// Settings page

pub fn settings_title(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Settings",
        AppLanguage::French => "Paramètres",
        AppLanguage::Romanian => "Setări",
    }
}

pub fn settings_general(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::Romanian => "General",
        AppLanguage::French => "Général",
    }
}

pub fn settings_theme(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Theme",
        AppLanguage::French => "Thème",
        AppLanguage::Romanian => "Temă",
    }
}

pub fn settings_language(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Language",
        AppLanguage::French => "Langue",
        AppLanguage::Romanian => "Limbă",
    }
}

pub fn modal_close(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Close",
        AppLanguage::French => "Fermer",
        AppLanguage::Romanian => "Închide",
    }
}

// Dashboard

pub fn current_power_consumption(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Current power consumption",
        AppLanguage::French => "Consommation actuelle",
        AppLanguage::Romanian => "Consum curent",
    }
}

pub fn database_migrating_title(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Preparing database",
        AppLanguage::French => "Préparation de la base de données",
        AppLanguage::Romanian => "Bază de date în pregătire",
    }
}

pub fn database_migrating_description(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => {
            "WattSeal is waiting for the collector to finish migrating the database. Retrying every second…"
        }
        AppLanguage::French => {
            "WattSeal attend que le collecteur termine la migration de la base de données. Nouvel essai chaque seconde…"
        }
        AppLanguage::Romanian => {
            "WattSeal așteaptă colectorul să termine de migrat baza de date. Reîncercare în fiecare secundă..."
        }
    }
}

pub fn all_time(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "All Time",
        AppLanguage::French => "Depuis le début",
        AppLanguage::Romanian => "De la început",
    }
}

pub fn total(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::French | AppLanguage::Romanian => "Total",
    }
}

pub fn emissions(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Emissions",
        AppLanguage::French => "Émissions",
        AppLanguage::Romanian => "Emisii",
    }
}

pub fn zero_carbon_intensity_warning(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "\u{26a0} Choose a real carbon intensity in the settings! \u{26a0}",
        AppLanguage::French => "\u{26a0} Choisissez une intensité carbone réelle dans les paramètres ! \u{26a0}",
        AppLanguage::Romanian => "\u{26a0} Alegeți o intensitate de carbon reală în setări! \u{26a0}",
    }
}

// Info page

pub fn cpu(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::French | AppLanguage::Romanian => "CPU",
    }
}

pub fn processor_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Processor Information",
        AppLanguage::French => "Informations processeur",
        AppLanguage::Romanian => "Informații procesor",
    }
}

pub fn model(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::Romanian => "Model",
        AppLanguage::French => "Modèle",
    }
}

pub fn cores(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Cores",
        AppLanguage::French => "Cœurs",
        AppLanguage::Romanian => "Nuclee",
    }
}

pub fn cores_and_threads(language: AppLanguage, physical: u16, logical: u16) -> String {
    match language {
        AppLanguage::English => format!("{} cores / {} threads", physical, logical),
        AppLanguage::French => format!("{} cœurs / {} threads", physical, logical),
        AppLanguage::Romanian => format!("{} nuclee / {} thread-uri", physical, logical),
    }
}

pub fn gpu(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::French | AppLanguage::Romanian => "GPU",
    }
}

pub fn graphics_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Graphics Information",
        AppLanguage::French => "Informations graphiques",
        AppLanguage::Romanian => "Informații grafice",
    }
}

pub fn graphics_processor_n(language: AppLanguage, n: usize) -> String {
    match language {
        AppLanguage::English => format!("Graphics Processor {}", n),
        AppLanguage::French => format!("Processeur graphique {}", n),
        AppLanguage::Romanian => format!("Procesor grafic {}", n),
    }
}

pub fn memory(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Memory",
        AppLanguage::French => "Mémoire",
        AppLanguage::Romanian => "Memorie",
    }
}

pub fn ram_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "RAM Information",
        AppLanguage::French => "Informations RAM",
        AppLanguage::Romanian => "Informații RAM",
    }
}

pub fn total_memory(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Total Memory",
        AppLanguage::French => "Mémoire totale",
        AppLanguage::Romanian => "Memorie Totală",
    }
}

pub fn swap(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::French => "Swap",
        AppLanguage::Romanian => "Swap",
    }
}

pub fn system(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "System",
        AppLanguage::French => "Système",
        AppLanguage::Romanian => "Sistem",
    }
}

pub fn os_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "OS Information",
        AppLanguage::French => "Informations OS",
        AppLanguage::Romanian => "Informații Sistem de Operare",
    }
}

pub fn operating_system(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Operating System",
        AppLanguage::French => "Système d'exploitation",
        AppLanguage::Romanian => "Sistem de operare",
    }
}

pub fn hostname(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Hostname",
        AppLanguage::French => "Nom d'hôte",
        AppLanguage::Romanian => "Nume de gazdă",
    }
}

pub fn storage(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Storage",
        AppLanguage::French => "Stockage",
        AppLanguage::Romanian => "Stocare",
    }
}

pub fn disk_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Disk Information",
        AppLanguage::French => "Informations disque",
        AppLanguage::Romanian => "Informații disc",
    }
}

pub fn disk(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Disk",
        AppLanguage::French => "Disque",
        AppLanguage::Romanian => "Disc",
    }
}

pub fn disk_n(language: AppLanguage, n: usize) -> String {
    match language {
        AppLanguage::English | AppLanguage::French | AppLanguage::Romanian => format!("{} {}", disk(language), n),
    }
}

pub fn space(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Space",
        AppLanguage::French => "Espace",
        AppLanguage::Romanian => "Spațiu",
    }
}

pub fn network(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Network",
        AppLanguage::French => "Réseau",
        AppLanguage::Romanian => "Rețea",
    }
}

pub fn battery(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Battery",
        AppLanguage::French => "Batterie",
        AppLanguage::Romanian => "Baterie",
    }
}

pub fn battery_status(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Battery Status",
        AppLanguage::French => "État de la batterie",
        AppLanguage::Romanian => "Stare Baterie",
    }
}

pub fn process(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Processes",
        AppLanguage::French => "Processus",
        AppLanguage::Romanian => "Procese",
    }
}

pub fn name(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Name",
        AppLanguage::French => "Nom",
        AppLanguage::Romanian => "Nume",
    }
}

pub fn capacity(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Capacity",
        AppLanguage::French => "Capacité",
        AppLanguage::Romanian => "Capacitate",
    }
}

pub fn capacity_wh_cycles(language: AppLanguage, cap_wh: f32, cycles: u32) -> String {
    let val = format_number(cap_wh as f64, 1, language);
    match language {
        AppLanguage::English => format!("{} Wh ({} cycles)", val, cycles),
        AppLanguage::French => format!("{} Wh ({} cycles)", val, cycles),
        AppLanguage::Romanian => format!("{} Wh ({} cicluri)", val, cycles),
    }
}

pub fn capacity_wh_only(language: AppLanguage, cap_wh: f32) -> String {
    let val = format_number(cap_wh as f64, 1, language);
    match language {
        AppLanguage::English | AppLanguage::French | AppLanguage::Romanian => format!("{} Wh", val),
    }
}

pub fn na_with_cycles(language: AppLanguage, cycles: u32) -> String {
    match language {
        AppLanguage::English => format!("N/A ({} cycles)", cycles),
        AppLanguage::French => format!("N/A ({} cycles)", cycles),
        AppLanguage::Romanian => format!("N/A ({} cicluri)", cycles),
    }
}

pub fn display(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Display",
        AppLanguage::French => "Écran",
        AppLanguage::Romanian => "Ecran",
    }
}

pub fn screen_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Screen Information",
        AppLanguage::French => "Informations écran",
        AppLanguage::Romanian => "Informații ecran",
    }
}

pub fn mode(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Mode",
        AppLanguage::French => "Mode",
        AppLanguage::Romanian => "Mod",
    }
}

pub fn primary_display(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Primary Display",
        AppLanguage::French => "Écran principal",
        AppLanguage::Romanian => "Ecran principal",
    }
}

pub fn secondary_display(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Secondary Display",
        AppLanguage::French => "Écran secondaire",
        AppLanguage::Romanian => "Ecran secundar",
    }
}

// General

pub fn na(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::French | AppLanguage::Romanian => "N/A",
    }
}

pub fn no_data_available(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "No data available",
        AppLanguage::French => "Aucune donnée disponible",
        AppLanguage::Romanian => "Nu există date disponibile",
    }
}

// Charts

pub fn power_label(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Power:",
        AppLanguage::French => "Puissance :",
        AppLanguage::Romanian => "Putere:",
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
        AppLanguage::Romanian => format!("Valoare: {}", value_text),
    }
}

pub fn tooltip_time(language: AppLanguage, time_text: &str) -> String {
    match language {
        AppLanguage::English => format!("Time: {}", time_text),
        AppLanguage::French => format!("Heure : {}", time_text),
        AppLanguage::Romanian => format!("Timp: {}", time_text),
    }
}

// Process list

pub fn application(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::French => "Application",
        AppLanguage::Romanian => "Aplicație",
    }
}

pub fn power(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Power",
        AppLanguage::French => "Puissance",
        AppLanguage::Romanian => "Putere",
    }
}

pub fn energy(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Energy",
        AppLanguage::French => "Énergie",
        AppLanguage::Romanian => "Energie",
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
        AppLanguage::English | AppLanguage::French | AppLanguage::Romanian => "RAM",
    }
}

pub fn disk_read(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Disk read",
        AppLanguage::French => "Lecture disque",
        AppLanguage::Romanian => "Citire disc",
    }
}

pub fn disk_write(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Disk write",
        AppLanguage::French => "Écriture disque",
        AppLanguage::Romanian => "Scriere disc",
    }
}

// Time ranges

pub fn last_minute(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Minute",
        AppLanguage::French => "Dernière minute",
        AppLanguage::Romanian => "Ultimul minut",
    }
}

pub fn last_hour(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Hour",
        AppLanguage::French => "Dernière heure",
        AppLanguage::Romanian => "Ultima oră",
    }
}

pub fn last_24_hours(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last 24 Hours",
        AppLanguage::French => "Dernières 24 heures",
        AppLanguage::Romanian => "Ultimele 24 ore",
    }
}

pub fn last_week(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Week",
        AppLanguage::French => "Dernière semaine",
        AppLanguage::Romanian => "Ultima săptămână",
    }
}

pub fn last_month(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Month",
        AppLanguage::French => "Dernier mois",
        AppLanguage::Romanian => "Ultima lună",
    }
}

pub fn last_year(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Year",
        AppLanguage::French => "Dernière année",
        AppLanguage::Romanian => "Ultimul an",
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
        AppLanguage::Romanian => "Putere",
    }
}

pub fn metric_energy(language: AppLanguage) -> &'static str {
    energy(language)
}

pub fn metric_usage(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Usage",
        AppLanguage::French => "Utilisation",
        AppLanguage::Romanian => "Utilizare",
    }
}

pub fn metric_speed(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Speed",
        AppLanguage::French => "Vitesse",
        AppLanguage::Romanian => "Viteză",
    }
}

pub fn metric_type_name(language: AppLanguage, metric: MetricKind) -> &'static str {
    match metric {
        MetricKind::Power => metric_power(language),
        MetricKind::Usage => metric_usage(language),
        MetricKind::Speed => metric_speed(language),
    }
}

// Labels

pub fn label_usage(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::French | AppLanguage::Romanian => metric_usage(language),
    }
}

pub fn label_read(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Read",
        AppLanguage::French => "Lecture",
        AppLanguage::Romanian => "Citire",
    }
}

pub fn label_write(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Write",
        AppLanguage::French => "Écriture",
        AppLanguage::Romanian => "Scriere",
    }
}

pub fn label_download(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Download",
        AppLanguage::French => "Téléchargement",
        AppLanguage::Romanian => "Descărcare",
    }
}

pub fn label_upload(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Upload",
        AppLanguage::French => "Envoi",
        AppLanguage::Romanian => "Încărcare",
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
            AppLanguage::Romanian => "Necunoscut",
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
        (AppLanguage::Romanian, "Disk") => "Disc",
        (AppLanguage::Romanian, "Network") => "Rețea",
        (AppLanguage::Romanian, "Processes") => "Procese",
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
        AppLanguage::Romanian => "Cost electricitate",
    }
}

pub fn kwh_cost_placeholder(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "e.g. 0.20",
        AppLanguage::French => "ex. 0.20",
        AppLanguage::Romanian => "ex: 0.20",
    }
}

pub fn kwh_cost_invalid(language: AppLanguage, currency_symbol: &str) -> String {
    let text = match language {
        AppLanguage::English => "Enter a positive number",
        AppLanguage::French => "Entrez un nombre positif",
        AppLanguage::Romanian => "Introduceți un număr pozitiv",
    };
    return format!("{} ({}/kWh)", text, currency_symbol);
}

pub fn setup_choose_electricity(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Select your electricity price",
        AppLanguage::French => "Sélectionnez votre tarif d'électricité",
        AppLanguage::Romanian => "Selectați tariful dvs. de electricitate",
    }
}

pub fn custom_kwh_cost_placeholder(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "e.g. 0.25",
        AppLanguage::French => "ex. 0.25",
        AppLanguage::Romanian => "ex: 0.25",
    }
}

pub fn electricity_bill(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Estimated Bill",
        AppLanguage::French => "Facture estimée",
        AppLanguage::Romanian => "Factură estimată",
    }
}

pub fn settings_carbon_intensity(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Carbon Intensity",
        AppLanguage::French => "Intensité carbone",
        AppLanguage::Romanian => "Intensitate Carbon",
    }
}

pub fn setup_welcome_title(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Welcome to WattSeal",
        AppLanguage::French => "Bienvenue sur WattSeal",
        AppLanguage::Romanian => "Bine ați venit la WattSeal",
    }
}

pub fn setup_choose_language(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Choose your language",
        AppLanguage::French => "Choisissez votre langue",
        AppLanguage::Romanian => "Alegeți limba",
    }
}

pub fn setup_choose_carbon(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Select the carbon intensity of your electricity grid",
        AppLanguage::French => "Sélectionnez l'intensité carbone de votre réseau électrique",
        AppLanguage::Romanian => "Selectați intensitatea de carbon a rețelei electrice",
    }
}

pub fn setup_confirm(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Confirm",
        AppLanguage::French => "Confirmer",
        AppLanguage::Romanian => "Confirmați",
    }
}

pub fn custom_carbon_placeholder(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "e.g. 300",
        AppLanguage::French => "ex. 300",
        AppLanguage::Romanian => "ex: 300",
    }
}

pub fn custom_carbon_invalid(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Enter a positive number (g CO₂/kWh)",
        AppLanguage::French => "Entrez un nombre positif (g CO₂/kWh)",
        AppLanguage::Romanian => "Introduceți un număr pozitiv (g CO₂/kWh)",
    }
}

// Info modal

pub fn info_modal_current_power(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Current power",
        AppLanguage::French => "Puissance actuelle",
        AppLanguage::Romanian => "Putere actuală",
    }
}

pub fn info_modal_all_time_power(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "All-time energy",
        AppLanguage::French => "Énergie totale",
        AppLanguage::Romanian => "Energie totală",
    }
}

pub fn info_modal_current_top_consumer(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Current highest consumer",
        AppLanguage::French => "Plus gros consommateur actuel",
        AppLanguage::Romanian => "Cel mai mare consumator actual",
    }
}

pub fn info_modal_all_time_top_consumer(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "All-time top consumer",
        AppLanguage::French => "Plus gros consommateur total",
        AppLanguage::Romanian => "Cel mai mare consumator total",
    }
}

pub fn info_modal_top_process(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Top process",
        AppLanguage::French => "Processus le plus gourmand",
        AppLanguage::Romanian => "Cel mai intensiv proces",
    }
}

pub fn info_modal_coming_soon(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Coming soon",
        AppLanguage::French => "Bientôt disponible",
        AppLanguage::Romanian => "În curând",
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
        return total(language).to_string();
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
                AppLanguage::Romanian => "Emisii de carbon".to_string(),
            },
            _ => match language {
                AppLanguage::English => "Info".to_string(),
                AppLanguage::French => "Info".to_string(),
                AppLanguage::Romanian => "Info".to_string(),
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
            AppLanguage::Romanian => {
                "Unitatea centrală de procesare (CPU) este creierul calculatorului dumneavoastră. \
                 El execută toate instrucțiunile și calculele.\n\n\
                 Principalii factori de consum:\n\
                 \u{2022} Frecvențele mai mari cresc consumul\n\
                 \u{2022} Mai multe nuclee active = consum mai ridicat\n\
                 \u{2022} Sarcinile intensive (compilare, encodare) cresc consumul\n\
                 \u{2022} Tensiuni ridicate (overclocking) cresc consumul"
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
                 \u{2022} Charges IA et machine learning\n\
                 \u{2022} Utilisation élevée de la VRAM"
            }
            AppLanguage::Romanian => {
                "Unitatea de Procesare Grafică (GPU) se ocupă de randarea grafică și calcule \
                 paralele.\n\n\
                 Principalii consumatori:\n\
                 \u{2022} Randarea 3D și jocurile\n\
                 \u{2022} Encodare/decodare video\n\
                 \u{2022} Sarcini IA și învățare automată\n\
                 \u{2022} Utilizare intensă a VRAM-ului și lățime de bandă a memoriei"
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
                 \u{2022} Frequent read/write operations\n\n\
                 Overall, RAM consumes power as long as the system is on, even when idle."
            }
            AppLanguage::French => {
                "La RAM (mémoire vive) fournit un stockage temporaire rapide pour les \
                 programmes en cours et les données actives.\n\n\
                 Principaux consommateurs d'énergie :\n\
                 \u{2022} Fréquences mémoire plus élevées (MHz)\n\
                 \u{2022} Plus de modules mémoire actifs\n\
                 \u{2022} Opérations de lecture/écriture fréquentes\n\n\
                 Globalement, la RAM consomme de l'énergie tant que le système est allumé, même au repos."
            }
            AppLanguage::Romanian => {
                "Memoria RAM (Random Access Memory) oferă stocare temporară rapidă pentru\
                 programele în execuție și datele active.\n\n\
                 Consumatori principali:\n\
                 \u{2022} Frecvențe înalte de memorie (MHz)\n\
                 \u{2022} Mai multe module de memorie\n\
                 \u{2022} Citiri/Scrieri frecvente\n\n\
                 Pe scurt, memoria RAM consumă curent constant, cât timp sistemul este pornit,\
                 chiar și în stand-by."
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
            AppLanguage::Romanian => {
                "Discurile de stocare (SSD / HDD) oferă spațiu de stocare permanent pentru fișiere \
                 și sistem.\n\n\
                 Principalii consumatori:\n\
                 \u{2022} Operații de citire/scriere frecvente\n\
                 \u{2022} Rotirea discurilor (HDD)\n\
                 \u{2022} Operații de scriere NAND (SSD)\n\
                 \u{2022} Căutari și indexări pe disc"
            }
        };
    } else if key == NetworkData::table_name_static() {
        // Source for network emissions:
        // D. Al Kez, A. M. Foley, D. Laverty, D. F. Del Rio, and B. Sovacool, “Exploring the sustainability challenges facing digitalization and internet data centers,” Journal of Cleaner Production, vol. 371, no. 371, p. 133633, Aug. 2022, doi: https://doi.org/10.1016/j.jclepro.2022.133633.
        return match language {
            AppLanguage::English => {
                "Network interfaces handle data transmission between your computer \
                 and other devices or the internet.\n\n\
                 Main power consumers:\n\
                 \u{2022} High data throughput\n\
                 \u{2022} Wi-Fi radio transmission\n\
                 \u{2022} Active network connections\n\
                 \u{2022} Bluetooth and wireless peripherals\n\n\
                Note: Internet usage generates indirect emissions from network infrastructure and remote servers (28\u{2013}63 g CO₂/GB), which are not included in WattSeal's measurements for your PC."
            }
            AppLanguage::French => {
                "Les interfaces réseau gèrent la transmission de données entre votre \
                 ordinateur et d'autres appareils ou internet.\n\n\
                 Principaux consommateurs d'énergie :\n\
                 \u{2022} Débit de données élevé\n\
                 \u{2022} Transmission radio Wi-Fi\n\
                 \u{2022} Connexions réseau actives\n\
                 \u{2022} Bluetooth et périphériques sans fil\n\n\
                 Note : L'utilisation d'internet engendre des émissions indirectes dues aux infrastructures réseau et aux serveurs distants (28 à 63 g CO₂/Go), qui ne sont pas incluses dans les mesures de WattSeal pour votre PC."
            }
            AppLanguage::Romanian => {
                "Interfețele de rețea se ocupă de transmiterea datelor între calculatorul dvs. și \
                 alte dispozitive sau internet.\n\n\
                 Principalii consumatori:\n\
                 \u{2022} Debit ridicat de date\n\
                 \u{2022} Transmisii Wi-Fi\n\
                 \u{2022} Conexiuni de rețea active\n\
                 \u{2022} Periferice Bluetooth sau wireless\n\n\
                Notă: Utilizarea internetului generează emisii indirecte prin infrastructura de rețea și servere remote (28\u{2013}63 g CO₂/GB), care nu sunt incluse în măsurătorile WattSeal pentru calculatorul dumneavoastră."
            }
        };
    } else if key == ProcessData::table_name_static() {
        return match language {
            AppLanguage::English => {
                "Shows which applications consume the most power on your system.\n\n\
                 Power is estimated based on CPU and GPU (highest variable consumption) usage of each process.\n\
                 Background processes and services also contribute to total consumption."
            }
            AppLanguage::French => {
                "Montre quelles applications consomment le plus d'énergie sur votre \
                 système.\n\n\
                 La puissance est estimée à partir de l'utilisation CPU et GPU \
                 (consommations variables les plus élevées) \
                 de chaque processus.\n\
                 Les processus en arrière-plan et les services \
                 contribuent aussi à la consommation totale."
            }
            AppLanguage::Romanian => {
                "Arată care sunt aplicațiile cu cel mai mare consum de curent din sistem.\n\n\
                 Consumul este estimat pe baza utilizării de CPU și GPU (cel mai mare consum \
                 variabil) al fiecărui proces.\n\
                 Procesele și serviciile ascunse contribuie de asemenea la consumul total."
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
            AppLanguage::Romanian => {
                "Arată consumul total de curent al sistemului.\n\n\
                 Aceasta este suma totală a tuturor componentelor hardware (CPU, GPU, RAM, Disc, \
                 Rețea). Înțelegerea componentei care consumă cel mai mult ajută la optimizarea \
                 consumului de energie."
            }
        };
    } else if key == "carbon_emissions" {
        return match language {
            // Source for manufacturing emissions:
            // N. Six, “What’s the carbon footprint of a computer?,” Le Monde.fr, Apr. 30, 2023. https://www.lemonde.fr/en/pixels/article/2023/04/30/what-s-the-carbon-footprint-of-a-computer_6024865_13.html
            // Source for water consumption:
            // M. Yañez-Barnuevo, “Data Centers and Water Consumption | Article | EESI,” Eesi.org, Jun. 25, 2025. https://www.eesi.org/articles/view/data-centers-and-water-consumption
            AppLanguage::English => {
                "These emissions reflect only the CO₂ generated by the electricity \
                 your PC consumes while running.\n\n\
                 To estimate your full carbon footprint, you should also add:\n\
                 \u{2022} Manufacturing and transport emissions (~250\u{2013}500 kg CO₂eq one-time), add more for gaming PCs, additional monitors and peripherals\n\
                 \u{2022} Network infrastructure emissions (28\u{2013}63 g CO₂/GB), which are increasing with the rise of AI\n\
                 \u{2022} Hardware disposal and e-waste emissions (varies widely, leaching of toxic materials in landfills)\n\n\
                 Note: Digital technology also consumes a lot of water for datacenter cooling (around 1,9L/kWh) and mineral extraction, especially in water-stressed regions."
            }
            AppLanguage::French => {
                "Ces émissions reflètent le CO₂ généré par l'électricité \
                 consommée par votre PC en fonctionnement.\n\n\
                 Pour estimer l'empreinte carbone complète de votre ordinateur, vous devriez aussi ajouter :\n\
                 \u{2022} Émissions de fabrication et de transport (~250\u{2013}500 kg CO₂eq)\n\
                 \u{2022} Émissions d'infrastructure réseau (28 à 63 g CO₂/Go), en constante augmentation avec l'essor de l'IA\n\
                 \u{2022} Émissions liées à la fin de vie du matériel (varie largement, écoulement de matériaux toxiques dans les décharges)\n\n\
                 Note: Le numérique consomme aussi énormément d'eau pour le refroidissement des datacenters (environ 1,9L/kWh) et l'extraction des minerais, notamment dans des régions en stress hydrique."
            }
            AppLanguage::Romanian => {
                "Aceste emisii reflectă doar CO₂ generat de energia electrică consumată de PC în timpul funcționării.\n\n\
                 Pentru a estima amprenta de carbon completă, ar trebui să adăugați și:\n\
                 \u{2022} Emisiile din fabricație și transport (~250\u{2013}500 kg CO₂e, o singură dată), mai mult pentru PC-urile de gaming, monitoare suplimentare și periferice\n\
                 \u{2022} Emisiile din infrastructura rețelei (28\u{2013}63 g CO₂/GB), care cresc odată cu avansul IA\n\
                 \u{2022} Emisiile legate de eliminarea hardware-ului și deșeurilor electronice (variază mult, scurgeri de materiale toxice în depozite)\n\n\
                 Notă: Tehnologia digitală consumă și multă apă pentru răcirea centrelor de date (aprox. 1,9 L/kWh) și pentru extracția mineralelor, în special în regiuni cu stres hidric."
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
                     \u{2022} Power plan settings affect all components\n\n\
                     You can optimize this by disabling unnecessary startup programs, background processes running in the taskbar (weather, news...), and using power-saving modes."
                }
                AppLanguage::French => {
                    "Votre système d'exploitation gère toutes les ressources matérielles \
                     et les logiciels en cours.\n\n\
                     Impact sur la consommation :\n\
                     \u{2022} Services en arrière-plan et tâches planifiées\n\
                     \u{2022} Indexation et mises à jour du système\n\
                     \u{2022} Les paramètres du plan d'alimentation affectent tous les \
                     composants\n\n\
                    Vous pouvez optimiser cela en désactivant les programmes de démarrage \
                    inutiles, les processus d'arrière-plan dans la barre des tâches (météo, actualités...), et en utilisant les modes d'économie d'énergie."
                }
                AppLanguage::Romanian => {
                    "Sistemul de operare gestionează toate resursele hardware și software-ul care rulează.\n\n\
                     Impact asupra consumului de energie:\n\
                     \u{2022} Servicii de fundal și sarcini programate\n\
                     \u{2022} Indexarea și actualizările sistemului\n\
                     \u{2022} Setările planului de energie afectează toate componentele\n\n\
                     Puteți optimiza acest consum prin dezactivarea programelor de pornire inutile, \
                     proceselor de fundal care rulează în bara de activități (vreme, știri...), \
                     și prin utilizarea modurilor de economisire a energiei."
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
                AppLanguage::Romanian => {
                    "Bateria stochează energie pentru utilizare portabilă și influențează modul în care este gestionat consumul.\n\n\
                     Aspecte cheie:\n\
                     \u{2022} Numărul de cicluri reflectă sănătatea și îmbătrânirea bateriei\n\
                     \u{2022} Capacitatea proiectată scade în timp\n\
                     \u{2022} Utilizarea pe baterie declanșează adesea moduri de economisire a energiei\n\
                     \u{2022} Încărcarea rapidă generează mai multă căldură și consumă mai multă energie"
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
                AppLanguage::Romanian => {
                    "Display-urile sunt un mare consumator de energie, în special la luminozitate ridicată.\n\n\
                     Principalii consumatori:\n\
                     \u{2022} Luminozitatea ecranului (factorul principal)\n\
                     \u{2022} Rate de reîmprospătare mai mari (Hz)\n\
                     \u{2022} Rezoluții mai înalte\n\
                     \u{2022} HDR și gamă largă de culori"
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
                AppLanguage::Romanian => {
                    "Discurile de stocare (SSD / HDD) oferă stocare permanentă pentru fișierele și sistemul dvs.\n\n\
                     Principalii consumatori:\n\
                     \u{2022} Operații de citire/scriere susținute\n\
                     \u{2022} Rotirea platanelor (HDD)\n\
                     \u{2022} Operații de scriere NAND (SSD)\n\
                     \u{2022} Căutări și indexări pe disc"
                }
            },
            _ => match language {
                AppLanguage::English => "No additional information available for this component.",
                AppLanguage::French => "Aucune information supplémentaire disponible pour ce composant.",
                AppLanguage::Romanian => "Nu sunt disponibile informații suplimentare pentru această componentă.",
            },
        };
    }
}

/// Label for measured (app-tracked) CO₂ emissions row in the carbon info modal.
pub fn carbon_info_measured(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Measured by WattSeal",
        AppLanguage::French => "Mesuré par WattSeal",
        AppLanguage::Romanian => "Măsurat de WattSeal",
    }
}

/// Label for estimated base (manufacturing + transport) CO₂ row.
pub fn carbon_info_base(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Estimated base (manufacturing + transport)",
        AppLanguage::French => "Base estimée (fabrication + transport)",
        AppLanguage::Romanian => "Estimare de bază (fabricare + transport)",
    }
}

/// Label for estimated annual network CO₂ row.
pub fn carbon_info_annual(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Estimated annual (network usage)",
        AppLanguage::French => "Annuel estimé (utilisation réseau)",
        AppLanguage::Romanian => "Estimare anuală (utilizare rețea)",
    }
}

/// Label for estimated all-time CO₂ total row.
pub fn carbon_info_all_time(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Estimated all-time total",
        AppLanguage::French => "Total estimé (toutes sources)",
        AppLanguage::Romanian => "Total estimat (toate sursele)",
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
    pub metric: MetricKind,
    language: AppLanguage,
}

impl TranslatedMetricType {
    pub fn new(metric: MetricKind, language: AppLanguage) -> Self {
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
        AppLanguage::Romanian => "Închide WattSeal",
    }
}

pub fn close_dialog_description(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Do you want to close only the window, or also stop the background collector?",
        AppLanguage::French => "Voulez-vous fermer uniquement l'interface, ou aussi arrêter le collecteur ?",
        AppLanguage::Romanian => "Doriți să închideți doar fereastra sau și să opriți colectorul de fundal?",
    }
}

pub fn close_ui_only(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Close window only",
        AppLanguage::French => "Fermer l'interface",
        AppLanguage::Romanian => "Închide doar fereastra",
    }
}

pub fn close_everything(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Close everything",
        AppLanguage::French => "Tout fermer",
        AppLanguage::Romanian => "Închide tot",
    }
}

// Localized Theme & Country Presets

pub fn theme_name(language: AppLanguage, theme: AppTheme) -> &'static str {
    match language {
        AppLanguage::English => theme.name(),
        AppLanguage::French => match theme {
            AppTheme::DeepOcean => "Chasse",
            AppTheme::OceanLight => "Baignade",
            AppTheme::EcoEnergy => "Dodo",
            AppTheme::EcoEnergyLight => "Bataille d'eau",
            AppTheme::GeothermalCore => "Bronzette",
            AppTheme::SolarUmbra => "Détente",
        },
        AppLanguage::Romanian => theme.name(),
    }
}

pub fn country_preset_name<'a>(language: AppLanguage, label: &'a str) -> &'a str {
    match language {
        AppLanguage::English => label,
        AppLanguage::French => match label {
            "France" => "France",
            "Germany" => "Allemagne",
            "Spain" => "Espagne",
            "Italy" => "Italie",
            "Netherlands" => "Pays-Bas",
            "Switzerland" => "Suisse",
            "UK" => "Royaume-Uni",
            "USA (average)" => "États-Unis (moyenne)",
            "China" => "Chine",
            "India" => "Inde",
            "Indonesia" => "Indonésie",
            "Philippines" => "Philippines",
            "Sweden" => "Suède",
            "Poland" => "Pologne",
            "World average" => "Moyenne mondiale",
            "Custom" => "Personnalisé",
            other => other,
        },
        AppLanguage::Romanian => match label {
            "France" => "Franța",
            "Germany" => "Germania",
            "Spain" => "Spania",
            "Italy" => "Italia",
            "Netherlands" => "Olanda",
            "Switzerland" => "Elveția",
            "UK" => "Regatul Unit",
            "USA (average)" => "SUA (medie)",
            "China" => "China",
            "India" => "India",
            "Indonesia" => "Indonezia",
            "Philippines" => "Filipine",
            "Sweden" => "Suedia",
            "Poland" => "Polonia",
            "World average" => "Media mondială",
            "Custom" => "Personalizat",
            other => other,
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslatedTheme {
    pub theme: AppTheme,
    language: AppLanguage,
}

impl TranslatedTheme {
    pub fn new(theme: AppTheme, language: AppLanguage) -> Self {
        Self { theme, language }
    }

    pub fn all(language: AppLanguage) -> Vec<Self> {
        AppTheme::all().iter().map(|&t| Self::new(t, language)).collect()
    }
}

impl std::fmt::Display for TranslatedTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", theme_name(self.language, self.theme))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranslatedCarbonIntensity {
    pub intensity: CarbonIntensity,
    language: AppLanguage,
}

impl TranslatedCarbonIntensity {
    pub fn new(intensity: CarbonIntensity, language: AppLanguage) -> Self {
        Self { intensity, language }
    }

    pub fn all(language: AppLanguage) -> Vec<Self> {
        CarbonIntensity::PRESETS
            .iter()
            .map(|&p| Self::new(p, language))
            .collect()
    }
}

impl std::fmt::Display for TranslatedCarbonIntensity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let country = country_preset_name(self.language, self.intensity.label);
        if self.intensity.is_custom() {
            write!(f, "{}", country)
        } else {
            write!(
                f,
                "{} ({} g/kWh)",
                country,
                format_number(self.intensity.g_per_kwh, 0, self.language)
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranslatedElectricityCost {
    pub cost: ElectricityCost,
    language: AppLanguage,
}

impl TranslatedElectricityCost {
    pub fn new(cost: ElectricityCost, language: AppLanguage) -> Self {
        Self { cost, language }
    }

    pub fn all(language: AppLanguage) -> Vec<Self> {
        ElectricityCost::PRESETS
            .iter()
            .map(|&p| Self::new(p, language))
            .collect()
    }
}

impl std::fmt::Display for TranslatedElectricityCost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let country = country_preset_name(self.language, self.cost.label);
        if self.cost.is_custom() {
            write!(f, "{}", country)
        } else {
            write!(
                f,
                "{} ({} {}/kWh)",
                country,
                format_number(self.cost.price_per_kwh, 2, self.language),
                self.cost.currency_symbol
            )
        }
    }
}

// Unit Formatting (Information & Data Rates)

pub fn format_bytes_gb(bytes: u64, language: AppLanguage) -> String {
    if bytes == 0 {
        return na(language).to_string();
    }
    let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let unit = match language {
        AppLanguage::English => "GB",
        AppLanguage::French => "Go",
        AppLanguage::Romanian => "GB",
    };
    format!("{} {}", format_number(gb, 2, language), unit)
}

pub fn format_mb_per_sec(mb: f64, language: AppLanguage) -> String {
    let unit = match language {
        AppLanguage::English => "MB/s",
        AppLanguage::French => "Mo/s",
        AppLanguage::Romanian => "MB/s",
    };
    format!("{} {}", format_number(mb, 1, language), unit)
}

pub fn metric_unit(language: AppLanguage, metric: MetricKind) -> &'static str {
    match metric {
        MetricKind::Power => "W",
        MetricKind::Usage => "%",
        MetricKind::Speed => match language {
            AppLanguage::English => "MB/s",
            AppLanguage::French => "Mo/s",
            AppLanguage::Romanian => "MB/s",
        },
    }
}

pub fn metric_effective_unit(language: AppLanguage, metric: MetricKind, energy_mode: bool) -> &'static str {
    if metric == MetricKind::Power && energy_mode {
        "Wh"
    } else {
        metric_unit(language, metric)
    }
}

// Localized Number & Metric Unit Formatting

/// Formats a floating point number with specified decimal precision and language-specific separators.
pub fn format_number(val: f64, decimals: usize, language: AppLanguage) -> String {
    if val.is_nan() || val.is_infinite() {
        return if decimals > 0 {
            format!(
                "0{}{}",
                match language {
                    AppLanguage::English => ".",
                    AppLanguage::French => ",",
                    AppLanguage::Romanian => ",",
                },
                "0".repeat(decimals)
            )
        } else {
            "0".to_string()
        };
    }

    let thousands_sep = match language {
        AppLanguage::English => ",",
        AppLanguage::French => " ",
        AppLanguage::Romanian => " ",
    };
    let decimal_sep = match language {
        AppLanguage::English => ".",
        AppLanguage::French => ",",
        AppLanguage::Romanian => ",",
    };

    let is_negative = val < 0.0;
    let formatted = format!("{:.1$}", val.abs(), decimals);
    let parts: Vec<&str> = formatted.split('.').collect();
    let int_digits = parts[0];

    let mut formatted_int = String::with_capacity(int_digits.len() + int_digits.len() / 3);
    let len = int_digits.len();
    for (i, ch) in int_digits.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            formatted_int.push_str(thousands_sep);
        }
        formatted_int.push(ch);
    }

    let sign = if is_negative && (int_digits != "0" || parts.get(1).map_or(false, |d| d.chars().any(|c| c != '0'))) {
        "-"
    } else {
        ""
    };

    if decimals > 0 && parts.len() > 1 {
        format!("{}{}{}{}", sign, formatted_int, decimal_sep, parts[1])
    } else {
        format!("{}{}", sign, formatted_int)
    }
}

/// Formats energy consumption in Wh into a localized value string and appropriate unit (Wh, kWh, MWh, GWh).
pub fn format_energy(energy_wh: f64, language: AppLanguage) -> (String, &'static str) {
    let energy_wh = energy_wh.max(0.0);
    if energy_wh < 1_000.0 {
        (format_number(energy_wh, 1, language), "Wh")
    } else if energy_wh < 1_000_000.0 {
        (format_number(energy_wh / 1_000.0, 1, language), "kWh")
    } else if energy_wh < 1_000_000_000.0 {
        (format_number(energy_wh / 1_000_000.0, 1, language), "MWh")
    } else {
        (format_number(energy_wh / 1_000_000_000.0, 1, language), "GWh")
    }
}

/// Formats CO₂ emissions in grams into a localized value string and appropriate unit (g CO₂, kg CO₂, t CO₂).
pub fn format_emissions(co2_grams: f64, language: AppLanguage) -> (String, &'static str) {
    let co2_grams = co2_grams.max(0.0);
    if co2_grams < 1_000.0 {
        (format_number(co2_grams, 1, language), "g CO₂")
    } else if co2_grams < 1_000_000.0 {
        (format_number(co2_grams / 1_000.0, 1, language), "kg CO₂")
    } else {
        (format_number(co2_grams / 1_000_000.0, 1, language), "t CO₂")
    }
}
