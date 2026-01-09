use chrono::{DateTime, Utc};
use common::SensorData;

use crate::{pages::Page, themes::AppTheme};

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    NavigateTo(Page),
    ChangeTheme(AppTheme),
    UpdateChartData(Vec<(DateTime<Utc>, SensorData)>),
    Redraw,
    LoadChartEvents(i64),
}
