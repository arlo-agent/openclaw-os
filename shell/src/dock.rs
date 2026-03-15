use crate::theme::{self, OpenClawPalette};
use crate::widgets::glass_card;
use iced::widget::{button, container, row, text, Space};
use iced::{Alignment, Element, Length, Padding};

#[derive(Debug, Clone)]
pub enum DockMessage {
    ToggleVoice,
    ToggleText,
}

pub fn view_dock<'a>(show: bool, palette: &OpenClawPalette) -> Element<'a, DockMessage> {
    if !show {
        return Space::new(0, 0).into();
    }

    let p = *palette;

    let voice_btn = button(text("🎤").size(20).color(p.text_primary))
        .on_press(DockMessage::ToggleVoice)
        .padding(Padding::from(theme::GRID * 1.5))
        .style(button::text);

    let text_btn = button(text("⌨").size(20).color(p.text_primary))
        .on_press(DockMessage::ToggleText)
        .padding(Padding::from(theme::GRID * 1.5))
        .style(button::text);

    let dock_content = row![voice_btn, Space::with_width(theme::GRID * 2.0), text_btn]
        .align_y(Alignment::Center);

    let dock_style = glass_card::glass_container_with_palette(&p);
    container(
        container(dock_content)
            .padding(Padding::from([theme::GRID, theme::GRID * 3.0]))
            .style(move |_theme: &_| dock_style),
    )
    .center_x(Length::Fill)
    .padding(Padding::from(theme::GRID * 2.0))
    .into()
}
