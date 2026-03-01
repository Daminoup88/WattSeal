use common::HardwareInfo;
use iced::{
    Color, Element, Length, Padding,
    widget::{Column, Container, Row, Scrollable, Space},
};

use crate::{
    components::hardware_card::{InfoCard, InfoField, hardware_card},
    icons,
    message::Message,
    styles::{
        scrollable::ScrollableStyle,
        style_constants::{PADDING_LARGE, SPACING_LARGE},
    },
    themes::AppTheme,
    translations::{
        self, all_time, battery, battery_status, capacity, capacity_wh_cycles, capacity_wh_only, cores,
        cores_and_threads, cpu, current_power_consumption, disk, disk_information, disk_n, disk_read, disk_write,
        display, emissions, gpu, graphics_information, graphics_processor_n, hostname, memory, mode, model, na,
        na_with_cycles, name, operating_system, os_information, primary_display, processor_information, ram,
        ram_information, screen_information, secondary_display, space, storage, swap, system, total_memory,
        translate_label,
    },
    types::AppLanguage,
};

const CARD_HEIGHT: f32 = 180.0;

pub struct InfoPage;

impl InfoPage {
    pub fn new() -> Self {
        Self
    }

    pub fn view(&self, hw: &HardwareInfo, theme: AppTheme, language: AppLanguage) -> Element<'_, Message, AppTheme> {
        let pal = theme.palette();

        let mut specs: Vec<InfoCard> = Vec::new();

        specs.push(InfoCard::new(
            icons::CPU,
            pal.primary,
            cpu(language).to_string(),
            processor_information(language).to_string(),
            InfoField::new(model(language), &hw.cpu.name),
            Some(InfoField::new(
                cores(language),
                cores_and_threads(language, hw.cpu.physical_cores, hw.cpu.logical_cores),
            )),
        ));

        if hw.gpus.is_empty() {
            specs.push(InfoCard::new(
                icons::GPU,
                pal.danger,
                gpu(language).to_string(),
                graphics_information(language).to_string(),
                InfoField::new(model(language), na(language)),
                None,
            ));
        } else {
            for (i, gpu) in hw.gpus.iter().enumerate() {
                let subtitle = graphics_processor_n(language, i + 1);
                specs.push(InfoCard::new(
                    icons::GPU,
                    pal.danger,
                    translations::gpu(language).to_string(),
                    subtitle,
                    InfoField::new(model(language), gpu.as_str()),
                    None,
                ));
            }
        }

        specs.push(InfoCard::new(
            icons::RAM,
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
        ));

        specs.push(InfoCard::new(
            icons::SYSTEM,
            pal.success,
            system(language).to_string(),
            os_information(language).to_string(),
            InfoField::new(operating_system(language), &hw.system.os),
            Some(InfoField::new(hostname(language), &hw.system.hostname)),
        ));

        if hw.disks.is_empty() {
            specs.push(InfoCard::new(
                icons::STORAGE,
                pal.primary,
                storage(language).to_string(),
                disk_information(language).to_string(),
                InfoField::new(disk(language), na(language)),
                Some(InfoField::new(space(language), na(language))),
            ));
        } else {
            for (i, disk) in hw.disks.iter().enumerate() {
                let subtitle = disk_n(language, i + 1);
                specs.push(InfoCard::new(
                    icons::STORAGE,
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
                icons::BATTERY,
                pal.warning,
                battery(language).to_string(),
                battery_status(language).to_string(),
                InfoField::new(name(language), hw.battery.name.as_deref().unwrap_or(na(language))),
                Some(InfoField::new(translations::capacity(language), capacity)),
            ));
        } else {
            specs.push(InfoCard::new(
                icons::BATTERY,
                pal.warning,
                battery(language).to_string(),
                battery_status(language).to_string(),
                InfoField::new(name(language), na(language)),
                Some(InfoField::new(capacity(language), na(language))),
            ));
        }

        if hw.displays.is_empty() {
            specs.push(InfoCard::new(
                icons::DISPLAY,
                pal.success,
                display(language).to_string(),
                screen_information(language).to_string(),
                InfoField::new(model(language), na(language)),
                Some(InfoField::new(mode(language), na(language))),
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
                    icons::DISPLAY,
                    pal.success,
                    display(language).to_string(),
                    subtitle.to_string(),
                    InfoField::new(model(language), &d.model),
                    Some(InfoField::new(
                        mode(language),
                        format_display_mode(&d.resolution, d.refresh_rate_hz),
                    )),
                ));
            }
        }

        let cards = specs
            .into_iter()
            .map(|card| {
                hardware_card(
                    card.icon_svg,
                    card.accent,
                    &card.title,
                    &card.subtitle,
                    card.field,
                    card.optional_field,
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
