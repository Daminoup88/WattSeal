pub mod dashboard;
pub mod info;
pub mod optimization;
pub mod settings;

use crate::translations::{page_dashboard, page_info, page_optimization};

macro_rules! define_pages {
    ($($variant:ident),+ $(,)?) => {
        #[derive(Debug, PartialEq, Clone, Copy)]
        pub enum Page {
            $($variant,)+
        }

        impl Page {
            pub fn all() -> Vec<Self> {
                vec![$(Page::$variant,)+]
            }
        }

        impl std::fmt::Display for Page {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Page::$variant => write!(f, stringify!($variant)),)+
                }
            }
        }
    };
}

define_pages!(Dashboard, Info, Optimization);

impl Page {
    pub fn translated_name(&self, language: crate::types::AppLanguage) -> &'static str {
        match self {
            Page::Dashboard => page_dashboard(language),
            Page::Info => page_info(language),
            Page::Optimization => page_optimization(language),
        }
    }
}
