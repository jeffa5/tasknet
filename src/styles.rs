use seed_styles::{ColorTheme, CssColor, SeedBreakpoint, Theme};

#[derive(Hash, PartialEq, Eq, Clone)]
pub enum Color {
    Background,
    MainText,
    Primary,
    MutedPrimary,
    DarkPrimary,
    MutedSecondary,
    Secondary,
    DarkSecondary,
    Highlight,
}

impl ColorTheme for Color {}

#[allow(clippy::unreadable_literal)]
pub fn light_theme() -> Theme {
    Theme::new("light_theme").set_color(Color::Background, CssColor::Hex(0xE5E7EB))
}
