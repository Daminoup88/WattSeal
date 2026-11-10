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
        | AppLanguage::PortugueseBrazil
        | AppLanguage::Russian => "WattSeal",
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
        AppLanguage::Russian => "Панель",
    }
}

pub fn page_info(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::German | AppLanguage::Romanian | AppLanguage::PortugueseBrazil => "Info",
        AppLanguage::French => "Infos",
        AppLanguage::Chinese => "信息",
        AppLanguage::Russian => "Инфо",
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
        AppLanguage::Russian => "Настройки",
    }
}

pub fn settings_general(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::Romanian => "General",
        AppLanguage::German => "Allgemein",
        AppLanguage::French => "Général",
        AppLanguage::Chinese => "常规",
        AppLanguage::PortugueseBrazil => "Geral",
        AppLanguage::Russian => "Общие",
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
        AppLanguage::Russian => "Тема",
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
        AppLanguage::Russian => "Язык",
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
        AppLanguage::Russian => "Запускать при старте системы",
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
        AppLanguage::Russian => "Открыть папку данных",
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
        AppLanguage::Russian => "Открыть папку журналов",
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
        AppLanguage::Russian => "Открыть папку данных и журналов",
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
        AppLanguage::Russian => "Закрыть",
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
        AppLanguage::Russian => "Поставить звезду на GitHub",
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
        AppLanguage::Russian => "Сообщить о проблеме",
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
        AppLanguage::Russian => "Текущее энергопотребление",
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
        AppLanguage::Russian => "Подготовка базы данных",
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
        AppLanguage::Russian => {
            "WattSeal ожидает завершения миграции базы данных сборщиком. Повторная попытка каждую секунду…"
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
        AppLanguage::Russian => "За всё время",
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
        AppLanguage::Russian => "Всего",
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
        AppLanguage::Russian => "Выбросы",
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
        AppLanguage::Russian => "\u{26a0} Выберите реальную углеродную интенсивность в настройках! \u{26a0}",
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
        | AppLanguage::PortugueseBrazil
        | AppLanguage::Russian => "CPU",
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
        AppLanguage::Russian => "Информация о процессоре",
    }
}

pub fn model(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::Romanian => "Model",
        AppLanguage::German => "Modell",
        AppLanguage::French => "Modèle",
        AppLanguage::Chinese => "型号",
        AppLanguage::PortugueseBrazil => "Modelo",
        AppLanguage::Russian => "Модель",
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
        AppLanguage::Russian => "Ядра",
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
        AppLanguage::Russian => format!("{} ядер / {} потоков", physical, logical),
    }
}

pub fn gpu(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English
        | AppLanguage::German
        | AppLanguage::French
        | AppLanguage::Chinese
        | AppLanguage::Romanian
        | AppLanguage::PortugueseBrazil
        | AppLanguage::Russian => "GPU",
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
        AppLanguage::Russian => "Информация о графике",
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
        AppLanguage::Russian => format!("Графический процессор {}", n),
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
        AppLanguage::Russian => "Память",
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
        AppLanguage::Russian => "Информация о RAM",
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
        AppLanguage::Russian => "Общий объём памяти",
    }
}

pub fn swap(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English | AppLanguage::French | AppLanguage::Chinese | AppLanguage::Romanian => "Swap",
        AppLanguage::German => "Auslagerungsspeicher",
        AppLanguage::PortugueseBrazil => "Memória virtual",
        AppLanguage::Russian => "Файл подкачки",
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
        AppLanguage::Russian => "Система",
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
        AppLanguage::Russian => "Информация об ОС",
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
        AppLanguage::Russian => "Операционная система",
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
        AppLanguage::Russian => "Имя компьютера",
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
        AppLanguage::Russian => "Накопители",
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
        AppLanguage::Russian => "Информация о дисках",
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
        AppLanguage::Russian => "Диск",
    }
}

pub fn disk_n(language: AppLanguage, n: usize) -> String {
    match language {
        AppLanguage::English
        | AppLanguage::German
        | AppLanguage::French
        | AppLanguage::Chinese
        | AppLanguage::Romanian
        | AppLanguage::PortugueseBrazil
        | AppLanguage::Russian => {
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
        AppLanguage::Russian => "Объём",
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
        AppLanguage::Russian => "Сеть",
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
        AppLanguage::Russian => "Батарея",
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
        AppLanguage::Russian => "Состояние батареи",
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
        AppLanguage::Russian => "Процессы",
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
        AppLanguage::Russian => "Имя",
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
        AppLanguage::Russian => "Ёмкость",
    }
}

pub fn capacity_wh_cycles(language: AppLanguage, cap_wh: f32, cycles: u32) -> String {
    let val = format_number(cap_wh as f64, 1, language);
    match language {
        AppLanguage::English | AppLanguage::French | AppLanguage::Chinese => format!("{} Wh ({} cycles)", val, cycles),
        AppLanguage::German => format!("{} Wh ({} Ladezyklen)", val, cycles),
        AppLanguage::Romanian => format!("{} Wh ({} cicluri)", val, cycles),
        AppLanguage::PortugueseBrazil => format!("{} Wh ({} ciclos)", val, cycles),
        AppLanguage::Russian => format!("{} Вт·ч ({} циклов)", val, cycles),
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
        AppLanguage::Russian => format!("{} Вт·ч", val),
    }
}

pub fn na_with_cycles(language: AppLanguage, cycles: u32) -> String {
    match language {
        AppLanguage::English | AppLanguage::French | AppLanguage::Chinese => format!("N/A ({} cycles)", cycles),
        AppLanguage::German => format!("N/A ({} Ladezyklen)", cycles),
        AppLanguage::Romanian => format!("N/A ({} cicluri)", cycles),
        AppLanguage::PortugueseBrazil => format!("N/D ({} ciclos)", cycles),
        AppLanguage::Russian => format!("Н/Д ({} циклов)", cycles),
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
        AppLanguage::Russian => "Дисплей",
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
        AppLanguage::Russian => "Информация об экране",
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
        AppLanguage::Russian => "Режим",
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
        AppLanguage::Russian => "Основной дисплей",
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
        AppLanguage::Russian => "Дополнительный дисплей",
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
        AppLanguage::Russian => "Н/Д",
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
        AppLanguage::Russian => "Данные недоступны",
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
        AppLanguage::Russian => "Мощность:",
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
        AppLanguage::Russian => format!("Значение: {}", value_text),
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
        AppLanguage::Russian => format!("Время: {}", time_text),
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
        AppLanguage::Russian => "Приложение",
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
        AppLanguage::Russian => "Мощность",
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
        AppLanguage::Russian => "Энергия",
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
        | AppLanguage::PortugueseBrazil
        | AppLanguage::Russian => "RAM",
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
        AppLanguage::Russian => "Чтение с диска",
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
        AppLanguage::Russian => "Запись на диск",
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
        AppLanguage::Russian => "Последняя минута",
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
        AppLanguage::Russian => "Последний час",
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
        AppLanguage::Russian => "Последние 24 часа",
    }
}

pub fn last_week(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English =>
