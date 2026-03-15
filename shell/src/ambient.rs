use crate::theme::{self, OpenClawPalette};
use chrono::Local;
use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Element, Length};

/// Render the ambient clock display
pub fn view_clock<'a, M: 'a>(palette: &OpenClawPalette) -> Element<'a, M> {
    let now = Local::now();

    let time_str = now.format("%H:%M").to_string();
    let date_str = now.format("%A, %B %-d").to_string();

    let clock = column![
        text(time_str)
            .size(theme::FONT_DISPLAY)
            .color(palette.text_primary),
        text(date_str)
            .size(theme::FONT_HEADING)
            .color(palette.text_secondary),
    ]
    .align_x(Alignment::Center)
    .spacing(theme::GRID as u16);

    container(clock)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Small status indicator dots (connectivity, agent status, etc.)
pub fn view_status_dots<'a, M: 'a>(
    connected: bool,
    agent_active: bool,
    palette: &OpenClawPalette,
) -> Element<'a, M> {
    let dot = |color: iced::Color| {
        container(Space::new(6, 6))
            .style(move |_theme: &_| container::Style {
                background: Some(iced::Background::Color(color)),
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
    };

    let conn_color = if connected {
        palette.cyan_bright
    } else {
        palette.coral_bright
    };
    let agent_color = if agent_active {
        palette.coral_mid
    } else {
        palette.text_muted
    };

    row![dot(conn_color), Space::with_width(6), dot(agent_color)].into()
}
