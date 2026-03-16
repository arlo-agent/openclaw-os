//! Welcome / Onboarding wizard — first-boot experience.
//!
//! Multi-screen wizard: Welcome → Name → Voice → Auth → Ollama/ApiKey → Messaging → Ready
//! Navigation buttons pinned at card footer; content scrolls above.

use crate::messaging_setup::{MessagingMessage, MessagingState};
use crate::ollama::{OllamaClient, OllamaStatus};
use crate::theme::{self, OpenClawPalette, BORDER_RADIUS};
use crate::widgets::glass_card;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Color, Element, Length, Padding};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};
use rand::Rng;

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
    // Model pulling
    pub pulling_model: bool,
    pub pull_progress: f32,
    pub pull_status: String,
    pub pull_complete: bool,
    pub pull_error: Option<String>,
    // Messaging
    pub messaging: MessagingState,
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
            pulling_model: false,
            pull_progress: 0.0,
            pull_status: String::new(),
            pull_complete: false,
            pull_error: None,
            messaging: MessagingState::default(),
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
    PullModel(String),
    PullProgress(f32, String),
    PullComplete,
    PullError(String),
    SkipMessaging,
    Messaging(MessagingMessage),
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
        WelcomeMessage::PullModel(_model) => {
            // Handled in main.rs — triggers ollama_client.pull_model()
            state.pulling_model = true;
            state.pull_progress = 0.0;
            state.pull_status = "Starting download...".to_string();
            state.pull_complete = false;
            state.pull_error = None;
        }
        WelcomeMessage::PullProgress(pct, status) => {
            state.pull_progress = pct;
            state.pull_status = status;
        }
        WelcomeMessage::PullComplete => {
            state.pulling_model = false;
            state.pull_complete = true;
            state.pull_status = "Download complete!".to_string();
        }
        WelcomeMessage::PullError(e) => {
            state.pulling_model = false;
            state.pull_error = Some(e);
        }
        WelcomeMessage::SkipMessaging => {
            state.step = WizardStep::Ready;
        }
        WelcomeMessage::Messaging(msg) => {
            crate::messaging_setup::update_messaging(&mut state.messaging, msg);
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

// ── Config writing ──────────────────────────────────────────────────────

/// Generate a random 32-char hex token for gateway auth
fn generate_random_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Write config and return the gateway token (so the shell can connect)
pub fn write_wizard_config(state: &WelcomeState) -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let openclaw_dir = format!("{}/.openclaw", home);
    let workspace_dir = format!("{}/workspace", openclaw_dir);

    // Create directories
    let _ = std::fs::create_dir_all(&openclaw_dir);
    let _ = std::fs::create_dir_all(&workspace_dir);
    let _ = std::fs::create_dir_all(format!("{}/voice", workspace_dir));

    let agent_name = if state.agent_name.is_empty() {
        "Atlas"
    } else {
        &state.agent_name
    };

    // 1. Write openclaw.json — must match the real OpenClaw config schema
    // Top-level keys: auth, models, agents, channels, gateway, plugins, etc.
    // NOT "llm" — that's not a valid key.
    let (provider_key, model_id, auth_profile) = match state.auth_choice {
        AuthProvider::Ollama => {
            let model = state.selected_model.as_deref().unwrap_or("llama3.3");
            ("ollama", model.to_string(), serde_json::json!({
                "provider": "ollama",
                "mode": "api_key"
            }))
        }
        AuthProvider::Anthropic => {
            ("anthropic", "claude-sonnet-4-20250514".to_string(), serde_json::json!({
                "provider": "anthropic",
                "mode": "token"
            }))
        }
        AuthProvider::OpenAI => {
            ("openai", "gpt-4o".to_string(), serde_json::json!({
                "provider": "openai",
                "mode": "token"
            }))
        }
        AuthProvider::OpenRouter => {
            ("openrouter", "openrouter/auto".to_string(), serde_json::json!({
                "provider": "openrouter",
                "mode": "token"
            }))
        }
    };

    let full_model_ref = format!("{}/{}", provider_key, model_id);

    // Build provider config
    let mut providers = serde_json::json!({});
    match state.auth_choice {
        AuthProvider::Ollama => {
            providers[provider_key] = serde_json::json!({
                "baseUrl": "http://127.0.0.1:11434",
                "auth": "api-key",
                "api": "openai-completions",
                "models": [{
                    "id": model_id,
                    "name": model_id
                }]
            });
        }
        AuthProvider::Anthropic => {
            // Anthropic is built-in, no need to define provider
        }
        AuthProvider::OpenAI => {
            // OpenAI is built-in, no need to define provider
        }
        AuthProvider::OpenRouter => {
            providers[provider_key] = serde_json::json!({
                "baseUrl": "https://openrouter.ai/api/v1",
                "auth": "api-key",
                "api": "openai-completions",
                "models": [{
                    "id": "openrouter/auto",
                    "name": "Auto (best available)"
                }]
            });
        }
    }

    // Generate a random gateway auth token
    let gateway_token = generate_random_token();

    let mut config = serde_json::json!({
        "auth": {
            "profiles": {
                provider_key: auth_profile
            }
        },
        "agents": {
            "defaults": {
                "model": {
                    "primary": full_model_ref
                },
                "workspace": format!("{}/workspace", openclaw_dir)
            },
            "list": [{
                "id": "main",
                "default": true
            }]
        },
        "gateway": {
            "port": 18789,
            "mode": "local",
            "bind": "loopback",
            "auth": {
                "mode": "token",
                "token": gateway_token
            }
        }
    });

    // Add models.providers only if we have custom providers (Ollama, OpenRouter)
    if providers.as_object().map(|o| !o.is_empty()).unwrap_or(false) {
        config["models"] = serde_json::json!({
            "providers": providers
        });
    }

    // Add API key to env hint for cloud providers
    if !state.api_key.is_empty() && state.auth_choice != AuthProvider::Ollama {
        let env_var = match state.auth_choice {
            AuthProvider::Anthropic => "ANTHROPIC_API_KEY",
            AuthProvider::OpenAI => "OPENAI_API_KEY",
            AuthProvider::OpenRouter => "OPENROUTER_API_KEY",
            _ => "",
        };
        if !env_var.is_empty() {
            // Write API key to a secrets file (not in the config JSON)
            let secrets_content = format!("{}={}\n", env_var, state.api_key);
            let _ = std::fs::write(format!("{}/.env", openclaw_dir), secrets_content);
            eprintln!("[wizard] API key written to {}/.env as {}", openclaw_dir, env_var);
        }
    }

    // Add messaging channels
    let messaging_configs = crate::messaging_setup::generate_messaging_config(&state.messaging);
    if !messaging_configs.is_empty() {
        let mut channels = serde_json::json!({});
        for ch_config in &messaging_configs {
            let ch_type = ch_config.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
            let mut ch_obj = ch_config.clone();
            ch_obj.as_object_mut().map(|o| {
                o.remove("type");
                o.insert("enabled".to_string(), serde_json::Value::Bool(true));
            });
            channels[ch_type] = ch_obj;
        }
        config["channels"] = channels;

        // Add matching plugin entries
        let mut plugin_entries = serde_json::json!({});
        for ch_config in &messaging_configs {
            let ch_type = ch_config.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
            plugin_entries[ch_type] = serde_json::json!({ "enabled": true });
        }
        config["plugins"] = serde_json::json!({ "entries": plugin_entries });
    }

    let config_json = serde_json::to_string_pretty(&config).unwrap_or_default();
    let _ = std::fs::write(format!("{}/openclaw.json", openclaw_dir), config_json);

    // 2. Write IDENTITY.md
    let identity = format!("# IDENTITY.md\n\n- **Name:** {}\n", agent_name);
    let _ = std::fs::write(format!("{}/IDENTITY.md", workspace_dir), identity);

    // 3. Write voice/config.json
    let voice = state
        .selected_voice
        .as_deref()
        .unwrap_or("warm-female");
    let voice_config = serde_json::json!({
        "voice": voice,
        "wakeWord": format!("hey {}", agent_name.to_lowercase())
    });
    let voice_json = serde_json::to_string_pretty(&voice_config).unwrap_or_default();
    let _ = std::fs::write(format!("{}/voice/config.json", workspace_dir), voice_json);

    eprintln!(
        "[wizard] Config written to {}/openclaw.json, {}/IDENTITY.md, {}/voice/config.json",
        openclaw_dir, workspace_dir, workspace_dir
    );

    // 4. Start or restart the gateway so it picks up the new config
    start_or_restart_gateway();

    Some(gateway_token)
}

/// Start or restart the OpenClaw gateway after writing config.
///
/// Tries multiple strategies:
/// 1. `openclaw gateway restart` (if already running)
/// 2. `openclaw gateway start` (if not running)
/// 3. `systemctl restart openclaw-gateway` (ClawOS with systemd)
///
/// Runs in a background thread so the UI doesn't block.
fn start_or_restart_gateway() {
    std::thread::spawn(|| {
        eprintln!("[wizard] Starting/restarting gateway...");

        // Try openclaw CLI first (works on macOS + Linux with openclaw installed)
        let restart = std::process::Command::new("openclaw")
            .args(["gateway", "restart"])
            .output();

        match restart {
            Ok(out) if out.status.success() => {
                eprintln!("[wizard] Gateway restarted via 'openclaw gateway restart'");
                return;
            }
            _ => {}
        }

        // Try starting if restart failed (maybe it wasn't running)
        let start = std::process::Command::new("openclaw")
            .args(["gateway", "start"])
            .output();

        match start {
            Ok(out) if out.status.success() => {
                eprintln!("[wizard] Gateway started via 'openclaw gateway start'");
                return;
            }
            _ => {}
        }

        // Fallback: systemd (ClawOS)
        let systemd = std::process::Command::new("systemctl")
            .args(["restart", "openclaw-gateway"])
            .output();

        match systemd {
            Ok(out) if out.status.success() => {
                eprintln!("[wizard] Gateway restarted via systemctl");
                return;
            }
            _ => {}
        }

        eprintln!("[wizard] Could not start gateway — run 'openclaw gateway start' manually");
    });
}

// ── View ────────────────────────────────────────────────────────────────

pub fn view_welcome<'a>(
    state: &'a WelcomeState,
    palette: &OpenClawPalette,
) -> Element<'a, WelcomeMessage> {
    let p = *palette;

    // Screen content (no nav buttons)
    let content: Element<'a, WelcomeMessage> = match state.step {
        WizardStep::Welcome => view_step_welcome(&p),
        WizardStep::NameAgent => view_step_name(&state.agent_name, &p),
        WizardStep::ChooseVoice => {
            view_step_voice(&state.agent_name, state.selected_voice.as_deref(), &p)
        }
        WizardStep::AuthChoice => view_step_auth(state.auth_choice, &state.ollama_status, &p),
        WizardStep::OllamaSetup => view_step_ollama(state, &p),
        WizardStep::ApiKey => view_step_apikey(state.auth_choice, &state.api_key, &p),
        WizardStep::ConnectMessaging => view_step_messaging(&state.messaging, &p),
        WizardStep::Ready => view_step_ready(&state.agent_name, &p),
    };

    // Footer nav (pinned at bottom)
    let footer: Element<'static, WelcomeMessage> = match state.step {
        WizardStep::Welcome | WizardStep::Ready => {
            // These screens have their own buttons in content
            Space::with_height(0).into()
        }
        WizardStep::ConnectMessaging => {
            container(
                row![
                    secondary_btn("Skip for now", WelcomeMessage::SkipMessaging, &p),
                    Space::with_width(Length::Fill),
                    primary_btn("Next", Bootstrap::ArrowRight, WelcomeMessage::NextStep, &p),
                ]
                .align_y(Alignment::Center)
                .width(Length::Fill),
            )
            .padding(Padding { top: theme::GRID * 1.5, right: theme::GRID * 4.0, bottom: theme::GRID * 2.0, left: theme::GRID * 4.0 })
            .width(Length::Fill)
            .into()
        }
        WizardStep::OllamaSetup if state.pulling_model => {
            // Disable Next while pulling
            container(
                row![
                    secondary_btn("← Back", WelcomeMessage::PrevStep, &p),
                    Space::with_width(Length::Fill),
                    caption("Downloading...", &p),
                ]
                .align_y(Alignment::Center)
                .width(Length::Fill),
            )
            .padding(Padding { top: theme::GRID * 1.5, right: theme::GRID * 4.0, bottom: theme::GRID * 2.0, left: theme::GRID * 4.0 })
            .width(Length::Fill)
            .into()
        }
        _ => {
            container(back_and_next(&p, true))
                .padding(Padding { top: theme::GRID * 1.5, right: theme::GRID * 4.0, bottom: theme::GRID * 2.0, left: theme::GRID * 4.0 })
                .width(Length::Fill)
                .into()
        }
    };

    // Separator line above footer
    let separator = container(Space::new(Length::Fill, 1))
        .width(Length::Fill)
        .style(move |_: &_| container::Style {
            background: Some(iced::Background::Color(p.border_subtle)),
            ..Default::default()
        });

    let has_footer = !matches!(state.step, WizardStep::Welcome | WizardStep::Ready);

    // Assemble: scrollable content + separator + fixed footer
    let card_content = if has_footer {
        column![
            scrollable(
                container(content)
                    .padding(Padding::from(theme::GRID * 4.0))
                    .width(Length::Fill),
            )
            .height(Length::Fill),
            separator,
            footer,
        ]
        .height(Length::Fill)
    } else {
        column![
            scrollable(
                container(content)
                    .padding(Padding::from(theme::GRID * 4.0))
                    .width(Length::Fill),
            )
            .height(Length::Fill),
        ]
        .height(Length::Fill)
    };

    // Glass card wrapper with fixed height
    let card_style = glass_card::glass_container_with_palette(&p);

    container(
        container(card_content)
            .max_width(600)
            .height(520)
            .style(move |_: &_| card_style),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
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

    row(items)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
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

/// Progress bar widget — coral fill on muted background
fn progress_bar(
    progress: f32,
    palette: &OpenClawPalette,
) -> Element<'static, WelcomeMessage> {
    let p = *palette;
    let pct = progress.clamp(0.0, 100.0);

    // Background track
    let bg_style = move |_: &_| container::Style {
        background: Some(iced::Background::Color(p.surface_interactive)),
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    };

    // Fill
    let fill_style = move |_: &_| container::Style {
        background: Some(iced::Background::Color(p.coral_bright)),
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    };

    let fill_width = Length::FillPortion((pct * 10.0) as u16);
    let empty_width = Length::FillPortion(((100.0 - pct) * 10.0) as u16);

    container(
        row![
            container(Space::new(fill_width, 8))
                .style(fill_style),
            Space::new(empty_width, 8),
        ]
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .style(bg_style)
    .into()
}

// ── Individual screens (content only, no nav) ───────────────────────────

fn view_step_welcome(p: &OpenClawPalette) -> Element<'static, WelcomeMessage> {
    column![
        Space::with_height(theme::GRID * 3.0),
        container(
            text("OpenClaw")
                .size(theme::FONT_DISPLAY)
                .color(p.coral_bright),
        )
        .center_x(Length::Fill),
        Space::with_height(theme::GRID * 2.0),
        container(
            text("Your AI assistant lives here.")
                .size(theme::FONT_HEADING)
                .color(p.text_secondary),
        )
        .center_x(Length::Fill),
        Space::with_height(theme::GRID),
        container(
            text("Let's get you set up.")
                .size(theme::FONT_BODY)
                .color(p.text_muted),
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

    let male_voices: Vec<(&str, &str)> = vec![
        ("Calm", "calm-male"),
        ("Energetic", "energetic-male"),
        ("Deep", "deep-male"),
    ];
    let female_voices: Vec<(&str, &str)> = vec![
        ("Warm", "warm-female"),
        ("Clear", "clear-female"),
        ("Bright", "bright-female"),
    ];

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
                display,
                "",
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

        items.push(radio_option(
            provider.label(),
            provider.description(),
            is_sel,
            WelcomeMessage::SetAuthChoice(*provider),
            p,
        ));

        // Add Ollama status indicator below the radio option
        if *provider == AuthProvider::Ollama {
            let (status_icon, status_color, status_text): (Bootstrap, Color, String) = match ollama_status {
                OllamaStatus::Running(v) => (
                    Bootstrap::CheckCircleFill,
                    p.cyan_bright,
                    format!("Detected (v{})", v),
                ),
                OllamaStatus::NotRunning => (
                    Bootstrap::ExclamationTriangleFill,
                    p.coral_mid,
                    "Installed but not running".to_string(),
                ),
                OllamaStatus::NotInstalled => (
                    Bootstrap::XCircleFill,
                    p.coral_bright,
                    "Not detected".to_string(),
                ),
                OllamaStatus::Checking => (
                    Bootstrap::Hourglass,
                    p.text_muted,
                    "Checking...".to_string(),
                ),
                OllamaStatus::Unknown => (
                    Bootstrap::Hourglass,
                    p.text_muted,
                    String::new(),
                ),
            };

            if !status_text.is_empty() {
                items.push(
                    container(
                        row![
                            Space::with_width(30), // indent under radio label
                            bicon(status_icon, 12.0, status_color),
                            Space::with_width(6),
                            text(status_text)
                                .size(theme::FONT_CAPTION)
                                .color(status_color),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .into(),
                );
            }
        }

        items.push(Space::with_height(theme::GRID).into());
    }

    column(items).width(Length::Fill).into()
}

fn view_step_ollama(
    state: &WelcomeState,
    p: &OpenClawPalette,
) -> Element<'static, WelcomeMessage> {
    let status = &state.ollama_status;
    let available = &state.available_models;
    let selected = state.selected_model.as_deref();

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

            if available.is_empty() && !state.pull_complete {
                items.push(body_text(
                    "No models found. Select one to download:",
                    p,
                ));
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

                // Download button or progress
                items.push(Space::with_height(theme::GRID * 1.5).into());

                if state.pulling_model {
                    // Progress bar
                    items.push(progress_bar(state.pull_progress, p));
                    items.push(Space::with_height(theme::GRID * 0.5).into());
                    let status_text = format!(
                        "{} — {:.0}%",
                        state.pull_status, state.pull_progress
                    );
                    items.push(caption(&status_text, p));
                } else if let Some(ref err) = state.pull_error {
                    items.push(
                        text(format!("Error: {}", err))
                            .size(theme::FONT_CAPTION)
                            .color(p.coral_bright)
                            .into(),
                    );
                    items.push(Space::with_height(theme::GRID).into());
                    if let Some(ref model) = state.selected_model {
                        let pp = *p;
                        items.push(
                            button(
                                container(
                                    row![
                                        bicon(Bootstrap::ArrowRepeat, 14.0, Color::WHITE),
                                        Space::with_width(6),
                                        text("Retry")
                                            .size(theme::FONT_BODY)
                                            .color(Color::WHITE),
                                    ]
                                    .align_y(Alignment::Center),
                                )
                                .padding(Padding::from([
                                    theme::GRID,
                                    theme::GRID * 2.0,
                                ]))
                                .style(move |_: &_| container::Style {
                                    background: Some(iced::Background::Color(
                                        pp.coral_bright,
                                    )),
                                    border: iced::Border {
                                        radius: BORDER_RADIUS.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            )
                            .on_press(WelcomeMessage::PullModel(model.clone()))
                            .style(button::text)
                            .into(),
                        );
                    }
                } else if let Some(ref model) = state.selected_model {
                    // Download button
                    let pp = *p;
                    items.push(
                        button(
                            container(
                                row![
                                    bicon(Bootstrap::Download, 14.0, Color::WHITE),
                                    Space::with_width(6),
                                    text(format!("Download {}", model))
                                        .size(theme::FONT_BODY)
                                        .color(Color::WHITE),
                                ]
                                .align_y(Alignment::Center),
                            )
                            .padding(Padding::from([
                                theme::GRID,
                                theme::GRID * 2.0,
                            ]))
                            .style(move |_: &_| container::Style {
                                background: Some(iced::Background::Color(pp.coral_bright)),
                                border: iced::Border {
                                    radius: BORDER_RADIUS.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                        )
                        .on_press(WelcomeMessage::PullModel(model.clone()))
                        .style(button::text)
                        .into(),
                    );
                }
            } else if state.pull_complete && available.is_empty() {
                // Just finished pulling
                items.push(
                    row![
                        bicon(Bootstrap::CheckCircleFill, 16.0, p.cyan_bright),
                        Space::with_width(8),
                        text(format!(
                            "Model {} downloaded successfully!",
                            state.selected_model.as_deref().unwrap_or("unknown")
                        ))
                        .size(theme::FONT_BODY)
                        .color(p.cyan_bright),
                    ]
                    .align_y(Alignment::Center)
                    .into(),
                );
            } else {
                // Has local models
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

    column(items).width(Length::Fill).into()
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
    ]
    .width(Length::Fill)
    .into()
}

fn view_step_messaging<'a>(
    messaging_state: &'a MessagingState,
    p: &OpenClawPalette,
) -> Element<'a, WelcomeMessage> {
    let messaging_view = crate::messaging_setup::view_messaging_setup(messaging_state, p);
    let mapped: Element<'a, WelcomeMessage> = messaging_view.map(WelcomeMessage::Messaging);

    column![
        heading("Connect your messaging", p),
        Space::with_height(theme::GRID),
        body_text("You can do this later.", p),
        Space::with_height(theme::GRID * 2.0),
        mapped,
    ]
    .width(Length::Fill)
    .into()
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
            row![
                bicon(Bootstrap::Stars, theme::FONT_DISPLAY * 0.5, p.coral_bright),
                Space::with_width(12),
                text("All set!")
                    .size(theme::FONT_DISPLAY * 0.6)
                    .color(p.coral_bright),
            ]
            .align_y(Alignment::Center),
        )
        .center_x(Length::Fill),
        Space::with_height(theme::GRID * 2.0),
        container(
            text(ready_text)
                .size(theme::FONT_HEADING)
                .color(p.text_primary),
        )
        .center_x(Length::Fill),
        Space::with_height(theme::GRID * 3.0),
        container(
            text(try_text)
                .size(theme::FONT_BODY)
                .color(p.text_secondary),
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
