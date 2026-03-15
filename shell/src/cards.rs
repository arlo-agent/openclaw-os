use crate::theme;
use crate::widgets::glass_card;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length, Padding};

#[derive(Debug, Clone)]
pub enum CardType {
    Message,
    Alert,
    Status,
    Info,
}

impl CardType {
    pub fn accent(&self) -> Color {
        match self {
            CardType::Message => theme::PRIMARY,
            CardType::Alert => theme::ALERT,
            CardType::Status => theme::SUCCESS,
            CardType::Info => theme::TEXT_SECONDARY,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            CardType::Message => "Message",
            CardType::Alert => "Alert",
            CardType::Status => "Status",
            CardType::Info => "Info",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Card {
    pub card_type: CardType,
    pub title: String,
    pub body: String,
    pub offset_x: f32, // for slide-in animation
}

impl Card {
    pub fn new(card_type: CardType, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            card_type,
            title: title.into(),
            body: body.into(),
            offset_x: 300.0, // start offscreen right
        }
    }

    /// Animate the card sliding in (spring-like: approach 0 with decay)
    pub fn tick(&mut self) {
        // Simple spring: move 12% closer to target each frame
        self.offset_x *= 0.88;
        if self.offset_x.abs() < 0.5 {
            self.offset_x = 0.0;
        }
    }

    pub fn is_settled(&self) -> bool {
        self.offset_x == 0.0
    }
}

#[derive(Debug, Clone)]
pub enum CardMessage {
    Dismiss(usize),
}

pub fn view_cards<'a>(cards: &'a [Card]) -> Element<'a, CardMessage> {
    let card_views: Vec<Element<'a, CardMessage>> = cards
        .iter()
        .enumerate()
        .map(|(i, card)| {
            let accent_dot = container(Space::new(8, 8))
                .style(move |_theme: &_| container::Style {
                    background: Some(iced::Background::Color(card.card_type.accent())),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

            let header = row![
                accent_dot,
                Space::with_width(8),
                text(card.card_type.label())
                    .size(theme::FONT_CAPTION)
                    .color(theme::TEXT_SECONDARY),
                Space::with_width(Length::Fill),
                button(text("✕").size(14).color(theme::TEXT_SECONDARY))
                    .on_press(CardMessage::Dismiss(i))
                    .padding(4)
                    .style(button::text),
            ]
            .align_y(Alignment::Center);

            let content = column![
                header,
                text(&card.title)
                    .size(theme::FONT_HEADING)
                    .color(theme::TEXT_PRIMARY),
                text(&card.body)
                    .size(theme::FONT_BODY)
                    .color(theme::TEXT_SECONDARY),
            ]
            .spacing(theme::GRID as u16);

            container(content)
                .padding(Padding::from(theme::GRID * 2.0))
                .width(360)
                .style(glass_card::glass_container)
                .into()
        })
        .collect();

    column(card_views)
        .spacing(theme::GRID as u16)
        .into()
}
