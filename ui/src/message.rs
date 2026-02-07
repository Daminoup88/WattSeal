use chrono::{DateTime, Local};
use common::SensorData;

use crate::{pages::Page, themes::AppTheme, types::TimeRange};

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    NavigateTo(Page),
    ChangeTheme(AppTheme),
    ChangeChartMetricType(String),
    ChangeChartTimeRange(String, TimeRange),
    UpdateChartData(Vec<(DateTime<Local>, SensorData)>),
    ReplaceChartData(String, Vec<(DateTime<Local>, SensorData)>),
    FetchChartData(String, TimeRange),
    Redraw,
    LoadChartEvents(i64),
}
