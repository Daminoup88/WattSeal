use common::{CPUData, DatabaseEntry, DiskData, GPUData, MetricKind, NetworkData, ProcessData, RamData, TotalData};

use crate::{
    themes::AppTheme,
    types::{
        AppLanguage, CarbonIntensity, CountryKey, ElectricityCost, ProcessLimit, PsuClass, PsuEfficiency, TimeRange,
    },
};

// Window title

pub fn app_name(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English
        | AppLanguage::German
        | AppLanguage::French
        | AppLanguage::Chinese
        | AppLanguage::Romanian
        | AppLanguage::PortugueseBrazil => "WattSeal",
    }
}

// Page titles

pub fn page_dashboard(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Dashboard",
        AppLanguage::German => "Übersicht",
        AppLanguage::French => "Tableau de bord",
        AppLanguage::Chinese => "仪表盘",
        AppLanguage::Romanian => "Tablou de bord",
        AppLanguage::PortugueseBrazil => "Painel",
    }
}

pub fn page_info(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::German | AppLanguage::Romanian | AppLanguage::PortugueseBrazil => "Info",
        AppLanguage::French => "Infos",
        AppLanguage::Chinese => "信息",
    }
}

// Settings page

pub fn settings_title(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Settings",
        AppLanguage::German => "Einstellungen",
        AppLanguage::French => "Paramètres",
        AppLanguage::Chinese => "设置",
        AppLanguage::Romanian => "Setări",
        AppLanguage::PortugueseBrazil => "Configurações",
    }
}

pub fn settings_general(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::Romanian => "General",
        AppLanguage::German => "Allgemein",
        AppLanguage::French => "Général",
        AppLanguage::Chinese => "常规",
        AppLanguage::PortugueseBrazil => "Geral",
    }
}

pub fn settings_theme(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Theme",
        AppLanguage::German => "Darstellung",
        AppLanguage::French => "Thème",
        AppLanguage::Chinese => "主题",
        AppLanguage::Romanian => "Temă",
        AppLanguage::PortugueseBrazil => "Tema",
    }
}

pub fn settings_language(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Language",
        AppLanguage::German => "Sprache",
        AppLanguage::French => "Langue",
        AppLanguage::Chinese => "语言",
        AppLanguage::Romanian => "Limbă",
        AppLanguage::PortugueseBrazil => "Idioma",
    }
}

pub fn settings_launch_on_startup(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Launch on startup",
        AppLanguage::German => "Beim Systemstart starten",
        AppLanguage::French => "Lancer au démarrage",
        AppLanguage::Chinese => "开机自启动",
        AppLanguage::Romanian => "Pornește la conectare",
        AppLanguage::PortugueseBrazil => "Iniciar com o sistema",
    }
}

pub fn settings_open_data_folder(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Open data folder",
        AppLanguage::German => "Datenordner öffnen",
        AppLanguage::French => "Ouvrir le dossier des données",
        AppLanguage::Chinese => "打开数据文件夹",
        AppLanguage::Romanian => "Deschide folderul de date",
        AppLanguage::PortugueseBrazil => "Abrir pasta de dados",
    }
}

pub fn settings_open_logs_folder(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Open logs folder",
        AppLanguage::German => "Protokollordner öffnen",
        AppLanguage::French => "Ouvrir le dossier des logs",
        AppLanguage::Chinese => "打开日志文件夹",
        AppLanguage::Romanian => "Deschide folderul de jurnale",
        AppLanguage::PortugueseBrazil => "Abrir pasta de logs",
    }
}

pub fn settings_open_data_and_logs_folder(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Open data and logs folder",
        AppLanguage::German => "Daten- und Protokollordner öffnen",
        AppLanguage::French => "Ouvrir le dossier des données et logs",
        AppLanguage::Chinese => "打开数据与日志文件夹",
        AppLanguage::Romanian => "Deschide folderul de date și jurnale",
        AppLanguage::PortugueseBrazil => "Abrir pasta de dados e logs",
    }
}

pub fn modal_close(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Close",
        AppLanguage::German => "Schließen",
        AppLanguage::French => "Fermer",
        AppLanguage::Chinese => "关闭",
        AppLanguage::Romanian => "Închide",
        AppLanguage::PortugueseBrazil => "Fechar",
    }
}

pub fn star_on_github(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Star on GitHub",
        AppLanguage::German => "GitHub-Stern vergeben",
        AppLanguage::French => "Étoiler sur GitHub",
        AppLanguage::Chinese => "在 GitHub 上加星",
        AppLanguage::Romanian => "Acordă o stea pe GitHub",
        AppLanguage::PortugueseBrazil => "Favoritar no GitHub",
    }
}

pub fn report_issue(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Report Issue",
        AppLanguage::German => "Problem melden",
        AppLanguage::French => "Signaler un bug",
        AppLanguage::Chinese => "报告问题",
        AppLanguage::Romanian => "Raportează o problemă",
        AppLanguage::PortugueseBrazil => "Informar um problema",
    }
}

// Dashboard

pub fn current_power_consumption(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Current power consumption",
        AppLanguage::German => "Aktuelle Leistungsaufnahme",
        AppLanguage::French => "Consommation actuelle",
        AppLanguage::Chinese => "当前功耗",
        AppLanguage::Romanian => "Consum curent",
        AppLanguage::PortugueseBrazil => "Consumo atual",
    }
}

pub fn database_migrating_title(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Preparing database",
        AppLanguage::German => "Datenbank wird vorbereitet",
        AppLanguage::French => "Préparation de la base de données",
        AppLanguage::Chinese => "准备数据库",
        AppLanguage::Romanian => "Bază de date în pregătire",
        AppLanguage::PortugueseBrazil => "Preparando o banco de dados",
    }
}

pub fn database_migrating_description(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => {
            "WattSeal is waiting for the collector to finish migrating the database. Retrying every second…"
        }
        AppLanguage::German => {
            "WattSeal wartet, bis der Datensammler die Datenbankmigration abgeschlossen hat. Neuer Versuch jede Sekunde…"
        }
        AppLanguage::French => {
            "WattSeal attend que le collecteur termine la migration de la base de données. Nouvel essai chaque seconde…"
        }
        AppLanguage::Chinese => "WattSeal 正在等待采集器完成数据库迁移，每秒重试一次…",
        AppLanguage::Romanian => {
            "WattSeal așteaptă colectorul să termine de migrat baza de date. Reîncercare în fiecare secundă..."
        }
        AppLanguage::PortugueseBrazil => {
            "O WattSeal está aguardando o coletor terminar a migração do banco de dados. Tentando novamente a cada segundo…"
        }
    }
}

pub fn all_time(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "All Time",
        AppLanguage::German => "Gesamt",
        AppLanguage::French => "Depuis le début",
        AppLanguage::Chinese => "累计",
        AppLanguage::Romanian => "De la început",
        AppLanguage::PortugueseBrazil => "Todo o período",
    }
}

pub fn total(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English
        | AppLanguage::French
        | AppLanguage::Chinese
        | AppLanguage::Romanian
        | AppLanguage::PortugueseBrazil => "Total",
        AppLanguage::German => "Gesamt",
    }
}

pub fn emissions(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Emissions",
        AppLanguage::German => "Emissionen",
        AppLanguage::French => "Émissions",
        AppLanguage::Chinese => "碳排放",
        AppLanguage::Romanian => "Emisii",
        AppLanguage::PortugueseBrazil => "Emissões",
    }
}

pub fn zero_carbon_intensity_warning(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "\u{26a0} Choose a real carbon intensity in the settings! \u{26a0}",
        AppLanguage::German => "\u{26a0} Wähle in den Einstellungen eine reale CO₂-Intensität aus! \u{26a0}",
        AppLanguage::French => "\u{26a0} Choisissez une intensité carbone réelle dans les paramètres ! \u{26a0}",
        AppLanguage::Chinese => "\u{26a0} 请在设置中选择真实的电网碳强度！ \u{26a0}",
        AppLanguage::Romanian => "\u{26a0} Alegeți o intensitate de carbon reală în setări! \u{26a0}",
        AppLanguage::PortugueseBrazil => "\u{26a0} Escolha uma intensidade de carbono real nas configurações! \u{26a0}",
    }
}

// Info page

pub fn cpu(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English
        | AppLanguage::German
        | AppLanguage::French
        | AppLanguage::Chinese
        | AppLanguage::Romanian
        | AppLanguage::PortugueseBrazil => "CPU",
    }
}

pub fn processor_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Processor Information",
        AppLanguage::German => "Prozessorinformationen",
        AppLanguage::French => "Informations processeur",
        AppLanguage::Chinese => "处理器信息",
        AppLanguage::Romanian => "Informații procesor",
        AppLanguage::PortugueseBrazil => "Informações do processador",
    }
}

pub fn model(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::Romanian => "Model",
        AppLanguage::German => "Modell",
        AppLanguage::French => "Modèle",
        AppLanguage::Chinese => "型号",
        AppLanguage::PortugueseBrazil => "Modelo",
    }
}

pub fn cores(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Cores",
        AppLanguage::German => "Kerne",
        AppLanguage::French => "Cœurs",
        AppLanguage::Chinese => "核心",
        AppLanguage::Romanian => "Nuclee",
        AppLanguage::PortugueseBrazil => "Núcleos",
    }
}

pub fn cores_and_threads(language: AppLanguage, physical: u16, logical: u16) -> String {
    match language {
        AppLanguage::English => format!("{} cores / {} threads", physical, logical),
        AppLanguage::German => format!("{} Kerne / {} Threads", physical, logical),
        AppLanguage::French => format!("{} cœurs / {} threads", physical, logical),
        AppLanguage::Chinese => format!("{} 核 / {} 线程", physical, logical),
        AppLanguage::Romanian => format!("{} nuclee / {} thread-uri", physical, logical),
        AppLanguage::PortugueseBrazil => format!("{} núcleos / {} threads", physical, logical),
    }
}

pub fn gpu(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English
        | AppLanguage::German
        | AppLanguage::French
        | AppLanguage::Chinese
        | AppLanguage::Romanian
        | AppLanguage::PortugueseBrazil => "GPU",
    }
}

pub fn graphics_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Graphics Information",
        AppLanguage::German => "Grafikinformationen",
        AppLanguage::French => "Informations graphiques",
        AppLanguage::Chinese => "显卡信息",
        AppLanguage::Romanian => "Informații grafice",
        AppLanguage::PortugueseBrazil => "Informações gráficas",
    }
}

pub fn graphics_processor_n(language: AppLanguage, n: usize) -> String {
    match language {
        AppLanguage::English => format!("Graphics Processor {}", n),
        AppLanguage::German => format!("Grafikprozessor {}", n),
        AppLanguage::French => format!("Processeur graphique {}", n),
        AppLanguage::Chinese => format!("图形处理器 {}", n),
        AppLanguage::Romanian => format!("Procesor grafic {}", n),
        AppLanguage::PortugueseBrazil => format!("Processador gráfico {}", n),
    }
}

pub fn memory(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Memory",
        AppLanguage::German => "Arbeitsspeicher",
        AppLanguage::French => "Mémoire",
        AppLanguage::Chinese => "内存",
        AppLanguage::Romanian => "Memorie",
        AppLanguage::PortugueseBrazil => "Memória",
    }
}

pub fn ram_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "RAM Information",
        AppLanguage::German => "Arbeitsspeicherinformationen",
        AppLanguage::French => "Informations RAM",
        AppLanguage::Chinese => "内存信息",
        AppLanguage::Romanian => "Informații RAM",
        AppLanguage::PortugueseBrazil => "Informações da RAM",
    }
}

pub fn total_memory(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Total Memory",
        AppLanguage::German => "Gesamter Arbeitsspeicher",
        AppLanguage::French => "Mémoire totale",
        AppLanguage::Chinese => "总内存",
        AppLanguage::Romanian => "Memorie Totală",
        AppLanguage::PortugueseBrazil => "Memória total",
    }
}

pub fn swap(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::French | AppLanguage::Chinese | AppLanguage::Romanian => "Swap",
        AppLanguage::German => "Auslagerungsspeicher",
        AppLanguage::PortugueseBrazil => "Memória virtual",
    }
}

pub fn system(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "System",
        AppLanguage::German => "System",
        AppLanguage::French => "Système",
        AppLanguage::Chinese => "系统",
        AppLanguage::Romanian => "Sistem",
        AppLanguage::PortugueseBrazil => "Sistema",
    }
}

pub fn os_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "OS Information",
        AppLanguage::German => "Betriebssysteminformationen",
        AppLanguage::French => "Informations OS",
        AppLanguage::Chinese => "操作系统信息",
        AppLanguage::Romanian => "Informații Sistem de Operare",
        AppLanguage::PortugueseBrazil => "Informações do SO",
    }
}

pub fn operating_system(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Operating System",
        AppLanguage::German => "Betriebssystem",
        AppLanguage::French => "Système d'exploitation",
        AppLanguage::Chinese => "操作系统",
        AppLanguage::Romanian => "Sistem de operare",
        AppLanguage::PortugueseBrazil => "Sistema operacional",
    }
}

pub fn hostname(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Hostname",
        AppLanguage::German => "Hostname",
        AppLanguage::French => "Nom d'hôte",
        AppLanguage::Chinese => "主机名",
        AppLanguage::Romanian => "Nume de gazdă",
        AppLanguage::PortugueseBrazil => "Nome do computador",
    }
}

pub fn storage(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Storage",
        AppLanguage::German => "Speicher",
        AppLanguage::French => "Stockage",
        AppLanguage::Chinese => "存储",
        AppLanguage::Romanian => "Stocare",
        AppLanguage::PortugueseBrazil => "Armazenamento",
    }
}

pub fn disk_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Disk Information",
        AppLanguage::German => "Datenträgerinformationen",
        AppLanguage::French => "Informations disque",
        AppLanguage::Chinese => "磁盘信息",
        AppLanguage::Romanian => "Informații disc",
        AppLanguage::PortugueseBrazil => "Informações do disco",
    }
}

pub fn disk(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Disk",
        AppLanguage::German => "Laufwerk",
        AppLanguage::French => "Disque",
        AppLanguage::Chinese => "磁盘",
        AppLanguage::Romanian => "Disc",
        AppLanguage::PortugueseBrazil => "Disco",
    }
}

pub fn disk_n(language: AppLanguage, n: usize) -> String {
    match language {
        AppLanguage::English
        | AppLanguage::German
        | AppLanguage::French
        | AppLanguage::Chinese
        | AppLanguage::Romanian
        | AppLanguage::PortugueseBrazil => {
            format!("{} {}", disk(language), n)
        }
    }
}

pub fn space(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Space",
        AppLanguage::German => "Speicherplatz",
        AppLanguage::French => "Espace",
        AppLanguage::Chinese => "空间",
        AppLanguage::Romanian => "Spațiu",
        AppLanguage::PortugueseBrazil => "Espaço",
    }
}

pub fn network(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Network",
        AppLanguage::German => "Netzwerk",
        AppLanguage::French => "Réseau",
        AppLanguage::Chinese => "网络",
        AppLanguage::Romanian => "Rețea",
        AppLanguage::PortugueseBrazil => "Rede",
    }
}

pub fn battery(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Battery",
        AppLanguage::German => "Akku",
        AppLanguage::French => "Batterie",
        AppLanguage::Chinese => "电池",
        AppLanguage::Romanian => "Baterie",
        AppLanguage::PortugueseBrazil => "Bateria",
    }
}

pub fn battery_status(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Battery Status",
        AppLanguage::German => "Akkustatus",
        AppLanguage::French => "État de la batterie",
        AppLanguage::Chinese => "电池状态",
        AppLanguage::Romanian => "Stare Baterie",
        AppLanguage::PortugueseBrazil => "Status da bateria",
    }
}

pub fn process(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Processes",
        AppLanguage::German => "Prozesse",
        AppLanguage::French => "Processus",
        AppLanguage::Chinese => "进程",
        AppLanguage::Romanian => "Procese",
        AppLanguage::PortugueseBrazil => "Processos",
    }
}

pub fn name(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Name",
        AppLanguage::German => "Name",
        AppLanguage::French => "Nom",
        AppLanguage::Chinese => "名称",
        AppLanguage::Romanian => "Nume",
        AppLanguage::PortugueseBrazil => "Nome",
    }
}

pub fn capacity(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Capacity",
        AppLanguage::German => "Kapazität",
        AppLanguage::French => "Capacité",
        AppLanguage::Chinese => "容量",
        AppLanguage::Romanian => "Capacitate",
        AppLanguage::PortugueseBrazil => "Capacidade",
    }
}

pub fn capacity_wh_cycles(language: AppLanguage, cap_wh: f32, cycles: u32) -> String {
    let val = format_number(cap_wh as f64, 1, language);
    match language {
        AppLanguage::English | AppLanguage::French | AppLanguage::Chinese => format!("{} Wh ({} cycles)", val, cycles),
        AppLanguage::German => format!("{} Wh ({} Ladezyklen)", val, cycles),
        AppLanguage::Romanian => format!("{} Wh ({} cicluri)", val, cycles),
        AppLanguage::PortugueseBrazil => format!("{} Wh ({} ciclos)", val, cycles),
    }
}

pub fn capacity_wh_only(language: AppLanguage, cap_wh: f32) -> String {
    let val = format_number(cap_wh as f64, 1, language);
    match language {
        AppLanguage::English
        | AppLanguage::German
        | AppLanguage::French
        | AppLanguage::Chinese
        | AppLanguage::Romanian
        | AppLanguage::PortugueseBrazil => {
            format!("{} Wh", val)
        }
    }
}

pub fn na_with_cycles(language: AppLanguage, cycles: u32) -> String {
    match language {
        AppLanguage::English | AppLanguage::French | AppLanguage::Chinese => format!("N/A ({} cycles)", cycles),
        AppLanguage::German => format!("N/A ({} Ladezyklen)", cycles),
        AppLanguage::Romanian => format!("N/A ({} cicluri)", cycles),
        AppLanguage::PortugueseBrazil => format!("N/D ({} ciclos)", cycles),
    }
}

pub fn display(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Display",
        AppLanguage::German => "Bildschirm",
        AppLanguage::French => "Écran",
        AppLanguage::Chinese => "显示器",
        AppLanguage::Romanian => "Ecran",
        AppLanguage::PortugueseBrazil => "Tela",
    }
}

pub fn screen_information(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Screen Information",
        AppLanguage::German => "Bildschirminformationen",
        AppLanguage::French => "Informations écran",
        AppLanguage::Chinese => "屏幕信息",
        AppLanguage::Romanian => "Informații ecran",
        AppLanguage::PortugueseBrazil => "Informações da tela",
    }
}

pub fn mode(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Mode",
        AppLanguage::German => "Modus",
        AppLanguage::French => "Mode",
        AppLanguage::Chinese => "模式",
        AppLanguage::Romanian => "Mod",
        AppLanguage::PortugueseBrazil => "Modo",
    }
}

pub fn primary_display(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Primary Display",
        AppLanguage::German => "Primärer Bildschirm",
        AppLanguage::French => "Écran principal",
        AppLanguage::Chinese => "主显示器",
        AppLanguage::Romanian => "Ecran principal",
        AppLanguage::PortugueseBrazil => "Tela principal",
    }
}

pub fn secondary_display(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Secondary Display",
        AppLanguage::German => "Sekundärer Bildschirm",
        AppLanguage::French => "Écran secondaire",
        AppLanguage::Chinese => "副显示器",
        AppLanguage::Romanian => "Ecran secundar",
        AppLanguage::PortugueseBrazil => "Tela secundária",
    }
}

// General

pub fn na(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English
        | AppLanguage::German
        | AppLanguage::French
        | AppLanguage::Chinese
        | AppLanguage::Romanian => "N/A",
        AppLanguage::PortugueseBrazil => "N/D",
    }
}

pub fn no_data_available(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "No data available",
        AppLanguage::German => "Keine Daten verfügbar",
        AppLanguage::French => "Aucune donnée disponible",
        AppLanguage::Chinese => "暂无数据",
        AppLanguage::Romanian => "Nu există date disponibile",
        AppLanguage::PortugueseBrazil => "Sem dados disponíveis",
    }
}

// Charts

pub fn power_label(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Power:",
        AppLanguage::German => "Leistung:",
        AppLanguage::French => "Puissance :",
        AppLanguage::Chinese => "功率：",
        AppLanguage::Romanian => "Putere:",
        AppLanguage::PortugueseBrazil => "Potência:",
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
        AppLanguage::German => format!("Wert: {}", value_text),
        AppLanguage::French => format!("Valeur : {}", value_text),
        AppLanguage::Chinese => format!("数值：{}", value_text),
        AppLanguage::Romanian => format!("Valoare: {}", value_text),
        AppLanguage::PortugueseBrazil => format!("Valor: {}", value_text),
    }
}

pub fn tooltip_time(language: AppLanguage, time_text: &str) -> String {
    match language {
        AppLanguage::English => format!("Time: {}", time_text),
        AppLanguage::German => format!("Zeit: {}", time_text),
        AppLanguage::French => format!("Heure : {}", time_text),
        AppLanguage::Chinese => format!("时间：{}", time_text),
        AppLanguage::Romanian => format!("Timp: {}", time_text),
        AppLanguage::PortugueseBrazil => format!("Hora: {}", time_text),
    }
}

// Process list

pub fn application(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::French => "Application",
        AppLanguage::German => "Anwendung",
        AppLanguage::Chinese => "应用",
        AppLanguage::Romanian => "Aplicație",
        AppLanguage::PortugueseBrazil => "Aplicativo",
    }
}

pub fn power(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Power",
        AppLanguage::German => "Leistung",
        AppLanguage::French => "Puissance",
        AppLanguage::Chinese => "功率",
        AppLanguage::Romanian => "Putere",
        AppLanguage::PortugueseBrazil => "Potência",
    }
}

pub fn energy(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Energy",
        AppLanguage::German => "Energie",
        AppLanguage::French => "Énergie",
        AppLanguage::Chinese => "能耗",
        AppLanguage::Romanian => "Energie",
        AppLanguage::PortugueseBrazil => "Energia",
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
        AppLanguage::English
        | AppLanguage::German
        | AppLanguage::French
        | AppLanguage::Chinese
        | AppLanguage::Romanian
        | AppLanguage::PortugueseBrazil => "RAM",
    }
}

pub fn disk_read(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Disk read",
        AppLanguage::German => "Datenträger lesen",
        AppLanguage::French => "Lecture disque",
        AppLanguage::Chinese => "磁盘读取",
        AppLanguage::Romanian => "Citire disc",
        AppLanguage::PortugueseBrazil => "Leitura de disco",
    }
}

pub fn disk_write(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Disk write",
        AppLanguage::German => "Datenträger schreiben",
        AppLanguage::French => "Écriture disque",
        AppLanguage::Chinese => "磁盘写入",
        AppLanguage::Romanian => "Scriere disc",
        AppLanguage::PortugueseBrazil => "Gravação em disco",
    }
}

// Time ranges

pub fn last_minute(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Minute",
        AppLanguage::German => "Letzte Minute",
        AppLanguage::French => "Dernière minute",
        AppLanguage::Chinese => "最近 1 分钟",
        AppLanguage::Romanian => "Ultimul minut",
        AppLanguage::PortugueseBrazil => "Último minuto",
    }
}

pub fn last_hour(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Hour",
        AppLanguage::German => "Letzte Stunde",
        AppLanguage::French => "Dernière heure",
        AppLanguage::Chinese => "最近 1 小时",
        AppLanguage::Romanian => "Ultima oră",
        AppLanguage::PortugueseBrazil => "Última hora",
    }
}

pub fn last_24_hours(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last 24 Hours",
        AppLanguage::German => "Letzte 24 h",
        AppLanguage::French => "Dernières 24 heures",
        AppLanguage::Chinese => "最近 24 小时",
        AppLanguage::Romanian => "Ultimele 24 ore",
        AppLanguage::PortugueseBrazil => "Últimas 24 horas",
    }
}

pub fn last_week(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Week",
        AppLanguage::German => "Letzte Woche",
        AppLanguage::French => "Dernière semaine",
        AppLanguage::Chinese => "最近 1 周",
        AppLanguage::Romanian => "Ultima săptămână",
        AppLanguage::PortugueseBrazil => "Última semana",
    }
}

pub fn last_month(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Month",
        AppLanguage::German => "Letzter Monat",
        AppLanguage::French => "Dernier mois",
        AppLanguage::Chinese => "最近 1 个月",
        AppLanguage::Romanian => "Ultima lună",
        AppLanguage::PortugueseBrazil => "Último mês",
    }
}

pub fn last_year(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Last Year",
        AppLanguage::German => "Letztes Jahr",
        AppLanguage::French => "Dernière année",
        AppLanguage::Chinese => "最近 1 年",
        AppLanguage::Romanian => "Ultimul an",
        AppLanguage::PortugueseBrazil => "Último ano",
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
        AppLanguage::German => "Leistung",
        AppLanguage::French => "Puissance",
        AppLanguage::Chinese => "功率",
        AppLanguage::Romanian => "Putere",
        AppLanguage::PortugueseBrazil => "Potência",
    }
}

pub fn metric_energy(language: AppLanguage) -> &'static str {
    energy(language)
}

pub fn metric_usage(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Usage",
        AppLanguage::German => "Auslastung",
        AppLanguage::French => "Utilisation",
        AppLanguage::Chinese => "使用率",
        AppLanguage::Romanian => "Utilizare",
        AppLanguage::PortugueseBrazil => "Uso",
    }
}

pub fn metric_speed(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Speed",
        AppLanguage::German => "Rate",
        AppLanguage::French => "Vitesse",
        AppLanguage::Chinese => "速度",
        AppLanguage::Romanian => "Viteză",
        AppLanguage::PortugueseBrazil => "Velocidade",
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
        AppLanguage::English
        | AppLanguage::German
        | AppLanguage::French
        | AppLanguage::Chinese
        | AppLanguage::Romanian
        | AppLanguage::PortugueseBrazil => metric_usage(language),
    }
}

pub fn label_read(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Read",
        AppLanguage::German => "Lesen",
        AppLanguage::French => "Lecture",
        AppLanguage::Chinese => "读取",
        AppLanguage::Romanian => "Citire",
        AppLanguage::PortugueseBrazil => "Leitura",
    }
}

pub fn label_write(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Write",
        AppLanguage::German => "Schreiben",
        AppLanguage::French => "Écriture",
        AppLanguage::Chinese => "写入",
        AppLanguage::Romanian => "Scriere",
        AppLanguage::PortugueseBrazil => "Gravação",
    }
}

pub fn label_download(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::PortugueseBrazil => "Download",
        AppLanguage::German => "Download",
        AppLanguage::French => "Téléchargement",
        AppLanguage::Chinese => "下载",
        AppLanguage::Romanian => "Descărcare",
    }
}

pub fn label_upload(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::PortugueseBrazil => "Upload",
        AppLanguage::German => "Upload",
        AppLanguage::French => "Envoi",
        AppLanguage::Chinese => "上传",
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
            AppLanguage::German => "Unbekannt",
            AppLanguage::French => "Inconnu",
            AppLanguage::Chinese => "未知",
            AppLanguage::Romanian => "Necunoscut",
            AppLanguage::PortugueseBrazil => "Desconhecido",
        },
    }
}

pub fn sensor_name<'a>(language: AppLanguage, english_name: &'a str) -> &'a str {
    match english_name {
        "CPU" | "GPU" | "RAM" => english_name,
        "Disk" => match language {
            AppLanguage::German => "Laufwerk",
            AppLanguage::French => "Disque",
            AppLanguage::Chinese => "磁盘",
            AppLanguage::Romanian => "Disc",
            AppLanguage::PortugueseBrazil => "Disco",
            _ => english_name,
        },
        "Network" => match language {
            AppLanguage::German => "Netzwerk",
            AppLanguage::French => "Réseau",
            AppLanguage::Chinese => "网络",
            AppLanguage::Romanian => "Rețea",
            AppLanguage::PortugueseBrazil => "Rede",
            _ => english_name,
        },
        "Processes" => match language {
            AppLanguage::German => "Prozesse",
            AppLanguage::French => "Processus",
            AppLanguage::Chinese => "进程",
            AppLanguage::Romanian => "Procese",
            AppLanguage::PortugueseBrazil => "Processos",
            _ => english_name,
        },
        "Total" => match language {
            AppLanguage::German => "Gesamt",
            _ => english_name,
        },
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
        AppLanguage::German => "Strompreis",
        AppLanguage::French => "Coût de l'électricité",
        AppLanguage::Chinese => "电价",
        AppLanguage::Romanian => "Cost electricitate",
        AppLanguage::PortugueseBrazil => "Tarifa de energia",
    }
}

pub fn kwh_cost_placeholder(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "e.g. 0.20",
        AppLanguage::German => "z. B. 0.20",
        AppLanguage::French => "ex. 0.20",
        AppLanguage::Chinese => "例如 0.20",
        AppLanguage::Romanian => "ex: 0.20",
        AppLanguage::PortugueseBrazil => "ex.: 0.20",
    }
}

pub fn kwh_cost_invalid(language: AppLanguage, currency_symbol: &str) -> String {
    let text = match language {
        AppLanguage::English => "Enter a positive number",
        AppLanguage::German => "Gib eine positive Zahl ein",
        AppLanguage::French => "Entrez un nombre positif",
        AppLanguage::Romanian => "Introduceți un număr pozitiv",
        AppLanguage::Chinese => "请输入正数",
        AppLanguage::PortugueseBrazil => "Digite um número positivo",
    };
    return format!("{} ({}/kWh)", text, currency_symbol);
}

pub fn setup_choose_electricity(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Select your electricity price",
        AppLanguage::German => "Wähle deinen Strompreis aus",
        AppLanguage::French => "Sélectionnez votre tarif d'électricité",
        AppLanguage::Chinese => "选择你的电价",
        AppLanguage::Romanian => "Selectați tariful dvs. de electricitate",
        AppLanguage::PortugueseBrazil => "Selecione a tarifa da sua energia elétrica",
    }
}

pub fn custom_kwh_cost_placeholder(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "e.g. 0.25",
        AppLanguage::German => "z. B. 0.25",
        AppLanguage::French => "ex. 0.25",
        AppLanguage::Chinese => "例如 0.25",
        AppLanguage::Romanian => "ex: 0.25",
        AppLanguage::PortugueseBrazil => "ex.: 0.25",
    }
}

pub fn electricity_bill(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Estimated Bill",
        AppLanguage::German => "Geschätzte Stromkosten",
        AppLanguage::French => "Facture estimée",
        AppLanguage::Chinese => "预估电费",
        AppLanguage::Romanian => "Factură estimată",
        AppLanguage::PortugueseBrazil => "Fatura estimada",
    }
}

pub fn settings_carbon_intensity(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Carbon Intensity",
        AppLanguage::German => "CO₂-Intensität",
        AppLanguage::French => "Intensité carbone",
        AppLanguage::Chinese => "碳强度",
        AppLanguage::Romanian => "Intensitate Carbon",
        AppLanguage::PortugueseBrazil => "Intensidade de carbono",
    }
}

pub fn setup_welcome_title(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Welcome to WattSeal",
        AppLanguage::German => "Willkommen bei WattSeal",
        AppLanguage::French => "Bienvenue sur WattSeal",
        AppLanguage::Chinese => "欢迎使用 WattSeal",
        AppLanguage::Romanian => "Bine ați venit la WattSeal",
        AppLanguage::PortugueseBrazil => "Bem-vindo ao WattSeal",
    }
}

pub fn setup_choose_language(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Choose your language",
        AppLanguage::German => "Wähle deine Sprache",
        AppLanguage::French => "Choisissez votre langue",
        AppLanguage::Chinese => "选择语言",
        AppLanguage::Romanian => "Alegeți limba",
        AppLanguage::PortugueseBrazil => "Escolha seu idioma",
    }
}

pub fn setup_choose_carbon(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Select the carbon intensity of your electricity grid",
        AppLanguage::German => "Wähle die CO₂-Intensität deines Stromnetzes aus",
        AppLanguage::French => "Sélectionnez l'intensité carbone de votre réseau électrique",
        AppLanguage::Chinese => "选择你所在电网的碳强度",
        AppLanguage::Romanian => "Selectați intensitatea de carbon a rețelei electrice",
        AppLanguage::PortugueseBrazil => "Selecione a intensidade de carbono da sua rede elétrica",
    }
}

pub fn setup_confirm(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Confirm",
        AppLanguage::German => "Bestätigen",
        AppLanguage::French => "Confirmer",
        AppLanguage::Chinese => "确认",
        AppLanguage::Romanian => "Confirmați",
        AppLanguage::PortugueseBrazil => "Confirmar",
    }
}

pub fn custom_carbon_placeholder(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "e.g. 300",
        AppLanguage::German => "z. B. 300",
        AppLanguage::French => "ex. 300",
        AppLanguage::Chinese => "例如 300",
        AppLanguage::Romanian => "ex: 300",
        AppLanguage::PortugueseBrazil => "ex.: 300",
    }
}

pub fn custom_carbon_invalid(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Enter a positive number (g CO₂/kWh)",
        AppLanguage::German => "Gib eine positive Zahl ein (g CO₂/kWh)",
        AppLanguage::French => "Entrez un nombre positif (g CO₂/kWh)",
        AppLanguage::Chinese => "请输入正数 (g CO₂/kWh)",
        AppLanguage::Romanian => "Introduceți un număr pozitiv (g CO₂/kWh)",
        AppLanguage::PortugueseBrazil => "Digite um número positivo (g CO₂/kWh)",
    }
}

pub fn settings_psu_efficiency(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "PSU Efficiency",
        AppLanguage::German => "Netzteil-Effizienz",
        AppLanguage::French => "Efficacité de l'alimentation",
        AppLanguage::Chinese => "电源效率",
        AppLanguage::Romanian => "Eficiență sursă alimentare",
        AppLanguage::PortugueseBrazil => "Eficiência da fonte",
    }
}

pub fn setup_choose_psu_efficiency(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Select your PSU efficiency class (optional)",
        AppLanguage::German => "Wähle die Effizienzklasse deines Netzteils (optional)",
        AppLanguage::French => "Sélectionnez la classe d'efficacité de votre alimentation (optionnel)",
        AppLanguage::Chinese => "选择你的电源效率等级（可选）",
        AppLanguage::Romanian => "Selectați clasa de eficiență a sursei de alimentare (opțional)",
        AppLanguage::PortugueseBrazil => "Selecione a classe de eficiência da sua fonte (opcional)",
    }
}

pub fn custom_psu_efficiency_placeholder(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "e.g. η = 0.87",
        AppLanguage::German => "z. B. η = 0.87",
        AppLanguage::French => "ex. η = 0.87",
        AppLanguage::Chinese => "例如 η = 0.87",
        AppLanguage::Romanian => "ex: η = 0.87",
        AppLanguage::PortugueseBrazil => "ex.: η = 0.87",
    }
}

pub fn custom_psu_efficiency_invalid(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Enter η between 0.01 and 1.0",
        AppLanguage::German => "Gib η zwischen 0.01 und 1.0 ein",
        AppLanguage::French => "Entrez η entre 0.01 et 1.0",
        AppLanguage::Chinese => "请输入 0.01 到 1.0 之间的 η 值",
        AppLanguage::Romanian => "Introduceți η între 0.01 și 1.0",
        AppLanguage::PortugueseBrazil => "Digite η entre 0.01 e 1.0",
    }
}

// Info modal

pub fn info_modal_current_power(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Current power",
        AppLanguage::German => "Aktuelle Leistung",
        AppLanguage::French => "Puissance actuelle",
        AppLanguage::Chinese => "当前功率",
        AppLanguage::Romanian => "Putere actuală",
        AppLanguage::PortugueseBrazil => "Potência atual",
    }
}

pub fn info_modal_all_time_power(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "All-time energy",
        AppLanguage::German => "Gesamtenergie",
        AppLanguage::French => "Énergie totale",
        AppLanguage::Chinese => "累计能耗",
        AppLanguage::Romanian => "Energie totală",
        AppLanguage::PortugueseBrazil => "Energia acumulada",
    }
}

pub fn info_modal_current_top_consumer(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Current highest consumer",
        AppLanguage::German => "Aktuell größter Verbraucher",
        AppLanguage::French => "Plus gros consommateur actuel",
        AppLanguage::Chinese => "当前最大耗电",
        AppLanguage::Romanian => "Cel mai mare consumator actual",
        AppLanguage::PortugueseBrazil => "Maior consumidor atual",
    }
}

pub fn info_modal_all_time_top_consumer(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "All-time top consumer",
        AppLanguage::German => "Größter Gesamtverbraucher",
        AppLanguage::French => "Plus gros consommateur total",
        AppLanguage::Chinese => "累计最大耗电",
        AppLanguage::Romanian => "Cel mai mare consumator total",
        AppLanguage::PortugueseBrazil => "Maior consumidor acumulado",
    }
}

pub fn info_modal_top_process(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Top process",
        AppLanguage::German => "Verbrauchsstärkster Prozess",
        AppLanguage::French => "Processus le plus gourmand",
        AppLanguage::Chinese => "耗电最高进程",
        AppLanguage::Romanian => "Cel mai intensiv proces",
        AppLanguage::PortugueseBrazil => "Processo que mais consome",
    }
}

pub fn info_modal_coming_soon(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Coming soon",
        AppLanguage::German => "Demnächst verfügbar",
        AppLanguage::French => "Bientôt disponible",
        AppLanguage::Chinese => "即将推出",
        AppLanguage::Romanian => "În curând",
        AppLanguage::PortugueseBrazil => "Em breve",
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
                AppLanguage::German => "CO₂-Emissionen".to_string(),
                AppLanguage::French => "Émissions carbone".to_string(),
                AppLanguage::Chinese => "碳排放".to_string(),
                AppLanguage::Romanian => "Emisii de carbon".to_string(),
                AppLanguage::PortugueseBrazil => "Emissões de carbono".to_string(),
            },
            _ => match language {
                AppLanguage::English | AppLanguage::German | AppLanguage::French | AppLanguage::Romanian => {
                    "Info".to_string()
                }
                AppLanguage::Chinese => "信息".to_string(),
                AppLanguage::PortugueseBrazil => "Informações".to_string(),
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
            AppLanguage::German => {
                "Die CPU (Central Processing Unit) ist das Gehirn deines Computers. \
                 Sie führt alle Anweisungen und Berechnungen aus.\n\n\
                 Hauptverbraucher:\n\
                 \u{2022} Höhere Taktfrequenzen erhöhen die Leistungsaufnahme\n\
                 \u{2022} Mehr aktive Kerne verbrauchen mehr Energie\n\
                 \u{2022} Rechenintensive Aufgaben wie Kompilierung und Kodierung erhöhen die Auslastung\n\
                 \u{2022} Höhere Spannungen durch Übertaktung steigern den Verbrauch"
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
            AppLanguage::Chinese => {
                "CPU (中央处理器) 是计算机的核心\u{ff0c}负责执行所有指令和计算。\n\n\
                 主要耗电因素：更高主频、更多活跃核心、编译转码等重负载、超频。"
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
            AppLanguage::PortugueseBrazil => {
                "A CPU (Unidade Central de Processamento) funciona como o cérebro do computador e executa as instruções e os cálculos.\n\
                 \n\
                 O que faz o consumo subir:\n\
                 \u{2022} Frequência de clock mais alta\n\
                 \u{2022} Mais núcleos ativos\n\
                 \u{2022} Tarefas pesadas, como compilação e codificação\n\
                 \u{2022} Tensão mais alta, como no overclock"
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
            AppLanguage::German => {
                "Die GPU (Graphics Processing Unit) übernimmt die Grafikdarstellung und \
                 parallele Berechnungen.\n\n\
                 Hauptverbraucher:\n\
                 \u{2022} 3D-Darstellung und Spiele\n\
                 \u{2022} Video-Kodierung und -Dekodierung\n\
                 \u{2022} KI- und Machine-Learning-Aufgaben\n\
                 \u{2022} Hohe VRAM-Auslastung und Speicherbandbreite"
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
            AppLanguage::Chinese => {
                "GPU 负责图形渲染与并行计算。\n\n\
                 主要耗电\u{ff1a}3D/游戏、视频编解码、AI 负载、高显存占用。"
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
            AppLanguage::PortugueseBrazil => {
                "A GPU (Unidade de Processamento Gráfico) cuida da renderização de gráficos e dos cálculos em paralelo.\n\
                 \n\
                 O que mais consome energia:\n\
                 \u{2022} Jogos e renderização 3D\n\
                 \u{2022} Codificação e decodificação de vídeo\n\
                 \u{2022} Tarefas de IA e aprendizado de máquina\n\
                 \u{2022} Uso alto de VRAM e de largura de banda da memória"
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
            AppLanguage::German => {
                "Der Arbeitsspeicher (RAM) bietet schnellen temporären Speicher für laufende \
                 Programme und aktive Daten.\n\n\
                 Hauptverbraucher:\n\
                 \u{2022} Höhere Speichertaktfrequenzen (MHz)\n\
                 \u{2022} Mehr aktive Speichermodule\n\
                 \u{2022} Häufige Lese- und Schreibvorgänge\n\n\
                 Der Arbeitsspeicher verbraucht Energie, solange das System eingeschaltet ist, auch im Leerlauf."
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
            AppLanguage::Chinese => {
                "内存为运行程序提供高速临时存储。\n\n\
                 主要耗电：更高频率、更多模块、频繁读写；开机即耗电。"
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
            AppLanguage::PortugueseBrazil => {
                "A RAM (Memória de Acesso Aleatório) guarda, de forma temporária e com acesso rápido, os programas em execução e os dados em uso.\n\
                 \n\
                 O que aumenta o consumo:\n\
                 \u{2022} Frequências de memória mais altas (MHz)\n\
                 \u{2022} Mais módulos ativos\n\
                 \u{2022} Operações de leitura/gravação frequentes\n\
                 \n\
                 Em geral, a RAM gasta energia sempre que o sistema está ligado, mesmo em repouso."
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
            AppLanguage::German => {
                "Datenträger (SSD / HDD) speichern deine Dateien und dein System dauerhaft.\n\n\
                 Hauptverbraucher:\n\
                 \u{2022} Andauernde Lese- und Schreibvorgänge\n\
                 \u{2022} Rotierende Magnetscheiben (HDD)\n\
                 \u{2022} NAND-Schreibvorgänge (SSD)\n\
                 \u{2022} Suchvorgänge und Indizierung"
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
            AppLanguage::Chinese => {
                "存储设备用于永久保存数据。\n\n\
                 主要耗电\u{ff1a}持续读写、机械盘旋转、SSD 写入、寻道索引。"
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
            AppLanguage::PortugueseBrazil => {
                "Unidades de armazenamento (SSD / HDD) guardam seus arquivos e o sistema de forma permanente.\n\
                 \n\
                 Principais consumidores de energia:\n\
                 \u{2022} Operações contínuas de leitura/gravação\n\
                 \u{2022} Pratos girando (HDD)\n\
                 \u{2022} Operações de gravação NAND (SSD)\n\
                 \u{2022} Busca e indexação no disco"
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
            AppLanguage::German => {
                "Netzwerkschnittstellen übertragen Daten zwischen deinem Computer, anderen Geräten und dem Internet.\n\n\
                 Hauptverbraucher:\n\
                 \u{2022} Hoher Datendurchsatz\n\
                 \u{2022} WLAN-Funkübertragung\n\
                 \u{2022} Aktive Netzwerkverbindungen\n\
                 \u{2022} Bluetooth und drahtlose Peripheriegeräte\n\n\
                 Hinweis: Die Internetnutzung verursacht indirekte Emissionen durch Netzwerkinfrastruktur und entfernte Server (28\u{2013}63 g CO₂/GB). Diese sind nicht in WattSeals Messungen für deinen PC enthalten."
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
            AppLanguage::Chinese => {
                "网络接口负责计算机与其他设备或互联网之间的数据传输。\n\n\
                 主要耗电因素\u{ff1a}高数据吞吐量、Wi-Fi 无线传输、活跃的网络连接、蓝牙和无线外设。\n\n\
                 注意：使用互联网会通过网络基础设施和远程服务器产生间接排放 (28\u{2013}63 g CO₂/GB)\u{ff0c}这部分未包含在 WattSeal 对您电脑的测量中。"
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
            AppLanguage::PortugueseBrazil => {
                "As interfaces de rede cuidam da transmissão de dados entre o seu computador e outros dispositivos ou a internet.\n\
                 \n\
                 Principais consumidores de energia:\n\
                 \u{2022} Alto volume de dados trafegados\n\
                 \u{2022} Transmissão de rádio Wi-Fi\n\
                 \u{2022} Conexões de rede ativas\n\
                 \u{2022} Bluetooth e periféricos sem fio\n\
                 \n\
                 Observação: o uso da internet gera emissões indiretas da infraestrutura de rede e de servidores remotos (28\u{2013}63 g CO₂/GB), que não estão incluídas nas medições do WattSeal para o seu PC."
            }
        };
    } else if key == ProcessData::table_name_static() {
        return match language {
            AppLanguage::English => {
                "Shows which applications consume the most power on your system.\n\n\
                 Power is estimated based on CPU and GPU (highest variable consumption) usage of each process.\n\
                 Background processes and services also contribute to total consumption."
            }
            AppLanguage::German => {
                "Zeigt, welche Anwendungen auf deinem System am meisten Leistung aufnehmen.\n\n\
                 Die Leistung wird anhand der CPU- und GPU-Auslastung jedes Prozesses geschätzt. \
                 Hintergrundprozesse und Dienste tragen ebenfalls zum Gesamtverbrauch bei."
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
            AppLanguage::Chinese => {
                "显示哪些应用最耗电。\n\n\
                 功耗按各进程 CPU/GPU 使用率估算，后台服务也计入。"
            }
            AppLanguage::Romanian => {
                "Arată care sunt aplicațiile cu cel mai mare consum de curent din sistem.\n\n\
                 Consumul este estimat pe baza utilizării de CPU și GPU (cel mai mare consum \
                 variabil) al fiecărui proces.\n\
                 Procesele și serviciile ascunse contribuie de asemenea la consumul total."
            }
            AppLanguage::PortugueseBrazil => {
                "Mostra quais aplicativos consomem mais energia no seu sistema.\n\
                 \n\
                 A potência é estimada com base no uso de CPU e GPU (maior consumo variável) de cada processo.\n\
                 Processos e serviços em segundo plano também contribuem para o consumo total."
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
            AppLanguage::German => {
                "Zeigt die gesamte Leistungsaufnahme deines Systems.\n\n\
                 Sie setzt sich aus allen Hardwarekomponenten zusammen (CPU, GPU, RAM, Datenträger, \
                 Netzwerk). Wenn du weißt, welche Komponente am meisten verbraucht, kannst du den \
                 Energieverbrauch gezielt optimieren."
            }
            AppLanguage::French => {
                "Affiche la consommation totale de votre système.\n\n\
                 C'est la somme de tous les composants (CPU, GPU, RAM, Disque, \
                 Réseau). Comprendre quel composant consomme le plus aide à \
                 optimiser la consommation d'énergie."
            }
            AppLanguage::Chinese => {
                "显示整机总功耗，为各硬件组件之和。\n\n\
                 了解哪个组件最耗电有助于优化用电。"
            }
            AppLanguage::Romanian => {
                "Arată consumul total de curent al sistemului.\n\n\
                 Aceasta este suma totală a tuturor componentelor hardware (CPU, GPU, RAM, Disc, \
                 Rețea). Înțelegerea componentei care consumă cel mai mult ajută la optimizarea \
                 consumului de energie."
            }
            AppLanguage::PortugueseBrazil => {
                "Mostra o consumo total de energia do seu sistema.\n\
                 \n\
                 Esse valor é a soma do consumo de todos os componentes de hardware (CPU, GPU, RAM, disco e rede). Saber qual componente consome mais ajuda a otimizar o uso de energia."
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
            AppLanguage::German => {
                "Diese Emissionen berücksichtigen nur das CO₂, das durch den Stromverbrauch \
                 deines laufenden PCs entsteht.\n\n\
                 Für eine vollständige CO₂-Bilanz solltest du zusätzlich einbeziehen:\n\
                 \u{2022} Emissionen aus Herstellung und Transport (einmalig etwa 250\u{2013}500 kg CO₂eq), bei Gaming-PCs, zusätzlichen Bildschirmen und Peripheriegeräten entsprechend mehr\n\
                 \u{2022} Emissionen der Netzwerkinfrastruktur (28\u{2013}63 g CO₂/GB), die mit der Verbreitung von KI steigen\n\
                 \u{2022} Emissionen durch Entsorgung und Elektroschrott, einschließlich giftiger Stoffe in Deponien\n\n\
                 Hinweis: Digitale Technik benötigt außerdem viel Wasser für die Kühlung von Rechenzentren (etwa 1,9 l/kWh) und für den Rohstoffabbau, besonders in Regionen mit Wasserknappheit."
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
            AppLanguage::Chinese => {
                "此处仅反映电脑运行时消耗电力所产生的 CO₂。\n\n\
                 完整碳足迹还需考虑：\n\
                 \u{2022} 制造与运输排放 (约 250\u{2013}500 kg CO₂eq\u{ff0c}一次性)\u{ff0c}游戏电脑、额外显示器及外设更高\n\
                 \u{2022} 网络基础设施排放 (28\u{2013}63 g CO₂/GB) \u{ff0c}随 AI 兴起而持续增长\n\
                 \u{2022} 硬件废弃与电子垃圾排放 (差异较大，涉及填埋场有毒物质渗漏) \n\n\
                 注：数字技术也消耗大量水资源用于数据中心冷却 (约 1.9L/kWh) 及矿物开采，在水资源紧张地区尤为严重。"
            }
            AppLanguage::Romanian => {
                "Aceste emisii reflectă doar CO₂ generat de energia electrică consumată de PC în timpul funcționării.\n\n\
                 Pentru a estima amprenta de carbon completă, ar trebui să adăugați și:\n\
                 \u{2022} Emisiile din fabricație și transport (~250\u{2013}500 kg CO₂e, o singură dată), mai mult pentru PC-urile de gaming, monitoare suplimentare și periferice\n\
                 \u{2022} Emisiile din infrastructura rețelei (28\u{2013}63 g CO₂/GB), care cresc odată cu avansul IA\n\
                 \u{2022} Emisiile legate de eliminarea hardware-ului și deșeurilor electronice (variază mult, scurgeri de materiale toxice în depozite)\n\n\
                 Notă: Tehnologia digitală consumă și multă apă pentru răcirea centrelor de date (aprox. 1,9 L/kWh) și pentru extracția mineralelor, în special în regiuni cu stres hidric."
            }
            AppLanguage::PortugueseBrazil => {
                "Estas emissões refletem apenas o CO₂ gerado pela eletricidade que o seu PC consome enquanto está ligado.\n\
                 \n\
                 Para estimar a sua pegada de carbono completa, você também deve somar:\n\
                 \u{2022} Emissões de fabricação e transporte (~250\u{2013}500 kg CO₂eq, uma única vez), mais ainda para PCs gamer, monitores adicionais e periféricos\n\
                 \u{2022} Emissões da infraestrutura de rede (28\u{2013}63 g CO₂/GB), que crescem com o avanço da IA\n\
                 \u{2022} Emissões do descarte de hardware e do lixo eletrônico (variam muito; materiais tóxicos vazam em aterros)\n\
                 \n\
                 Observação: a tecnologia digital também consome muita água para resfriar datacenters (cerca de 1,9 L/kWh) e para a extração de minerais, especialmente em regiões com escassez de água."
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
                AppLanguage::German => {
                    "Dein Betriebssystem verwaltet alle Hardwareressourcen und laufenden Programme.\n\n\
                     Auswirkungen auf den Verbrauch:\n\
                     \u{2022} Hintergrunddienste und geplante Aufgaben\n\
                     \u{2022} Systemindizierung und Aktualisierungen\n\
                     \u{2022} Die Einstellungen des Energiesparplans wirken sich auf alle Komponenten aus\n\n\
                     Du kannst den Verbrauch senken, indem du unnötige Autostartprogramme und \
                     Hintergrundprozesse in der Taskleiste deaktivierst und Energiesparmodi verwendest."
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
                AppLanguage::Chinese => {
                    "操作系统管理所有硬件资源及运行中的软件。\n\n\
                     对功耗的影响：后台服务与计划任务、系统索引与更新、电源方案设置影响所有组件。\n\n\
                     可禁用不必要的自启程序、任务栏后台进程（天气、新闻等），并使用省电模式来优化。"
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
                AppLanguage::PortugueseBrazil => {
                    "O seu sistema operacional gerencia todos os recursos de hardware e os programas em execução.\n\
                 \n\
                 Impacto no consumo:\n\
                 \u{2022} Serviços em segundo plano e tarefas agendadas\n\
                 \u{2022} Indexação e atualizações do sistema\n\
                 \u{2022} As configurações do plano de energia afetam todos os componentes\n\
                 \n\
                 Você pode otimizar isso desativando programas desnecessários na inicialização, processos em segundo plano na barra de tarefas (clima, notícias...) e usando modos de economia de energia."
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
                AppLanguage::German => {
                    "Der Akku speichert Energie für den mobilen Betrieb und beeinflusst die Energieverwaltung.\n\n\
                     Wichtige Informationen:\n\
                     \u{2022} Die Anzahl der Ladezyklen zeigt Alterung und Zustand des Akkus\n\
                     \u{2022} Die ursprüngliche Kapazität nimmt mit der Zeit ab\n\
                     \u{2022} Im Akkubetrieb werden häufig Energiesparmodi aktiviert\n\
                     \u{2022} Schnellladen erzeugt mehr Wärme und benötigt mehr Energie"
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
                AppLanguage::Chinese => {
                    "电池为便携使用储存能量，并影响电源管理行为。\n\n\
                     要点：循环次数反映电池健康与老化程度、设计容量随时间衰减、使用电池时常触发省电模式、快充发热更大且耗能更多。"
                }
                AppLanguage::Romanian => {
                    "Bateria stochează energie pentru utilizare portabilă și influențează modul în care este gestionat consumul.\n\n\
                     Aspecte cheie:\n\
                     \u{2022} Numărul de cicluri reflectă sănătatea și îmbătrânirea bateriei\n\
                     \u{2022} Capacitatea proiectată scade în timp\n\
                     \u{2022} Utilizarea pe baterie declanșează adesea moduri de economisire a energiei\n\
                     \u{2022} Încărcarea rapidă generează mai multă căldură și consumă mai multă energie"
                }
                AppLanguage::PortugueseBrazil => {
                    "A bateria armazena energia para uso portátil e influencia como a energia é gerenciada.\n\
                 \n\
                 Fatos importantes:\n\
                 \u{2022} A contagem de ciclos reflete a saúde e o envelhecimento da bateria\n\
                 \u{2022} A capacidade de projeto diminui com o tempo\n\
                 \u{2022} Usar a bateria costuma ativar modos de economia de energia\n\
                 \u{2022} O carregamento rápido gera mais calor e consome mais energia"
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
                AppLanguage::German => {
                    "Bildschirme gehören zu den größten Verbrauchern, besonders bei hoher Helligkeit.\n\n\
                     Hauptverbraucher:\n\
                     \u{2022} Bildschirmhelligkeit (größter Faktor)\n\
                     \u{2022} Höhere Bildwiederholraten (Hz)\n\
                     \u{2022} Höhere Auflösungen\n\
                     \u{2022} HDR und ein großer Farbraum"
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
                AppLanguage::Chinese => {
                    "显示器是耗电大户，尤其在高亮度下。\n\n\
                     主要耗电因素：屏幕亮度 (影响最大) 、更高的刷新率、更高的分辨率、HDR 与广色域。"
                }
                AppLanguage::Romanian => {
                    "Display-urile sunt un mare consumator de energie, în special la luminozitate ridicată.\n\n\
                     Principalii consumatori:\n\
                     \u{2022} Luminozitatea ecranului (factorul principal)\n\
                     \u{2022} Rate de reîmprospătare mai mari (Hz)\n\
                     \u{2022} Rezoluții mai înalte\n\
                     \u{2022} HDR și gamă largă de culori"
                }
                AppLanguage::PortugueseBrazil => {
                    "As telas são grandes consumidoras de energia, principalmente com brilho alto.\n\
                 \n\
                 Principais consumidores de energia:\n\
                 \u{2022} Brilho da tela (o fator mais importante)\n\
                 \u{2022} Taxas de atualização mais altas (Hz)\n\
                 \u{2022} Resoluções mais altas\n\
                 \u{2022} HDR e gama de cores ampla"
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
                AppLanguage::German => {
                    "Datenträger (SSD / HDD) speichern deine Dateien und dein System dauerhaft.\n\n\
                     Hauptverbraucher:\n\
                     \u{2022} Andauernde Lese- und Schreibvorgänge\n\
                     \u{2022} Rotierende Magnetscheiben (HDD)\n\
                     \u{2022} NAND-Schreibvorgänge (SSD)\n\
                     \u{2022} Suchvorgänge und Indizierung"
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
                AppLanguage::Chinese => {
                    "存储设备 (SSD / HDD) 为文件与系统提供永久存储。\n\n\
                     主要耗电因素：持续读写操作、机械盘片旋转 (HDD)、NAND 写入操作 (SSD)、磁盘寻道与索引。"
                }
                AppLanguage::Romanian => {
                    "Discurile de stocare (SSD / HDD) oferă stocare permanentă pentru fișierele și sistemul dvs.\n\n\
                     Principalii consumatori:\n\
                     \u{2022} Operații de citire/scriere susținute\n\
                     \u{2022} Rotirea platanelor (HDD)\n\
                     \u{2022} Operații de scriere NAND (SSD)\n\
                     \u{2022} Căutări și indexări pe disc"
                }
                AppLanguage::PortugueseBrazil => {
                    "Unidades de armazenamento (SSD / HDD) guardam seus arquivos e o sistema de forma permanente.\n\
                 \n\
                 Principais consumidores de energia:\n\
                 \u{2022} Operações contínuas de leitura/gravação\n\
                 \u{2022} Pratos girando (HDD)\n\
                 \u{2022} Operações de gravação NAND (SSD)\n\
                 \u{2022} Busca e indexação no disco"
                }
            },
            _ => match language {
                AppLanguage::English => "No additional information available for this component.",
                AppLanguage::German => "Für diese Komponente sind keine weiteren Informationen verfügbar.",
                AppLanguage::French => "Aucune information supplémentaire disponible pour ce composant.",
                AppLanguage::Chinese => "暂无该组件的更多信息。",
                AppLanguage::Romanian => "Nu sunt disponibile informații suplimentare pentru această componentă.",
                AppLanguage::PortugueseBrazil => "Nenhuma informação adicional disponível para este componente.",
            },
        };
    }
}

/// Label for measured (app-tracked) CO₂ emissions row in the carbon info modal.
pub fn carbon_info_measured(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Measured by WattSeal",
        AppLanguage::German => "Von WattSeal gemessen",
        AppLanguage::French => "Mesuré par WattSeal",
        AppLanguage::Chinese => "WattSeal 实测",
        AppLanguage::Romanian => "Măsurat de WattSeal",
        AppLanguage::PortugueseBrazil => "Medido pelo WattSeal",
    }
}

/// Label for estimated base (manufacturing + transport) CO₂ row.
pub fn carbon_info_base(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Estimated base (manufacturing + transport)",
        AppLanguage::German => "Geschätzte Basis (Herstellung und Transport)",
        AppLanguage::French => "Base estimée (fabrication + transport)",
        AppLanguage::Chinese => "预估基础 (制造与运输)",
        AppLanguage::Romanian => "Estimare de bază (fabricare + transport)",
        AppLanguage::PortugueseBrazil => "Base estimada (fabricação + transporte)",
    }
}

/// Label for estimated annual network CO₂ row.
pub fn carbon_info_annual(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Estimated annual (network usage)",
        AppLanguage::German => "Jährliche Schätzung (Netzwerknutzung)",
        AppLanguage::French => "Annuel estimé (utilisation réseau)",
        AppLanguage::Chinese => "预估年度 (网络使用)",
        AppLanguage::Romanian => "Estimare anuală (utilizare rețea)",
        AppLanguage::PortugueseBrazil => "Estimativa anual (uso de rede)",
    }
}

/// Label for estimated all-time CO₂ total row.
pub fn carbon_info_all_time(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Estimated all-time total",
        AppLanguage::German => "Geschätzte Gesamtemissionen",
        AppLanguage::French => "Total estimé (toutes sources)",
        AppLanguage::Chinese => "预估累计总量",
        AppLanguage::Romanian => "Total estimat (toate sursele)",
        AppLanguage::PortugueseBrazil => "Total estimado de todo o período",
    }
}

// Pick lists

pub fn process_limit_name(language: AppLanguage, limit: ProcessLimit) -> &'static str {
    match limit {
        ProcessLimit::Five => match language {
            AppLanguage::English
            | AppLanguage::German
            | AppLanguage::French
            | AppLanguage::Romanian
            | AppLanguage::PortugueseBrazil => "Top 5",
            AppLanguage::Chinese => "前 5",
        },
        ProcessLimit::Ten => match language {
            AppLanguage::English
            | AppLanguage::German
            | AppLanguage::French
            | AppLanguage::Romanian
            | AppLanguage::PortugueseBrazil => "Top 10",
            AppLanguage::Chinese => "前 10",
        },
        ProcessLimit::Custom => custom_preset_name(language),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslatedProcessLimit {
    pub limit: ProcessLimit,
    language: AppLanguage,
}

impl TranslatedProcessLimit {
    pub fn new(limit: ProcessLimit, language: AppLanguage) -> Self {
        Self { limit, language }
    }

    pub fn options(language: AppLanguage) -> Vec<Self> {
        ProcessLimit::all()
            .iter()
            .map(|&limit| Self::new(limit, language))
            .collect()
    }
}

impl std::fmt::Display for TranslatedProcessLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", process_limit_name(self.language, self.limit))
    }
}

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

pub fn settings_close_behavior(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "When closing",
        AppLanguage::German => "Beim Schließen",
        AppLanguage::French => "À la fermeture",
        AppLanguage::Chinese => "关闭时",
        AppLanguage::Romanian => "La închidere",
        AppLanguage::PortugueseBrazil => "Ao fechar",
    }
}

pub fn close_remember_choice(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Remember my choice",
        AppLanguage::German => "Auswahl merken",
        AppLanguage::French => "Mémoriser mon choix",
        AppLanguage::Chinese => "记住我的选择",
        AppLanguage::Romanian => "Reține alegerea mea",
        AppLanguage::PortugueseBrazil => "Lembrar minha escolha",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TranslatedCloseBehavior {
    pub behavior: common::CloseBehavior,
    language: AppLanguage,
}

impl TranslatedCloseBehavior {
    pub fn new(behavior: common::CloseBehavior, language: AppLanguage) -> Self {
        Self { behavior, language }
    }

    pub fn all(language: AppLanguage) -> Vec<Self> {
        use common::CloseBehavior;
        [CloseBehavior::Ask, CloseBehavior::WindowOnly, CloseBehavior::Everything]
            .into_iter()
            .map(|behavior| Self::new(behavior, language))
            .collect()
    }
}

impl std::fmt::Display for TranslatedCloseBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self.behavior {
            common::CloseBehavior::Ask => match self.language {
                AppLanguage::English => "Ask every time",
                AppLanguage::German => "Jedes Mal fragen",
                AppLanguage::French => "Toujours demander",
                AppLanguage::Chinese => "每次询问",
                AppLanguage::Romanian => "Întreabă de fiecare dată",
                AppLanguage::PortugueseBrazil => "Perguntar sempre",
            },
            common::CloseBehavior::WindowOnly => close_ui_only(self.language),
            common::CloseBehavior::Everything => close_everything(self.language),
        };
        f.write_str(label)
    }
}

pub fn close_dialog_title(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Close WattSeal",
        AppLanguage::German => "WattSeal schließen",
        AppLanguage::French => "Fermer WattSeal",
        AppLanguage::Chinese => "关闭 WattSeal",
        AppLanguage::Romanian => "Închide WattSeal",
        AppLanguage::PortugueseBrazil => "Fechar o WattSeal",
    }
}

pub fn close_dialog_description(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Do you want to close only the window, or also stop the background collector?",
        AppLanguage::German => {
            "Möchtest du nur das Fenster schließen oder auch den Datensammler im Hintergrund beenden?"
        }
        AppLanguage::French => "Voulez-vous fermer uniquement l'interface, ou aussi arrêter le collecteur ?",
        AppLanguage::Chinese => "仅关闭窗口，还是同时停止后台采集？",
        AppLanguage::Romanian => "Doriți să închideți doar fereastra sau și să opriți colectorul de fundal?",
        AppLanguage::PortugueseBrazil => "Deseja fechar somente a janela ou também parar o coletor em segundo plano?",
    }
}

pub fn close_ui_only(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Close window only",
        AppLanguage::German => "Nur Fenster schließen",
        AppLanguage::French => "Fermer l'interface",
        AppLanguage::Chinese => "仅关闭窗口",
        AppLanguage::Romanian => "Închide doar fereastra",
        AppLanguage::PortugueseBrazil => "Fechar somente a janela",
    }
}

pub fn close_everything(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Close everything",
        AppLanguage::German => "Alles schließen",
        AppLanguage::French => "Tout fermer",
        AppLanguage::Chinese => "完全退出",
        AppLanguage::Romanian => "Închide tot",
        AppLanguage::PortugueseBrazil => "Fechar tudo",
    }
}

// Localized Theme & Country Presets

pub fn theme_name(language: AppLanguage, theme: AppTheme) -> &'static str {
    match theme {
        AppTheme::DeepOcean => match language {
            AppLanguage::English | AppLanguage::Romanian | AppLanguage::PortugueseBrazil => "Hunting",
            AppLanguage::German => "Jagen",
            AppLanguage::French => "Chasse",
            AppLanguage::Chinese => "捕猎",
        },
        AppTheme::OceanLight => match language {
            AppLanguage::English | AppLanguage::Romanian | AppLanguage::PortugueseBrazil => "Swimming",
            AppLanguage::German => "Schwimmen",
            AppLanguage::French => "Baignade",
            AppLanguage::Chinese => "畅游",
        },
        AppTheme::EcoEnergy => match language {
            AppLanguage::English | AppLanguage::Romanian | AppLanguage::PortugueseBrazil => "Sleeping",
            AppLanguage::German => "Schlafen",
            AppLanguage::French => "Dodo",
            AppLanguage::Chinese => "酣睡",
        },
        AppTheme::EcoEnergyLight => match language {
            AppLanguage::English | AppLanguage::Romanian | AppLanguage::PortugueseBrazil => "Splashing",
            AppLanguage::German => "Planschen",
            AppLanguage::French => "Bataille d'eau",
            AppLanguage::Chinese => "戏水",
        },
        AppTheme::GeothermalCore => match language {
            AppLanguage::English | AppLanguage::Romanian | AppLanguage::PortugueseBrazil => "Sunbathing",
            AppLanguage::German => "Sonnenbaden",
            AppLanguage::French => "Bronzette",
            AppLanguage::Chinese => "晒日",
        },
        AppTheme::SolarUmbra => match language {
            AppLanguage::English | AppLanguage::Romanian | AppLanguage::PortugueseBrazil => "Lounging",
            AppLanguage::German => "Entspannen",
            AppLanguage::French => "Détente",
            AppLanguage::Chinese => "小憩",
        },
    }
}

pub fn country_name(language: AppLanguage, country: CountryKey) -> &'static str {
    match country {
        CountryKey::France => match language {
            AppLanguage::English | AppLanguage::French => "France",
            AppLanguage::German => "Frankreich",
            AppLanguage::Romanian => "Franța",
            AppLanguage::Chinese => "法国",
            AppLanguage::PortugueseBrazil => "França",
        },
        CountryKey::Germany => match language {
            AppLanguage::English => "Germany",
            AppLanguage::German => "Deutschland",
            AppLanguage::French => "Allemagne",
            AppLanguage::Romanian => "Germania",
            AppLanguage::Chinese => "德国",
            AppLanguage::PortugueseBrazil => "Alemanha",
        },
        CountryKey::Spain => match language {
            AppLanguage::English => "Spain",
            AppLanguage::German => "Spanien",
            AppLanguage::French => "Espagne",
            AppLanguage::Romanian => "Spania",
            AppLanguage::Chinese => "西班牙",
            AppLanguage::PortugueseBrazil => "Espanha",
        },
        CountryKey::Italy => match language {
            AppLanguage::English => "Italy",
            AppLanguage::German => "Italien",
            AppLanguage::French => "Italie",
            AppLanguage::Romanian => "Italia",
            AppLanguage::Chinese => "意大利",
            AppLanguage::PortugueseBrazil => "Itália",
        },
        CountryKey::Netherlands => match language {
            AppLanguage::English => "Netherlands",
            AppLanguage::German => "Niederlande",
            AppLanguage::French => "Pays-Bas",
            AppLanguage::Romanian => "Olanda",
            AppLanguage::Chinese => "荷兰",
            AppLanguage::PortugueseBrazil => "Holanda",
        },
        CountryKey::Switzerland => match language {
            AppLanguage::English => "Switzerland",
            AppLanguage::German => "Schweiz",
            AppLanguage::French => "Suisse",
            AppLanguage::Romanian => "Elveția",
            AppLanguage::Chinese => "瑞士",
            AppLanguage::PortugueseBrazil => "Suíça",
        },
        CountryKey::Belgium => match language {
            AppLanguage::English => "Belgium",
            AppLanguage::German => "Belgien",
            AppLanguage::French => "Belgique",
            AppLanguage::Romanian => "Belgia",
            AppLanguage::Chinese => "比利时",
            AppLanguage::PortugueseBrazil => "Bélgica",
        },
        CountryKey::Portugal => match language {
            AppLanguage::English | AppLanguage::German | AppLanguage::French | AppLanguage::PortugueseBrazil => {
                "Portugal"
            }
            AppLanguage::Romanian => "Portugalia",
            AppLanguage::Chinese => "葡萄牙",
        },
        CountryKey::Brazil => match language {
            AppLanguage::English => "Brazil",
            AppLanguage::German => "Brasilien",
            AppLanguage::French => "Brésil",
            AppLanguage::Romanian => "Brazilia",
            AppLanguage::Chinese => "巴西",
            AppLanguage::PortugueseBrazil => "Brasil",
        },
        CountryKey::UK => match language {
            AppLanguage::English => "UK",
            AppLanguage::German => "Vereinigtes Königreich",
            AppLanguage::French => "Royaume-Uni",
            AppLanguage::Romanian => "Regatul Unit",
            AppLanguage::Chinese => "英国",
            AppLanguage::PortugueseBrazil => "Reino Unido",
        },
        CountryKey::USA => match language {
            AppLanguage::English => "USA (average)",
            AppLanguage::German => "USA (Durchschnitt)",
            AppLanguage::French => "États-Unis (moyenne)",
            AppLanguage::Romanian => "SUA (medie)",
            AppLanguage::Chinese => "美国（平均）",
            AppLanguage::PortugueseBrazil => "Estados Unidos (média)",
        },
        CountryKey::China => match language {
            AppLanguage::English | AppLanguage::German | AppLanguage::Romanian | AppLanguage::PortugueseBrazil => {
                "China"
            }
            AppLanguage::French => "Chine",
            AppLanguage::Chinese => "中国",
        },
        CountryKey::India => match language {
            AppLanguage::English | AppLanguage::Romanian => "India",
            AppLanguage::German => "Indien",
            AppLanguage::French => "Inde",
            AppLanguage::Chinese => "印度",
            AppLanguage::PortugueseBrazil => "Índia",
        },
        CountryKey::Indonesia => match language {
            AppLanguage::English => "Indonesia",
            AppLanguage::German => "Indonesien",
            AppLanguage::French => "Indonésie",
            AppLanguage::Romanian => "Indonezia",
            AppLanguage::Chinese => "印度尼西亚",
            AppLanguage::PortugueseBrazil => "Indonésia",
        },
        CountryKey::Philippines => match language {
            AppLanguage::English | AppLanguage::French => "Philippines",
            AppLanguage::German => "Philippinen",
            AppLanguage::Romanian => "Filipine",
            AppLanguage::Chinese => "菲律宾",
            AppLanguage::PortugueseBrazil => "Filipinas",
        },
        CountryKey::Australia => match language {
            AppLanguage::English | AppLanguage::Romanian => "Australia",
            AppLanguage::German => "Australien",
            AppLanguage::French => "Australie",
            AppLanguage::Chinese => "澳大利亚",
            AppLanguage::PortugueseBrazil => "Austrália",
        },
        CountryKey::Sweden => match language {
            AppLanguage::English => "Sweden",
            AppLanguage::German => "Schweden",
            AppLanguage::French => "Suède",
            AppLanguage::Romanian => "Suedia",
            AppLanguage::Chinese => "瑞典",
            AppLanguage::PortugueseBrazil => "Suécia",
        },
        CountryKey::Poland => match language {
            AppLanguage::English => "Poland",
            AppLanguage::German => "Polen",
            AppLanguage::French => "Pologne",
            AppLanguage::Romanian => "Polonia",
            AppLanguage::Chinese => "波兰",
            AppLanguage::PortugueseBrazil => "Polônia",
        },
        CountryKey::Romania => match language {
            AppLanguage::English => "Romania",
            AppLanguage::German => "Rumänien",
            AppLanguage::French => "Roumanie",
            AppLanguage::Romanian => "România",
            AppLanguage::Chinese => "罗马尼亚",
            AppLanguage::PortugueseBrazil => "Romênia",
        },
        CountryKey::Iceland => match language {
            AppLanguage::English => "Iceland",
            AppLanguage::German => "Island",
            AppLanguage::French => "Islande",
            AppLanguage::Romanian => "Islanda",
            AppLanguage::Chinese => "冰岛",
            AppLanguage::PortugueseBrazil => "Islândia",
        },
        CountryKey::Georgia => match language {
            AppLanguage::English | AppLanguage::Romanian => "Georgia",
            AppLanguage::German => "Georgien",
            AppLanguage::French => "Géorgie",
            AppLanguage::Chinese => "格鲁吉亚",
            AppLanguage::PortugueseBrazil => "Geórgia",
        },
        CountryKey::World => match language {
            AppLanguage::English => "World average",
            AppLanguage::German => "Weltdurchschnitt",
            AppLanguage::French => "Moyenne mondiale",
            AppLanguage::Romanian => "Media mondială",
            AppLanguage::Chinese => "全球平均",
            AppLanguage::PortugueseBrazil => "Média mundial",
        },
        CountryKey::Custom => custom_preset_name(language),
    }
}

pub fn custom_preset_name(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Custom",
        AppLanguage::German => "Benutzerdefiniert",
        AppLanguage::French => "Personnalisé",
        AppLanguage::Chinese => "自定义",
        AppLanguage::Romanian => "Personalizat",
        AppLanguage::PortugueseBrazil => "Personalizado",
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

pub fn psu_class_name(language: AppLanguage, class: PsuClass) -> &'static str {
    match class {
        PsuClass::None => match language {
            AppLanguage::English => "Perfect",
            AppLanguage::German => "Perfekt",
            AppLanguage::French => "Parfaite",
            AppLanguage::Chinese => "完美",
            AppLanguage::Romanian => "Perfect",
            AppLanguage::PortugueseBrazil => "Perfeita",
        },
        PsuClass::Basic80 => "80+",
        PsuClass::Bronze80 => "80+ Bronze",
        PsuClass::Silver80 => "80+ Silver",
        PsuClass::Gold80 => "80+ Gold",
        PsuClass::Platinum80 => "80+ Platinum",
        PsuClass::Titanium80 => "80+ Titanium",
        PsuClass::Custom => custom_preset_name(language),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranslatedPsuEfficiency {
    pub efficiency: PsuEfficiency,
    language: AppLanguage,
}

impl TranslatedPsuEfficiency {
    pub fn new(efficiency: PsuEfficiency, language: AppLanguage) -> Self {
        Self { efficiency, language }
    }

    pub fn all(language: AppLanguage) -> Vec<Self> {
        PsuEfficiency::presets()
            .into_iter()
            .map(|p| Self::new(p, language))
            .collect()
    }
}

impl std::fmt::Display for TranslatedPsuEfficiency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = psu_class_name(self.language, self.efficiency.class);
        if self.efficiency.is_custom() {
            write!(f, "{}", name)
        } else {
            write!(f, "{} ({:.0}%)", name, self.efficiency.eta * 100.0)
        }
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
        CarbonIntensity::presets()
            .into_iter()
            .map(|p| Self::new(p, language))
            .collect()
    }
}

impl std::fmt::Display for TranslatedCarbonIntensity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let country = country_name(self.language, self.intensity.country);
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
        ElectricityCost::presets()
            .into_iter()
            .map(|p| Self::new(p, language))
            .collect()
    }
}

impl std::fmt::Display for TranslatedElectricityCost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let country = country_name(self.language, self.cost.country);
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
        AppLanguage::English
        | AppLanguage::German
        | AppLanguage::Chinese
        | AppLanguage::Romanian
        | AppLanguage::PortugueseBrazil => "GB",
        AppLanguage::French => "Go",
    };
    format!("{} {}", format_number(gb, 2, language), unit)
}

pub fn format_mb_per_sec(mb: f64, language: AppLanguage) -> String {
    format!(
        "{} {}",
        format_number(mb, 1, language),
        metric_unit(language, MetricKind::Speed)
    )
}

pub fn metric_unit(language: AppLanguage, metric: MetricKind) -> &'static str {
    match metric {
        MetricKind::Power => "W",
        MetricKind::Usage => "%",
        MetricKind::Speed => match language {
            AppLanguage::English
            | AppLanguage::German
            | AppLanguage::Chinese
            | AppLanguage::Romanian
            | AppLanguage::PortugueseBrazil => "MB/s",
            AppLanguage::French => "Mo/s",
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
                    AppLanguage::English | AppLanguage::Chinese => ".",
                    AppLanguage::German | AppLanguage::French | AppLanguage::Romanian => ",",
                    AppLanguage::PortugueseBrazil => ",",
                },
                "0".repeat(decimals)
            )
        } else {
            "0".to_string()
        };
    }

    let thousands_sep = match language {
        AppLanguage::English => ",",
        AppLanguage::German => ".",
        AppLanguage::French | AppLanguage::Romanian => " ",
        AppLanguage::Chinese => "",
        AppLanguage::PortugueseBrazil => ".",
    };
    let decimal_sep = match language {
        AppLanguage::English | AppLanguage::Chinese => ".",
        AppLanguage::German | AppLanguage::French | AppLanguage::Romanian => ",",
        AppLanguage::PortugueseBrazil => ",",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn german_language_is_selectable_and_round_trips() {
        assert!(AppLanguage::all().contains(&AppLanguage::German));
        assert_eq!(AppLanguage::German.code(), "DE");
        assert_eq!(AppLanguage::from_code("DE"), AppLanguage::German);
        assert_eq!(AppLanguage::German.to_string(), "Deutsch");
    }

    #[test]
    fn german_number_format_uses_local_separators() {
        assert_eq!(format_number(12_345.67, 2, AppLanguage::German), "12.345,67");
        assert_eq!(format_number(-0.5, 1, AppLanguage::German), "-0,5");
    }

    #[test]
    fn german_labels_cover_core_ui_surfaces() {
        assert_eq!(page_dashboard(AppLanguage::German), "Übersicht");
        assert_eq!(settings_title(AppLanguage::German), "Einstellungen");
        assert_eq!(sensor_name(AppLanguage::German, "Processes"), "Prozesse");
        assert_eq!(sensor_name(AppLanguage::German, "Total"), "Gesamt");
        assert_eq!(country_name(AppLanguage::German, CountryKey::Germany), "Deutschland");
        assert_eq!(close_everything(AppLanguage::German), "Alles schließen");
    }

    #[test]
    fn brazilian_portuguese_is_selectable_and_round_trips() {
        assert!(AppLanguage::all().contains(&AppLanguage::PortugueseBrazil));
        assert_eq!(AppLanguage::PortugueseBrazil.code(), "PT-BR");
        assert_eq!(AppLanguage::from_code("PT-BR"), AppLanguage::PortugueseBrazil);
        assert_eq!(AppLanguage::from_code("PT"), AppLanguage::PortugueseBrazil);
        assert_eq!(AppLanguage::PortugueseBrazil.to_string(), "Português (Brasil)");
    }

    #[test]
    fn brazilian_portuguese_number_format_uses_local_separators() {
        assert_eq!(format_number(12_345.67, 2, AppLanguage::PortugueseBrazil), "12.345,67");
        assert_eq!(format_number(-0.5, 1, AppLanguage::PortugueseBrazil), "-0,5");
        assert_eq!(format_number(f64::NAN, 1, AppLanguage::PortugueseBrazil), "0,0");
    }

    #[test]
    fn brazilian_portuguese_labels_cover_core_ui_surfaces() {
        assert_eq!(page_dashboard(AppLanguage::PortugueseBrazil), "Painel");
        assert_eq!(settings_title(AppLanguage::PortugueseBrazil), "Configurações");
        assert_eq!(sensor_name(AppLanguage::PortugueseBrazil, "Processes"), "Processos");
        assert_eq!(
            country_name(AppLanguage::PortugueseBrazil, CountryKey::Brazil),
            "Brasil"
        );
        assert_eq!(close_everything(AppLanguage::PortugueseBrazil), "Fechar tudo");
        assert_eq!(format_bytes_gb(1 << 30, AppLanguage::PortugueseBrazil), "1,00 GB");
    }
}
