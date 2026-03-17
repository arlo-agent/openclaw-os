#![allow(dead_code, unused_imports, unused_variables)]

mod about;
mod ambient;
mod cards;
mod conversation;
mod device_identity;
mod gateway;
mod messaging_setup;
mod notifications;
mod ollama;
mod spotlight;
mod theme;
mod welcome;
mod widgets;

use cards::{Card, CardMessage, CardType};
use conversation::ChatMessage;
use gateway::{Gateway, GatewayConfig, GatewayEvent};
use iced::widget::{column, container, row, stack, Space};
use iced::{Alignment, Element, Length, Padding, Size, Subscription, Theme};
use notifications::{NotificationMessage, NotificationState};
use spotlight::{ChatOverlayMessage, LogoAction};
use ollama::{OllamaClient, OllamaEvent, OllamaStatus};
use std::time::{Duration, Instant};
use theme::{OpenClawPalette, ThemeMode};
use welcome::{WelcomeMessage, WelcomeState, WizardStep};
use widgets::particle_field::ParticleField;
extern crate iced_fonts;

fn main() -> iced::Result {
    iced::application("OpenClaw OS", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .font(iced_fonts::BOOTSTRAP_FONT_BYTES)
        .window_size(Size::new(1280.0, 720.0))
        .antialiasing(true)
        .run()
}

#[derive(Debug, Clone)]
enum AppView {
    Welcome,
    Ambient,
}

struct App {
    view: AppView,
    particles: ParticleField,
    cards: Vec<Card>,
    chat_messages: Vec<ChatMessage>,
    chat_input: String,
    show_chat: bool,
    connected: bool,
    agent_active: bool,
    agent_thinking: bool,
    window_size: (f32, f32),
    theme_mode: ThemeMode,
    gateway: Gateway,
    welcome: WelcomeState,
    ollama_client: OllamaClient,
    notifs: NotificationState,
    show_about: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    Card(CardMessage),
    Welcome(WelcomeMessage),
    Notification(NotificationMessage),
    Chat(ChatOverlayMessage),
    Logo(LogoAction),
    About(about::AboutMessage),
}

impl Default for App {
    fn default() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let config = GatewayConfig::from_args(&args);
        let is_mock = config.mock;
        let gw = Gateway::new(config);
        let connected = gw.connected;

        let force_welcome = args.iter().any(|a| a == "--welcome" || a == "--onboard");
        let has_config = std::path::Path::new("/home/openclaw/.config/openclaw/openclaw.json").exists()
            || std::path::Path::new("/etc/openclaw/openclaw.json").exists();
        let show_welcome = force_welcome || !has_config;

        let demo_cards = if is_mock || show_welcome {
            Vec::new()
        } else {
            vec![
                Card::new(
                    CardType::Message,
                    "New message from Francis",
                    "Hey, how's the shell UI coming along?",
                ),
                Card::new(
                    CardType::Status,
                    "System Update",
                    "NixOS generation 42 applied successfully.",
                ),
                Card::new(
                    CardType::Alert,
                    "Calendar",
                    "Team standup in 15 minutes",
                ),
            ]
        };

        let ollama_client = OllamaClient::new();
        if show_welcome {
            ollama_client.check_status();
        }

        let mut welcome_state = WelcomeState::default();
        if let Some(pos) = args.iter().position(|a| a == "--auth-choice") {
            if let Some(choice) = args.get(pos + 1) {
                match choice.to_lowercase().as_str() {
                    "ollama" => welcome_state.auth_choice = welcome::AuthProvider::Ollama,
                    "anthropic" => welcome_state.auth_choice = welcome::AuthProvider::Anthropic,
                    "openai" => welcome_state.auth_choice = welcome::AuthProvider::OpenAI,
                    "openrouter" => welcome_state.auth_choice = welcome::AuthProvider::OpenRouter,
                    _ => {}
                }
            }
        }

        let initial_view = if show_welcome {
            AppView::Welcome
        } else {
            AppView::Ambient
        };

        Self {
            view: initial_view,
            particles: ParticleField::new(),
            cards: demo_cards,
            chat_messages: Vec::new(),
            chat_input: String::new(),
            show_chat: false,
            connected,
            agent_active: true,
            agent_thinking: false,
            window_size: (1280.0, 720.0),
            theme_mode: ThemeMode::default(),
            gateway: gw,
            welcome: welcome_state,
            ollama_client,
            notifs: NotificationState::default(),
            show_about: false,
        }
    }
}

impl App {
    fn palette(&self) -> OpenClawPalette {
        OpenClawPalette::from_mode(self.theme_mode)
    }

    fn theme(&self) -> Theme {
        let p = self.palette();
        Theme::custom("OpenClaw".into(), p.to_iced_palette())
    }

    fn send_message(&mut self) {
        if !self.chat_input.is_empty() {
            let user_msg = ChatMessage::new(true, self.chat_input.clone());
            self.chat_messages.push(user_msg);

            self.gateway.send_message(&self.chat_input);
            self.agent_thinking = true;

            self.chat_input.clear();
        }
    }

    fn process_gateway_events(&mut self) {
        let events = self.gateway.drain_events();
        for event in events {
            match event {
                GatewayEvent::AgentResponse(text) => {
                    self.agent_thinking = false;
                    if !text.is_empty() {
                        let msg_index = self.chat_messages.len();
                        let agent_msg = ChatMessage::new(false, text.clone());
                        self.chat_messages.push(agent_msg);
                        if !self.show_chat {
                            self.notifs.push(msg_index, &text);
                        }
                    }
                }
                GatewayEvent::Notification {
                    channel,
                    title,
                    body,
                } => {
                    let card_type = match channel.to_lowercase().as_str() {
                        "telegram" | "whatsapp" | "discord" => CardType::Message,
                        "calendar" => CardType::Alert,
                        _ => CardType::Status,
                    };
                    self.cards
                        .push(Card::new(card_type, title, body));
                }
                GatewayEvent::ConnectionStatus(connected) => {
                    self.connected = connected;
                }
            }
        }
    }

    fn process_ollama_events(&mut self) {
        let events = self.ollama_client.drain_events();
        for event in events {
            match event {
                OllamaEvent::StatusChecked(status) => {
                    let detected = matches!(status, OllamaStatus::Running(_));
                    self.welcome.ollama_status = status;
                    if detected {
                        self.ollama_client.list_models();
                    }
                }
                OllamaEvent::ModelsListed(models) => {
                    self.welcome.available_models =
                        models.iter().map(|m| m.display_name()).collect();
                    if self.welcome.selected_model.is_none() && !models.is_empty() {
                        self.welcome.selected_model = Some(models[0].name.clone());
                    }
                }
                OllamaEvent::ModelPullProgress { percent, status, .. } => {
                    self.welcome.pull_progress = percent;
                    self.welcome.pull_status = status;
                }
                OllamaEvent::ModelPullComplete(model) => {
                    self.welcome.pulling_model = false;
                    self.welcome.pull_complete = true;
                    self.welcome.pull_status = format!("{} downloaded!", model);
                    self.ollama_client.list_models();
                }
                OllamaEvent::ModelPullError { error, .. } => {
                    self.welcome.pulling_model = false;
                    self.welcome.pull_error = Some(error);
                }
                OllamaEvent::Error(e) => {
                    eprintln!("[ollama] Error: {}", e);
                }
            }
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(_now) => {
                self.particles.tick(self.window_size.0, self.window_size.1);
                for card in &mut self.cards {
                    card.tick();
                }
                self.gateway.tick();
                self.process_gateway_events();
                self.notifs.tick();
                if matches!(self.view, AppView::Welcome) {
                    self.process_ollama_events();
                }
            }
            Message::Card(CardMessage::Dismiss(i)) => {
                if i < self.cards.len() {
                    self.cards.remove(i);
                }
            }
            Message::Welcome(welcome_msg) => {
                let pull_request = if let WelcomeMessage::PullModel(ref model) = welcome_msg {
                    Some(model.clone())
                } else {
                    None
                };

                let was_ollama_step = matches!(self.welcome.step, WizardStep::OllamaSetup);
                let finished = welcome::update_welcome(&mut self.welcome, welcome_msg);
                let is_ollama_step = matches!(self.welcome.step, WizardStep::OllamaSetup);

                if let Some(model) = pull_request {
                    self.ollama_client.pull_model(&model);
                }

                if !was_ollama_step && is_ollama_step {
                    self.ollama_client.check_status();
                }

                if finished {
                    if let Some(token) = welcome::write_wizard_config(&self.welcome) {
                        let new_config = GatewayConfig {
                            mock: false,
                            host: self.gateway.config.host.clone(),
                            port: self.gateway.config.port,
                            token: Some(token),
                        };
                        self.gateway = Gateway::new(new_config);
                    }
                    self.view = AppView::Ambient;
                }
            }
            Message::Notification(notif_msg) => match notif_msg {
                NotificationMessage::TogglePanel => {
                    self.notifs.panel_open = !self.notifs.panel_open;
                }
                NotificationMessage::ClickNotification(msg_index) => {
                    if let Some(idx) = self.notifs.notifications.iter().position(|n| n.message_index == msg_index) {
                        self.notifs.mark_read(idx);
                    }
                    self.notifs.panel_open = false;
                    // Open chat overlay to show the conversation
                    self.show_chat = true;
                }
                NotificationMessage::DismissToast => {
                    if let Some(idx) = self.notifs.notifications.iter().rposition(|n| n.toast_visible()) {
                        self.notifs.mark_read(idx);
                    }
                }
                NotificationMessage::MarkAllRead => {
                    self.notifs.mark_all_read();
                    self.notifs.panel_open = false;
                }
            },
            Message::Chat(chat_msg) => match chat_msg {
                ChatOverlayMessage::Toggle => {
                    self.show_chat = !self.show_chat;
                }
                ChatOverlayMessage::InputChanged(val) => {
                    self.chat_input = val;
                }
                ChatOverlayMessage::Submit => {
                    self.send_message();
                    self.notifs.mark_all_read();
                }
                ChatOverlayMessage::LinkClicked(url) => {
                    let _ = open::that(url.as_str());
                }
            },
            Message::Logo(logo_msg) => match logo_msg {
                LogoAction::ShowAbout => {
                    self.show_about = true;
                }
            },
            Message::About(about_msg) => match about_msg {
                about::AboutMessage::Close => {
                    self.show_about = false;
                }
                about::AboutMessage::OpenTerminal => {
                    self.show_about = false;
                    let _ = std::process::Command::new("foot").spawn();
                }
                about::AboutMessage::OpenBrowser => {
                    self.show_about = false;
                    let _ = std::process::Command::new("chromium").spawn();
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let palette = self.palette();

        let bg = container(Space::new(Length::Fill, Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme: &_| container::Style {
                background: Some(iced::Background::Color(palette.bg_deep)),
                ..Default::default()
            });

        let particles = self
            .particles
            .view()
            .map(|_: ()| Message::Tick(Instant::now()));

        let main_content: Element<Message> = match &self.view {
            AppView::Welcome => {
                welcome::view_welcome(&self.welcome, &palette)
                    .map(Message::Welcome)
            }
            AppView::Ambient => {
                let clock =
                    ambient::view_clock(&palette).map(|_: ()| Message::Tick(Instant::now()));

                let cards_view = cards::view_cards(&self.cards, &palette).map(Message::Card);

                let right_panel = container(cards_view)
                    .padding(Padding::from(theme::GRID * 3.0))
                    .width(400)
                    .height(Length::Fill);

                let left = column![clock]
                    .width(Length::Fill)
                    .height(Length::Fill);

                let base_layout: Element<Message> = row![left, right_panel]
                    .height(Length::Fill)
                    .into();

                // Logo button (top-left) + FAB (bottom-right) overlay
                let logo_btn = spotlight::view_logo_button(&palette).map(Message::Logo);
                let fab = spotlight::view_fab(&palette).map(Message::Chat);

                let with_buttons: Element<Message> = stack![base_layout, logo_btn, fab]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into();

                // Notification overlay
                if self.notifs.panel_open {
                    let panel = notifications::view_panel(&self.notifs, &palette)
                        .map(Message::Notification);
                    stack![with_buttons, panel]
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                } else if let Some(toast) = self.notifs.active_toast() {
                    let toast_view = notifications::view_toast(toast, &palette)
                        .map(Message::Notification);
                    stack![with_buttons, toast_view]
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                } else {
                    with_buttons
                }
            }
        };

        let layout = column![
            container(main_content)
                .width(Length::Fill)
                .height(Length::Fill),
        ]
        .height(Length::Fill);

        let base = stack![bg, particles, layout]
            .width(Length::Fill)
            .height(Length::Fill);

        // Layer: chat spotlight overlay
        let with_chat = if self.show_chat {
            let chat_overlay = spotlight::view_spotlight(
                &self.chat_input,
                &self.chat_messages,
                self.agent_thinking,
                self.connected,
                &palette,
            )
            .map(Message::Chat);
            stack![base, chat_overlay]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            base.into()
        };

        // Layer: about modal (on top of everything)
        if self.show_about {
            let about_overlay = about::view_about(self.connected, self.agent_active, &palette)
                .map(Message::About);
            stack![with_chat, about_overlay]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            with_chat
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick(Instant::now()))
    }
}
