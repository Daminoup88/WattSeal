use iced::{
    Alignment, Color, Element, Length, Padding,
    widget::{Button, Column, Container, Row, Scrollable, Svg, Text, button, svg},
};

use crate::{
    message::Message,
    styles::{
        button::ButtonStyle,
        container::ContainerStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_SMALL, FONT_SIZE_SUBTITLE, PADDING_LARGE, SPACING_LARGE,
            SPACING_MEDIUM,
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
    pub field: InfoField,
    pub optional_field: Option<InfoField>,
    pub info_key: Option<String>,
}

impl InfoCard {
    pub fn new(
        icon_svg: &'static [u8],
        accent: Color,
        title: impl Into<String>,
        subtitle: impl Into<String>,
        field: InfoField,
        optional_field: Option<InfoField>,
        info_key: Option<String>,
    ) -> Self {
        Self {
            icon_svg,
            accent,
            title: title.into(),
            subtitle: subtitle.into(),
            field,
            optional_field,
            info_key,
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
    field: InfoField,
    optional_field: Option<InfoField>,
    on_info: Option<Message>,
) -> Element<'a, Message, AppTheme> {
    let icon = Svg::new(svg::Handle::from_memory(icon_svg))
        .width(22)
        .height(22)
        .class(SvgStyle::Tinted(accent));

    let icon_badge = Container::new(icon).padding(8).class(ContainerStyle::IconBadge(accent));

    let title_col = Column::new()
        .push(Text::new(title.to_owned()).size(FONT_SIZE_SUBTITLE).font(FONT_BOLD))
        .push(
            Text::new(subtitle.to_owned())
                .size(FONT_SIZE_SMALL)
                .class(TextStyle::Muted),
        );

    let mut header = Row::new()
        .spacing(SPACING_MEDIUM)
        .align_y(Alignment::Center)
        .push(icon_badge)
        .push(title_col.width(Length::Fill));

    if let Some(msg) = on_info {
        let info_btn: Button<'a, Message, AppTheme> = button(Text::new("?").size(FONT_SIZE_BODY).font(FONT_BOLD))
            .class(ButtonStyle::InfoHelp)
            .on_press(msg)
            .padding(Padding::from([2, 8]));
        header = header.push(info_btn);
    }

    let mut content = Column::new().spacing(SPACING_LARGE).push(header);

    let to_field_column = |field: InfoField| {
        Column::new()
            .spacing(2)
            .width(Length::FillPortion(1))
            .push(Text::new(field.label).size(FONT_SIZE_SMALL).class(TextStyle::Muted))
            .push(Text::new(field.value).size(FONT_SIZE_SUBTITLE).font(FONT_BOLD))
    };

    let mut fields_row = Row::new().spacing(SPACING_LARGE).push(to_field_column(field));
    if let Some(second) = optional_field {
        fields_row = fields_row.push(to_field_column(second));
    }

    content = content.push(fields_row);

    Container::new(Scrollable::new(content).width(Length::Fill).height(Length::Fill))
        .padding(PADDING_LARGE)
        .width(Length::Fill)
        .class(ContainerStyle::ComponentCard)
        .into()
}
