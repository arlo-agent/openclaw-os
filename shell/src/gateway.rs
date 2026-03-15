//! Gateway integration — communicates with OpenClaw gateway (real or mock).
//!
//! Mock mode: simulates agent responses with canned replies.
//! Real mode: connects to OpenClaw gateway via HTTP.

use std::time::{Duration, Instant};

/// Events produced by the gateway for the app to consume
#[derive(Debug, Clone)]
pub enum GatewayEvent {
    /// Agent responded to a user message
    AgentResponse(String),
    /// A notification card arrived (channel, title, body)
    Notification {
        channel: String,
        title: String,
        body: String,
    },
    /// Connection status changed
    ConnectionStatus(bool),
}

/// Configuration for the gateway connection
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub mock: bool,
    pub base_url: String,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        let mock = std::env::var("OPENCLAW_MOCK")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
            || std::env::args().any(|a| a == "--mock");

        let base_url = std::env::var("OPENCLAW_GATEWAY_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        Self { mock, base_url }
    }
}

/// Gateway client — handles mock and real communication
pub struct Gateway {
    pub config: GatewayConfig,
    /// Pending events to be consumed by the app
    pending_events: Vec<GatewayEvent>,
    /// For mock mode: schedule fake notifications
    last_mock_notification: Option<Instant>,
    mock_notification_index: usize,
}

impl Gateway {
    pub fn new(config: GatewayConfig) -> Self {
        Self {
            config,
            pending_events: Vec::new(),
            last_mock_notification: None,
            mock_notification_index: 0,
        }
    }

    /// Send a user message to the agent.
    /// In mock mode, generates a canned response.
    /// In real mode, would POST to the gateway.
    pub fn send_message(&mut self, message: &str) {
        if self.config.mock {
            let response = self.mock_response(message);
            self.pending_events
                .push(GatewayEvent::AgentResponse(response));
        } else {
            // Real mode: HTTP POST to gateway
            // For now, fall back to mock response since we can't do async here easily
            // TODO: Use iced::Command for async HTTP
            let response = format!(
                "Gateway at {} received: \"{}\" (HTTP integration pending)",
                self.config.base_url, message
            );
            self.pending_events
                .push(GatewayEvent::AgentResponse(response));
        }
    }

    /// Tick — called each frame. Generates periodic mock events.
    pub fn tick(&mut self) {
        if self.config.mock {
            self.tick_mock();
        }
    }

    /// Drain all pending events
    pub fn drain_events(&mut self) -> Vec<GatewayEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Whether we're in mock mode
    pub fn is_mock(&self) -> bool {
        self.config.mock
    }

    // --- Mock implementation ---

    fn mock_response(&self, message: &str) -> String {
        let lower = message.to_lowercase();
        if lower.contains("hello") || lower.contains("hi") {
            "Hey! I'm the OpenClaw agent running in mock mode. How can I help?".to_string()
        } else if lower.contains("weather") {
            "It's currently 14°C and cloudy in Dublin. Typical! ☁️".to_string()
        } else if lower.contains("time") {
            let now = chrono::Local::now();
            format!("It's currently {}.", now.format("%H:%M on %A, %B %-d"))
        } else if lower.contains("status") {
            "All systems nominal. Gateway: mock mode. Channels: none connected.".to_string()
        } else {
            format!(
                "I heard you say: \"{}\". I'm running in mock mode — connect a real gateway for full agent capabilities.",
                message
            )
        }
    }

    fn tick_mock(&mut self) {
        let now = Instant::now();
        let interval = Duration::from_secs(30);

        let should_notify = match self.last_mock_notification {
            None => true,
            Some(last) => now.duration_since(last) >= interval,
        };

        if should_notify && self.mock_notification_index < MOCK_NOTIFICATIONS.len() {
            let (channel, title, body) = MOCK_NOTIFICATIONS[self.mock_notification_index];
            self.pending_events.push(GatewayEvent::Notification {
                channel: channel.to_string(),
                title: title.to_string(),
                body: body.to_string(),
            });
            self.mock_notification_index += 1;
            self.last_mock_notification = Some(now);
        }
    }
}

const MOCK_NOTIFICATIONS: &[(&str, &str, &str)] = &[
    (
        "Telegram",
        "Message from Francis",
        "How's the shell UI coming along?",
    ),
    (
        "System",
        "NixOS Update",
        "Generation 43 built successfully. Restart to apply.",
    ),
    (
        "Calendar",
        "Upcoming Event",
        "Team standup in 15 minutes.",
    ),
    (
        "GitHub",
        "PR Review",
        "New review comment on openclaw-os #42.",
    ),
];
