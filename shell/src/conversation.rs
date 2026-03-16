use crate::theme::{self, OpenClawPalette};
use crate::widgets::glass_card;
use iced::widget::{button, column, container, markdown, row, scrollable, text, Space};
use iced::widget::text::Shaping;
use iced::{Alignment, Element, Length, Padding, Theme};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

pub struct ChatMessage {
    pub from_user: bool,
    pub content: String,
    /// Pre-parsed markdown items for agent messages
    pub markdown_items: Vec<markdown::Item>,
}

impl ChatMessage {
    pub fn new(from_user: bool, content: impl Into<String>) -> Self {
        let content = content.into();
        let markdown_items = if from_user {
            Vec::new()
        } else {
            markdown::parse(&content).collect()
        };
        Self {
            from_user,
            content,
            markdown_items,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConversationMessage {
    Back,
    LinkClicked(markdown::Url),
}

/// Render the conversation view — messages only (no input, dock handles that).
/// Includes a back button to return to ambient.
pub fn view_conversation<'a>(
    messages: &'a [ChatMessage],
    thinking: bool,
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

        let content_widget: Element<'a, ConversationMessage> = if msg.from_user {
            // User messages: plain text
            text(&msg.content)
                .size(theme::FONT_BODY)
                .color(p.text_primary)
                .shaping(Shaping::Advanced)
                .into()
        } else {
            // Agent messages: rendered markdown
            markdown::view(&msg.markdown_items, markdown::Settings::default(), markdown::Style::from_palette(iced::theme::Palette {
                background: p.bg_deep,
                text: p.text_primary,
                primary: p.coral_bright,
                success: p.cyan_bright,
                danger: p.coral_dark,
            }))
            .map(ConversationMessage::LinkClicked)
            .into()
        };

        let bubble = container(
            column![
                text(label)
                    .size(theme::FONT_CAPTION)
                    .color(p.text_secondary),
                content_widget,
            ]
            .spacing(4),
        )
        .padding(Padding::from(theme::GRID * 1.5))
        .max_width(600)
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
        row![
            text(Bootstrap::ArrowLeft.to_string()).font(BOOTSTRAP_FONT).size(16).color(p.text_secondary),
            Space::with_width(6),
            text("Back").size(theme::FONT_BODY).color(p.text_secondary),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(ConversationMessage::Back)
    .padding(Padding::from([theme::GRID * 0.5, theme::GRID]))
    .style(button::text);

    let header = container(back_btn)
        .padding(Padding::from(theme::GRID));

    // Thinking indicator
    if thinking {
        let dots = "Thinking...";
        let thinking_bubble = container(
            text(dots)
                .size(theme::FONT_BODY)
                .color(p.text_muted),
        )
        .padding(Padding::from(theme::GRID * 1.5))
        .max_width(200)
        .style(move |_theme: &_| glass_card::glass_container_with_palette(&p));

        msg_views.push(
            container(thinking_bubble)
                .width(Length::Fill)
                .align_x(Alignment::Start)
                .into(),
        );
    }

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
