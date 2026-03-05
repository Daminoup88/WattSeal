pub mod dashboard;
pub mod info;
pub mod settings;

use crate::{
    translations::{page_dashboard, page_info},
    types::AppLanguage,
};

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

define_pages!(Dashboard, Info);

impl Page {
    /// Returns the translated page name for the given language.
    pub fn translated_name(&self, language: AppLanguage) -> &'static str {
        match self {
            Page::Dashboard => page_dashboard(language),
            Page::Info => page_info(language),
        }
    }
}
