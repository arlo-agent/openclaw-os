mod ambient;
mod cards;
mod conversation;
mod dock;
mod theme;
mod widgets;

use cards::{Card, CardMessage, CardType};
use conversation::{ChatMessage, ConversationMessage};
use dock::DockMessage;
use iced::widget::{column, container, row, stack, Space};
use iced::{Element, Length, Padding, Size, Subscription, Theme};
use std::time::{Duration, Instant};
use theme::{OpenClawPalette, ThemeMode};
use widgets::particle_field::ParticleField;

fn main() -> iced::Result {
    iced::application("OpenClaw OS", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .window_size(Size::new(1280.0, 720.0))
        .antialiasing(true)
        .run()
}

#[derive(Debug, Clone)]
enum AppView {
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
    listening: bool,
    window_size: (f32, f32),
    theme_mode: ThemeMode,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    Card(CardMessage),
    Conversation(ConversationMessage),
    Dock(DockMessage),
}

impl Default for App {
    fn default() -> Self {
        let demo_cards = vec![
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
        ];

        Self {
            view: AppView::Ambient,
            particles: ParticleField::new(),
            cards: demo_cards,
            chat_messages: Vec::new(),
            dock_input: String::new(),
            connected: true,
            agent_active: true,
            listening: false,
            window_size: (1280.0, 720.0),
            theme_mode: ThemeMode::default(),
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

            let response = format!(
                "I heard you say: \"{}\". I'm the OpenClaw agent — voice pipeline coming soon!",
                self.dock_input
            );
            let agent_msg = ChatMessage::new(false, response);
            self.chat_messages.push(agent_msg);

            self.dock_input.clear();

            // Switch to conversation view when sending a message
            self.view = AppView::Conversation;
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(_now) => {
                self.particles.tick(self.window_size.0, self.window_size.1);
                for card in &mut self.cards {
                    card.tick();
                }
                for msg in &mut self.chat_messages {
                    if !msg.is_fully_revealed() {
                        msg.tick_typewriter();
                    }
                }
            }
            Message::Card(CardMessage::Dismiss(i)) => {
                if i < self.cards.len() {
                    self.cards.remove(i);
                }
            }
            Message::Conversation(conv_msg) => match conv_msg {
                ConversationMessage::InputChanged(val) => {
                    self.dock_input = val;
                }
                ConversationMessage::Submit => {
                    self.send_message();
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

        let main_content: Element<Message> = match &self.view {
            AppView::Ambient => {
                let clock =
                    ambient::view_clock(&palette).map(|_: ()| Message::Tick(Instant::now()));
                let status =
                    ambient::view_status_dots(self.connected, self.agent_active, &palette)
                        .map(|_: ()| Message::Tick(Instant::now()));

                let cards_view = cards::view_cards(&self.cards, &palette).map(Message::Card);

                let right_panel = container(cards_view)
                    .padding(Padding::from(theme::GRID * 3.0))
                    .width(400)
                    .height(Length::Fill);

                let status_bar =
                    container(status).padding(Padding::from(theme::GRID * 2.0));

                let left = column![status_bar, clock]
                    .width(Length::Fill)
                    .height(Length::Fill);

                row![left, right_panel].height(Length::Fill).into()
            }
            AppView::Conversation => {
                conversation::view_conversation(&self.chat_messages, &self.dock_input, &palette)
                    .map(Message::Conversation)
            }
        };

        let dock_view =
            dock::view_dock(&self.dock_input, self.listening, &palette).map(Message::Dock);

        let layout = column![
            container(main_content)
                .width(Length::Fill)
                .height(Length::Fill),
            dock_view,
        ]
        .height(Length::Fill);

        stack![bg, particles, layout]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick(Instant::now()))
    }
}
