mod ambient;
mod cards;
mod conversation;
mod device_identity;
mod dock;
mod gateway;
mod messaging_setup;
mod notifications;
mod ollama;
mod statusbar;
mod theme;
mod welcome;
mod widgets;

use cards::{Card, CardMessage, CardType};
use conversation::{ChatMessage, ConversationMessage};
use dock::DockMessage;
use gateway::{Gateway, GatewayConfig, GatewayEvent};
use iced::widget::{column, container, row, stack, Space};
use iced::{Alignment, Element, Length, Padding, Size, Subscription, Theme};
use notifications::{NotificationMessage, NotificationState};
use statusbar::StatusBarMessage;
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
    Conversation,
}

struct App {
    view: AppView,
    particles: ParticleField,
    cards: Vec<Card>,
    chat_messages: Vec<ChatMessage>,
    dock_input: String,
    connected: bool,
    agent_active: bool,
    agent_thinking: bool,
    listening: bool,
    window_size: (f32, f32),
    theme_mode: ThemeMode,
    gateway: Gateway,
    welcome: WelcomeState,
    ollama_client: OllamaClient,
    notifs: NotificationState,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    Card(CardMessage),
    Conversation(ConversationMessage),
    Dock(DockMessage),
    Welcome(WelcomeMessage),
    Notification(NotificationMessage),
    StatusBar(StatusBarMessage),
}

impl Default for App {
    fn default() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let config = GatewayConfig::from_args(&args);
        let is_mock = config.mock;
        let gw = Gateway::new(config);
        let connected = gw.connected;

        // Check for --welcome or --onboard flag to launch the wizard
        let show_welcome = args.iter().any(|a| a == "--welcome" || a == "--onboard");

        // Only show demo cards if NOT in mock mode (mock will generate its own)
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

        // Set up Ollama client and kick off status check
        let ollama_client = OllamaClient::new();
        if show_welcome {
            ollama_client.check_status();
        }

        // Detect --auth-choice from CLI args
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
            dock_input: String::new(),
            connected,
            agent_active: true,
            agent_thinking: false,
            listening: false,
            window_size: (1280.0, 720.0),
            theme_mode: ThemeMode::default(),
            gateway: gw,
            welcome: welcome_state,
            ollama_client,
            notifs: NotificationState::default(),
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
        if !self.dock_input.is_empty() {
            let user_msg = ChatMessage::new(true, self.dock_input.clone());
            self.chat_messages.push(user_msg);

            // Send to gateway
            self.gateway.send_message(&self.dock_input);
            self.agent_thinking = true;

            self.dock_input.clear();
            self.view = AppView::Conversation;
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
                        // Create notification if not currently viewing conversation
                        if !matches!(self.view, AppView::Conversation) {
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
                    // Refresh model list
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
                // Gateway tick + process events
                self.gateway.tick();
                self.process_gateway_events();
                // Notification tick (toast timing)
                self.notifs.tick();
                // Ollama tick (during welcome wizard)
                if matches!(self.view, AppView::Welcome) {
                    self.process_ollama_events();
                }
            }
            Message::Card(CardMessage::Dismiss(i)) => {
                if i < self.cards.len() {
                    self.cards.remove(i);
                }
            }
            Message::Conversation(conv_msg) => match conv_msg {
                ConversationMessage::Back => {
                    self.view = AppView::Ambient;
                }
                ConversationMessage::LinkClicked(url) => {
                    // Open URL in default browser
                    let _ = open::that(url.as_str());
                }
            },
            Message::Welcome(welcome_msg) => {
                // Check if this is a PullModel request before update consumes it
                let pull_request = if let WelcomeMessage::PullModel(ref model) = welcome_msg {
                    Some(model.clone())
                } else {
                    None
                };

                let was_ollama_step = matches!(self.welcome.step, WizardStep::OllamaSetup);
                let finished = welcome::update_welcome(&mut self.welcome, welcome_msg);
                let is_ollama_step = matches!(self.welcome.step, WizardStep::OllamaSetup);

                // Trigger model pull if requested
                if let Some(model) = pull_request {
                    self.ollama_client.pull_model(&model);
                }

                if !was_ollama_step && is_ollama_step {
                    self.ollama_client.check_status();
                }

                if finished {
                    // Write config files, start gateway, get the token
                    if let Some(token) = welcome::write_wizard_config(&self.welcome) {
                        // Reconnect using the same host/port from original config,
                        // but with the new token
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
                    // Mark as read, close panel, navigate to conversation
                    // Find the notification with this message_index and mark it read
                    if let Some(idx) = self.notifs.notifications.iter().position(|n| n.message_index == msg_index) {
                        self.notifs.mark_read(idx);
                    }
                    self.notifs.panel_open = false;
                    self.view = AppView::Conversation;
                    // TODO: scroll to msg_index in conversation view
                }
                NotificationMessage::DismissToast => {
                    // Mark the latest toast as read
                    if let Some(idx) = self.notifs.notifications.iter().rposition(|n| n.toast_visible()) {
                        self.notifs.mark_read(idx);
                    }
                }
                NotificationMessage::MarkAllRead => {
                    self.notifs.mark_all_read();
                    self.notifs.panel_open = false;
                }
            },
            Message::StatusBar(sb_msg) => match sb_msg {
                StatusBarMessage::ToggleTheme => {
                    self.theme_mode = self.theme_mode.toggle();
                    self.particles.set_theme_mode(self.theme_mode);
                }
                StatusBarMessage::Notification(notif_msg) => {
                    // Delegate to notification handler
                    match notif_msg {
                        NotificationMessage::TogglePanel => {
                            self.notifs.panel_open = !self.notifs.panel_open;
                        }
                        NotificationMessage::ClickNotification(msg_index) => {
                            if let Some(idx) = self.notifs.notifications.iter().position(|n| n.message_index == msg_index) {
                                self.notifs.mark_read(idx);
                            }
                            self.notifs.panel_open = false;
                            self.view = AppView::Conversation;
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
                    }
                }
            },
            Message::Dock(dock_msg) => match dock_msg {
                DockMessage::ToggleVoice => {
                    self.listening = !self.listening;
                }
                DockMessage::InputChanged(val) => {
                    self.dock_input = val;
                }
                DockMessage::Submit => {
                    self.send_message();
                    // Mark all notifications as read when user sends a message
                    self.notifs.mark_all_read();
                }
                DockMessage::ToggleTheme => {
                    self.theme_mode = self.theme_mode.toggle();
                    self.particles.set_theme_mode(self.theme_mode);
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

        // Global status bar (visible on all screens except welcome wizard)
        let show_statusbar = !matches!(self.view, AppView::Welcome);

        let status_bar_view: Element<Message> = if show_statusbar {
            statusbar::view_statusbar(
                self.connected,
                self.agent_active,
                self.theme_mode,
                &self.notifs,
                &palette,
            )
            .map(Message::StatusBar)
        } else {
            Space::new(0, 0).into()
        };

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

                // Toast notification overlay or notification panel
                let mut left_items: Vec<Element<Message>> = Vec::new();

                if self.notifs.panel_open {
                    let panel = notifications::view_panel(&self.notifs, &palette)
                        .map(Message::Notification);
                    left_items.push(panel);
                } else if let Some(toast) = self.notifs.active_toast() {
                    let toast_view = notifications::view_toast(toast, &palette)
                        .map(Message::Notification);
                    left_items.push(toast_view);
                }

                left_items.push(clock);

                let left = column(left_items)
                    .width(Length::Fill)
                    .height(Length::Fill);

                row![left, right_panel].height(Length::Fill).into()
            }
            AppView::Conversation => {
                conversation::view_conversation(&self.chat_messages, self.agent_thinking, &palette)
                    .map(Message::Conversation)
            }
        };

        let show_dock = !matches!(self.view, AppView::Welcome);

        let layout = if show_dock {
            let dock_view =
                dock::view_dock(&self.dock_input, self.listening, &palette, self.theme_mode)
                    .map(Message::Dock);
            column![
                status_bar_view,
                container(main_content)
                    .width(Length::Fill)
                    .height(Length::Fill),
                dock_view,
            ]
            .height(Length::Fill)
        } else {
            column![
                container(main_content)
                    .width(Length::Fill)
                    .height(Length::Fill),
            ]
            .height(Length::Fill)
        };

        stack![bg, particles, layout]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick(Instant::now()))
    }
}
