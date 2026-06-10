use common::{CPUData, DatabaseEntry, DiskData, GPUData, HardwareInfo, RamData};
use iced::{
    Element, Length, Padding,
    widget::{Column, Container, Row, Scrollable},
};

use crate::{
    components::hardware_card::{InfoCard, InfoField, hardware_card},
    icons::Icon,
    message::Message,
    styles::{
        scrollable::ScrollableStyle,
        style_constants::{PADDING_LARGE, SPACING_LARGE},
    },
    themes::AppTheme,
    translations::{
        self, battery, battery_status, capacity, capacity_wh_cycles, capacity_wh_only, cores, cores_and_threads, cpu,
        disk, disk_information, disk_n, display, gpu, graphics_information, graphics_processor_n, hostname, memory,
        mode, model, na, na_with_cycles, name, operating_system, os_information, primary_display,
        processor_information, ram_information, screen_information, secondary_display, space, storage, swap, system,
        total_memory,
    },
    types::AppLanguage,
};

const CARD_HEIGHT: f32 = 180.0;

/// Hardware information page listing system components.
pub struct InfoPage;

impl InfoPage {
    pub fn new() -> Self {
        Self
    }

    pub fn view(&self, hw: &HardwareInfo, theme: AppTheme, language: AppLanguage) -> Element<'_, Message, AppTheme> {
        let pal = theme.palette();

        let mut specs: Vec<InfoCard> = Vec::new();

        specs.push(InfoCard::new(
            Icon::CPU,
            pal.primary,
            cpu(language).to_string(),
            processor_information(language).to_string(),
            InfoField::new(model(language), &hw.cpu.name),
            Some(InfoField::new(
                cores(language),
                cores_and_threads(language, hw.cpu.physical_cores, hw.cpu.logical_cores),
            )),
            Some(CPUData::table_name_static().to_string()),
        ));

        if hw.gpus.is_empty() {
            specs.push(InfoCard::new(
                Icon::GPU,
                pal.danger,
                gpu(language).to_string(),
                graphics_information(language).to_string(),
                InfoField::new(model(language), na(language)),
                None,
                Some(GPUData::table_name_static().to_string()),
            ));
        } else {
            for (i, gpu) in hw.gpus.iter().enumerate() {
                let subtitle = graphics_processor_n(language, i + 1);
                specs.push(InfoCard::new(
                    Icon::GPU,
                    pal.danger,
                    translations::gpu(language).to_string(),
                    subtitle,
                    InfoField::new(model(language), gpu.as_str()),
                    None,
                    Some(GPUData::table_name_static().to_string()),
                ));
            }
        }

        specs.push(InfoCard::new(
            Icon::RAM,
            pal.warning,
            memory(language).to_string(),
            ram_information(language).to_string(),
            InfoField::new(
                total_memory(language),
                format_bytes_gb(hw.memory.total_ram_bytes, language),
            ),
            Some(InfoField::new(
                swap(language),
                format_bytes_gb(hw.memory.total_swap_bytes, language),
            )),
            Some(RamData::table_name_static().to_string()),
        ));

        specs.push(InfoCard::new(
            Icon::System,
            pal.success,
            system(language).to_string(),
            os_information(language).to_string(),
            InfoField::new(operating_system(language), &hw.system.os),
            Some(InfoField::new(hostname(language), &hw.system.hostname)),
            Some("system".to_string()),
        ));

        if hw.disks.is_empty() {
            specs.push(InfoCard::new(
                Icon::Storage,
                pal.primary,
                storage(language).to_string(),
                disk_information(language).to_string(),
                InfoField::new(disk(language), na(language)),
                Some(InfoField::new(space(language), na(language))),
                Some(DiskData::table_name_static().to_string()),
            ));
        } else {
            for (i, disk) in hw.disks.iter().enumerate() {
                let subtitle = disk_n(language, i + 1);
                specs.push(InfoCard::new(
                    Icon::Storage,
                    pal.primary,
                    storage(language).to_string(),
                    subtitle,
                    InfoField::new(
                        translations::disk(language),
                        format!("{} ({})", disk.mount_point, disk.name),
                    ),
                    Some(InfoField::new(
                        space(language),
                        format!(
                            "{} / {}",
                            format_bytes_gb(disk.used_bytes, language),
                            format_bytes_gb(disk.total_bytes, language)
                        ),
                    )),
                    Some(DiskData::table_name_static().to_string()),
                ));
            }
        }

        if hw.battery.present {
            let capacity = match (hw.battery.design_capacity_wh, hw.battery.cycle_count) {
                (Some(cap), Some(cycles)) => capacity_wh_cycles(language, cap, cycles),
                (Some(cap), None) => capacity_wh_only(language, cap),
                (None, Some(cycles)) => na_with_cycles(language, cycles),
                (None, None) => na(language).to_string(),
            };

            specs.push(InfoCard::new(
                Icon::Battery,
                pal.warning,
                battery(language).to_string(),
                battery_status(language).to_string(),
                InfoField::new(name(language), hw.battery.name.as_deref().unwrap_or(na(language))),
                Some(InfoField::new(translations::capacity(language), capacity)),
                Some("battery".to_string()),
            ));
        } else {
            specs.push(InfoCard::new(
                Icon::Battery,
                pal.warning,
                battery(language).to_string(),
                battery_status(language).to_string(),
                InfoField::new(name(language), na(language)),
                Some(InfoField::new(capacity(language), na(language))),
                Some("battery".to_string()),
            ));
        }

        if hw.displays.is_empty() {
            specs.push(InfoCard::new(
                Icon::Display,
                pal.success,
                display(language).to_string(),
                screen_information(language).to_string(),
                InfoField::new(model(language), na(language)),
                Some(InfoField::new(mode(language), na(language))),
                Some("display".to_string()),
            ));
        } else {
            let mut displays = hw.displays.iter().collect::<Vec<_>>();
            displays.sort_by_key(|d| !d.is_primary);

            for d in displays {
                let subtitle = if d.is_primary {
                    primary_display(language)
                } else {
                    secondary_display(language)
                };
                specs.push(InfoCard::new(
                    Icon::Display,
                    pal.success,
                    display(language).to_string(),
                    subtitle.to_string(),
                    InfoField::new(model(language), &d.model),
                    Some(InfoField::new(
                        mode(language),
                        format_display_mode(&d.resolution, d.refresh_rate_hz),
                    )),
                    Some("display".to_string()),
                ));
            }
        }

        let cards = specs
            .into_iter()
            .map(|card| {
                let on_info = card.info_key.map(|key| Message::OpenInfoModal(key));
                hardware_card(
                    card.icon,
                    card.accent,
                    &card.title,
                    &card.subtitle,
                    card.field,
                    card.optional_field,
                    on_info,
                )
            })
            .collect::<Vec<_>>();

        let mut card_rows = Column::new().spacing(SPACING_LARGE);

        let mut row = Row::new().spacing(SPACING_LARGE);

        for (i, card) in cards.into_iter().enumerate() {
            if i % 3 == 0 && i != 0 {
                card_rows = card_rows.push(row);
                row = Row::new().spacing(SPACING_LARGE);
            }

            row = row.push(
                Container::new(card)
                    .width(Length::FillPortion(1))
                    .height(Length::Fixed(CARD_HEIGHT)),
            );
        }
        card_rows = card_rows.push(row);

        Scrollable::new(
            Container::new(card_rows)
                .width(Length::Fill)
                .padding(Padding::from(PADDING_LARGE)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .class(ScrollableStyle::Standard)
        .into()
    }
}

fn format_bytes_gb(bytes: u64, language: AppLanguage) -> String {
    if bytes == 0 {
        return na(language).to_string();
    }
    let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    format!("{:.2} GB", gb)
}

fn format_display_mode(resolution: &str, refresh_rate_hz: u32) -> String {
    if refresh_rate_hz > 0 {
        format!("{} @ {} Hz", resolution, refresh_rate_hz)
    } else {
        resolution.to_string()
    }
}
