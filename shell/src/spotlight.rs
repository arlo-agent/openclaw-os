//! Spotlight-style chat overlay — centered floating prompt, like macOS Spotlight.

use crate::conversation::ChatMessage;
use crate::theme::{self, OpenClawPalette, BORDER_RADIUS};
use crate::widgets::glass_card;
use iced::widget::{button, column, container, markdown, row, scrollable, text, text_input, Space};
use iced::widget::text::Shaping;
use iced::{Alignment, Color, Element, Length, Padding, Shadow, Vector};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

#[derive(Debug, Clone)]
pub enum ChatOverlayMessage {
    Toggle,
    InputChanged(String),
    Submit,
    LinkClicked(markdown::Url),
}

fn bicon(i: Bootstrap, size: f32, color: Color) -> iced::widget::Text<'static> {
    text(i.to_string())
        .font(BOOTSTRAP_FONT)
        .size(size)
        .color(color)
}

/// Render the full-screen spotlight overlay: backdrop + centered glass card.
pub fn view_spotlight<'a>(
    input: &str,
    messages: &'a [ChatMessage],
    thinking: bool,
    connected: bool,
    palette: &OpenClawPalette,
) -> Element<'a, ChatOverlayMessage> {
    let p = *palette;

    // --- Backdrop (click to close) ---
    let backdrop = button(
        container(Space::new(Length::Fill, Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(ChatOverlayMessage::Toggle)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_: &_, _: button::Status| button::Style {
        background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.45))),
        text_color: Color::TRANSPARENT,
        ..Default::default()
    });

    // --- Build message bubbles ---
    let has_messages = !messages.is_empty();
    let mut msg_views: Vec<Element<'a, ChatOverlayMessage>> = Vec::new();

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

        let content_widget: Element<'a, ChatOverlayMessage> = if msg.from_user {
            text(&msg.content)
                .size(theme::FONT_BODY)
                .color(p.text_primary)
                .shaping(Shaping::Advanced)
                .into()
        } else {
            markdown::view(
                &msg.markdown_items,
                markdown::Settings::default(),
                markdown::Style::from_palette(iced::theme::Palette {
                    background: p.bg_deep,
                    text: p.text_primary,
                    primary: p.coral_bright,
                    success: p.cyan_bright,
                    danger: p.coral_dark,
                }),
            )
            .map(ChatOverlayMessage::LinkClicked)
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
        .max_width(560)
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

    // Thinking indicator
    if thinking {
        let thinking_bubble = container(
            text("Thinking...")
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

    // --- Input bar ---
    let has_text = !input.is_empty();
    let is_active = has_text && connected;

    let mic_color = p.text_primary;
    let mut mic_btn = button(bicon(Bootstrap::Mic, 18.0, if connected { mic_color } else { p.text_muted }))
        .padding(Padding::from([theme::GRID, theme::GRID * 1.2]))
        .style(button::text);
    if connected {
        // Mic is placeholder for now — no voice action
        mic_btn = mic_btn;
    }

    let placeholder = if connected {
        "Talk to your agent..."
    } else {
        "Connecting to your agent..."
    };

    let mut input_field = text_input(placeholder, input)
        .padding(Padding::from([theme::GRID, theme::GRID * 1.5]))
        .size(theme::FONT_BODY);

    if connected {
        input_field = input_field
            .on_input(ChatOverlayMessage::InputChanged)
            .on_submit(ChatOverlayMessage::Submit);
    }

    let input_field = input_field.style(move |_theme, status| {
        let _focused = matches!(status, text_input::Status::Focused);
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

    let send_btn = if has_text && connected {
        button(bicon(Bootstrap::SendFill, 16.0, p.coral_bright))
            .on_press(ChatOverlayMessage::Submit)
            .padding(Padding::from([theme::GRID, theme::GRID * 1.2]))
            .style(button::text)
    } else {
        button(bicon(Bootstrap::Send, 16.0, p.text_muted))
            .padding(Padding::from([theme::GRID, theme::GRID * 1.2]))
            .style(button::text)
    };

    let input_row = row![
        mic_btn,
        container(input_field).width(Length::Fill),
        send_btn,
    ]
    .spacing(theme::GRID as u16 / 2)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let input_pill_style = {
        let base = glass_card::glass_container_with_palette(&p);
        let border_color = if is_active { p.border_accent } else { base.border.color };
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

    let input_bar = container(input_row)
        .padding(Padding::from([theme::GRID * 0.5, theme::GRID * 1.5]))
        .style(move |_theme: &_| input_pill_style)
        .width(Length::Fill);

    // --- Assemble card content ---
    let card_content: Element<'a, ChatOverlayMessage> = if has_messages || thinking {
        let messages_scroll = scrollable(
            column(msg_views)
                .spacing(theme::GRID as u16)
                .padding(Padding::from(theme::GRID)),
        )
        .height(Length::Shrink);

        // Wrap scrollable in a max-height container
        let scroll_area = container(messages_scroll)
            .max_height(400)
            .width(Length::Fill);

        column![
            scroll_area,
            Space::with_height(theme::GRID),
            input_bar,
        ]
        .width(Length::Fill)
        .into()
    } else {
        // Input-only mode (no messages yet)
        input_bar.into()
    };

    // --- Glass card wrapper ---
    let card_style = glass_card::glass_container_with_palette(&p);

    let card = container(
        container(card_content)
            .max_width(640)
            .padding(Padding::from(theme::GRID * 1.5))
            .style(move |_: &_| container::Style {
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                    offset: Vector::new(0.0, 8.0),
                    blur_radius: 32.0,
                },
                ..card_style
            }),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fill);

    // Stack backdrop + card
    iced::widget::stack![backdrop, card]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Render the floating action button (FAB) for toggling the chat overlay.
/// Positioned bottom-right.
pub fn view_fab(palette: &OpenClawPalette) -> Element<'static, ChatOverlayMessage> {
    let p = *palette;

    let fab = button(
        container(bicon(Bootstrap::ChatDots, 22.0, Color::WHITE))
            .padding(Padding::from(theme::GRID * 1.5))
            .style(move |_: &_| container::Style {
                background: Some(iced::Background::Color(p.coral_bright)),
                border: iced::Border {
                    radius: 28.0.into(),
                    ..Default::default()
                },
                shadow: Shadow {
                    color: Color::from_rgba(
                        p.coral_bright.r,
                        p.coral_bright.g,
                        p.coral_bright.b,
                        0.4,
                    ),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 16.0,
                },
                ..Default::default()
            }),
    )
    .on_press(ChatOverlayMessage::Toggle)
    .style(button::text);

    container(fab)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::End)
        .align_y(Alignment::End)
        .padding(Padding::from(theme::GRID * 3.0))
        .into()
}

/// Render a small logo button for the top-left corner (opens About).
pub fn view_logo_button(palette: &OpenClawPalette) -> Element<'static, LogoAction> {
    let p = *palette;

    static LOGO_BYTES: &[u8] = include_bytes!("../assets/logo-32.png");
    let logo_handle = iced::widget::image::Handle::from_bytes(LOGO_BYTES);
    let logo_img = iced::widget::image(logo_handle).width(24).height(24);

    let btn = button(logo_img)
        .on_press(LogoAction::ShowAbout)
        .padding(Padding::from(theme::GRID))
        .style(move |_: &_, _: button::Status| button::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                p.bg_surface.r,
                p.bg_surface.g,
                p.bg_surface.b,
                0.5,
            ))),
            border: iced::Border {
                radius: 12.0.into(),
                color: p.border_subtle,
                width: 1.0,
            },
            text_color: Color::TRANSPARENT,
            ..Default::default()
        });

    container(btn)
        .width(Length::Fill)
        .height(Length::Shrink)
        .align_x(Alignment::Start)
        .padding(Padding::from(theme::GRID * 2.0))
        .into()
}

#[derive(Debug, Clone)]
pub enum LogoAction {
    ShowAbout,
}
