use iced::{Element, widget::Text};

use crate::{message::Message, themes::AppTheme, translations::optimization_content, types::AppLanguage};

pub struct OptimizationPage {}

impl OptimizationPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, message: Message) {
        match message {
            _ => {
                todo!()
            }
        }
    }

    pub fn view(&self, language: AppLanguage) -> Element<'_, Message, AppTheme> {
        Text::new(optimization_content(language)).into()
    }
}
