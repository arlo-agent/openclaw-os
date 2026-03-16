//! Messaging channel configuration for the onboarding wizard.
//!
//! Each channel can be expanded to show its configuration fields.

use crate::theme::{self, OpenClawPalette, BORDER_RADIUS};
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Color, Element, Length, Padding};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

// ── Data types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagingChannel {
    Telegram,
    WhatsApp,
    Discord,
    Signal,
    Slack,
}

impl MessagingChannel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Telegram => "Telegram",
            Self::WhatsApp => "WhatsApp",
            Self::Discord => "Discord",
            Self::Signal => "Signal",
            Self::Slack => "Slack",
        }
    }

    pub fn icon(&self) -> Bootstrap {
        match self {
            Self::Telegram => Bootstrap::Telegram,
            Self::WhatsApp => Bootstrap::Whatsapp,
            Self::Discord => Bootstrap::Discord,
            Self::Signal => Bootstrap::ChatDots,
            Self::Slack => Bootstrap::Slack,
        }
    }

    pub fn config_type(&self) -> &'static str {
        match self {
            Self::Telegram => "telegram",
            Self::WhatsApp => "whatsapp",
            Self::Discord => "discord",
            Self::Signal => "signal",
            Self::Slack => "slack",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChannelStatus {
    NotConfigured,
    Configuring,
    Configured(String), // summary text
}

#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub channel: MessagingChannel,
    pub status: ChannelStatus,
    // Telegram
    pub telegram_bot_token: String,
    pub telegram_chat_id: String,
    // Discord
    pub discord_bot_token: String,
    pub discord_guild_id: String,
    // Signal
    pub signal_phone: String,
    // Slack
    pub slack_bot_token: String,
    pub slack_channel: String,
}

impl ChannelConfig {
    fn new(channel: MessagingChannel) -> Self {
        Self {
            channel,
            status: ChannelStatus::NotConfigured,
            telegram_bot_token: String::new(),
            telegram_chat_id: String::new(),
            discord_bot_token: String::new(),
            discord_guild_id: String::new(),
            signal_phone: String::new(),
            slack_bot_token: String::new(),
            slack_channel: String::new(),
        }
    }
}

pub struct MessagingState {
    pub channels: Vec<ChannelConfig>,
    pub expanded: Option<MessagingChannel>,
}

impl Default for MessagingState {
    fn default() -> Self {
        Self {
            channels: vec![
                ChannelConfig::new(MessagingChannel::Telegram),
                ChannelConfig::new(MessagingChannel::WhatsApp),
                ChannelConfig::new(MessagingChannel::Discord),
                ChannelConfig::new(MessagingChannel::Signal),
                ChannelConfig::new(MessagingChannel::Slack),
            ],
            expanded: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MessagingMessage {
    Expand(MessagingChannel),
    Collapse,
    SetTelegramToken(String),
    SetTelegramChatId(String),
    SetDiscordToken(String),
    SetDiscordGuildId(String),
    SetSignalPhone(String),
    SetSlackToken(String),
    SetSlackChannel(String),
    SaveChannel(MessagingChannel),
}

// ── Update ──────────────────────────────────────────────────────────────

pub fn update_messaging(state: &mut MessagingState, msg: MessagingMessage) {
    match msg {
        MessagingMessage::Expand(ch) => {
            // Set channel to Configuring
            if let Some(cfg) = state.channels.iter_mut().find(|c| c.channel == ch) {
                if cfg.status == ChannelStatus::NotConfigured {
                    cfg.status = ChannelStatus::Configuring;
                }
            }
            state.expanded = Some(ch);
        }
        MessagingMessage::Collapse => {
            // Revert if not saved
            if let Some(expanded) = state.expanded {
                if let Some(cfg) = state
                    .channels
                    .iter_mut()
                    .find(|c| c.channel == expanded)
                {
                    if cfg.status == ChannelStatus::Configuring {
                        cfg.status = ChannelStatus::NotConfigured;
                    }
                }
            }
            state.expanded = None;
        }
        MessagingMessage::SetTelegramToken(v) => {
            if let Some(cfg) = state
                .channels
                .iter_mut()
                .find(|c| c.channel == MessagingChannel::Telegram)
            {
                cfg.telegram_bot_token = v;
            }
        }
        MessagingMessage::SetTelegramChatId(v) => {
            if let Some(cfg) = state
                .channels
                .iter_mut()
                .find(|c| c.channel == MessagingChannel::Telegram)
            {
                cfg.telegram_chat_id = v;
            }
        }
        MessagingMessage::SetDiscordToken(v) => {
            if let Some(cfg) = state
                .channels
                .iter_mut()
                .find(|c| c.channel == MessagingChannel::Discord)
            {
                cfg.discord_bot_token = v;
            }
        }
        MessagingMessage::SetDiscordGuildId(v) => {
            if let Some(cfg) = state
                .channels
                .iter_mut()
                .find(|c| c.channel == MessagingChannel::Discord)
            {
                cfg.discord_guild_id = v;
            }
        }
        MessagingMessage::SetSignalPhone(v) => {
            if let Some(cfg) = state
                .channels
                .iter_mut()
                .find(|c| c.channel == MessagingChannel::Signal)
            {
                cfg.signal_phone = v;
            }
        }
        MessagingMessage::SetSlackToken(v) => {
            if let Some(cfg) = state
                .channels
                .iter_mut()
                .find(|c| c.channel == MessagingChannel::Slack)
            {
                cfg.slack_bot_token = v;
            }
        }
        MessagingMessage::SetSlackChannel(v) => {
            if let Some(cfg) = state
                .channels
                .iter_mut()
                .find(|c| c.channel == MessagingChannel::Slack)
            {
                cfg.slack_channel = v;
            }
        }
        MessagingMessage::SaveChannel(ch) => {
            if let Some(cfg) = state.channels.iter_mut().find(|c| c.channel == ch) {
                let summary = match ch {
                    MessagingChannel::Telegram => {
                        if cfg.telegram_bot_token.is_empty() {
                            return;
                        }
                        format!("Bot: ...{}", &cfg.telegram_bot_token[cfg.telegram_bot_token.len().saturating_sub(6)..])
                    }
                    MessagingChannel::WhatsApp => "QR setup pending".to_string(),
                    MessagingChannel::Discord => {
                        if cfg.discord_bot_token.is_empty() {
                            return;
                        }
                        "Bot connected".to_string()
                    }
                    MessagingChannel::Signal => {
                        if cfg.signal_phone.is_empty() {
                            return;
                        }
                        format!("Phone: {}", cfg.signal_phone)
                    }
                    MessagingChannel::Slack => {
                        if cfg.slack_bot_token.is_empty() {
                            return;
                        }
                        format!("Channel: {}", cfg.slack_channel)
                    }
                };
                cfg.status = ChannelStatus::Configured(summary);
            }
            state.expanded = None;
        }
    }
}

// ── Config generation ───────────────────────────────────────────────────

pub fn generate_messaging_config(state: &MessagingState) -> Vec<serde_json::Value> {
    let mut configs = Vec::new();

    for cfg in &state.channels {
        if let ChannelStatus::Configured(_) = &cfg.status {
            let val = match cfg.channel {
                MessagingChannel::Telegram => serde_json::json!({
                    "type": "telegram",
                    "botToken": cfg.telegram_bot_token,
                    "chatId": cfg.telegram_chat_id
                }),
                MessagingChannel::WhatsApp => serde_json::json!({
                    "type": "whatsapp",
                    "note": "QR pairing required after setup"
                }),
                MessagingChannel::Discord => serde_json::json!({
                    "type": "discord",
                    "botToken": cfg.discord_bot_token,
                    "guildId": cfg.discord_guild_id
                }),
                MessagingChannel::Signal => serde_json::json!({
                    "type": "signal",
                    "phone": cfg.signal_phone
                }),
                MessagingChannel::Slack => serde_json::json!({
                    "type": "slack",
                    "botToken": cfg.slack_bot_token,
                    "channel": cfg.slack_channel
                }),
            };
            configs.push(val);
        }
    }

    configs
}

// ── View ────────────────────────────────────────────────────────────────

fn bicon(i: Bootstrap, size: f32, color: Color) -> iced::widget::Text<'static> {
    text(i.to_string())
        .font(BOOTSTRAP_FONT)
        .size(size)
        .color(color)
}

pub fn view_messaging_setup<'a>(
    state: &'a MessagingState,
    palette: &OpenClawPalette,
) -> Element<'a, MessagingMessage> {
    let p = *palette;
    let mut items: Vec<Element<'a, MessagingMessage>> = Vec::new();

    for cfg in &state.channels {
        let ch = cfg.channel;
        let is_expanded = state.expanded == Some(ch);

        // Channel header row
        let status_dot_color = match &cfg.status {
            ChannelStatus::Configured(_) => p.cyan_bright,
            ChannelStatus::Configuring => p.coral_mid,
            ChannelStatus::NotConfigured => p.text_muted,
        };

        let right_side: Element<'static, MessagingMessage> = match &cfg.status {
            ChannelStatus::Configured(summary) => {
                row![
                    text(format!("✓ {}", summary))
                        .size(theme::FONT_CAPTION)
                        .color(p.cyan_bright),
                ]
                .into()
            }
            _ if is_expanded => Space::with_width(0).into(),
            _ => {
                button(
                    text("Configure")
                        .size(theme::FONT_CAPTION)
                        .color(p.text_muted),
                )
                .on_press(MessagingMessage::Expand(ch))
                .padding(Padding::from([4, 8]))
                .style(button::text)
                .into()
            }
        };

        let header = container(
            row![
                // Status dot
                container(Space::new(6, 6)).style(move |_: &_| container::Style {
                    background: Some(iced::Background::Color(status_dot_color)),
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                Space::with_width(10),
                bicon(ch.icon(), 18.0, p.text_primary),
                Space::with_width(10),
                text(ch.label())
                    .size(theme::FONT_BODY)
                    .color(p.text_primary),
                Space::with_width(Length::Fill),
                right_side,
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .padding(Padding::from([theme::GRID, theme::GRID * 1.5]))
        .width(Length::Fill);

        items.push(header.into());

        // Expanded config panel
        if is_expanded {
            let config_panel: Element<'a, MessagingMessage> =
                view_channel_config(cfg, &p);
            items.push(config_panel);
        }

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

    column(items).width(Length::Fill).into()
}

fn view_channel_config<'a>(
    cfg: &'a ChannelConfig,
    p: &OpenClawPalette,
) -> Element<'a, MessagingMessage> {
    let pp = *p;
    let ch = cfg.channel;

    let fields: Element<'a, MessagingMessage> = match ch {
        MessagingChannel::Telegram => {
            column![
                text("Bot Token")
                    .size(theme::FONT_CAPTION)
                    .color(pp.text_muted),
                text_input("123456:ABC-DEF...", &cfg.telegram_bot_token)
                    .on_input(MessagingMessage::SetTelegramToken)
                    .padding(Padding::from([theme::GRID * 0.8, theme::GRID]))
                    .size(theme::FONT_BODY),
                Space::with_height(theme::GRID),
                text("Chat ID")
                    .size(theme::FONT_CAPTION)
                    .color(pp.text_muted),
                text_input("-100123456789", &cfg.telegram_chat_id)
                    .on_input(MessagingMessage::SetTelegramChatId)
                    .padding(Padding::from([theme::GRID * 0.8, theme::GRID]))
                    .size(theme::FONT_BODY),
                Space::with_height(theme::GRID * 0.5),
                text("Get a token from @BotFather on Telegram")
                    .size(theme::FONT_CAPTION)
                    .color(pp.text_muted),
            ]
            .spacing(4)
            .into()
        }
        MessagingChannel::WhatsApp => {
            column![
                text("WhatsApp connects via QR code.")
                    .size(theme::FONT_BODY)
                    .color(pp.text_secondary),
                Space::with_height(theme::GRID * 0.5),
                text("After setup, a QR code will appear for you to scan with your phone.")
                    .size(theme::FONT_CAPTION)
                    .color(pp.text_muted),
            ]
            .spacing(4)
            .into()
        }
        MessagingChannel::Discord => {
            column![
                text("Bot Token")
                    .size(theme::FONT_CAPTION)
                    .color(pp.text_muted),
                text_input("MTIz...", &cfg.discord_bot_token)
                    .on_input(MessagingMessage::SetDiscordToken)
                    .padding(Padding::from([theme::GRID * 0.8, theme::GRID]))
                    .size(theme::FONT_BODY),
                Space::with_height(theme::GRID),
                text("Guild (Server) ID")
                    .size(theme::FONT_CAPTION)
                    .color(pp.text_muted),
                text_input("123456789012345678", &cfg.discord_guild_id)
                    .on_input(MessagingMessage::SetDiscordGuildId)
                    .padding(Padding::from([theme::GRID * 0.8, theme::GRID]))
                    .size(theme::FONT_BODY),
            ]
            .spacing(4)
            .into()
        }
        MessagingChannel::Signal => {
            column![
                text("Phone Number")
                    .size(theme::FONT_CAPTION)
                    .color(pp.text_muted),
                text_input("+1234567890", &cfg.signal_phone)
                    .on_input(MessagingMessage::SetSignalPhone)
                    .padding(Padding::from([theme::GRID * 0.8, theme::GRID]))
                    .size(theme::FONT_BODY),
            ]
            .spacing(4)
            .into()
        }
        MessagingChannel::Slack => {
            column![
                text("Bot Token")
                    .size(theme::FONT_CAPTION)
                    .color(pp.text_muted),
                text_input("xoxb-...", &cfg.slack_bot_token)
                    .on_input(MessagingMessage::SetSlackToken)
                    .padding(Padding::from([theme::GRID * 0.8, theme::GRID]))
                    .size(theme::FONT_BODY),
                Space::with_height(theme::GRID),
                text("Channel ID")
                    .size(theme::FONT_CAPTION)
                    .color(pp.text_muted),
                text_input("C0123456789", &cfg.slack_channel)
                    .on_input(MessagingMessage::SetSlackChannel)
                    .padding(Padding::from([theme::GRID * 0.8, theme::GRID]))
                    .size(theme::FONT_BODY),
            ]
            .spacing(4)
            .into()
        }
    };

    // Save and Cancel buttons
    let save_style = move |_: &_| container::Style {
        background: Some(iced::Background::Color(pp.coral_bright)),
        border: iced::Border {
            radius: (BORDER_RADIUS * 0.75).into(),
            ..Default::default()
        },
        ..Default::default()
    };

    let btns = row![
        button(
            container(
                text("Save")
                    .size(theme::FONT_BODY)
                    .color(Color::WHITE),
            )
            .padding(Padding::from([theme::GRID * 0.8, theme::GRID * 2.0]))
            .style(save_style),
        )
        .on_press(MessagingMessage::SaveChannel(ch))
        .style(button::text),
        Space::with_width(theme::GRID),
        button(
            text("Cancel")
                .size(theme::FONT_BODY)
                .color(pp.text_secondary),
        )
        .on_press(MessagingMessage::Collapse)
        .padding(Padding::from([theme::GRID * 0.8, theme::GRID * 2.0]))
        .style(button::text),
    ]
    .align_y(Alignment::Center);

    container(
        column![fields, Space::with_height(theme::GRID * 1.5), btns,]
            .spacing(0)
            .width(Length::Fill),
    )
    .padding(Padding {
        top: theme::GRID,
        right: theme::GRID * 1.5,
        bottom: theme::GRID * 1.5,
        left: theme::GRID * 4.0, // indent left to align under channel name
    })
    .width(Length::Fill)
    .style(move |_: &_| container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            pp.coral_bright.r,
            pp.coral_bright.g,
            pp.coral_bright.b,
            0.03,
        ))),
        ..Default::default()
    })
    .into()
}
