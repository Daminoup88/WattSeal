use iced::widget::checkbox::{self, Catalog, Status, Style};

use crate::themes::AppTheme;

impl Catalog for AppTheme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {}

    fn style(&self, _class: &Self::Class<'_>, status: Status) -> Style {
        checkbox::primary(&self.to_iced_theme(), status)
    }
}
