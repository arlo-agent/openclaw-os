//! Welcome / Onboarding wizard — first-boot experience.
//!
//! Multi-screen wizard: Welcome → Name → Voice → Auth → Ollama/ApiKey → Messaging → Ready

use crate::ollama::{OllamaClient, OllamaStatus};
use crate::theme::{self, OpenClawPalette, BORDER_RADIUS};
use crate::widgets::glass_card;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Color, Element, Length, Padding};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

// ── Data types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardStep {
    Welcome,
    NameAgent,
    ChooseVoice,
    AuthChoice,
    OllamaSetup,
    ApiKey,
    ConnectMessaging,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthProvider {
    Ollama,
    Anthropic,
    OpenAI,
    OpenRouter,
}

impl AuthProvider {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ollama => "Ollama (local)",
            Self::Anthropic => "Anthropic",
            Self::OpenAI => "OpenAI",
            Self::OpenRouter => "OpenRouter",
        }
    }
    pub fn description(&self) -> &'static str {
        match self {
            Self::Ollama => "Runs on this device, no cloud, no API key needed",
            Self::Anthropic => "Claude models, requires API key",
            Self::OpenAI => "GPT models, requires API key",
            Self::OpenRouter => "Access many models, requires API key",
        }
    }
    pub fn key_hint(&self) -> &'static str {
        match self {
            Self::Anthropic => "sk-ant-...",
            Self::OpenAI => "sk-...",
            Self::OpenRouter => "sk-or-...",
            _ => "",
        }
    }
}

/// Full wizard state
pub struct WelcomeState {
    pub step: WizardStep,
    pub agent_name: String,
    pub selected_voice: Option<String>,
    pub auth_choice: AuthProvider,
    pub api_key: String,
    pub selected_model: Option<String>,
    pub available_models: Vec<String>,
    pub ollama_status: OllamaStatus,
}

impl Default for WelcomeState {
    fn default() -> Self {
        Self {
            step: WizardStep::Welcome,
            agent_name: String::new(),
            selected_voice: None,
            auth_choice: AuthProvider::Ollama,
            api_key: String::new(),
            selected_model: None,
            available_models: Vec::new(),
            ollama_status: OllamaStatus::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WelcomeMessage {
    NextStep,
    PrevStep,
    SetName(String),
    SelectNameSuggestion(String),
    SelectVoice(String),
    SetAuthChoice(AuthProvider),
    SetApiKey(String),
    SelectModel(String),
    SkipMessaging,
    Finish,
}

// ── Update logic ────────────────────────────────────────────────────────

/// Returns true when wizard is complete (Finish)
pub fn update_welcome(state: &mut WelcomeState, message: WelcomeMessage) -> bool {
    match message {
        WelcomeMessage::NextStep => {
            state.step = next_step(state.step, state.auth_choice);
        }
        WelcomeMessage::PrevStep => {
            state.step = prev_step(state.step, state.auth_choice);
        }
        WelcomeMessage::SetName(n) => {
            state.agent_name = n;
        }
        WelcomeMessage::SelectNameSuggestion(n) => {
            state.agent_name = n;
        }
        WelcomeMessage::SelectVoice(v) => {
            state.selected_voice = Some(v);
        }
        WelcomeMessage::SetAuthChoice(a) => {
            state.auth_choice = a;
        }
        WelcomeMessage::SetApiKey(k) => {
            state.api_key = k;
        }
        WelcomeMessage::SelectModel(m) => {
            state.selected_model = Some(m);
        }
        WelcomeMessage::SkipMessaging => {
            state.step = WizardStep::Ready;
        }
        WelcomeMessage::Finish => {
            return true;
        }
    }
    false
}

fn next_step(current: WizardStep, auth: AuthProvider) -> WizardStep {
    match current {
        WizardStep::Welcome => WizardStep::NameAgent,
        WizardStep::NameAgent => WizardStep::ChooseVoice,
        WizardStep::ChooseVoice => WizardStep::AuthChoice,
        WizardStep::AuthChoice => {
            if auth == AuthProvider::Ollama {
                WizardStep::OllamaSetup
            } else {
                WizardStep::ApiKey
            }
        }
        WizardStep::OllamaSetup => WizardStep::ConnectMessaging,
        WizardStep::ApiKey => WizardStep::ConnectMessaging,
        WizardStep::ConnectMessaging => WizardStep::Ready,
        WizardStep::Ready => WizardStep::Ready,
    }
}

fn prev_step(current: WizardStep, auth: AuthProvider) -> WizardStep {
    match current {
        WizardStep::Welcome => WizardStep::Welcome,
        WizardStep::NameAgent => WizardStep::Welcome,
        WizardStep::ChooseVoice => WizardStep::NameAgent,
        WizardStep::AuthChoice => WizardStep::ChooseVoice,
        WizardStep::OllamaSetup => WizardStep::AuthChoice,
        WizardStep::ApiKey => WizardStep::AuthChoice,
        WizardStep::ConnectMessaging => {
            if auth == AuthProvider::Ollama {
                WizardStep::OllamaSetup
            } else {
                WizardStep::ApiKey
            }
        }
        WizardStep::Ready => WizardStep::ConnectMessaging,
    }
}

// ── View ────────────────────────────────────────────────────────────────

pub fn view_welcome<'a>(
    state: &'a WelcomeState,
    palette: &OpenClawPalette,
) -> Element<'a, WelcomeMessage> {
    let p = *palette;

    let inner: Element<'a, WelcomeMessage> = match state.step {
        WizardStep::Welcome => view_step_welcome(&p),
        WizardStep::NameAgent => view_step_name(&state.agent_name, &p),
        WizardStep::ChooseVoice => {
            view_step_voice(&state.agent_name, state.selected_voice.as_deref(), &p)
        }
        WizardStep::AuthChoice => view_step_auth(state.auth_choice, &state.ollama_status, &p),
        WizardStep::OllamaSetup => view_step_ollama(
            &state.ollama_status,
            &state.available_models,
            state.selected_model.as_deref(),
            &p,
        ),
        WizardStep::ApiKey => view_step_apikey(state.auth_choice, &state.api_key, &p),
        WizardStep::ConnectMessaging => view_step_messaging(&p),
        WizardStep::Ready => view_step_ready(&state.agent_name, &p),
    };

    // Wrap in a centered glass card
    let card_style = glass_card::glass_container_with_palette(&p);

    let card = container(
        container(inner)
            .padding(Padding::from(theme::GRID * 4.0))
            .max_width(600)
            .style(move |_: &_| card_style),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fill);

    card.into()
}

// ── Helper widgets ──────────────────────────────────────────────────────

fn bicon(i: Bootstrap, size: f32, color: Color) -> iced::widget::Text<'static> {
    text(i.to_string())
        .font(BOOTSTRAP_FONT)
        .size(size)
        .color(color)
}

fn primary_btn(
    label: &str,
    icon: Bootstrap,
    msg: WelcomeMessage,
    palette: &OpenClawPalette,
) -> Element<'static, WelcomeMessage> {
    let p = *palette;
    let label_owned = label.to_string();
    button(
        container(
            row![
                text(label_owned)
                    .size(theme::FONT_BODY)
                    .color(Color::WHITE),
                Space::with_width(8),
                bicon(icon, 14.0, Color::WHITE),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([theme::GRID * 1.2, theme::GRID * 2.5]))
        .style(move |_: &_| container::Style {
            background: Some(iced::Background::Color(p.coral_bright)),
            border: iced::Border {
                radius: BORDER_RADIUS.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    )
    .on_press(msg)
    .style(button::text)
    .into()
}

fn secondary_btn(
    label: &str,
    msg: WelcomeMessage,
    palette: &OpenClawPalette,
) -> Element<'static, WelcomeMessage> {
    let p = *palette;
    button(
        text(label.to_string())
            .size(theme::FONT_BODY)
            .color(p.text_secondary),
    )
    .on_press(msg)
    .padding(Padding::from([theme::GRID, theme::GRID * 2.0]))
    .style(button::text)
    .into()
}

fn back_and_next(palette: &OpenClawPalette, show_back: bool) -> Element<'static, WelcomeMessage> {
    let p = *palette;
    let mut items: Vec<Element<'static, WelcomeMessage>> = Vec::new();

    if show_back {
        items.push(secondary_btn("← Back", WelcomeMessage::PrevStep, &p));
    }
    items.push(Space::with_width(Length::Fill).into());
    items.push(primary_btn(
        "Next",
        Bootstrap::ArrowRight,
        WelcomeMessage::NextStep,
        &p,
    ));

    row(items).align_y(Alignment::Center).width(Length::Fill).into()
}

fn heading(s: &str, palette: &OpenClawPalette) -> Element<'static, WelcomeMessage> {
    text(s.to_string())
        .size(theme::FONT_HEADING)
        .color(palette.text_primary)
        .into()
}

fn heading_owned(s: String, palette: &OpenClawPalette) -> Element<'static, WelcomeMessage> {
    text(s)
        .size(theme::FONT_HEADING)
        .color(palette.text_primary)
        .into()
}

fn body_text(s: &str, palette: &OpenClawPalette) -> Element<'static, WelcomeMessage> {
    text(s.to_string())
        .size(theme::FONT_BODY)
        .color(palette.text_secondary)
        .into()
}

fn caption(s: &str, palette: &OpenClawPalette) -> Element<'static, WelcomeMessage> {
    text(s.to_string())
        .size(theme::FONT_CAPTION)
        .color(palette.text_muted)
        .into()
}

fn radio_option(
    label_str: &str,
    description: &str,
    selected: bool,
    msg: WelcomeMessage,
    palette: &OpenClawPalette,
) -> Element<'static, WelcomeMessage> {
    let p = *palette;
    let icon = if selected {
        Bootstrap::RecordCircleFill
    } else {
        Bootstrap::Circle
    };
    let icon_color = if selected {
        p.coral_bright
    } else {
        p.text_muted
    };
    let border_color = if selected {
        p.border_accent
    } else {
        p.border_subtle
    };

    let label_owned = label_str.to_string();
    let desc_owned = description.to_string();

    button(
        container(
            row![
                bicon(icon, 18.0, icon_color),
                Space::with_width(12),
                column![
                    text(label_owned)
                        .size(theme::FONT_BODY)
                        .color(p.text_primary),
                    text(desc_owned)
                        .size(theme::FONT_CAPTION)
                        .color(p.text_secondary),
                ]
                .spacing(2),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([theme::GRID * 1.5, theme::GRID * 2.0]))
        .width(Length::Fill)
        .style(move |_: &_| container::Style {
            background: Some(iced::Background::Color(if selected {
                Color::from_rgba(p.coral_bright.r, p.coral_bright.g, p.coral_bright.b, 0.08)
            } else {
                Color::TRANSPARENT
            })),
            border: iced::Border {
                color: border_color,
                width: 1.0,
                radius: (BORDER_RADIUS * 0.75).into(),
            },
            ..Default::default()
        }),
    )
    .on_press(msg)
    .style(button::text)
    .width(Length::Fill)
    .into()
}

// ── Individual screens ──────────────────────────────────────────────────

fn view_step_welcome(p: &OpenClawPalette) -> Element<'static, WelcomeMessage> {
    column![
        Space::with_height(theme::GRID * 3.0),
        container(
            text("OpenClaw")
                .size(theme::FONT_DISPLAY)
                .color(p.coral_bright)
        )
        .center_x(Length::Fill),
        Space::with_height(theme::GRID * 2.0),
        container(
            text("Your AI assistant lives here.")
                .size(theme::FONT_HEADING)
                .color(p.text_secondary)
        )
        .center_x(Length::Fill),
        Space::with_height(theme::GRID),
        container(
            text("Let's get you set up.")
                .size(theme::FONT_BODY)
                .color(p.text_muted)
        )
        .center_x(Length::Fill),
        Space::with_height(theme::GRID * 4.0),
        container(primary_btn(
            "Get Started",
            Bootstrap::ArrowRight,
            WelcomeMessage::NextStep,
            p,
        ))
        .center_x(Length::Fill),
        Space::with_height(theme::GRID * 2.0),
        container(caption("Language: English", p)).center_x(Length::Fill),
    ]
    .spacing(0)
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn view_step_name<'a>(current_name: &'a str, p: &OpenClawPalette) -> Element<'a, WelcomeMessage> {
    let suggestions = ["Nova", "Atlas", "Sage", "Echo", "Iris", "Max"];

    let make_suggestion = |name: &str| -> Element<'static, WelcomeMessage> {
        let selected = current_name == name;
        let bg = if selected {
            Color::from_rgba(p.coral_bright.r, p.coral_bright.g, p.coral_bright.b, 0.15)
        } else {
            Color::TRANSPARENT
        };
        let border = if selected {
            p.border_accent
        } else {
            p.border_subtle
        };
        button(
            container(
                text(name.to_string())
                    .size(theme::FONT_BODY)
                    .color(p.text_primary),
            )
            .padding(Padding::from([theme::GRID * 0.8, theme::GRID * 2.0]))
            .style(move |_: &_| container::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    color: border,
                    width: 1.0,
                    radius: (BORDER_RADIUS * 0.75).into(),
                },
                ..Default::default()
            }),
        )
        .on_press(WelcomeMessage::SelectNameSuggestion(name.to_string()))
        .style(button::text)
        .into()
    };

    let row1 = row![
        make_suggestion(suggestions[0]),
        make_suggestion(suggestions[1]),
        make_suggestion(suggestions[2]),
    ]
    .spacing(theme::GRID as u16)
    .align_y(Alignment::Center);

    let row2 = row![
        make_suggestion(suggestions[3]),
        make_suggestion(suggestions[4]),
        make_suggestion(suggestions[5]),
    ]
    .spacing(theme::GRID as u16)
    .align_y(Alignment::Center);

    column![
        heading("What would you like to call your assistant?", p),
        Space::with_height(theme::GRID * 2.0),
        text_input("Enter a name...", current_name)
            .on_input(WelcomeMessage::SetName)
            .padding(Padding::from([theme::GRID * 1.2, theme::GRID * 1.5]))
            .size(theme::FONT_BODY),
        Space::with_height(theme::GRID * 2.0),
        caption("Suggestions:", p),
        Space::with_height(theme::GRID),
        row1,
        Space::with_height(theme::GRID),
        row2,
        Space::with_height(theme::GRID * 3.0),
        back_and_next(p, true),
    ]
    .width(Length::Fill)
    .into()
}

fn view_step_voice(
    agent_name: &str,
    selected: Option<&str>,
    p: &OpenClawPalette,
) -> Element<'static, WelcomeMessage> {
    let name_display = if agent_name.is_empty() {
        "your assistant".to_string()
    } else {
        agent_name.to_string()
    };

    let male_voices: Vec<(&str, &str)> = vec![("Calm", "calm-male"), ("Energetic", "energetic-male"), ("Deep", "deep-male")];
    let female_voices: Vec<(&str, &str)> = vec![("Warm", "warm-female"), ("Clear", "clear-female"), ("Bright", "bright-female")];

    let selected_owned = selected.map(String::from);

    let make_col = |label: &str, voices: Vec<(&str, &str)>| -> Element<'static, WelcomeMessage> {
        let mut items: Vec<Element<'static, WelcomeMessage>> = vec![
            text(label.to_string())
                .size(theme::FONT_BODY)
                .color(p.text_secondary)
                .into(),
            Space::with_height(theme::GRID).into(),
        ];
        for (display, key) in voices {
            let is_sel = selected_owned.as_deref() == Some(key);
            items.push(radio_option(
                &display.to_string(),
                &String::new(),
                is_sel,
                WelcomeMessage::SelectVoice(key.to_string()),
                p,
            ));
            items.push(Space::with_height(theme::GRID * 0.5).into());
        }
        column(items).width(Length::Fill).into()
    };

    let title = format!("Choose a voice for {}", name_display);

    column![
        heading_owned(title, p),
        Space::with_height(theme::GRID * 2.0),
        row![
            make_col("Male", male_voices),
            Space::with_width(theme::GRID * 2.0),
            make_col("Female", female_voices),
        ]
        .width(Length::Fill),
        Space::with_height(theme::GRID * 3.0),
        back_and_next(p, true),
    ]
    .width(Length::Fill)
    .into()
}

fn view_step_auth(
    current: AuthProvider,
    ollama_status: &OllamaStatus,
    p: &OpenClawPalette,
) -> Element<'static, WelcomeMessage> {
    let providers = [
        AuthProvider::Ollama,
        AuthProvider::Anthropic,
        AuthProvider::OpenAI,
        AuthProvider::OpenRouter,
    ];

    let mut items: Vec<Element<'static, WelcomeMessage>> = vec![
        heading("Choose your AI provider", p),
        Space::with_height(theme::GRID * 2.0).into(),
    ];

    for provider in &providers {
        let is_sel = current == *provider;
        let desc = if *provider == AuthProvider::Ollama {
            let status_str = match ollama_status {
                OllamaStatus::Running(v) => format!("  ✓ Detected (v{})", v),
                OllamaStatus::NotRunning => "  ⚠ Installed but not running".to_string(),
                OllamaStatus::NotInstalled => "  ✗ Not detected".to_string(),
                OllamaStatus::Checking => "  ⏳ Checking...".to_string(),
                OllamaStatus::Unknown => String::new(),
            };
            format!("{}{}", provider.description(), status_str)
        } else {
            provider.description().to_string()
        };

        items.push(radio_option(
            provider.label(),
            &desc,
            is_sel,
            WelcomeMessage::SetAuthChoice(*provider),
            p,
        ));
        items.push(Space::with_height(theme::GRID).into());
    }

    items.push(Space::with_height(theme::GRID * 2.0).into());
    items.push(back_and_next(p, true));

    column(items).width(Length::Fill).into()
}

fn view_step_ollama(
    status: &OllamaStatus,
    available: &[String],
    selected: Option<&str>,
    p: &OpenClawPalette,
) -> Element<'static, WelcomeMessage> {
    let mut items: Vec<Element<'static, WelcomeMessage>> = vec![
        heading("Set up Ollama", p),
        Space::with_height(theme::GRID).into(),
    ];

    match status {
        OllamaStatus::Running(v) => {
            items.push(
                row![
                    bicon(Bootstrap::CheckCircleFill, 18.0, p.cyan_bright),
                    Space::with_width(8),
                    text(format!("Ollama is running (v{})", v))
                        .size(theme::FONT_BODY)
                        .color(p.cyan_bright),
                ]
                .align_y(Alignment::Center)
                .into(),
            );
            items.push(Space::with_height(theme::GRID * 2.0).into());

            if available.is_empty() {
                items.push(body_text("No models found. Pull a model to get started.", p));
                items.push(Space::with_height(theme::GRID).into());
                items.push(caption("Recommended models:", p));
                items.push(Space::with_height(theme::GRID).into());

                for (name, desc) in OllamaClient::recommended_models() {
                    let is_sel = selected == Some(name);
                    items.push(radio_option(
                        name,
                        desc,
                        is_sel,
                        WelcomeMessage::SelectModel(name.to_string()),
                        p,
                    ));
                    items.push(Space::with_height(theme::GRID * 0.5).into());
                }
            } else {
                items.push(body_text("Select a model:", p));
                items.push(Space::with_height(theme::GRID).into());

                for model_name in available {
                    let is_sel = selected.map(|s| s == model_name).unwrap_or(false);
                    items.push(radio_option(
                        model_name,
                        "",
                        is_sel,
                        WelcomeMessage::SelectModel(model_name.clone()),
                        p,
                    ));
                    items.push(Space::with_height(theme::GRID * 0.5).into());
                }
            }
        }
        OllamaStatus::NotInstalled => {
            items.push(
                row![
                    bicon(Bootstrap::ExclamationTriangleFill, 18.0, p.coral_bright),
                    Space::with_width(8),
                    text("Ollama not found")
                        .size(theme::FONT_BODY)
                        .color(p.coral_bright),
                ]
                .align_y(Alignment::Center)
                .into(),
            );
            items.push(Space::with_height(theme::GRID * 2.0).into());
            items.push(body_text("Install Ollama to run models locally:", p));
            items.push(Space::with_height(theme::GRID).into());
            items.push(
                text("curl -fsSL https://ollama.com/install.sh | sh")
                    .size(theme::FONT_CAPTION)
                    .color(p.cyan_bright)
                    .into(),
            );
        }
        OllamaStatus::NotRunning => {
            items.push(
                row![
                    bicon(Bootstrap::ExclamationCircleFill, 18.0, p.coral_mid),
                    Space::with_width(8),
                    text("Ollama is installed but not running")
                        .size(theme::FONT_BODY)
                        .color(p.coral_mid),
                ]
                .align_y(Alignment::Center)
                .into(),
            );
            items.push(Space::with_height(theme::GRID * 2.0).into());
            items.push(body_text("Start Ollama with:", p));
            items.push(Space::with_height(theme::GRID).into());
            items.push(
                text("ollama serve")
                    .size(theme::FONT_CAPTION)
                    .color(p.cyan_bright)
                    .into(),
            );
        }
        _ => {
            items.push(body_text("Checking Ollama status...", p));
        }
    }

    items.push(Space::with_height(theme::GRID * 3.0).into());
    items.push(back_and_next(p, true));

    let content = column(items).width(Length::Fill);
    scrollable(content).height(Length::Fill).into()
}

fn view_step_apikey<'a>(
    provider: AuthProvider,
    current_key: &'a str,
    p: &OpenClawPalette,
) -> Element<'a, WelcomeMessage> {
    let title = format!("Connect to {}", provider.label());

    column![
        heading_owned(title, p),
        Space::with_height(theme::GRID),
        body_text(provider.description(), p),
        Space::with_height(theme::GRID * 3.0),
        caption("Paste your API key:", p),
        Space::with_height(theme::GRID),
        text_input(provider.key_hint(), current_key)
            .on_input(WelcomeMessage::SetApiKey)
            .padding(Padding::from([theme::GRID * 1.2, theme::GRID * 1.5]))
            .size(theme::FONT_BODY),
        Space::with_height(theme::GRID * 2.0),
        row![
            bicon(Bootstrap::QuestionCircle, 14.0, p.text_muted),
            Space::with_width(6),
            text("Don't have one? Check the provider's website for a key.")
                .size(theme::FONT_CAPTION)
                .color(p.text_muted),
        ]
        .align_y(Alignment::Center),
        Space::with_height(theme::GRID * 4.0),
        back_and_next(p, true),
    ]
    .width(Length::Fill)
    .into()
}

fn view_step_messaging(p: &OpenClawPalette) -> Element<'static, WelcomeMessage> {
    let channels = [
        (Bootstrap::Telegram, "Telegram"),
        (Bootstrap::Whatsapp, "WhatsApp"),
        (Bootstrap::Discord, "Discord"),
        (Bootstrap::ChatDots, "Signal"),
        (Bootstrap::Slack, "Slack"),
    ];

    let mut items: Vec<Element<'static, WelcomeMessage>> = vec![
        heading("Connect your messaging", p),
        Space::with_height(theme::GRID).into(),
        body_text("You can do this later.", p),
        Space::with_height(theme::GRID * 2.0).into(),
    ];

    for (icon, name) in &channels {
        let pp = *p;
        let channel_row = button(
            container(
                row![
                    bicon(*icon, 20.0, pp.text_primary),
                    Space::with_width(12),
                    text(*name)
                        .size(theme::FONT_BODY)
                        .color(pp.text_primary),
                    Space::with_width(Length::Fill),
                    text("Add")
                        .size(theme::FONT_CAPTION)
                        .color(pp.text_muted),
                    Space::with_width(4),
                    bicon(Bootstrap::ChevronRight, 12.0, pp.text_muted),
                ]
                .align_y(Alignment::Center)
                .width(Length::Fill),
            )
            .padding(Padding::from([theme::GRID * 1.5, theme::GRID * 2.0]))
            .width(Length::Fill)
            .style(move |_: &_| container::Style {
                border: iced::Border {
                    color: pp.border_subtle,
                    width: 1.0,
                    radius: (BORDER_RADIUS * 0.75).into(),
                },
                ..Default::default()
            }),
        )
        .style(button::text)
        .width(Length::Fill);

        items.push(channel_row.into());
        items.push(Space::with_height(theme::GRID).into());
    }

    items.push(Space::with_height(theme::GRID * 2.0).into());

    // Bottom nav: Skip + Next
    let nav = row![
        secondary_btn("Skip for now", WelcomeMessage::SkipMessaging, p),
        Space::with_width(Length::Fill),
        primary_btn("Next", Bootstrap::ArrowRight, WelcomeMessage::NextStep, p),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    items.push(nav.into());

    column(items).width(Length::Fill).into()
}

fn view_step_ready(agent_name: &str, p: &OpenClawPalette) -> Element<'static, WelcomeMessage> {
    let name = if agent_name.is_empty() {
        "Your assistant".to_string()
    } else {
        agent_name.to_string()
    };

    let ready_text = format!("{} is ready.", name);
    let try_text = format!("Try saying:\n\"Hey {}, what can you do?\"", name);

    column![
        Space::with_height(theme::GRID * 3.0),
        container(
            text("✨ All set!")
                .size(theme::FONT_DISPLAY * 0.6)
                .color(p.coral_bright)
        )
        .center_x(Length::Fill),
        Space::with_height(theme::GRID * 2.0),
        container(
            text(ready_text)
                .size(theme::FONT_HEADING)
                .color(p.text_primary)
        )
        .center_x(Length::Fill),
        Space::with_height(theme::GRID * 3.0),
        container(
            text(try_text)
                .size(theme::FONT_BODY)
                .color(p.text_secondary)
        )
        .center_x(Length::Fill),
        Space::with_height(theme::GRID * 4.0),
        container(primary_btn(
            "Start using",
            Bootstrap::ArrowRight,
            WelcomeMessage::Finish,
            p,
        ))
        .center_x(Length::Fill),
        Space::with_height(theme::GRID * 2.0),
    ]
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .into()
}
