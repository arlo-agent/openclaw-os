use crate::theme;
use chrono::Local;
use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Element, Length};

/// Render the ambient clock display
pub fn view_clock<'a, M: 'a>() -> Element<'a, M> {
    let now = Local::now();

    let time_str = now.format("%H:%M").to_string();
    let date_str = now.format("%A, %B %-d").to_string();

    let clock = column![
        text(time_str)
            .size(theme::FONT_DISPLAY)
            .color(theme::TEXT_PRIMARY),
        text(date_str)
            .size(theme::FONT_HEADING)
            .color(theme::TEXT_SECONDARY),
    ]
    .align_x(Alignment::Center)
    .spacing(theme::GRID as u16);

    container(clock)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Small status indicator dots (connectivity, agent status, etc.)
pub fn view_status_dots<'a, M: 'a>(connected: bool, agent_active: bool) -> Element<'a, M> {
    let dot = |color| {
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
        theme::SUCCESS
    } else {
        theme::ERROR
    };
    let agent_color = if agent_active {
        theme::PRIMARY
    } else {
        theme::TEXT_SECONDARY
    };

    row![dot(conn_color), Space::with_width(6), dot(agent_color)]
        .into()
}
