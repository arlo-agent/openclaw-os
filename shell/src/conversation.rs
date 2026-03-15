use crate::theme::{self, OpenClawPalette};
use crate::widgets::glass_card;
use iced::widget::{column, container, row, scrollable, text, text_input, Space};
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
    InputChanged(String),
    Submit,
}

pub fn view_conversation<'a>(
    messages: &'a [ChatMessage],
    input_value: &str,
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

        let row = row![Space::new(0, 0), bubble]
            .width(Length::Fill)
            .align_y(Alignment::End);

        msg_views.push(
            container(row)
                .width(Length::Fill)
                .align_x(alignment)
                .into(),
        );
    }

    let messages_scroll = scrollable(
        column(msg_views)
            .spacing(theme::GRID as u16)
            .padding(Padding::from(theme::GRID * 2.0)),
    )
    .height(Length::Fill);

    let input_bar = container(
        text_input("Type a message...", input_value)
            .on_input(ConversationMessage::InputChanged)
            .on_submit(ConversationMessage::Submit)
            .padding(Padding::from(theme::GRID * 1.5))
            .size(theme::FONT_BODY),
    )
    .padding(Padding::from(theme::GRID * 2.0))
    .width(Length::Fill);

    column![messages_scroll, input_bar]
        .spacing(0)
        .height(Length::Fill)
        .into()
}
