//! About modal overlay — shown when clicking the logo in the status bar.

use crate::theme::{self, OpenClawPalette, BORDER_RADIUS};
use crate::widgets::glass_card;
use iced::widget::{button, column, container, image, row, stack, text, Space};
use iced::{Alignment, Color, Element, Length, Padding};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

static LOGO_BYTES: &[u8] = include_bytes!("../assets/logo-180.png");

const DOT_SIZE: f32 = 8.0;

#[derive(Debug, Clone)]
pub enum AboutMessage {
    Close,
    OpenTerminal,
}

fn bicon(i: Bootstrap, size: f32, color: Color) -> iced::widget::Text<'static> {
    text(i.to_string())
        .font(BOOTSTRAP_FONT)
        .size(size)
        .color(color)
}

fn status_dot(color: Color) -> Element<'static, AboutMessage> {
    container(Space::new(DOT_SIZE, DOT_SIZE))
        .style(move |_: &_| container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border {
                radius: (DOT_SIZE / 2.0).into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn status_row(
    dot_color: Color,
    label: &str,
    value: &str,
    palette: &OpenClawPalette,
) -> Element<'static, AboutMessage> {
    row![
        status_dot(dot_color),
        Space::with_width(8),
        text(label.to_string())
            .size(theme::FONT_CAPTION)
            .color(palette.text_muted),
        Space::with_width(6),
        text(value.to_string())
            .size(theme::FONT_CAPTION)
            .color(palette.text_primary),
    ]
    .align_y(Alignment::Center)
    .into()
}

pub fn view_about(
    connected: bool,
    agent_active: bool,
    palette: &OpenClawPalette,
) -> Element<'static, AboutMessage> {
    let p = *palette;

    // --- Backdrop (click to close) ---
    let backdrop = button(
        container(Space::new(Length::Fill, Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(AboutMessage::Close)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_: &_, _: button::Status| button::Style {
        background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
        text_color: Color::TRANSPARENT,
        ..Default::default()
    });

    // --- Card content ---

    // Logo
    let logo = container(
        image(image::Handle::from_bytes(LOGO_BYTES))
            .width(80)
            .height(80),
    )
    .center_x(Length::Fill);

    // Title
    let title = container(
        text("OpenClaw OS")
            .size(theme::FONT_DISPLAY * 0.5)
            .color(p.coral_bright),
    )
    .center_x(Length::Fill);

    // Version
    let version = container(
        text(format!("v{}", env!("CARGO_PKG_VERSION")))
            .size(theme::FONT_BODY)
            .color(p.text_muted),
    )
    .center_x(Length::Fill);

    // Status section
    let conn_color = if connected { p.cyan_bright } else { p.coral_bright };
    let conn_text = if connected { "Connected" } else { "Disconnected" };

    let agent_color = if agent_active { p.cyan_bright } else { p.text_muted };
    let agent_text = if agent_active { "Online" } else { "Offline" };

    let os_info = format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH);

    let status_section = container(
        column![
            status_row(conn_color, "Connection:", conn_text, &p),
            Space::with_height(6),
            status_row(agent_color, "Agent:", agent_text, &p),
            Space::with_height(6),
            status_row(p.text_secondary, "OS:", &os_info, &p),
        ]
        .align_x(Alignment::Start),
    )
    .padding(Padding::from([theme::GRID * 1.5, theme::GRID * 2.0]))
    .width(Length::Fill)
    .style(move |_: &_| container::Style {
        background: Some(iced::Background::Color(p.surface_interactive)),
        border: iced::Border {
            radius: (BORDER_RADIUS * 0.75).into(),
            ..Default::default()
        },
        ..Default::default()
    });

    // Open Terminal button
    let terminal_btn = container(
        button(
            container(
                row![
                    bicon(Bootstrap::Terminal, 14.0, Color::WHITE),
                    Space::with_width(8),
                    text("Open Terminal")
                        .size(theme::FONT_BODY)
                        .color(Color::WHITE),
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([theme::GRID * 1.2, theme::GRID * 2.5]))
            .style(move |_: &_| container::Style {
                background: Some(iced::Background::Color(p.coral_bright)),
                border: iced::Border {
                    radius: BORDER_RADIUS.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .on_press(AboutMessage::OpenTerminal)
        .style(button::text),
    )
    .center_x(Length::Fill);

    // Close button (top-right)
    let close_btn = button(bicon(Bootstrap::XLg, 16.0, p.text_muted))
        .on_press(AboutMessage::Close)
        .padding(Padding::from([4, 6]))
        .style(button::text);

    let card_body = column![
        Space::with_height(theme::GRID * 2.0),
        logo,
        Space::with_height(theme::GRID * 1.5),
        title,
        Space::with_height(theme::GRID * 0.5),
        version,
        Space::with_height(theme::GRID * 2.5),
        status_section,
        Space::with_height(theme::GRID * 2.5),
        terminal_btn,
        Space::with_height(theme::GRID * 2.0),
    ]
    .align_x(Alignment::Center)
    .width(Length::Fill);

    // Close button positioned at top-right
    let close_row = container(
        container(close_btn)
            .width(Length::Fill)
            .align_x(Alignment::End),
    )
    .padding(Padding::from([theme::GRID, theme::GRID]))
    .width(Length::Fill);

    let card_with_close = column![close_row, card_body]
        .width(Length::Fill);

    // Glass card
    let card_style = glass_card::glass_container_with_palette(&p);

    let card = container(
        container(card_with_close)
            .max_width(420)
            .style(move |_: &_| card_style)
            .padding(Padding { top: 0.0, right: theme::GRID * 2.0, bottom: theme::GRID * 2.0, left: theme::GRID * 2.0 }),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fill);

    // Stack: backdrop + card
    stack![backdrop, card]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
