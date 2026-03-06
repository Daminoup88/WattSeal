use iced::{
    Font,
    font::{Family, Weight},
};

// Icon font
/// Raw bytes of the embedded icon font (compiled into the binary).
pub const ICONS_BYTES: &[u8] = include_bytes!("../../../resources/wattseal-icons.ttf");

/// Raw bytes of the window icon

/// Font descriptor for the icon font.
pub const ICONS: Font = Font::with_name("wattseal-icons");

// Font sizes
pub const FONT_SIZE_SMALL: f32 = 12.0;
pub const FONT_SIZE_BODY: f32 = 14.0;
pub const FONT_SIZE_SUBTITLE: f32 = 16.0;
pub const FONT_SIZE_TITLE: f32 = 20.0;
pub const FONT_SIZE_HEADER: f32 = 24.0;
pub const FONT_SIZE_LARGE: f32 = 32.0;
pub const FONT_SIZE_HUGE: f32 = 48.0;

// Border styles
pub const BORDER_WIDTH: f32 = 1.5;
pub const BORDER_RADIUS_SMALL: f32 = 4.0;
pub const BORDER_RADIUS_MEDIUM: f32 = 8.0;
pub const BORDER_RADIUS_LARGE: f32 = 12.0;

// Spacing
pub const SPACING_SMALL: f32 = 4.0;
pub const SPACING_MEDIUM: f32 = 8.0;
pub const SPACING_LARGE: f32 = 16.0;
pub const SPACING_XLARGE: f32 = 24.0;

// Padding
pub const PADDING_SMALL: f32 = 4.0;
pub const PADDING_MEDIUM: f32 = 8.0;
pub const PADDING_LARGE: f32 = 16.0;
pub const PADDING_XLARGE: f32 = 24.0;

// Fonts
pub const FONT_BOLD: Font = Font {
    family: Family::SansSerif,
    weight: Weight::Bold,
    ..Font::DEFAULT
};

pub const FONT_MEDIUM: Font = Font {
    family: Family::SansSerif,
    weight: Weight::Medium,
    ..Font::DEFAULT
};
