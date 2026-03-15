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
use widgets::particle_field::ParticleField;

fn main() -> iced::Result {
    iced::application("OpenClaw OS", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| {
            Theme::custom(
                "OpenClaw".into(),
                iced::theme::Palette {
                    background: theme::BACKGROUND,
                    text: theme::TEXT_PRIMARY,
                    primary: theme::PRIMARY,
                    success: theme::SUCCESS,
                    danger: theme::ERROR,
                },
            )
        })
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
    chat_input: String,
    show_dock: bool,
    connected: bool,
    agent_active: bool,
    window_size: (f32, f32),
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
        // Demo cards
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
            chat_input: String::new(),
            show_dock: true,
            connected: true,
            agent_active: true,
            window_size: (1280.0, 720.0),
        }
    }
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(_now) => {
                self.particles.tick(self.window_size.0, self.window_size.1);
                for card in &mut self.cards {
                    card.tick();
                }
                // Typewriter effect for chat messages
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
                    self.chat_input = val;
                }
                ConversationMessage::Submit => {
                    if !self.chat_input.is_empty() {
                        let user_msg = ChatMessage::new(true, self.chat_input.clone());
                        self.chat_messages.push(user_msg);

                        // Demo agent response
                        let response = format!(
                            "I heard you say: \"{}\". I'm the OpenClaw agent — voice pipeline coming soon!",
                            self.chat_input
                        );
                        let agent_msg = ChatMessage::new(false, response);
                        self.chat_messages.push(agent_msg);

                        self.chat_input.clear();
                    }
                }
            },
            Message::Dock(dock_msg) => match dock_msg {
                DockMessage::ToggleVoice => {
                    // Placeholder: toggle back to ambient
                    self.view = match self.view {
                        AppView::Ambient => AppView::Ambient,
                        AppView::Conversation => AppView::Ambient,
                    };
                }
                DockMessage::ToggleText => {
                    self.view = match self.view {
                        AppView::Ambient => AppView::Conversation,
                        AppView::Conversation => AppView::Ambient,
                    };
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let bg = container(Space::new(Length::Fill, Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &_| container::Style {
                background: Some(iced::Background::Color(theme::BACKGROUND)),
                ..Default::default()
            });

        let particles = self.particles.view().map(|_: ()| Message::Tick(Instant::now()));

        let main_content: Element<Message> = match &self.view {
            AppView::Ambient => {
                let clock = ambient::view_clock().map(|_: ()| Message::Tick(Instant::now()));
                let status = ambient::view_status_dots(self.connected, self.agent_active)
                    .map(|_: ()| Message::Tick(Instant::now()));

                let cards_view = cards::view_cards(&self.cards).map(Message::Card);

                let right_panel = container(cards_view)
                    .padding(Padding::from(theme::GRID * 3.0))
                    .width(400)
                    .height(Length::Fill);

                let status_bar = container(status)
                    .padding(Padding::from(theme::GRID * 2.0));

                let left = column![status_bar, clock]
                    .width(Length::Fill)
                    .height(Length::Fill);

                row![left, right_panel].height(Length::Fill).into()
            }
            AppView::Conversation => {
                conversation::view_conversation(&self.chat_messages, &self.chat_input)
                    .map(Message::Conversation)
            }
        };

        let dock_view = dock::view_dock(self.show_dock).map(Message::Dock);

        let layout = column![
            container(main_content)
                .width(Length::Fill)
                .height(Length::Fill),
            dock_view,
        ]
        .height(Length::Fill);

        stack![
            bg,
            particles,
            layout,
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        // 60fps tick for animations
        iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick(Instant::now()))
    }
}
