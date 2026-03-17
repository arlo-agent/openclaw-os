//! macOS-style status bar — persistent across all views.
//!
//! Layout: [logo] [connection dot] [agent dot] ... [theme toggle] [bell+badge]

use crate::notifications::{NotificationMessage, NotificationState};
use crate::theme::{self, OpenClawPalette, ThemeMode};
use iced::widget::{button, container, image, row, text, Space};

// Embed the logo at compile time so it works regardless of install path
static LOGO_BYTES: &[u8] = include_bytes!("../assets/logo-32.png");
use iced::{Alignment, Color, Element, Length, Padding};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

const BAR_HEIGHT: f32 = 28.0;
const ICON_SIZE: f32 = 14.0;
const DOT_SIZE: f32 = 6.0;
const LOGO_SIZE: f32 = 16.0;

#[derive(Debug, Clone)]
pub enum StatusBarMessage {
    ToggleTheme,
    Notification(NotificationMessage),
}

fn bicon(i: Bootstrap, size: f32, color: Color) -> iced::widget::Text<'static> {
    text(i.to_string())
        .font(BOOTSTRAP_FONT)
        .size(size)
        .color(color)
}

fn status_dot(color: Color) -> Element<'static, StatusBarMessage> {
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

fn tooltip_dot(
    color: Color,
    _label: &str,
) -> Element<'static, StatusBarMessage> {
    // Simple dot for now — tooltip can be added later with iced tooltip widget
    status_dot(color)
}

pub fn view_statusbar(
    connected: bool,
    agent_active: bool,
    theme_mode: ThemeMode,
    notif_state: &NotificationState,
    palette: &OpenClawPalette,
) -> Element<'static, StatusBarMessage> {
    let p = *palette;

    // Background style — semi-transparent bar
    let bar_bg = move |_: &_| container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            p.bg_surface.r,
            p.bg_surface.g,
            p.bg_surface.b,
            0.7,
        ))),
        border: iced::Border {
            color: p.border_subtle,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    };

    // === Left section: Logo + status dots ===

    // Logo (embedded at compile time)
    let logo_handle = image::Handle::from_bytes(LOGO_BYTES);
    let logo = image(logo_handle)
        .width(LOGO_SIZE)
        .height(LOGO_SIZE);

    // Connection status
    let conn_color = if connected {
        p.cyan_bright
    } else {
        p.coral_bright
    };
    let conn_label = if connected { "Connected" } else { "Disconnected" };

    // Agent status
    let agent_color = if agent_active {
        p.cyan_bright
    } else {
        p.text_muted
    };
    let agent_label = if agent_active { "Agent online" } else { "Agent offline" };

    let left = row![
        logo,
        Space::with_width(10),
        tooltip_dot(conn_color, conn_label),
        Space::with_width(4),
        tooltip_dot(agent_color, agent_label),
    ]
    .align_y(Alignment::Center);

    // === Right section: Theme toggle + Bell ===

    // Theme toggle
    let theme_icon = match theme_mode {
        ThemeMode::Dark => Bootstrap::MoonStarsFill,
        ThemeMode::Light => Bootstrap::SunFill,
    };
    let theme_btn = button(bicon(theme_icon, ICON_SIZE, p.text_secondary))
        .on_press(StatusBarMessage::ToggleTheme)
        .padding(Padding::from([4, 6]))
        .style(button::text);

    // Bell with badge
    let unread = notif_state.unread_count();
    let bell_icon = if unread > 0 {
        Bootstrap::BellFill
    } else {
        Bootstrap::Bell
    };
    let bell_color = if unread > 0 {
        p.coral_bright
    } else {
        p.text_muted
    };

    let bell_content: Element<'static, StatusBarMessage> = if unread > 0 {
        // Bell with small badge dot overlaid
        let badge = container(Space::new(5.0, 5.0))
            .style(move |_: &_| container::Style {
                background: Some(iced::Background::Color(p.coral_bright)),
                border: iced::Border {
                    radius: 2.5.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
        row![
            bicon(bell_icon, ICON_SIZE, bell_color),
            container(badge)
                .padding(0),
        ]
        .spacing(0)
        .align_y(Alignment::Start)
        .into()
    } else {
        bicon(bell_icon, ICON_SIZE, bell_color).into()
    };

    let bell_btn = button(bell_content)
        .on_press(StatusBarMessage::Notification(
            NotificationMessage::TogglePanel,
        ))
        .padding(Padding::from([4, 6]))
        .style(button::text);

    // Date/time
    let now = chrono::Local::now();
    let datetime_str = now.format("%a %b %-d  %H:%M").to_string();
    let datetime = text(datetime_str)
        .size(ICON_SIZE - 1.0)
        .color(p.text_secondary);

    let right = row![
        theme_btn,
        Space::with_width(6),
        bell_btn,
        Space::with_width(12),
        datetime,
    ]
    .align_y(Alignment::Center);

    // === Full bar ===
    container(
        row![
            left,
            Space::with_width(Length::Fill),
            right,
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .padding(Padding::from([0.0, theme::GRID * 1.5])),
    )
    .width(Length::Fill)
    .height(BAR_HEIGHT)
    .style(bar_bg)
    .into()
}
