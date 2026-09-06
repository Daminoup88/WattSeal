use common::MetricKind;

use crate::{
    pages::Page,
    themes::AppTheme,
    types::{AppLanguage, CarbonIntensity, Currency, ElectricityCost, ProcessLimit, SensorRecord, TimeRange},
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
    ChangeCustomCurrency(Currency),
    ToggleLaunchOnStartup(bool),
    OpenSettings,
    CloseSettings,
    ChangeChartMetricType(String, MetricKind),
    ChangeChartTimeRange(String, TimeRange),
    ChangeProcessLimit(ProcessLimit),
    ChangeCustomProcessLimit(String),
    UpdateChartData(Vec<SensorRecord>),
    ReplaceChartData(String, Vec<SensorRecord>),
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
