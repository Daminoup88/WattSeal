use iced::{
    Alignment, Color, Element, Length,
    widget::{Column, Container, Row, Scrollable, Svg, Text, svg},
};

use crate::{
    message::Message,
    styles::{
        container::ContainerStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_SMALL, FONT_SIZE_SUBTITLE, PADDING_LARGE, SPACING_LARGE, SPACING_MEDIUM,
        },
        svg::SvgStyle,
        text::TextStyle,
    },
    themes::AppTheme,
};

pub struct InfoCard {
    pub icon_svg: &'static [u8],
    pub accent: Color,
    pub title: String,
    pub subtitle: String,
    pub fields: Vec<InfoField>,
}

impl InfoCard {
    pub fn new(
        icon_svg: &'static [u8],
        accent: Color,
        title: impl Into<String>,
        subtitle: impl Into<String>,
        fields: Vec<InfoField>,
    ) -> Self {
        Self {
            icon_svg,
            accent,
            title: title.into(),
            subtitle: subtitle.into(),
            fields,
        }
    }
}

pub struct InfoField {
    pub label: String,
    pub value: String,
}

impl InfoField {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

pub fn hardware_card<'a>(
    icon_svg: &'static [u8],
    accent: Color,
    title: &str,
    subtitle: &str,
    fields: Vec<InfoField>,
) -> Element<'a, Message, AppTheme> {
    let icon = Svg::new(svg::Handle::from_memory(icon_svg))
        .width(22)
        .height(22)
        .class(SvgStyle::Tinted(accent));

    let icon_badge = Container::new(icon).padding(8).class(ContainerStyle::IconBadge(accent));

    let header = Row::new()
        .spacing(SPACING_MEDIUM)
        .align_y(Alignment::Center)
        .push(icon_badge)
        .push(
            Column::new()
                .push(Text::new(title.to_owned()).size(FONT_SIZE_SUBTITLE).font(FONT_BOLD))
                .push(
                    Text::new(subtitle.to_owned())
                        .size(FONT_SIZE_SMALL)
                        .class(TextStyle::Muted),
                ),
        );

    let mut content = Column::new().spacing(SPACING_LARGE).push(header);

    let mut fields_content = Column::new().spacing(SPACING_LARGE);

    for chunk in fields.chunks(2) {
        let mut row = Row::new().spacing(SPACING_LARGE);
        for field in chunk {
            row = row.push(
                Column::new()
                    .spacing(2)
                    .width(Length::FillPortion(1))
                    .push(
                        Text::new(field.label.clone())
                            .size(FONT_SIZE_SMALL)
                            .class(TextStyle::Muted),
                    )
                    .push(Text::new(field.value.clone()).size(FONT_SIZE_SUBTITLE).font(FONT_BOLD)),
            );
        }
        fields_content = fields_content.push(row);
    }

    content = content.push(fields_content);

    Container::new(Scrollable::new(content).width(Length::Fill).height(Length::Fill))
        .padding(PADDING_LARGE)
        .width(Length::Fill)
        .class(ContainerStyle::ComponentCard)
        .into()
}
