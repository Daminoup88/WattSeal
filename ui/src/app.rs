use std::{collections::HashMap, time::SystemTime};

use chrono::{DateTime, Local};
use common::{
    AllTimeData, Database, DatabaseEntry, DatabaseError, HardwareInfo, ProcessData, SensorData, TotalData,
    generic_name_for_table,
};
use iced::{
    Alignment, Element, Length, Padding, Subscription, Task,
    time::{Duration, every},
    widget::{
        Button, Column, Container, Row, Scrollable, Space, Text, button, center, image, mouse_area, opaque, pick_list,
        stack, text_input,
    },
};

use crate::{
    components::{header::Header, helpers::modal, sensor_state::SensorState},
    message::Message,
    pages::{Page, dashboard::DashboardPage, info::InfoPage, optimization::OptimizationPage, settings::SettingsPage},
    styles::{
        button::ButtonStyle,
        container::ContainerStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_HEADER, FONT_SIZE_SMALL, FONT_SIZE_SUBTITLE, PADDING_LARGE,
            PADDING_XLARGE, SPACING_LARGE, SPACING_MEDIUM, SPACING_SMALL,
        },
        text::TextStyle,
    },
    themes::AppTheme,
    translations::{
        self, custom_carbon_invalid, custom_carbon_placeholder, info_modal_all_time_power, info_modal_current_power,
        info_modal_description, info_modal_title, info_modal_top_consumer, info_modal_top_process, modal_close, na,
        settings_carbon_intensity, settings_language, setup_choose_carbon, setup_choose_language, setup_confirm,
        setup_welcome_title, window_title,
    },
    types::{AppLanguage, CarbonIntensity, TimeRange},
};

const FPS: u64 = 1;

pub struct App {
    current_page: Page,
    sensors: HashMap<String, SensorState>,
    hardware_info: HardwareInfo,
    dashboard_page: DashboardPage,
    info_page: InfoPage,
    optimization_page: OptimizationPage,
    settings_page: SettingsPage,
    settings_open: bool,
    info_modal_open: Option<String>,
    language: AppLanguage,
    carbon_intensity: CarbonIntensity,
    custom_carbon_input: String,
    show_setup: bool,
    header: Header,
    theme: AppTheme,
    database: Database,
    all_time_data: AllTimeData,
    tick_count: u64,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let theme = AppTheme::EcoEnergy;
        let current_page = Page::Dashboard;
        let mut database = Database::new().expect("Failed to create database");

        let (language, carbon_intensity, show_setup) = match database.load_ui_settings() {
            Ok(Some((lang_str, ci))) => {
                let lang = AppLanguage::from_code(lang_str.as_str());
                (lang, CarbonIntensity::from_g_per_kwh(ci), false)
            }
            _ => (AppLanguage::default(), CarbonIntensity::PRESETS[8], true),
        };
        let custom_carbon_input = if carbon_intensity.is_custom() {
            format!("{:.0}", carbon_intensity.g_per_kwh)
        } else {
            String::new()
        };

        let sensors = database
            .get_tables()
            .into_iter()
            .map(|table_name| {
                let display_name = generic_name_for_table(table_name.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or(table_name.clone());
                (
                    table_name.clone(),
                    SensorState::new(table_name, display_name, theme, language),
                )
            })
            .collect();
        let hardware_info = database.get_hardware_info().unwrap_or_default();
        let dashboard_page = DashboardPage;
        let all_time_data = database.get_all_time_data().unwrap_or_default();
        let task = Task::done(Message::FetchAllChartsData(TimeRange::default()));

        (
            Self {
                current_page,
                sensors,
                dashboard_page,
                header: Header::new(Page::all(), current_page),
                info_page: InfoPage::new(),
                optimization_page: OptimizationPage::new(),
                settings_page: SettingsPage::new(),
                settings_open: false,
                info_modal_open: None,
                language,
                carbon_intensity,
                custom_carbon_input,
                show_setup,
                theme,
                database,
                hardware_info,
                all_time_data,
                tick_count: 0,
            },
            task,
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let data = self.load_latest_data(1);
                for (timestamp, sensor_data) in data.iter() {
                    if let Some(sensor) = self.sensors.get_mut(sensor_data.table_name()) {
                        sensor.push_data(*timestamp, sensor_data);
                    }
                }
                self.refresh_all_time_data();
                self.tick_count += 1;
                if self.tick_count % 10 == 0 {
                    let table_name = ProcessData::table_name_static();
                    let time_range = self
                        .sensors
                        .get(table_name)
                        .map(|s| s.current_time_range().clone())
                        .unwrap_or_default();
                    let process_data = self.load_process_data(time_range);
                    if let Some(sensor) = self.sensors.get_mut(table_name) {
                        sensor.load_history_batch(&process_data);
                    }
                }
                Task::none()
            }
            Message::NavigateTo(page) => {
                self.current_page = page;
                self.header.change_page(page);
                Task::none()
            }
            Message::ChangeTheme(theme) => {
                self.theme = theme;
                for sensor in self.sensors.values_mut() {
                    sensor.update_theme(theme);
                }
                Task::none()
            }
            Message::ChangeLanguage(language) => {
                self.language = language;
                for sensor in self.sensors.values_mut() {
                    sensor.update_language(language);
                }
                self.persist_ui_settings();
                Task::none()
            }
            Message::ChangeCarbonIntensity(ci) => {
                if ci.is_custom() {
                    if !self.carbon_intensity.is_custom() {
                        self.custom_carbon_input = String::new();
                    }
                    let g = self
                        .custom_carbon_input
                        .parse::<f64>()
                        .ok()
                        .filter(|&v| v > 0.0)
                        .unwrap_or(0.0);
                    self.carbon_intensity = CarbonIntensity {
                        label: "Custom",
                        g_per_kwh: g,
                    };
                } else {
                    self.carbon_intensity = ci;
                    self.persist_ui_settings();
                }
                Task::none()
            }
            Message::CustomCarbonInput(text) => {
                self.custom_carbon_input = text.clone();
                if let Some(val) = text.parse::<f64>().ok().filter(|&v| v > 0.0) {
                    self.carbon_intensity = CarbonIntensity {
                        label: "Custom",
                        g_per_kwh: val,
                    };
                    self.persist_ui_settings();
                } else {
                    self.carbon_intensity = CarbonIntensity {
                        label: "Custom",
                        g_per_kwh: 0.0,
                    };
                }
                Task::none()
            }
            Message::OpenSettings => {
                self.settings_open = true;
                Task::none()
            }
            Message::CloseSettings => {
                self.settings_open = false;
                Task::none()
            }
            Message::OpenInfoModal(target) => {
                self.info_modal_open = Some(target);
                Task::none()
            }
            Message::CloseInfoModal => {
                self.info_modal_open = None;
                Task::none()
            }
            Message::ChangeChartMetricType(table_name, metric_type) => {
                if let Some(sensor) = self.sensors.get_mut(&table_name) {
                    sensor.set_metric_type(metric_type);
                }
                Task::none()
            }
            Message::ChangeChartTimeRange(table_name, time_range) => {
                if let Some(sensor) = self.sensors.get_mut(&table_name) {
                    return sensor.update_time_range(time_range);
                }
                Task::none()
            }
            Message::FetchChartData(table_name, time_range) => {
                let data = self.load_history(&table_name, time_range);
                if let Some(sensor) = self.sensors.get_mut(&table_name) {
                    sensor.load_history_batch(&data);
                }
                Task::none()
            }
            Message::FetchAllChartsData(time_range) => {
                let table_names = self.database.get_tables();
                for table_name in table_names {
                    let data = self.load_history(&table_name, time_range.clone());
                    if let Some(sensor) = self.sensors.get_mut(&table_name) {
                        sensor.load_history_batch(&data);
                    }
                }
                Task::none()
            }
            Message::UpdateChartData(data) => {
                for (timestamp, sensor_data) in data.iter() {
                    if let Some(sensor) = self.sensors.get_mut(sensor_data.table_name()) {
                        sensor.push_data(*timestamp, sensor_data);
                    }
                }
                Task::none()
            }
            Message::ReplaceChartData(table_name, data) => {
                if let Some(sensor) = self.sensors.get_mut(&table_name) {
                    sensor.load_history_batch(&data);
                }
                Task::none()
            }
            Message::ConfirmSetup => {
                self.show_setup = false;
                self.persist_ui_settings();
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn get_hardware_info(&mut self) -> Result<HardwareInfo, DatabaseError> {
        self.database.get_hardware_info()
    }

    fn load_latest_data(&mut self, n: i64) -> Vec<(DateTime<Local>, SensorData)> {
        from_db(self.database.select_last_n_records(n))
    }

    fn load_history(&mut self, table_name: &str, time_range: TimeRange) -> Vec<(DateTime<Local>, SensorData)> {
        if table_name == ProcessData::table_name_static() {
            return self.load_process_data(time_range);
        }
        let result = match time_range {
            TimeRange::LastMinute => self.database.select_data_in_time_range(
                table_name,
                time_range.start_time().into(),
                time_range.end_time().into(),
            ),
            _ => {
                let window = time_range.granularity_seconds();
                self.database
                    .select_last_n_seconds_average(time_range.duration_secs(), table_name, window)
            }
        };
        from_db(result)
    }

    fn load_process_data(&mut self, time_range: TimeRange) -> Vec<(DateTime<Local>, SensorData)> {
        from_db(
            self.database
                .select_top_processes_average(time_range.duration_secs(), 10),
        )
    }

    pub fn view(&self) -> Element<'_, Message, AppTheme> {
        let page_content = match self.current_page {
            Page::Dashboard => {
                self.dashboard_page
                    .view(&self.sensors, &self.all_time_data, self.language, self.carbon_intensity)
            }
            Page::Info => self.info_page.view(&self.hardware_info, self.theme, self.language),
            Page::Optimization => self.optimization_page.view(self.language),
        };

        let content: Element<'_, Message, AppTheme> = Column::new()
            .push(self.header.view(self.language))
            .push(page_content)
            .into();

        if self.settings_open {
            modal(
                content,
                self.settings_page.view(
                    self.theme,
                    self.language,
                    self.carbon_intensity,
                    &self.custom_carbon_input,
                ),
                Message::CloseSettings,
            )
        } else if self.show_setup {
            modal(content, self.setup_view(), Message::ConfirmSetup)
        } else if let Some(ref target) = self.info_modal_open {
            modal(content, self.info_modal_view(target), Message::CloseInfoModal)
        } else {
            stack![content].into()
        }
    }

    fn info_modal_view(&self, target: &str) -> Element<'_, Message, AppTheme> {
        let language = self.language;

        let title = Text::new(info_modal_title(language, target))
            .size(FONT_SIZE_HEADER)
            .font(FONT_BOLD)
            .width(Length::Fill);

        let close_button: Button<'_, Message, AppTheme> = button(Text::new(modal_close(language)).size(FONT_SIZE_BODY))
            .class(ButtonStyle::Standard)
            .on_press(Message::CloseInfoModal);

        let top_row = Row::new()
            .spacing(SPACING_LARGE)
            .align_y(Alignment::Center)
            .push(title)
            .push(close_button);

        let description = Text::new(info_modal_description(language, target)).size(FONT_SIZE_BODY);

        let mut content = Column::new()
            .spacing(SPACING_LARGE)
            .align_x(Alignment::Start)
            .push(top_row)
            .push(description);

        if let Some(sensor) = self.sensors.get(target) {
            let power = sensor.get_latest_reading().and_then(|d| d.total_power_watts());

            let power_text = power
                .map(|p| format!("{:.1} W", p))
                .unwrap_or_else(|| na(language).to_string());

            let power_row = Row::new()
                .spacing(SPACING_MEDIUM)
                .align_y(Alignment::Center)
                .push(
                    Text::new(info_modal_current_power(language))
                        .size(FONT_SIZE_BODY)
                        .class(TextStyle::Muted),
                )
                .push(
                    Text::new(power_text)
                        .size(FONT_SIZE_SUBTITLE)
                        .font(FONT_BOLD)
                        .class(TextStyle::Primary),
                );

            if target != ProcessData::table_name_static() {
                content = content.push(power_row);
            }
        }

        if let Some(&energy) = self.all_time_data.components.get(target) {
            let energy_text = format!("{:.1} Wh", energy.max(0.0));
            let all_time_row = Row::new()
                .spacing(SPACING_MEDIUM)
                .align_y(Alignment::Center)
                .push(
                    Text::new(info_modal_all_time_power(language))
                        .size(FONT_SIZE_BODY)
                        .class(TextStyle::Muted),
                )
                .push(
                    Text::new(energy_text)
                        .size(FONT_SIZE_SUBTITLE)
                        .font(FONT_BOLD)
                        .class(TextStyle::Secondary),
                );
            content = content.push(all_time_row);
        }

        if target == TotalData::table_name_static() {
            if let Some((name, power)) = self.find_top_consumer() {
                let consumer_row = Row::new()
                    .spacing(SPACING_MEDIUM)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(info_modal_top_consumer(language))
                            .size(FONT_SIZE_BODY)
                            .class(TextStyle::Muted),
                    )
                    .push(
                        Text::new(format!("{} ({:.1} W)", name, power))
                            .size(FONT_SIZE_SUBTITLE)
                            .font(FONT_BOLD)
                            .class(TextStyle::Secondary),
                    );
                content = content.push(consumer_row);
            }
        }

        if target == ProcessData::table_name_static() {
            if let Some(proc_sensor) = self.sensors.get(ProcessData::table_name_static()) {
                if let Some(top_proc) = proc_sensor.get_top_process() {
                    let mut proc_row = Row::new().spacing(SPACING_MEDIUM).align_y(Alignment::Center).push(
                        Text::new(info_modal_top_process(language))
                            .size(FONT_SIZE_BODY)
                            .class(TextStyle::Muted),
                    );

                    if let Some(icon_handle) = proc_sensor.get_process_icon(top_proc) {
                        proc_row = proc_row.push(
                            image(icon_handle)
                                .width(Length::Fixed(16.0))
                                .height(Length::Fixed(16.0)),
                        );
                    }

                    proc_row = proc_row.push(
                        Text::new(format!("{} ({:.1} W)", top_proc.app_name, top_proc.process_power_watts))
                            .size(FONT_SIZE_SUBTITLE)
                            .font(FONT_BOLD)
                            .class(TextStyle::Primary),
                    );

                    content = content.push(proc_row);
                }
            }
        }

        Container::new(Scrollable::new(content).width(Length::Fill).height(Length::Shrink))
            .width(Length::Fixed(560.0))
            .max_height(500.0)
            .padding(PADDING_XLARGE)
            .class(ContainerStyle::ModalCard)
            .into()
    }

    fn find_top_consumer(&self) -> Option<(String, f64)> {
        self.sensors
            .iter()
            .filter(|(name, _)| *name != TotalData::table_name_static() && *name != ProcessData::table_name_static())
            .filter_map(|(_, sensor)| {
                let power = sensor.get_latest_reading().and_then(|d| d.total_power_watts())?;
                Some((sensor.name().to_string(), power))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn subscription(&self) -> Subscription<Message> {
        every(Duration::from_millis(1000 / FPS)).map(|_| Message::Tick)
    }

    pub fn title(&self) -> String {
        window_title(self.language).to_string()
    }

    pub fn theme(&self) -> AppTheme {
        self.theme
    }

    fn refresh_all_time_data(&mut self) {
        if let Ok(all_time_data) = self.database.get_all_time_data() {
            self.all_time_data = all_time_data;
        }
    }

    fn persist_ui_settings(&mut self) {
        let _ = self
            .database
            .save_ui_settings(self.language.code(), self.carbon_intensity.g_per_kwh);
    }

    fn setup_view(&self) -> Element<'_, Message, AppTheme> {
        let language = self.language;

        let title = Text::new(setup_welcome_title(language))
            .size(FONT_SIZE_HEADER)
            .font(FONT_BOLD)
            .width(Length::Fill);

        let lang_label = Text::new(setup_choose_language(language)).size(FONT_SIZE_BODY);
        let lang_picker = pick_list(AppLanguage::all(), Some(self.language), Message::ChangeLanguage)
            .width(Length::Fill)
            .padding(8);

        let ci_label = Text::new(setup_choose_carbon(language)).size(FONT_SIZE_BODY);
        let ci_picker = pick_list(
            CarbonIntensity::PRESETS.to_vec(),
            Some(self.carbon_intensity),
            Message::ChangeCarbonIntensity,
        )
        .width(Length::Fill)
        .padding(8);

        let custom_input_valid = self
            .custom_carbon_input
            .parse::<f64>()
            .ok()
            .filter(|&v| v > 0.0)
            .is_some();

        let carbon_section: Element<'_, Message, AppTheme> = if self.carbon_intensity.is_custom() {
            let input = text_input(custom_carbon_placeholder(language), &self.custom_carbon_input)
                .on_input(Message::CustomCarbonInput)
                .width(Length::Fill)
                .padding(8);
            let mut col = Column::new().spacing(SPACING_SMALL).push(ci_picker).push(input);
            if !self.custom_carbon_input.is_empty() && !custom_input_valid {
                col = col.push(
                    Text::new(custom_carbon_invalid(language))
                        .size(FONT_SIZE_SMALL)
                        .class(TextStyle::Muted),
                );
            }
            col.into()
        } else {
            ci_picker.into()
        };

        let can_confirm = !self.carbon_intensity.is_custom() || custom_input_valid;
        let confirm_btn = button(Text::new(setup_confirm(language)).size(FONT_SIZE_BODY))
            .class(ButtonStyle::Standard)
            .on_press_maybe(can_confirm.then_some(Message::ConfirmSetup));

        let content = Column::new()
            .spacing(SPACING_LARGE)
            .align_x(Alignment::Start)
            .push(title)
            .push(lang_label)
            .push(lang_picker)
            .push(ci_label)
            .push(carbon_section)
            .push(confirm_btn);

        Container::new(content)
            .width(Length::Fixed(520.0))
            .padding(PADDING_XLARGE)
            .class(ContainerStyle::ModalCard)
            .into()
    }
}

fn from_db(data: Result<Vec<(SystemTime, SensorData)>, DatabaseError>) -> Vec<(DateTime<Local>, SensorData)> {
    data.unwrap_or_default()
        .into_iter()
        .map(|(ts, data)| (ts.into(), data))
        .collect()
}
