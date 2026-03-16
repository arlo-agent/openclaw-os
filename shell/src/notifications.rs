//! Notification system — bell icon with badge, toast popups, click-to-navigate.
//!
//! Notifications appear as toasts on the ambient screen and accumulate
//! in a dropdown. Clicking a notification jumps to the conversation view
//! scrolled to that message.

use crate::theme::{self, OpenClawPalette, BORDER_RADIUS};
use crate::widgets::glass_card;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Color, Element, Length, Padding};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};
use std::time::Instant;

// ── Data ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Notification {
    /// Index into the chat_messages vec this notification refers to
    pub message_index: usize,
    /// Short preview of the message (truncated)
    pub preview: String,
    /// When the notification was created
    pub created_at: Instant,
    /// Whether the user has seen/dismissed it
    pub read: bool,
    /// For toast animation — seconds since creation
    pub age_secs: f32,
}

impl Notification {
    pub fn new(message_index: usize, content: &str) -> Self {
        // Truncate to ~80 chars for the preview
        let preview = if content.len() > 80 {
            format!("{}...", &content[..77])
        } else {
            content.to_string()
        };
        // Strip markdown formatting for preview
        let preview = preview
            .replace("**", "")
            .replace("##", "")
            .replace("```", "")
            .replace('\n', " ");

        Self {
            message_index,
            preview,
            created_at: Instant::now(),
            read: false,
            age_secs: 0.0,
        }
    }

    pub fn tick(&mut self) {
        self.age_secs = self.created_at.elapsed().as_secs_f32();
    }

    /// Toast is visible for 5 seconds, then fades
    pub fn toast_visible(&self) -> bool {
        self.age_secs < 6.0 && !self.read
    }

    /// Toast opacity (fades out in last second)
    pub fn toast_opacity(&self) -> f32 {
        if self.age_secs < 5.0 {
            1.0
        } else if self.age_secs < 6.0 {
            1.0 - (self.age_secs - 5.0)
        } else {
            0.0
        }
    }
}

pub struct NotificationState {
    pub notifications: Vec<Notification>,
    pub panel_open: bool,
}

impl Default for NotificationState {
    fn default() -> Self {
        Self {
            notifications: Vec::new(),
            panel_open: false,
        }
    }
}

impl NotificationState {
    /// Add a new notification for an agent message
    pub fn push(&mut self, message_index: usize, content: &str) {
        self.notifications.push(Notification::new(message_index, content));
    }

    /// Number of unread notifications
    pub fn unread_count(&self) -> usize {
        self.notifications.iter().filter(|n| !n.read).count()
    }

    /// Mark all as read
    pub fn mark_all_read(&mut self) {
        for n in &mut self.notifications {
            n.read = true;
        }
    }

    /// Mark a specific notification as read
    pub fn mark_read(&mut self, index: usize) {
        if let Some(n) = self.notifications.get_mut(index) {
            n.read = true;
        }
    }

    /// Tick all notifications (for toast timing)
    pub fn tick(&mut self) {
        for n in &mut self.notifications {
            n.tick();
        }
        // Remove old read notifications (keep last 50)
        if self.notifications.len() > 50 {
            self.notifications.retain(|n| !n.read || n.age_secs < 300.0);
        }
    }

    /// Get the latest visible toast (if any)
    pub fn active_toast(&self) -> Option<&Notification> {
        self.notifications.iter().rev().find(|n| n.toast_visible())
    }
}

#[derive(Debug, Clone)]
pub enum NotificationMessage {
    TogglePanel,
    ClickNotification(usize), // message_index to scroll to
    DismissToast,
    MarkAllRead,
}

// ── Views ───────────────────────────────────────────────────────────────

fn bicon(i: Bootstrap, size: f32, color: Color) -> iced::widget::Text<'static> {
    text(i.to_string())
        .font(BOOTSTRAP_FONT)
        .size(size)
        .color(color)
}

/// Bell icon with badge — goes in the status bar
pub fn view_bell(
    unread: usize,
    palette: &OpenClawPalette,
) -> Element<'static, NotificationMessage> {
    let p = *palette;
    let icon_color = if unread > 0 {
        p.coral_bright
    } else {
        p.text_muted
    };

    let bell = bicon(
        if unread > 0 { Bootstrap::BellFill } else { Bootstrap::Bell },
        18.0,
        icon_color,
    );

    let bell_btn = if unread > 0 {
        // Bell with badge dot
        let badge = container(Space::new(8, 8))
            .style(move |_: &_| container::Style {
                background: Some(iced::Background::Color(p.coral_bright)),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        button(
            row![bell, container(badge).padding(0)]
                .spacing(0)
                .align_y(Alignment::Start),
        )
        .on_press(NotificationMessage::TogglePanel)
        .padding(Padding::from([theme::GRID * 0.5, theme::GRID]))
        .style(button::text)
    } else {
        button(bell)
            .on_press(NotificationMessage::TogglePanel)
            .padding(Padding::from([theme::GRID * 0.5, theme::GRID]))
            .style(button::text)
    };

    bell_btn.into()
}

/// Toast popup — shows briefly when a new message arrives (ambient view only)
pub fn view_toast(
    notification: &Notification,
    palette: &OpenClawPalette,
) -> Element<'static, NotificationMessage> {
    let p = *palette;
    let opacity = notification.toast_opacity();
    let preview = notification.preview.clone();
    let msg_idx = notification.message_index;

    let card_style = {
        let base = glass_card::glass_container_with_palette(&p);
        container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                p.bg_surface.r, p.bg_surface.g, p.bg_surface.b, 0.9 * opacity,
            ))),
            border: iced::Border {
                color: Color::from_rgba(
                    p.border_accent.r, p.border_accent.g, p.border_accent.b, opacity,
                ),
                width: 1.0,
                radius: BORDER_RADIUS.into(),
            },
            ..base
        }
    };

    let content = button(
        container(
            row![
                bicon(Bootstrap::ChatLeftTextFill, 16.0, Color::from_rgba(
                    p.coral_bright.r, p.coral_bright.g, p.coral_bright.b, opacity,
                )),
                Space::with_width(10),
                text(preview)
                    .size(theme::FONT_BODY)
                    .color(Color::from_rgba(
                        p.text_primary.r, p.text_primary.g, p.text_primary.b, opacity,
                    )),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .padding(Padding::from([theme::GRID * 1.5, theme::GRID * 2.0]))
        .max_width(400)
        .style(move |_: &_| card_style),
    )
    .on_press(NotificationMessage::ClickNotification(msg_idx))
    .style(button::text);

    container(content)
        .width(Length::Fill)
        .align_x(Alignment::End)
        .padding(Padding::from([theme::GRID * 2.0, theme::GRID * 3.0]))
        .into()
}

/// Notification panel dropdown — shows all recent notifications
pub fn view_panel(
    state: &NotificationState,
    palette: &OpenClawPalette,
) -> Element<'static, NotificationMessage> {
    let p = *palette;

    let card_style = glass_card::glass_container_with_palette(&p);

    // Header
    let header = row![
        bicon(Bootstrap::BellFill, 16.0, p.text_primary),
        Space::with_width(8),
        text("Notifications")
            .size(theme::FONT_BODY)
            .color(p.text_primary),
        Space::with_width(Length::Fill),
        button(
            text("Clear all")
                .size(theme::FONT_CAPTION)
                .color(p.text_muted),
        )
        .on_press(NotificationMessage::MarkAllRead)
        .padding(Padding::from([4, 8]))
        .style(button::text),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    // Notification items
    let unread: Vec<&Notification> = state.notifications.iter().rev().take(20).collect();

    let mut items: Vec<Element<'static, NotificationMessage>> = Vec::new();

    if unread.is_empty() {
        items.push(
            container(
                text("No notifications")
                    .size(theme::FONT_BODY)
                    .color(p.text_muted),
            )
            .padding(Padding::from(theme::GRID * 2.0))
            .center_x(Length::Fill)
            .into(),
        );
    } else {
        for notif in &unread {
            let msg_idx = notif.message_index;
            let preview = notif.preview.clone();
            let is_unread = !notif.read;

            let dot_color = if is_unread {
                p.coral_bright
            } else {
                Color::TRANSPARENT
            };

            let item = button(
                container(
                    row![
                        container(Space::new(6, 6))
                            .style(move |_: &_| container::Style {
                                background: Some(iced::Background::Color(dot_color)),
                                border: iced::Border {
                                    radius: 3.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                        Space::with_width(10),
                        text(preview)
                            .size(theme::FONT_CAPTION)
                            .color(if is_unread { p.text_primary } else { p.text_secondary }),
                    ]
                    .align_y(Alignment::Center)
                    .width(Length::Fill),
                )
                .padding(Padding::from([theme::GRID, theme::GRID * 1.5]))
                .width(Length::Fill),
            )
            .on_press(NotificationMessage::ClickNotification(msg_idx))
            .style(button::text)
            .width(Length::Fill);

            items.push(item.into());

            // Separator
            items.push(
                container(Space::new(Length::Fill, 1))
                    .width(Length::Fill)
                    .style(move |_: &_| container::Style {
                        background: Some(iced::Background::Color(p.border_subtle)),
                        ..Default::default()
                    })
                    .into(),
            );
        }
    }

    let panel_content = column![
        container(header)
            .padding(Padding::from([theme::GRID * 1.5, theme::GRID * 2.0]))
            .width(Length::Fill),
        container(Space::new(Length::Fill, 1))
            .width(Length::Fill)
            .style(move |_: &_| container::Style {
                background: Some(iced::Background::Color(p.border_subtle)),
                ..Default::default()
            }),
        scrollable(column(items)).height(Length::Shrink),
    ];

    container(
        container(panel_content)
            .max_width(360)
            .max_height(400)
            .style(move |_: &_| card_style),
    )
    .width(Length::Fill)
    .align_x(Alignment::End)
    .padding(Padding {
        top: theme::GRID * 5.0,
        right: theme::GRID * 3.0,
        bottom: 0.0,
        left: 0.0,
    })
    .into()
}
