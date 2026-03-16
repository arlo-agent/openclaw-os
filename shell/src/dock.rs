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
    text(i.to_string()).font(BOOTSTRAP_FONT).size(size).color(color)
}

/// Render the pill-shaped dock with inline text input and icon buttons.
pub fn view_dock<'a>(
    input_value: &str,
    listening: bool,
    palette: &OpenClawPalette,
    _theme_mode: crate::theme::ThemeMode,
) -> Element<'a, DockMessage> {
    let p = *palette;

    // Mic button
    let mic_icon = if listening { Bootstrap::MicFill } else { Bootstrap::Mic };
    let mic_color = if listening { p.coral_bright } else { p.text_primary };
    let mic_btn = button(icon(mic_icon, 18.0, mic_color))
        .on_press(DockMessage::ToggleVoice)
        .padding(Padding::from([theme::GRID, theme::GRID * 1.2]))
        .style(button::text);

    // Text input
    let input = text_input("Talk to your agent...", input_value)
        .on_input(DockMessage::InputChanged)
        .on_submit(DockMessage::Submit)
        .padding(Padding::from([theme::GRID, theme::GRID * 1.5]))
        .size(theme::FONT_BODY);

    // Send button
    let send_btn = if input_value.is_empty() {
        button(icon(Bootstrap::Send, 16.0, p.text_muted))
            .padding(Padding::from([theme::GRID, theme::GRID * 1.2]))
            .style(button::text)
    } else {
        button(icon(Bootstrap::SendFill, 16.0, p.coral_bright))
            .on_press(DockMessage::Submit)
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

    // Glass pill style
    let pill_style = {
        let base = glass_card::glass_container_with_palette(&p);
        container::Style {
            border: iced::Border {
                radius: (BORDER_RADIUS * 2.0).into(),
                ..base.border
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
