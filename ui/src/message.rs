use chrono::{DateTime, Utc};
use common::SensorData;

use crate::{pages::Page, themes::AppTheme, types::TimeRange};

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    NavigateTo(Page),
    ChangeTheme(AppTheme),
    ChangeChartMetricType(String),
    ChangeChartTimeRange(String, TimeRange),
    UpdateChartData(Vec<(DateTime<Utc>, SensorData)>),
    ReplaceChartData(String, Vec<(DateTime<Utc>, SensorData)>),
    FetchChartData(String, TimeRange),
    Redraw,
    LoadChartEvents(i64),
}
