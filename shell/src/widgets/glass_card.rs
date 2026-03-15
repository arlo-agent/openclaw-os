use crate::theme;
use iced::widget::container;
use iced::{Color, Shadow, Theme, Vector};

/// Style function for a frosted-glass container
pub fn glass_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.1, 0.1, 0.18, 0.85,
        ))),
        border: iced::Border {
            color: theme::GLASS,
            width: 1.0,
            radius: theme::BORDER_RADIUS.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        text_color: Some(theme::TEXT_PRIMARY),
    }
}

/// Style function for a primary-accent glass container
pub fn glass_accent(theme: &Theme) -> container::Style {
    let mut style = glass_container(theme);
    style.border.color = Color::from_rgba(0.42, 0.39, 1.0, 0.2);
    style
}
