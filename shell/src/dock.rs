use crate::theme::{self, OpenClawPalette, BORDER_RADIUS};
use crate::widgets::glass_card;
use iced::widget::{button, container, row, text, text_input};
use iced::{Alignment, Color, Element, Length, Padding, Shadow, Vector};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

#[derive(Debug, Clone)]
pub enum DockMessage {
    ToggleVoice,
    InputChanged(String),
    Submit,
    ToggleTheme,
}

/// Helper to create a Bootstrap icon text widget
fn icon(i: Bootstrap, size: f32, color: Color) -> iced::widget::Text<'static> {
    text(i.to_string())
        .font(BOOTSTRAP_FONT)
        .size(size)
        .color(color)
}

/// Render the pill-shaped dock with inline text input and icon buttons.
/// Input is borderless; dock pill gets an accent border when input has text.
pub fn view_dock<'a>(
    input_value: &str,
    listening: bool,
    palette: &OpenClawPalette,
    _theme_mode: crate::theme::ThemeMode,
    _focused: bool,
    connected: bool,
) -> Element<'a, DockMessage> {
    let p = *palette;
    let has_text = !input_value.is_empty();
    let is_active = has_text && connected;

    // Mic button
    let mic_icon = if listening {
        Bootstrap::MicFill
    } else {
        Bootstrap::Mic
    };
    let mic_color = if listening {
        p.coral_bright
    } else {
        p.text_primary
    };
    let mut mic_btn = button(icon(mic_icon, 18.0, if connected { mic_color } else { p.text_muted }))
        .padding(Padding::from([theme::GRID, theme::GRID * 1.2]))
        .style(button::text);
    if connected {
        mic_btn = mic_btn.on_press(DockMessage::ToggleVoice);
    }

    // Borderless text input — disabled while connecting
    let placeholder = if connected {
        "Talk to your agent..."
    } else {
        "Connecting to your agent..."
    };

    let mut input = text_input(placeholder, input_value)
        .padding(Padding::from([theme::GRID, theme::GRID * 1.5]))
        .size(theme::FONT_BODY);

    if connected {
        input = input
            .on_input(DockMessage::InputChanged)
            .on_submit(DockMessage::Submit);
    }

    let input = input.style(move |_theme, status| {
            // Borderless input that blends with dock background
            let focused = matches!(status, text_input::Status::Focused);
            text_input::Style {
                background: iced::Background::Color(Color::TRANSPARENT),
                border: iced::Border {
                    width: 0.0,
                    radius: 0.0.into(),
                    color: Color::TRANSPARENT,
                },
                icon: p.text_muted,
                placeholder: p.text_muted,
                value: p.text_primary,
                selection: Color::from_rgba(
                    p.coral_bright.r,
                    p.coral_bright.g,
                    p.coral_bright.b,
                    0.3,
                ),
            }
        });

    // Send button
    let send_btn = if has_text && connected {
        button(icon(Bootstrap::SendFill, 16.0, p.coral_bright))
            .on_press(DockMessage::Submit)
            .padding(Padding::from([theme::GRID, theme::GRID * 1.2]))
            .style(button::text)
    } else {
        button(icon(Bootstrap::Send, 16.0, p.text_muted))
            .padding(Padding::from([theme::GRID, theme::GRID * 1.2]))
            .style(button::text)
    };

    let dock_content = row![
        mic_btn,
        container(input).width(Length::Fill),
        send_btn,
    ]
    .spacing(theme::GRID as u16 / 2)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    // Glass pill style — accent border when active/focused
    let pill_style = {
        let base = glass_card::glass_container_with_palette(&p);
        let border_color = if is_active {
            p.border_accent
        } else {
            base.border.color
        };
        let border_width = if is_active { 1.5 } else { 1.0 };

        container::Style {
            border: iced::Border {
                radius: (BORDER_RADIUS * 2.0).into(),
                color: border_color,
                width: border_width,
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: Vector::new(0.0, 6.0),
                blur_radius: 24.0,
            },
            ..base
        }
    };

    container(
        container(dock_content)
            .padding(Padding::from([theme::GRID * 0.5, theme::GRID * 1.5]))
            .style(move |_theme: &_| pill_style)
            .max_width(640),
    )
    .center_x(Length::Fill)
    .padding(Padding {
        top: 0.0,
        right: theme::GRID * 3.0,
        bottom: theme::GRID * 2.0,
        left: theme::GRID * 3.0,
    })
    .into()
}
