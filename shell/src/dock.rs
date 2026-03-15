use crate::theme::{self, OpenClawPalette, ThemeMode, BORDER_RADIUS};
use crate::widgets::glass_card;
use iced::widget::{button, container, row, text, text_input};
use iced::{Alignment, Color, Element, Length, Padding, Shadow, Vector};

#[derive(Debug, Clone)]
pub enum DockMessage {
    ToggleVoice,
    InputChanged(String),
    Submit,
    ToggleTheme,
}

/// Render the redesigned pill-shaped dock with inline text input.
/// Always visible at the bottom center.
pub fn view_dock<'a>(
    input_value: &str,
    _listening: bool,
    palette: &OpenClawPalette,
    theme_mode: ThemeMode,
) -> Element<'a, DockMessage> {
    let p = *palette;

    // Mic button (left)
    let mic_btn = button(text("🎤").size(20).color(p.text_primary))
        .on_press(DockMessage::ToggleVoice)
        .padding(Padding::from([theme::GRID, theme::GRID * 1.2]))
        .style(button::text);

    // Text input (center)
    let input = text_input("Talk to your agent...", input_value)
        .on_input(DockMessage::InputChanged)
        .on_submit(DockMessage::Submit)
        .padding(Padding::from([theme::GRID, theme::GRID * 1.5]))
        .size(theme::FONT_BODY);

    // Send button — only active when there's text
    let send_btn = if input_value.is_empty() {
        button(text("→").size(18).color(p.text_muted))
            .padding(Padding::from([theme::GRID, theme::GRID * 1.2]))
            .style(button::text)
    } else {
        button(text("→").size(18).color(p.coral_bright))
            .on_press(DockMessage::Submit)
            .padding(Padding::from([theme::GRID, theme::GRID * 1.2]))
            .style(button::text)
    };

    // Theme toggle (sun/moon icon)
    let theme_icon = match theme_mode {
        ThemeMode::Dark => "☀",  // show sun (to switch to light)
        ThemeMode::Light => "🌙", // show moon (to switch to dark)
    };
    let theme_btn = button(text(theme_icon).size(18).color(p.text_secondary))
        .on_press(DockMessage::ToggleTheme)
        .padding(Padding::from([theme::GRID, theme::GRID * 1.0]))
        .style(button::text);

    let dock_content = row![
        mic_btn,
        container(input).width(Length::Fill),
        send_btn,
        theme_btn,
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
