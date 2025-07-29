use ratatui::style::{Color, Modifier, Style};

//pub const SELECTED_BG: Color = Color::LightGreen;
pub const SELECTED_BG: Color = Color::Magenta;
pub const DEFAULT_STYLE: Style = Style {
    fg: Some(Color::Green),
    bg: Some(Color::Black),
    underline_color: None,
    add_modifier: Modifier::REVERSED,
    sub_modifier: Modifier::REVERSED,
};
pub const BATTERY_STYLE: Style = Style {
    fg: Some(Color::Green),
    bg: Some(Color::Black),
    underline_color: None,
    add_modifier: Modifier::REVERSED,
    sub_modifier: Modifier::REVERSED,
};
