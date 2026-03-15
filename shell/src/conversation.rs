use crate::theme::{self, OpenClawPalette};
use crate::widgets::glass_card;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length, Padding};

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub from_user: bool,
    pub content: String,
    pub revealed_chars: usize,
}

impl ChatMessage {
    pub fn new(from_user: bool, content: impl Into<String>) -> Self {
        let content = content.into();
        let revealed = if from_user { content.len() } else { 0 };
        Self {
            from_user,
            content,
            revealed_chars: revealed,
        }
    }

    pub fn tick_typewriter(&mut self) {
        if self.revealed_chars < self.content.len() {
            self.revealed_chars += 1;
        }
    }

    pub fn is_fully_revealed(&self) -> bool {
        self.revealed_chars >= self.content.len()
    }

    pub fn visible_text(&self) -> &str {
        let end = self
            .content
            .char_indices()
            .nth(self.revealed_chars)
            .map(|(i, _)| i)
            .unwrap_or(self.content.len());
        &self.content[..end]
    }
}

#[derive(Debug, Clone)]
pub enum ConversationMessage {
    Back,
}

/// Render the conversation view — messages only (no input, dock handles that).
/// Includes a back button to return to ambient.
pub fn view_conversation<'a>(
    messages: &'a [ChatMessage],
    palette: &OpenClawPalette,
) -> Element<'a, ConversationMessage> {
    let p = *palette;
    let mut msg_views: Vec<Element<'a, ConversationMessage>> = Vec::new();

    for msg in messages {
        let bubble_style = if msg.from_user {
            glass_card::glass_accent_with_palette(&p)
        } else {
            glass_card::glass_container_with_palette(&p)
        };

        let alignment = if msg.from_user {
            Alignment::End
        } else {
            Alignment::Start
        };

        let label = if msg.from_user { "You" } else { "Agent" };

        let bubble = container(
            column![
                text(label)
                    .size(theme::FONT_CAPTION)
                    .color(p.text_secondary),
                text(msg.visible_text())
                    .size(theme::FONT_BODY)
                    .color(p.text_primary),
            ]
            .spacing(4),
        )
        .padding(Padding::from(theme::GRID * 1.5))
        .max_width(500)
        .style(move |_theme: &_| bubble_style);

        let aligned_row = row![Space::new(0, 0), bubble]
            .width(Length::Fill)
            .align_y(Alignment::End);

        msg_views.push(
            container(aligned_row)
                .width(Length::Fill)
                .align_x(alignment)
                .into(),
        );
    }

    // Back button at top
    let back_btn = button(
        text("<  Back").size(theme::FONT_BODY).color(p.text_secondary),
    )
    .on_press(ConversationMessage::Back)
    .padding(Padding::from([theme::GRID * 0.5, theme::GRID]))
    .style(button::text);

    let header = container(back_btn)
        .padding(Padding::from(theme::GRID));

    let messages_scroll = scrollable(
        column(msg_views)
            .spacing(theme::GRID as u16)
            .padding(Padding::from(theme::GRID * 2.0)),
    )
    .height(Length::Fill);

    column![header, messages_scroll]
        .spacing(0)
        .height(Length::Fill)
        .into()
}
