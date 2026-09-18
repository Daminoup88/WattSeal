//! Overlay translations.
//!
//! The overlay is a standalone crate that must not depend on `ui` — doing so
//! would drag the whole dashboard into the overlay process — so the handful of
//! strings the overlay itself shows live here. The language is read from the
//! same place the dashboard keeps it (the `ui_settings` row), so the overlay
//! simply follows whatever language the user picked there.

use common::Database;

/// Languages the dashboard ships, identified by the codes it stores.
///
/// Mirrors `ui::types::AppLanguage`; kept local to this crate for the
/// dependency reason described above.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    English,
    German,
    French,
    Chinese,
    Romanian,
}

impl Language {
    /// Parses the code stored in `ui_settings.language` ("EN", "ZH", …).
    pub fn from_code(code: &str) -> Self {
        match code.to_ascii_uppercase().as_str() {
            "DE" => Self::German,
            "FR" => Self::French,
            "ZH" => Self::Chinese,
            "RO" => Self::Romanian,
            _ => Self::English,
        }
    }

    /// The language the dashboard is currently set to.
    ///
    /// Falls back to English when there is no database or no saved settings
    /// yet, which is also what a standalone `--overlay` run sees.
    pub fn from_database(database: Option<&Database>) -> Self {
        database
            .and_then(|database| database.load_ui_settings().ok().flatten())
            .map(|settings| Self::from_code(&settings.language))
            .unwrap_or_default()
    }
}

/// A value that knows how to label itself in a given language.
pub trait Localize {
    fn localize(&self, language: Language) -> &'static str;
}

/// A setting value paired with the language its label should be shown in.
///
/// `pick_list` renders both the options and the selected value through
/// `Display`, so the translated text has to travel with the value. This is the
/// same approach the dashboard uses for its `TranslatedMetricType`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Labeled<T> {
    pub value: T,
    language: Language,
}

impl<T: Localize> Labeled<T> {
    pub fn new(value: T, language: Language) -> Self {
        Self { value, language }
    }
}

impl<T: Localize> std::fmt::Display for Labeled<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value.localize(self.language))
    }
}

// Menu

pub fn menu_resume(language: Language) -> &'static str {
    match language {
        Language::English => "Resume",
        Language::German => "Weiter",
        Language::French => "Reprendre",
        Language::Chinese => "继续",
        Language::Romanian => "Reia",
    }
}

pub fn menu_settings(language: Language) -> &'static str {
    match language {
        Language::English => "Settings",
        Language::German => "Einstellungen",
        Language::French => "Paramètres",
        Language::Chinese => "设置",
        Language::Romanian => "Setări",
    }
}

pub fn menu_pin(language: Language) -> &'static str {
    match language {
        Language::English => "Pin",
        Language::German => "Anheften",
        Language::French => "Épingler",
        Language::Chinese => "固定",
        Language::Romanian => "Fixează",
    }
}

pub fn menu_unpin(language: Language) -> &'static str {
    match language {
        Language::English => "Unpin",
        Language::German => "Lösen",
        Language::French => "Détacher",
        Language::Chinese => "取消固定",
        Language::Romanian => "Anulează fixarea",
    }
}

pub fn menu_exit(language: Language) -> &'static str {
    match language {
        Language::English => "Exit",
        Language::German => "Beenden",
        Language::French => "Quitter",
        Language::Chinese => "退出",
        Language::Romanian => "Ieșire",
    }
}

// Section titles

pub fn section_appearance(language: Language) -> &'static str {
    match language {
        Language::English => "Appearance",
        Language::German => "Aussehen",
        Language::French => "Apparence",
        Language::Chinese => "外观",
        Language::Romanian => "Aspect",
    }
}

pub fn section_window(language: Language) -> &'static str {
    match language {
        Language::English => "Window",
        Language::German => "Fenster",
        Language::French => "Fenêtre",
        Language::Chinese => "窗口",
        Language::Romanian => "Fereastră",
    }
}

pub fn section_content(language: Language) -> &'static str {
    match language {
        Language::English => "Content",
        Language::German => "Inhalt",
        Language::French => "Contenu",
        Language::Chinese => "内容",
        Language::Romanian => "Conținut",
    }
}

// Settings

pub fn label_opacity(language: Language) -> &'static str {
    match language {
        Language::English => "Opacity",
        Language::German => "Deckkraft",
        Language::French => "Opacité",
        Language::Chinese => "不透明度",
        Language::Romanian => "Opacitate",
    }
}

pub fn label_bg_color(language: Language) -> &'static str {
    match language {
        Language::English => "Bg color",
        Language::German => "Hintergrund",
        Language::French => "Couleur de fond",
        Language::Chinese => "背景色",
        Language::Romanian => "Culoare fundal",
    }
}

pub fn label_text_color(language: Language) -> &'static str {
    match language {
        Language::English => "Text color",
        Language::German => "Textfarbe",
        Language::French => "Couleur du texte",
        Language::Chinese => "文字颜色",
        Language::Romanian => "Culoare text",
    }
}

pub fn label_transparency(language: Language) -> &'static str {
    match language {
        Language::English => "Transparency",
        Language::German => "Transparenz",
        Language::French => "Transparence",
        Language::Chinese => "透明度",
        Language::Romanian => "Transparență",
    }
}

pub fn label_layout(language: Language) -> &'static str {
    match language {
        Language::English => "Layout",
        Language::German => "Anordnung",
        Language::French => "Disposition",
        Language::Chinese => "布局",
        Language::Romanian => "Aranjare",
    }
}

pub fn label_density(language: Language) -> &'static str {
    match language {
        Language::English => "Density",
        Language::German => "Dichte",
        Language::French => "Densité",
        Language::Chinese => "密度",
        Language::Romanian => "Densitate",
    }
}

pub fn label_text_size(language: Language) -> &'static str {
    match language {
        Language::English => "Text size",
        Language::German => "Textgröße",
        Language::French => "Taille du texte",
        Language::Chinese => "文字大小",
        Language::Romanian => "Mărime text",
    }
}

/// Matches the dashboard's own wording for this setting.
pub fn label_theme(language: Language) -> &'static str {
    match language {
        Language::English => "Theme",
        Language::German => "Darstellung",
        Language::French => "Thème",
        Language::Chinese => "主题",
        Language::Romanian => "Temă",
    }
}

pub fn label_decimals(language: Language) -> &'static str {
    match language {
        Language::English => "Decimals",
        Language::German => "Nachkommastellen",
        Language::French => "Décimales",
        Language::Chinese => "小数位",
        Language::Romanian => "Zecimale",
    }
}

pub fn label_refresh(language: Language) -> &'static str {
    match language {
        Language::English => "Refresh",
        Language::German => "Aktualisierung",
        Language::French => "Rafraîchissement",
        Language::Chinese => "刷新",
        Language::Romanian => "Reîmprospătare",
    }
}

pub fn label_show_labels(language: Language) -> &'static str {
    match language {
        Language::English => "Show labels",
        Language::German => "Beschriftungen zeigen",
        Language::French => "Afficher les libellés",
        Language::Chinese => "显示名称",
        Language::Romanian => "Afișează etichetele",
    }
}

pub fn label_show_units(language: Language) -> &'static str {
    match language {
        Language::English => "Show units",
        Language::German => "Einheiten zeigen",
        Language::French => "Afficher les unités",
        Language::Chinese => "显示单位",
        Language::Romanian => "Afișează unitățile",
    }
}

pub fn label_short_labels(language: Language) -> &'static str {
    match language {
        Language::English => "Short labels (Total→T, CPU→C…)",
        Language::German => "Kurze Beschriftungen (Total→T, CPU→C…)",
        Language::French => "Libellés courts (Total→T, CPU→C…)",
        Language::Chinese => "缩写标签（总计→总、CPU→核…）",
        Language::Romanian => "Etichete scurte (Total→T, CPU→C…)",
    }
}

pub fn label_always_on_top(language: Language) -> &'static str {
    match language {
        Language::English => "Always on top",
        Language::German => "Immer im Vordergrund",
        Language::French => "Toujours au-dessus",
        Language::Chinese => "窗口置顶",
        Language::Romanian => "Mereu deasupra",
    }
}

pub fn label_pin_click_through(language: Language) -> &'static str {
    match language {
        Language::English => "Pin makes it click-through",
        Language::German => "Anheften macht klickdurchlässig",
        Language::French => "Épingler rend transparent aux clics",
        Language::Chinese => "固定时鼠标穿透",
        Language::Romanian => "Fixarea permite clicurile prin fereastră",
    }
}

/// Shown before the pixel value, which the caller appends.
pub fn label_width(language: Language) -> &'static str {
    match language {
        Language::English => "Width",
        Language::German => "Breite",
        Language::French => "Largeur",
        Language::Chinese => "宽度",
        Language::Romanian => "Lățime",
    }
}

pub fn label_top_count(language: Language) -> &'static str {
    match language {
        Language::English => "Top count",
        Language::German => "Anzahl Einträge",
        Language::French => "Nombre d'entrées",
        Language::Chinese => "显示数量",
        Language::Romanian => "Număr de intrări",
    }
}

pub fn button_done(language: Language) -> &'static str {
    match language {
        Language::English => "Done",
        Language::German => "Fertig",
        Language::French => "Terminé",
        Language::Chinese => "完成",
        Language::Romanian => "Gata",
    }
}

pub fn button_quit_overlay(language: Language) -> &'static str {
    match language {
        Language::English => "Quit overlay",
        Language::German => "Overlay beenden",
        Language::French => "Quitter la superposition",
        Language::Chinese => "退出悬浮窗",
        Language::Romanian => "Închide suprapunerea",
    }
}

// Hints

pub fn hint_pin_unavailable(language: Language) -> &'static str {
    match language {
        Language::English => "Pin locks the position here: click-through is not available on this platform.",
        Language::German => {
            "Anheften sperrt hier die Position: Klickdurchlässigkeit gibt es auf dieser Plattform nicht."
        }
        Language::French => {
            "Épingler verrouille la position ici : la transparence aux clics n'existe pas sur cette plateforme."
        }
        Language::Chinese => "此平台仅锁定位置：不支持鼠标穿透。",
        Language::Romanian => {
            "Fixarea blochează poziția aici: trecerea clicurilor nu este disponibilă pe această platformă."
        }
    }
}

pub fn hint_opacity_layered(language: Language) -> &'static str {
    match language {
        Language::English => {
            "One alpha covers the whole window, so text fades with the card. This GPU exposes no \
             alpha-capable surface, so only the card's color (not its alpha) is adjustable."
        }
        Language::German => {
            "Ein Alpha gilt für das ganze Fenster, der Text verblasst also mit der Karte. Diese GPU \
             bietet keine alpha-fähige Oberfläche, daher lässt sich nur die Farbe der Karte \
             einstellen, nicht ihr Alpha."
        }
        Language::French => {
            "Un seul alpha couvre toute la fenêtre : le texte s'estompe avec la carte. Ce GPU \
             n'expose aucune surface compatible alpha, seule la couleur de la carte est réglable, \
             pas son alpha."
        }
        Language::Chinese => {
            "整窗共用一个 alpha，文字会随卡片一起变淡。此 GPU 不提供支持 alpha 的绘制表面，\
             因此只能调整卡片颜色，无法调整其透明度。"
        }
        Language::Romanian => {
            "Un singur alpha acoperă toată fereastra, așa că textul se estompează odată cu cardul. \
             Acest GPU nu expune o suprafață compatibilă cu alpha, deci doar culoarea cardului este \
             reglabilă, nu și alpha."
        }
    }
}

pub fn hint_opacity_surface(language: Language) -> &'static str {
    match language {
        Language::English => "Surface: card and text alphas are independent.",
        Language::German => "Oberfläche: die Alphas von Karte und Text sind unabhängig.",
        Language::French => "Surface : les alphas de la carte et du texte sont indépendants.",
        Language::Chinese => "Surface 模式：卡片与文字的透明度相互独立。",
        Language::Romanian => "Suprafață: alpha-urile cardului și ale textului sunt independente.",
    }
}

pub fn hint_opacity_off(language: Language) -> &'static str {
    match language {
        Language::English => "Transparency is off — the overlay is fully opaque.",
        Language::German => "Transparenz ist aus — das Overlay ist vollständig deckend.",
        Language::French => "La transparence est désactivée : la superposition est opaque.",
        Language::Chinese => "透明度已关闭 —— 悬浮窗完全不透明。",
        Language::Romanian => "Transparența este oprită — suprapunerea este complet opacă.",
    }
}

// Metric names

pub fn metric_name(language: Language, metric: crate::config::Metric) -> &'static str {
    use crate::config::Metric;

    match metric {
        Metric::Total => match language {
            Language::English => "Total",
            Language::German => "Gesamt",
            Language::French => "Total",
            Language::Chinese => "总计",
            Language::Romanian => "Total",
        },
        Metric::Cpu => "CPU",
        Metric::Gpu => "GPU",
        Metric::Ram => "RAM",
        Metric::Disk => match language {
            Language::English => "Disk",
            Language::German => "Festplatte",
            Language::French => "Disque",
            Language::Chinese => "磁盘",
            Language::Romanian => "Disc",
        },
        Metric::Network => match language {
            Language::English => "Net",
            Language::German => "Netz",
            Language::French => "Réseau",
            Language::Chinese => "网络",
            Language::Romanian => "Rețea",
        },
        Metric::TopApps => match language {
            Language::English => "Top",
            Language::German => "Top",
            Language::French => "Top",
            Language::Chinese => "应用",
            Language::Romanian => "Top",
        },
    }
}

/// The `Top apps` entry in the Content list, longer than the in-widget label.
pub fn metric_top_apps_setting(language: Language) -> &'static str {
    match language {
        Language::English => "Top apps",
        Language::German => "Top-Apps",
        Language::French => "Applications principales",
        Language::Chinese => "应用排行",
        Language::Romanian => "Aplicații principale",
    }
}

/// The single-glyph label used by the `Short labels` mode.
pub fn metric_short_name(language: Language, metric: crate::config::Metric) -> &'static str {
    use crate::config::Metric;

    match language {
        // One Latin initial, exactly as the `Short labels` setting describes it.
        Language::English | Language::German | Language::French | Language::Romanian => metric.short_label(),
        // A Chinese reader gets more from one character than from a Latin initial.
        Language::Chinese => match metric {
            Metric::Total => "总",
            Metric::Cpu => "核",
            Metric::Gpu => "显",
            Metric::Ram => "存",
            Metric::Disk => "盘",
            Metric::Network => "网",
            Metric::TopApps => "应用",
        },
    }
}

impl Localize for crate::config::Metric {
    fn localize(&self, language: Language) -> &'static str {
        metric_name(language, *self)
    }
}

// Setting values (pick-list options)

impl Localize for crate::config::Layout {
    fn localize(&self, language: Language) -> &'static str {
        use crate::config::Layout;

        match self {
            Layout::Vertical => match language {
                Language::English | Language::French => "Vertical",
                Language::German => "Vertikal",
                Language::Chinese => "纵向",
                Language::Romanian => "Vertical",
            },
            Layout::Horizontal => match language {
                Language::English | Language::French | Language::German => "Horizontal",
                Language::Chinese => "横向",
                Language::Romanian => "Orizontal",
            },
        }
    }
}

impl Localize for crate::config::Density {
    fn localize(&self, language: Language) -> &'static str {
        use crate::config::Density;

        match self {
            Density::Ultra => match language {
                Language::Chinese => "极紧凑",
                _ => "Ultra",
            },
            Density::Compact => match language {
                Language::German => "Kompakt",
                Language::Chinese => "紧凑",
                _ => "Compact",
            },
            Density::Normal => match language {
                Language::Chinese => "普通",
                _ => "Normal",
            },
        }
    }
}

impl Localize for crate::config::FontSize {
    fn localize(&self, language: Language) -> &'static str {
        use crate::config::FontSize;

        match self {
            FontSize::Small => match language {
                Language::English => "Small",
                Language::German => "Klein",
                Language::French => "Petite",
                Language::Chinese => "小",
                Language::Romanian => "Mic",
            },
            FontSize::Medium => match language {
                Language::English => "Medium",
                Language::German => "Mittel",
                Language::French => "Moyenne",
                Language::Chinese => "中",
                Language::Romanian => "Mediu",
            },
            FontSize::Large => match language {
                Language::English => "Large",
                Language::German => "Groß",
                Language::French => "Grande",
                Language::Chinese => "大",
                Language::Romanian => "Mare",
            },
        }
    }
}

impl Localize for crate::config::Transparency {
    fn localize(&self, language: Language) -> &'static str {
        use crate::config::Transparency;

        match self {
            Transparency::Auto => match language {
                Language::German => "Automatisch",
                Language::Chinese => "自动",
                Language::Romanian => "Automat",
                _ => "Auto",
            },
            Transparency::Layered => match language {
                Language::English => "Layered",
                Language::German => "Ebenenfenster",
                Language::French => "Fenêtre en couches",
                Language::Chinese => "分层窗口",
                Language::Romanian => "Fereastră stratificată",
            },
            Transparency::Off => match language {
                Language::English => "Off",
                Language::German => "Aus",
                Language::French => "Désactivée",
                Language::Chinese => "关闭",
                Language::Romanian => "Oprit",
            },
        }
    }
}

impl Localize for crate::theme::ThemeChoice {
    fn localize(&self, language: Language) -> &'static str {
        use crate::theme::ThemeChoice;

        match self {
            ThemeChoice::Dark => match language {
                Language::English => "Dark",
                Language::German => "Dunkel",
                Language::French => "Sombre",
                Language::Chinese => "深色",
                Language::Romanian => "Întunecat",
            },
            ThemeChoice::Light => match language {
                Language::English => "Light",
                Language::German => "Hell",
                Language::French => "Clair",
                Language::Chinese => "浅色",
                Language::Romanian => "Luminos",
            },
        }
    }
}

impl Localize for crate::config::BgColor {
    fn localize(&self, language: Language) -> &'static str {
        use crate::config::BgColor;

        match self {
            BgColor::Auto => match language {
                Language::German => "Automatisch",
                Language::Chinese => "自动",
                Language::Romanian => "Automat",
                _ => "Auto",
            },
            BgColor::Slate => match language {
                Language::English => "Slate",
                Language::German => "Schiefer",
                Language::French => "Ardoise",
                Language::Chinese => "石板",
                Language::Romanian => "Ardezie",
            },
            BgColor::Graphite => match language {
                Language::English | Language::French => "Graphite",
                Language::German => "Graphit",
                Language::Chinese => "石墨",
                Language::Romanian => "Grafit",
            },
            BgColor::Navy => match language {
                Language::English => "Navy",
                Language::German => "Marine",
                Language::French => "Marine",
                Language::Chinese => "海军蓝",
                Language::Romanian => "Bleumarin",
            },
            BgColor::Plum => match language {
                Language::English => "Plum",
                Language::German => "Pflaume",
                Language::French => "Prune",
                Language::Chinese => "梅子",
                Language::Romanian => "Prun",
            },
            BgColor::Forest => match language {
                Language::English => "Forest",
                Language::German => "Wald",
                Language::French => "Forêt",
                Language::Chinese => "森林",
                Language::Romanian => "Pădure",
            },
            BgColor::Sand => match language {
                Language::English => "Sand",
                Language::German => "Sand",
                Language::French => "Sable",
                Language::Chinese => "沙色",
                Language::Romanian => "Nisip",
            },
            BgColor::White => match language {
                Language::English => "White",
                Language::German => "Weiß",
                Language::French => "Blanc",
                Language::Chinese => "白色",
                Language::Romanian => "Alb",
            },
        }
    }
}

impl Localize for crate::config::TextColor {
    fn localize(&self, language: Language) -> &'static str {
        use crate::config::TextColor;

        match self {
            TextColor::Auto => match language {
                Language::German => "Automatisch",
                Language::Chinese => "自动",
                Language::Romanian => "Automat",
                _ => "Auto",
            },
            TextColor::White => match language {
                Language::English => "White",
                Language::German => "Weiß",
                Language::French => "Blanc",
                Language::Chinese => "白色",
                Language::Romanian => "Alb",
            },
            TextColor::Silver => match language {
                Language::English => "Silver",
                Language::German => "Silber",
                Language::French => "Argent",
                Language::Chinese => "银色",
                Language::Romanian => "Argintiu",
            },
            TextColor::Cyan => match language {
                Language::English | Language::French | Language::Romanian => "Cyan",
                Language::German => "Türkis",
                Language::Chinese => "青色",
            },
            TextColor::Green => match language {
                Language::English => "Green",
                Language::German => "Grün",
                Language::French => "Vert",
                Language::Chinese => "绿色",
                Language::Romanian => "Verde",
            },
            TextColor::Amber => match language {
                Language::English => "Amber",
                Language::German => "Bernstein",
                Language::French => "Ambre",
                Language::Chinese => "琥珀",
                Language::Romanian => "Chihlimbar",
            },
            TextColor::Red => match language {
                Language::English => "Red",
                Language::German => "Rot",
                Language::French => "Rouge",
                Language::Chinese => "红色",
                Language::Romanian => "Roșu",
            },
            TextColor::Ink => match language {
                Language::English => "Ink",
                Language::German => "Tinte",
                Language::French => "Encre",
                Language::Chinese => "墨黑",
                Language::Romanian => "Cerneală",
            },
        }
    }
}
