use crate::theme::{OpenClawPalette, BORDER_RADIUS};
use iced::widget::container;
use iced::{Color, Shadow, Theme, Vector};

/// Style function for a frosted-glass container (uses dark palette as default)
pub fn glass_container(_theme: &Theme) -> container::Style {
    glass_container_with_palette(&OpenClawPalette::dark())
}

/// Style for frosted-glass using a specific palette
pub fn glass_container_with_palette(p: &OpenClawPalette) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(p.surface_card_strong)),
        border: iced::Border {
            color: p.border_subtle,
            width: 1.0,
            radius: BORDER_RADIUS.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        text_color: Some(p.text_primary),
    }
}

/// Style function for a primary-accent glass container
pub fn glass_accent(_theme: &Theme) -> container::Style {
    glass_accent_with_palette(&OpenClawPalette::dark())
}

/// Accent glass using a specific palette
pub fn glass_accent_with_palette(p: &OpenClawPalette) -> container::Style {
    let mut style = glass_container_with_palette(p);
    style.border.color = p.border_accent;
    style
}
