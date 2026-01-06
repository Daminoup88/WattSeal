use common::Event;

use crate::{pages::Page, themes::AppTheme};

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    NavigateTo(Page),
    ChangeTheme(AppTheme),
    UpdateChartData(Event),
}
