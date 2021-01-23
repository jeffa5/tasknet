use seed_styles::{ColorTheme, CssColor, Theme};

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
    Theme::new("light_theme")
        .set_color(Color::Background, CssColor::Hex(0xE5E7EB))
        .set_color(Color::MainText, seed_styles::seed_colors::Base::White)
        .set_color(Color::Primary, seed_styles::seed_colors::Blue::No3)
        .set_color(Color::MutedPrimary, seed_styles::seed_colors::Blue::No2)
        .set_color(Color::DarkPrimary, seed_styles::seed_colors::Blue::No4)
        .set_color(Color::Secondary, seed_styles::seed_colors::Red::No3)
        .set_color(Color::MutedSecondary, seed_styles::seed_colors::Red::No2)
        .set_color(Color::DarkSecondary, seed_styles::seed_colors::Red::No4)
        .set_color(Color::Highlight, seed_styles::seed_colors::Green::No4)
}
