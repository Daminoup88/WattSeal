use iced::{
    Element, Length,
    widget::{
        Text, center,
        container::Container,
        mouse_area, opaque,
        rule::horizontal,
        stack,
        text::{self},
    },
};

use crate::{
    message::Message,
    styles::{
        container::ContainerStyle,
        rule::RuleStyle,
        style_constants::{FONT_BOLD, FONT_SIZE_BODY},
        text::TextStyle,
    },
    themes::AppTheme,
    translations::no_data_available,
    types::AppLanguage,
};

/// Creates a styled text element with optional bold font.
pub fn text_widget<'a>(
    text: impl text::IntoFragment<'a>,
    size: f32,
    class: TextStyle,
    width: Length,
    is_bold: bool,
) -> Element<'a, Message, AppTheme> {
    let mut text = Text::new(text).size(size).class(class).width(width);
    if is_bold {
        text = text.font(FONT_BOLD);
    }
    text.into()
}

/// Returns a muted placeholder for empty data sections.
pub fn no_data_placeholder<'a>(language: AppLanguage) -> Element<'a, Message, AppTheme> {
    Text::new(no_data_available(language))
        .size(FONT_SIZE_BODY)
        .class(TextStyle::Muted)
        .into()
}

/// Wraps content in a modal overlay with click-outside dismiss.
pub fn modal<'a>(
    background: Element<'a, Message, AppTheme>,
    content: Element<'a, Message, AppTheme>,
    on_click_outside: Message,
) -> Element<'a, Message, AppTheme> {
    let mouse_area =
        mouse_area(center(opaque(content)).class(ContainerStyle::ModalBackdrop)).on_press(on_click_outside);

    stack![background, opaque(mouse_area)].into()
}

pub fn separator<'a>() -> Element<'a, Message, AppTheme> {
    Container::new(horizontal(1).class(RuleStyle::Subtle))
        .width(Length::Fill)
        .into()
}

pub fn format_gpu_display_names(names: &[String]) -> Vec<String> {
    names.iter().map(|name| format_gpu_display_name(name)).collect()
}

fn format_gpu_display_name(name: &str) -> String {
    let Some((model, suffix)) = name.rsplit_once(" (") else {
        return name.to_string();
    };
    let Some(index) = suffix.strip_suffix(')').and_then(|index| index.parse::<u32>().ok()) else {
        return name.to_string();
    };

    if index == 0 {
        model.to_string()
    } else {
        format!("{model} {}", index + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::format_gpu_display_names;

    #[test]
    fn single_gpu_for_a_vendor_has_no_index() {
        let names = vec!["Intel UHD Graphics 730 (0)".to_string()];

        assert_eq!(format_gpu_display_names(&names), ["Intel UHD Graphics 730"]);
    }

    #[test]
    fn first_gpu_has_no_index_and_later_gpu_uses_its_one_based_index() {
        let names = vec![
            "Intel UHD Graphics 730 (0)".to_string(),
            "Intel Iris Xe Graphics (1)".to_string(),
        ];

        assert_eq!(
            format_gpu_display_names(&names),
            ["Intel UHD Graphics 730", "Intel Iris Xe Graphics 2"]
        );
    }

    #[test]
    fn vendor_indices_start_independently_and_follow_vendor_order() {
        let names = vec![
            "Intel UHD Graphics 730 (0)".to_string(),
            "AMD Radeon RX 7800 XT (0)".to_string(),
            "Intel Iris Xe Graphics (1)".to_string(),
            "NVIDIA GeForce RTX 4080 (0)".to_string(),
            "AMD Radeon RX 7700 XT (1)".to_string(),
        ];

        assert_eq!(
            format_gpu_display_names(&names),
            [
                "Intel UHD Graphics 730",
                "AMD Radeon RX 7800 XT",
                "Intel Iris Xe Graphics 2",
                "NVIDIA GeForce RTX 4080",
                "AMD Radeon RX 7700 XT 2"
            ]
        );
    }

    #[test]
    fn single_gpu_vendor_stays_unindexed_when_another_vendor_has_multiple() {
        let names = vec![
            "Intel UHD Graphics 730 (0)".to_string(),
            "NVIDIA GeForce RTX 4080 (0)".to_string(),
            "NVIDIA GeForce RTX 4070 (1)".to_string(),
        ];

        assert_eq!(
            format_gpu_display_names(&names),
            [
                "Intel UHD Graphics 730",
                "NVIDIA GeForce RTX 4080",
                "NVIDIA GeForce RTX 4070 2"
            ]
        );
    }

    #[test]
    fn formatting_does_not_mutate_the_gpu_names() {
        let names = vec![
            "Intel UHD Graphics 730 (0)".to_string(),
            "Intel Iris Xe Graphics (1)".to_string(),
        ];
        let original_names = names.clone();

        let _ = format_gpu_display_names(&names);

        assert_eq!(names, original_names);
    }
}
