use common::HardwareInfo;
use iced::{
    Color, Element, Length, Padding,
    widget::{Container, Scrollable, grid},
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
};

const CARD_MAX_WIDTH: f32 = 350.0;

pub struct InfoPage;

impl InfoPage {
    pub fn new() -> Self {
        Self
    }

    pub fn view(&self, hw: &HardwareInfo, theme: AppTheme) -> Element<'_, Message, AppTheme> {
        let pal = theme.palette();

        let mut specs: Vec<InfoCard> = Vec::new();

        specs.push(InfoCard::new(
            icons::CPU,
            pal.primary,
            "CPU".to_string(),
            "Processor Information".to_string(),
            vec![
                InfoField::new("Model", &hw.cpu.name),
                InfoField::new(
                    "Cores",
                    format!("{} cores / {} threads", hw.cpu.physical_cores, hw.cpu.logical_cores),
                ),
                InfoField::new("Base Frequency", format!("{} MHz", hw.cpu.base_frequency_mhz)),
                InfoField::new("Architecture", &hw.cpu.architecture),
            ],
        ));

        if hw.gpus.is_empty() {
            specs.push(InfoCard::new(
                icons::GPU,
                pal.danger,
                "GPU".to_string(),
                "Graphics Information".to_string(),
                vec![InfoField::new("Model", "N/A")],
            ));
        } else {
            for (i, gpu) in hw.gpus.iter().enumerate() {
                let subtitle = format!("Graphics Processor {}", i + 1);
                specs.push(InfoCard::new(
                    icons::GPU,
                    pal.danger,
                    "GPU".to_string(),
                    subtitle,
                    vec![InfoField::new("Model", gpu.as_str())],
                ));
            }
        }

        specs.push(InfoCard::new(
            icons::RAM,
            pal.warning,
            "Memory".to_string(),
            "RAM Information".to_string(),
            vec![
                InfoField::new("Total Memory", format_bytes_gb(hw.memory.total_ram_bytes)),
                InfoField::new("Swap", format_bytes_gb(hw.memory.total_swap_bytes)),
            ],
        ));

        specs.push(InfoCard::new(
            icons::SYSTEM,
            pal.success,
            "System".to_string(),
            "OS Information".to_string(),
            vec![
                InfoField::new("Operating System", &hw.system.os),
                InfoField::new("Hostname", &hw.system.hostname),
            ],
        ));

        if hw.disks.is_empty() {
            let fields = vec![InfoField::new("Status", "No disk info")];
            specs.push(InfoCard::new(
                icons::STORAGE,
                pal.primary,
                "Storage".to_string(),
                "Disk Information".to_string(),
                fields,
            ));
        } else {
            for (i, disk) in hw.disks.iter().enumerate() {
                let fields = vec![
                    InfoField::new("Disk Name", &disk.name),
                    InfoField::new("Total Space", format_bytes_gb(disk.total_bytes)),
                    InfoField::new("Used", format_bytes_gb(disk.used_bytes)),
                ];
                let subtitle = format!("Disk {}", i + 1);
                specs.push(InfoCard::new(
                    icons::STORAGE,
                    pal.primary,
                    "Storage".to_string(),
                    subtitle,
                    fields,
                ));
            }
        }

        if hw.battery.present {
            let mut f = Vec::new();
            if let Some(name) = &hw.battery.name {
                f.push(InfoField::new("Name", name));
            }
            if let Some(cap) = hw.battery.design_capacity_wh {
                f.push(InfoField::new("Design Capacity", format!("{:.1} Wh", cap)));
            }
            if let Some(cycles) = hw.battery.cycle_count {
                f.push(InfoField::new("Cycle Count", cycles.to_string()));
            }
            if f.is_empty() {
                f.push(InfoField::new("Status", "Present"));
            }
            specs.push(InfoCard::new(
                icons::BATTERY,
                pal.warning,
                "Battery".to_string(),
                "Battery Status".to_string(),
                f,
            ));
        } else {
            specs.push(InfoCard::new(
                icons::BATTERY,
                pal.warning,
                "Battery".to_string(),
                "Battery Status".to_string(),
                vec![InfoField::new("Status", "Not present")],
            ));
        }

        if hw.displays.is_empty() {
            specs.push(InfoCard::new(
                icons::DISPLAY,
                pal.success,
                "Display".to_string(),
                "Screen Information".to_string(),
                vec![InfoField::new("Status", "No display info")],
            ));
        } else {
            for d in hw.displays.iter() {
                let subtitle = if d.is_primary {
                    "Primary Display"
                } else {
                    "Secondary Display"
                };
                let mut f = vec![
                    InfoField::new("Model", &d.model),
                    InfoField::new("Resolution", &d.resolution),
                ];
                if d.refresh_rate_hz > 0 {
                    f.push(InfoField::new("Refresh Rate", format!("{} Hz", d.refresh_rate_hz)));
                }
                specs.push(InfoCard::new(
                    icons::DISPLAY,
                    pal.success,
                    "Display".to_string(),
                    subtitle.to_string(),
                    f,
                ));
            }
        }

        let cards = specs
            .into_iter()
            .map(|card| hardware_card(card.icon_svg, card.accent, &card.title, &card.subtitle, card.fields))
            .collect::<Vec<_>>();

        let grid = grid(cards).fluid(CARD_MAX_WIDTH).spacing(SPACING_LARGE);

        Scrollable::new(
            Container::new(grid)
                .width(Length::Fill)
                .padding(Padding::from(PADDING_LARGE)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .class(ScrollableStyle::Standard)
        .into()
    }
}

fn format_bytes_gb(bytes: u64) -> String {
    if bytes == 0 {
        return "N/A".to_string();
    }
    let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    format!("{:.2} GB", gb)
}
