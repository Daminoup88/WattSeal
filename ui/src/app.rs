use std::{collections::HashMap, time::SystemTime};

use chrono::{DateTime, Local};
use common::{
    AllTimeData, Database, DatabaseEntry, DatabaseError, HardwareInfo, ProcessData, SensorData, TotalData, UiSettings,
    generic_name_for_table,
};
use iced::{
    Alignment, Element, Length, Subscription, Task, event,
    time::{Duration, every},
    widget::{Button, Column, Container, Row, Scrollable, Text, button, image, pick_list, stack, text_input},
    window,
};

use crate::{
    components::{footer::Footer, header::Header, helpers::modal, sensor_state::SensorState},
    message::Message,
    pages::{Page, dashboard::DashboardPage, info::InfoPage, settings::SettingsPage},
    styles::{
        button::ButtonStyle,
        container::ContainerStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_HEADER, FONT_SIZE_SMALL, FONT_SIZE_SUBTITLE, PADDING_XLARGE,
            SPACING_LARGE, SPACING_MEDIUM, SPACING_SMALL,
        },
        text::TextStyle,
    },
    themes::AppTheme,
    translations::{
        app_name, carbon_info_measured, close_dialog_description, close_dialog_title, close_everything, close_ui_only,
        custom_carbon_invalid, custom_carbon_placeholder, custom_kwh_cost_placeholder, info_modal_all_time_power,
        info_modal_all_time_top_consumer, info_modal_current_power, info_modal_current_top_consumer,
        info_modal_description, info_modal_title, info_modal_top_process, kwh_cost_invalid, modal_close, na,
        setup_choose_carbon, setup_choose_electricity, setup_choose_language, setup_confirm, setup_welcome_title,
    },
    types::{AppLanguage, CarbonIntensity, ElectricityCost, TimeRange},
};

const FPS: u64 = 1;

/// Main application state managing pages, sensors, and database.
pub struct App {
    current_page: Page,
    sensors: HashMap<String, SensorState>,
    hardware_info: HardwareInfo,
    dashboard_page: DashboardPage,
    info_page: InfoPage,
    settings_page: SettingsPage,
    settings_open: bool,
    info_modal_open: Option<String>,
    language: AppLanguage,
    carbon_intensity: CarbonIntensity,
    custom_carbon_input: String,
    electricity_cost: ElectricityCost,
    custom_kwh_cost_input: String,
    show_setup: bool,
    header: Header,
    footer: Footer,
    theme: AppTheme,
    database: Database,
    all_time_data: AllTimeData,
    tick_count: u64,
    show_close_dialog: bool,
}

impl App {
    /// Initializes the app, opens the database, and loads settings.
    pub fn new() -> (Self, Task<Message>) {
        let current_page = Page::Dashboard;
        let mut database = Database::new().expect("Failed to create database");

        let (language, carbon_intensity, theme, electricity_cost, show_setup) = match database.load_ui_settings() {
            Ok(Some(s)) => {
                let lang = AppLanguage::from_code(&s.language);
                let ci = CarbonIntensity::from_g_per_kwh(s.carbon_g_per_kwh);
                let theme = AppTheme::from_name(&s.theme);
                let ec = ElectricityCost::from_usd_per_kwh(s.kwh_cost_eur);
                (lang, ci, theme, ec, false)
            }
            _ => (
                AppLanguage::default(),
                CarbonIntensity::PRESETS[8],
                AppTheme::default(),
                ElectricityCost::PRESETS[8],
                true,
            ),
        };
        let custom_carbon_input = if carbon_intensity.is_custom() {
            format!("{:.0}", carbon_intensity.g_per_kwh)
        } else {
            String::new()
        };
        let custom_kwh_cost_input = if electricity_cost.is_custom() {
            format!("{:.4}", electricity_cost.usd_per_kwh)
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
                footer: Footer,
                info_page: InfoPage::new(),
                settings_page: SettingsPage::new(),
                settings_open: false,
                info_modal_open: None,
                language,
                carbon_intensity,
                custom_carbon_input,
                electricity_cost,
                custom_kwh_cost_input,
                show_setup,
                theme,
                database,
                hardware_info,
                all_time_data,
                tick_count: 0,
                show_close_dialog: false,
            },
            task,
        )
    }

    /// Handles incoming messages and returns follow-up tasks.
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
                self.persist_ui_settings();
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
            Message::ChangeElectricityCost(ec) => {
                if ec.is_custom() {
                    if !self.electricity_cost.is_custom() {
                        self.custom_kwh_cost_input = String::new();
                    }
                    let v = self
                        .custom_kwh_cost_input
                        .parse::<f64>()
                        .ok()
                        .filter(|&v| v >= 0.0)
                        .unwrap_or(0.0);
                    self.electricity_cost = ElectricityCost {
                        label: "Custom",
                        usd_per_kwh: v,
                    };
                } else {
                    self.electricity_cost = ec;
                    self.persist_ui_settings();
                }
                Task::none()
            }
            Message::CustomKwhCostInput(text) => {
                self.custom_kwh_cost_input = text.clone();
                if let Some(val) = text.parse::<f64>().ok().filter(|&v| v >= 0.0) {
                    self.electricity_cost = ElectricityCost {
                        label: "Custom",
                        usd_per_kwh: val,
                    };
                    self.persist_ui_settings();
                } else {
                    self.electricity_cost = ElectricityCost {
                        label: "Custom",
                        usd_per_kwh: 0.0,
                    };
                }
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
            Message::CloseRequested => {
                self.show_close_dialog = true;
                Task::none()
            }
            Message::CloseUIOnly => iced::exit(),
            Message::CloseAll => {
                std::process::exit(common::EXIT_CODE_SHUTDOWN_ALL);
            }
            Message::OpenUrl(url) => {
                open::that(&url).ok();
                Task::none()
            }
            _ => Task::none(),
        }
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
                    .select_last_n_seconds_average(time_range.seconds(), table_name, window)
            }
        };
        let mut data = from_db(result);

        // In energy mode, convert average watts → Wh per window
        if time_range.is_energy_mode() {
            let factor = time_range.power_scale_factor();
            for (_, sensor_data) in &mut data {
                sensor_data.scale_power(factor);
            }
        }

        data
    }

    fn load_process_data(&mut self, time_range: TimeRange) -> Vec<(DateTime<Local>, SensorData)> {
        from_db(
            self.database
                .select_top_processes_average(time_range.seconds(), 10, time_range.is_energy_mode()),
        )
    }

    /// Builds the root UI element tree.
    pub fn view(&self) -> Element<'_, Message, AppTheme> {
        let page_content = match self.current_page {
            Page::Dashboard => self.dashboard_page.view(
                &self.sensors,
                &self.all_time_data,
                self.language,
                self.carbon_intensity,
                self.electricity_cost.usd_per_kwh,
            ),
            Page::Info => self.info_page.view(&self.hardware_info, self.theme, self.language),
        };

        let content: Element<'_, Message, AppTheme> = Column::new()
            .push(self.header.view(self.language))
            .push(page_content)
            .push(self.footer.view(self.language))
            .into();

        if self.settings_open {
            modal(
                content,
                self.settings_page.view(
                    self.theme,
                    self.language,
                    self.carbon_intensity,
                    &self.custom_carbon_input,
                    self.electricity_cost,
                    &self.custom_kwh_cost_input,
                ),
                Message::CloseSettings,
            )
        } else if self.show_setup {
            modal(content, self.setup_view(), Message::ConfirmSetup)
        } else if self.show_close_dialog {
            modal(content, self.close_dialog_view(), Message::CloseUIOnly)
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
            if let Some((name, power)) = self.find_current_top_consumer() {
                let consumer_row = Row::new()
                    .spacing(SPACING_MEDIUM)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(info_modal_current_top_consumer(language))
                            .size(FONT_SIZE_BODY)
                            .class(TextStyle::Muted),
                    )
                    .push(
                        Text::new(format!("{} ({:.1} W)", name, power))
                            .size(FONT_SIZE_SUBTITLE)
                            .font(FONT_BOLD)
                            .class(TextStyle::Primary),
                    );
                content = content.push(consumer_row);
            }

            if let Some((name, energy)) = self.find_all_time_top_consumer() {
                let consumer_row = Row::new()
                    .spacing(SPACING_MEDIUM)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(info_modal_all_time_top_consumer(language))
                            .size(FONT_SIZE_BODY)
                            .class(TextStyle::Muted),
                    )
                    .push(
                        Text::new(format!("{} ({:.1} Wh)", name, energy))
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
                        Text::new(format!(
                            "{} ({:.1} {})",
                            top_proc.app_name,
                            top_proc.process_power_watts,
                            proc_sensor.current_time_range().power_unit()
                        ))
                        .size(FONT_SIZE_SUBTITLE)
                        .font(FONT_BOLD)
                        .class(TextStyle::Primary),
                    );

                    content = content.push(proc_row);
                }
            }
        }

        if target == "carbon_emissions" {
            let total_energy_wh = self
                .all_time_data
                .components
                .get(TotalData::table_name_static())
                .copied()
                .unwrap_or(0.0);
            let measured_g = (total_energy_wh / 1000.0) * self.carbon_intensity.g_per_kwh;

            let make_co2_row = |label: &'static str, value: String, style: TextStyle| {
                Row::new()
                    .spacing(SPACING_MEDIUM)
                    .align_y(Alignment::Center)
                    .push(Text::new(label).size(FONT_SIZE_BODY).class(TextStyle::Muted))
                    .push(Text::new(value).size(FONT_SIZE_SUBTITLE).font(FONT_BOLD).class(style))
            };

            let measured_row = make_co2_row(
                carbon_info_measured(language),
                format!("{:.1} g CO₂", measured_g.max(0.0)),
                TextStyle::Tertiary,
            );

            content = content.push(measured_row);
        }

        Container::new(Scrollable::new(content).width(Length::Fill).height(Length::Shrink))
            .width(Length::Fixed(560.0))
            .max_height(500.0)
            .padding(PADDING_XLARGE)
            .class(ContainerStyle::ModalCard)
            .into()
    }

    fn find_current_top_consumer(&self) -> Option<(String, f64)> {
        self.sensors
            .iter()
            .filter(|(name, _)| *name != TotalData::table_name_static() && *name != ProcessData::table_name_static())
            .filter_map(|(_, sensor)| {
                let power = sensor.get_latest_reading().and_then(|d| d.total_power_watts())?;
                Some((sensor.name().to_string(), power))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    fn find_all_time_top_consumer(&self) -> Option<(String, f64)> {
        self.all_time_data
            .components
            .iter()
            .filter(|(name, _)| *name != TotalData::table_name_static() && *name != ProcessData::table_name_static())
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, energy)| (info_modal_title(self.language, name), *energy))
    }

    /// Produces a 1 Hz tick and listens for window-close requests.
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            every(Duration::from_millis(1000 / FPS)).map(|_| Message::Tick),
            event::listen_with(|evt, _status, _id| {
                if let iced::Event::Window(window::Event::CloseRequested) = evt {
                    Some(Message::CloseRequested)
                } else {
                    None
                }
            }),
        ])
    }

    /// Returns the localized window title.
    pub fn title(&self) -> String {
        app_name(self.language).to_string()
    }

    /// Returns the active theme.
    pub fn theme(&self) -> AppTheme {
        self.theme
    }

    fn refresh_all_time_data(&mut self) {
        if let Ok(all_time_data) = self.database.get_all_time_data() {
            self.all_time_data = all_time_data;
        }
    }

    fn persist_ui_settings(&mut self) {
        let settings = UiSettings {
            language: self.language.code().to_string(),
            carbon_g_per_kwh: self.carbon_intensity.g_per_kwh,
            kwh_cost_eur: self.electricity_cost.usd_per_kwh,
            theme: self.theme.name().to_string(),
        };
        let _ = self.database.save_ui_settings(&settings);
    }

    fn close_dialog_view(&self) -> Element<'_, Message, AppTheme> {
        let language = self.language;

        let title = Text::new(close_dialog_title(language))
            .size(FONT_SIZE_HEADER)
            .font(FONT_BOLD)
            .width(Length::Fill);

        let description = Text::new(close_dialog_description(language)).size(FONT_SIZE_BODY);

        let ui_only_btn = button(Text::new(close_ui_only(language)).size(FONT_SIZE_BODY))
            .class(ButtonStyle::Standard)
            .on_press(Message::CloseUIOnly);

        let close_all_btn = button(Text::new(close_everything(language)).size(FONT_SIZE_BODY))
            .class(ButtonStyle::Standard)
            .on_press(Message::CloseAll);

        let buttons = Row::new().spacing(SPACING_MEDIUM).push(ui_only_btn).push(close_all_btn);

        let content = Column::new()
            .spacing(SPACING_LARGE)
            .align_x(Alignment::Start)
            .push(title)
            .push(description)
            .align_x(Alignment::Center)
            .push(buttons);

        Container::new(content)
            .width(Length::Fixed(520.0))
            .padding(PADDING_XLARGE)
            .class(ContainerStyle::ModalCard)
            .into()
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

        let custom_kwh_valid = self
            .custom_kwh_cost_input
            .parse::<f64>()
            .ok()
            .filter(|&v| v >= 0.0)
            .is_some();

        let ec_label = Text::new(setup_choose_electricity(language)).size(FONT_SIZE_BODY);
        let ec_picker = pick_list(
            ElectricityCost::PRESETS.to_vec(),
            Some(self.electricity_cost),
            Message::ChangeElectricityCost,
        )
        .width(Length::Fill)
        .padding(8);

        let electricity_section: Element<'_, Message, AppTheme> = if self.electricity_cost.is_custom() {
            let input = text_input(custom_kwh_cost_placeholder(language), &self.custom_kwh_cost_input)
                .on_input(Message::CustomKwhCostInput)
                .width(Length::Fill)
                .padding(8);
            let mut col = Column::new().spacing(SPACING_SMALL).push(ec_picker).push(
                Row::new()
                    .spacing(4)
                    .align_y(Alignment::Center)
                    .push(input)
                    .push(Text::new("$/kWh").size(FONT_SIZE_SMALL).class(TextStyle::Muted)),
            );
            if !self.custom_kwh_cost_input.is_empty() && !custom_kwh_valid {
                col = col.push(
                    Text::new(kwh_cost_invalid(language))
                        .size(FONT_SIZE_SMALL)
                        .class(TextStyle::Muted),
                );
            }
            col.into()
        } else {
            ec_picker.into()
        };

        let can_confirm = (!self.carbon_intensity.is_custom() || custom_input_valid)
            && (!self.electricity_cost.is_custom() || custom_kwh_valid);
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
            .push(ec_label)
            .push(electricity_section)
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
