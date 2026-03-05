use chrono::{DateTime, Local};
use common::{MetricType, SensorData};

use crate::{
    pages::Page,
    themes::AppTheme,
    types::{AppLanguage, CarbonIntensity, ElectricityCost, TimeRange},
};

/// UI event variants dispatched by user actions and background tasks.
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    NavigateTo(Page),
    ChangeTheme(AppTheme),
    ChangeLanguage(AppLanguage),
    ChangeCarbonIntensity(CarbonIntensity),
    CustomCarbonInput(String),
    ChangeElectricityCost(ElectricityCost),
    CustomKwhCostInput(String),
    OpenSettings,
    CloseSettings,
    ChangeChartMetricType(String, MetricType),
    ChangeChartTimeRange(String, TimeRange),
    UpdateChartData(Vec<(DateTime<Local>, SensorData)>),
    ReplaceChartData(String, Vec<(DateTime<Local>, SensorData)>),
    FetchChartData(String, TimeRange),
    FetchAllChartsData(TimeRange),
    Redraw,
    LoadChartEvents(i64),
    OpenInfoModal(String),
    CloseInfoModal,
    ConfirmSetup,
    CloseRequested,
    CloseUIOnly,
    CloseAll,
    OpenUrl(String),
}
