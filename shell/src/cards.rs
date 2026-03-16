use crate::theme::{self, OpenClawPalette};
use crate::widgets::glass_card;
use iced::widget::{button, column, container, row, text, Space};
use iced::widget::text::Shaping;
use iced::{Alignment, Color, Element, Length, Padding};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

#[derive(Debug, Clone)]
pub enum CardType {
    Message,
    Alert,
    Status,
    Info,
}

impl CardType {
    pub fn accent(&self, palette: &OpenClawPalette) -> Color {
        match self {
            CardType::Message => palette.coral_bright,
            CardType::Alert => palette.coral_mid,
            CardType::Status => palette.cyan_bright,
            CardType::Info => palette.text_secondary,
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
    pub offset_x: f32,
}

impl Card {
    pub fn new(card_type: CardType, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            card_type,
            title: title.into(),
            body: body.into(),
            offset_x: 300.0,
        }
    }

    pub fn tick(&mut self) {
        self.offset_x *= 0.88;
        if self.offset_x.abs() < 0.5 {
            self.offset_x = 0.0;
        }
    }

    pub fn _is_settled(&self) -> bool {
        self.offset_x == 0.0
    }
}

#[derive(Debug, Clone)]
pub enum CardMessage {
    Dismiss(usize),
}

pub fn view_cards<'a>(cards: &'a [Card], palette: &OpenClawPalette) -> Element<'a, CardMessage> {
    let p = *palette;
    let card_views: Vec<Element<'a, CardMessage>> = cards
        .iter()
        .enumerate()
        .map(|(i, card)| {
            let accent_color = card.card_type.accent(&p);
            let accent_dot = container(Space::new(8, 8)).style(move |_theme: &_| {
                container::Style {
                    background: Some(iced::Background::Color(accent_color)),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });

            let header = row![
                accent_dot,
                Space::with_width(8),
                text(card.card_type.label())
                    .size(theme::FONT_CAPTION)
                    .color(p.text_secondary),
                Space::with_width(Length::Fill),
                button(
                    text(Bootstrap::XLg.to_string())
                        .font(BOOTSTRAP_FONT)
                        .size(12)
                        .color(p.text_muted),
                )
                .on_press(CardMessage::Dismiss(i))
                .padding(4)
                .style(button::text),
            ]
            .align_y(Alignment::Center);

            let content = column![
                header,
                text(&card.title)
                    .size(theme::FONT_HEADING)
                    .color(p.text_primary),
                text(&card.body)
                    .size(theme::FONT_BODY)
                    .color(p.text_secondary)
                    .shaping(Shaping::Advanced),
            ]
            .spacing(theme::GRID as u16);

            let card_style = glass_card::glass_container_with_palette(&p);
            container(content)
                .padding(Padding::from(theme::GRID * 2.0))
                .width(360)
                .style(move |_theme: &_| card_style)
                .into()
        })
        .collect();

    column(card_views).spacing(theme::GRID as u16).into()
}
