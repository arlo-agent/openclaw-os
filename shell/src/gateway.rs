//! Gateway integration — communicates with OpenClaw gateway (real or mock).
//!
//! Mock mode: simulates agent responses with canned replies.
//! Real mode: connects to OpenClaw gateway via WebSocket.

use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use tungstenite::Message as WsMessage;

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
    pub host: String,
    pub port: u16,
    pub token: Option<String>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self::from_args(&std::env::args().collect::<Vec<_>>())
    }
}

impl GatewayConfig {
    pub fn from_args(args: &[String]) -> Self {
        let mock = std::env::var("OPENCLAW_MOCK")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
            || args.iter().any(|a| a == "--mock");

        let host = arg_value(args, "--host")
            .or_else(|| std::env::var("OPENCLAW_HOST").ok())
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let port = arg_value(args, "--port")
            .or_else(|| std::env::var("OPENCLAW_PORT").ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(18789u16);

        let token = arg_value(args, "--token")
            .or_else(|| std::env::var("OPENCLAW_GATEWAY_TOKEN").ok());

        Self {
            mock,
            host,
            port,
            token,
        }
    }
}

/// Extract `--key value` from args
fn arg_value(args: &[String], key: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == key)
        .map(|w| w[1].clone())
}

/// Session key used for all chat messages
const SESSION_KEY: &str = "agent:main:webchat";

/// Shared writer handle for the WebSocket
type WsWriter = Arc<Mutex<Option<tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>>>>;

/// Gateway client — handles mock and real communication
pub struct Gateway {
    pub config: GatewayConfig,
    /// Whether we're connected to the gateway
    pub connected: bool,
    /// Pending events to be consumed by the app
    pending_events: Vec<GatewayEvent>,
    /// For mock mode: schedule fake notifications
    last_mock_notification: Option<Instant>,
    mock_notification_index: usize,
    /// Channel receiver for events from the WS reader thread
    event_rx: Option<mpsc::Receiver<GatewayEvent>>,
    /// WebSocket writer (shared with connect thread, used by main thread to send)
    ws_writer: WsWriter,
    /// Atomic request ID counter
    next_id: Arc<AtomicU64>,
    /// Buffer for streaming responses: runId -> accumulated text
    streaming_buffer: std::collections::HashMap<String, String>,
    /// Track last reconnect attempt
    last_reconnect_attempt: Option<Instant>,
}

impl Gateway {
    pub fn new(config: GatewayConfig) -> Self {
        let ws_writer: WsWriter = Arc::new(Mutex::new(None));
        let next_id = Arc::new(AtomicU64::new(2)); // 1 is used for connect handshake
        let mut event_rx = None;
        let mut connected = false;

        if !config.mock {
            match Self::start_connection(&config, ws_writer.clone(), next_id.clone()) {
                Ok((rx, is_connected)) => {
                    event_rx = Some(rx);
                    connected = is_connected;
                }
                Err(e) => {
                    eprintln!("[gateway] Failed to connect: {}", e);
                }
            }
        }

        Self {
            config,
            connected,
            pending_events: Vec::new(),
            last_mock_notification: None,
            mock_notification_index: 0,
            event_rx,
            ws_writer,
            next_id,
            streaming_buffer: std::collections::HashMap::new(),
            last_reconnect_attempt: None,
        }
    }

    /// Start a WebSocket connection in a background thread
    fn start_connection(
        config: &GatewayConfig,
        ws_writer: WsWriter,
        _next_id: Arc<AtomicU64>,
    ) -> Result<(mpsc::Receiver<GatewayEvent>, bool), String> {
        let url_str = format!("ws://{}:{}/", config.host, config.port);
        eprintln!("[gateway] Connecting to {}...", url_str);

        let (mut socket, _response) =
            tungstenite::connect(&url_str).map_err(|e| format!("WebSocket connect failed: {}", e))?;

        eprintln!("[gateway] WebSocket connected, waiting for challenge...");

        // Step 1: Read the connect.challenge event
        let challenge_msg = socket
            .read()
            .map_err(|e| format!("Failed to read challenge: {}", e))?;

        if let WsMessage::Text(text) = &challenge_msg {
            let v: Value = serde_json::from_str(text)
                .map_err(|e| format!("Bad challenge JSON: {}", e))?;
            if v.get("event").and_then(|e| e.as_str()) == Some("connect.challenge") {
                eprintln!("[gateway] Received connect challenge");
            } else {
                eprintln!("[gateway] Unexpected first message: {}", text);
            }
        }

        // Step 2: Send connect request
        let auth = if let Some(ref token) = config.token {
            json!({"token": token})
        } else {
            json!({})
        };

        let connect_req = json!({
            "type": "req",
            "id": "1",
            "method": "connect",
            "params": {
                "minProtocol": 3,
                "maxProtocol": 3,
                "client": {
                    "id": "cli",
                    "version": "0.1.0",
                    "platform": "linux",
                    "mode": "cli"
                },
                "role": "operator",
                "scopes": ["operator.read", "operator.write", "operator.admin"],
                "caps": [],
                "commands": [],
                "permissions": {},
                "auth": auth
            }
        });

        socket
            .send(WsMessage::Text(connect_req.to_string().into()))
            .map_err(|e| format!("Failed to send connect: {}", e))?;

        eprintln!("[gateway] Sent connect request, waiting for hello-ok...");

        // Step 3: Read connect response
        let connect_resp = socket
            .read()
            .map_err(|e| format!("Failed to read connect response: {}", e))?;

        let mut is_connected = false;
        if let WsMessage::Text(text) = &connect_resp {
            let v: Value = serde_json::from_str(text).unwrap_or_default();
            if v.get("ok").and_then(|o| o.as_bool()) == Some(true) {
                eprintln!("[gateway] Connected! Protocol handshake complete.");
                is_connected = true;
            } else {
                eprintln!("[gateway] Connect rejected: {}", text);
                return Err(format!("Connect rejected: {}", text));
            }
        }

        // Store the writer half — we'll clone the socket for reading
        // tungstenite doesn't split easily, so we share the socket behind a mutex
        // The reader thread will lock to read, main thread locks to write
        {
            let mut writer = ws_writer.lock().unwrap();
            *writer = Some(socket);
        }

        // Step 4: Spawn reader thread
        let (tx, rx) = mpsc::channel();
        let ws_reader = ws_writer.clone();

        std::thread::spawn(move || {
            loop {
                // Lock to read one message
                let msg = {
                    let mut guard = ws_reader.lock().unwrap();
                    if let Some(ref mut ws) = *guard {
                        match ws.read() {
                            Ok(msg) => Some(msg),
                            Err(e) => {
                                eprintln!("[gateway] WS read error: {}", e);
                                *guard = None; // drop the dead socket
                                None
                            }
                        }
                    } else {
                        None
                    }
                };

                match msg {
                    Some(WsMessage::Text(text)) => {
                        if let Ok(v) = serde_json::from_str::<Value>(&text) {
                            Self::handle_ws_message(&v, &tx);
                        }
                    }
                    Some(WsMessage::Close(_)) => {
                        eprintln!("[gateway] WebSocket closed by server");
                        let _ = tx.send(GatewayEvent::ConnectionStatus(false));
                        break;
                    }
                    Some(WsMessage::Ping(data)) => {
                        // Send pong
                        let mut guard = ws_reader.lock().unwrap();
                        if let Some(ref mut ws) = *guard {
                            let _ = ws.send(WsMessage::Pong(data));
                        }
                    }
                    None => {
                        // Socket gone, signal disconnect
                        let _ = tx.send(GatewayEvent::ConnectionStatus(false));
                        break;
                    }
                    _ => {} // Binary, Pong, etc — ignore
                }
            }
            eprintln!("[gateway] Reader thread exiting");
        });

        Ok((rx, is_connected))
    }

    /// Handle a parsed WebSocket JSON message
    fn handle_ws_message(v: &Value, tx: &mpsc::Sender<GatewayEvent>) {
        let msg_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

        match msg_type {
            "event" => {
                let event_name = v.get("event").and_then(|e| e.as_str()).unwrap_or("");
                if event_name == "chat" {
                    if let Some(payload) = v.get("payload") {
                        let kind = payload.get("kind").and_then(|k| k.as_str()).unwrap_or("");
                        if kind == "text" {
                            let text = payload
                                .get("text")
                                .and_then(|t| t.as_str())
                                .unwrap_or("")
                                .to_string();
                            let done = payload
                                .get("done")
                                .and_then(|d| d.as_bool())
                                .unwrap_or(false);
                            let run_id = payload
                                .get("runId")
                                .and_then(|r| r.as_str())
                                .unwrap_or("unknown")
                                .to_string();

                            // Send a streaming chunk event — the main struct will buffer
                            let _ = tx.send(GatewayEvent::AgentResponse(format!(
                                "\x00STREAM\x00{}\x00{}\x00{}",
                                run_id,
                                if done { "1" } else { "0" },
                                text
                            )));
                        }
                    }
                }
            }
            "res" => {
                // Response to a request — we mostly care about errors
                if v.get("ok").and_then(|o| o.as_bool()) == Some(false) {
                    let err_msg = v
                        .get("error")
                        .and_then(|e| {
                            // error can be a string or an object with "message" field
                            e.as_str().map(|s| s.to_string())
                                .or_else(|| e.get("message").and_then(|m| m.as_str()).map(|s| s.to_string()))
                        })
                        .unwrap_or_else(|| format!("unknown error: {}", v));
                    eprintln!("[gateway] Request failed: {}", err_msg);
                }
            }
            _ => {}
        }
    }

    /// Allocate the next request ID
    fn next_request_id(&self) -> String {
        self.next_id.fetch_add(1, Ordering::Relaxed).to_string()
    }

    /// Send a user message to the agent.
    /// In mock mode, generates a canned response.
    /// In real mode, sends via WebSocket.
    pub fn send_message(&mut self, message: &str) {
        if self.config.mock {
            let response = self.mock_response(message);
            self.pending_events
                .push(GatewayEvent::AgentResponse(response));
        } else {
            let req_id = self.next_request_id();
            let req = json!({
                "type": "req",
                "id": req_id,
                "method": "chat.send",
                "params": {
                    "message": message,
                    "sessionKey": SESSION_KEY
                }
            });

            let mut guard = self.ws_writer.lock().unwrap();
            if let Some(ref mut ws) = *guard {
                match ws.send(WsMessage::Text(req.to_string().into())) {
                    Ok(_) => {
                        eprintln!("[gateway] Sent chat.send (id={})", req_id);
                    }
                    Err(e) => {
                        eprintln!("[gateway] Failed to send message: {}", e);
                        *guard = None;
                        self.connected = false;
                        self.pending_events.push(GatewayEvent::ConnectionStatus(false));
                        self.pending_events.push(GatewayEvent::AgentResponse(
                            "⚠ Connection lost. Message not sent.".to_string(),
                        ));
                    }
                }
            } else {
                self.pending_events.push(GatewayEvent::AgentResponse(
                    "⚠ Not connected to gateway.".to_string(),
                ));
            }
        }
    }

    /// Tick — called each frame. Processes incoming events, handles reconnect.
    pub fn tick(&mut self) {
        if self.config.mock {
            self.tick_mock();
            return;
        }

        // Drain events from the reader thread
        if let Some(ref rx) = self.event_rx {
            while let Ok(event) = rx.try_recv() {
                match &event {
                    GatewayEvent::AgentResponse(text) if text.starts_with("\x00STREAM\x00") => {
                        // Parse streaming chunk: \0STREAM\0<runId>\0<done>\0<text>
                        let parts: Vec<&str> = text.splitn(5, '\x00').collect();
                        if parts.len() == 5 {
                            let run_id = parts[2].to_string();
                            let done = parts[3] == "1";
                            let chunk = parts[4];

                            let buffer = self.streaming_buffer.entry(run_id.clone()).or_default();
                            buffer.push_str(chunk);

                            if done {
                                let full_text = self.streaming_buffer.remove(&run_id).unwrap_or_default();
                                self.pending_events
                                    .push(GatewayEvent::AgentResponse(full_text));
                            }
                        }
                    }
                    GatewayEvent::ConnectionStatus(status) => {
                        self.connected = *status;
                        self.pending_events.push(event);
                    }
                    _ => {
                        self.pending_events.push(event);
                    }
                }
            }
        }

        // Attempt reconnect if disconnected
        if !self.connected {
            let should_reconnect = match self.last_reconnect_attempt {
                None => true,
                Some(last) => last.elapsed() >= Duration::from_secs(5),
            };

            if should_reconnect {
                self.last_reconnect_attempt = Some(Instant::now());
                eprintln!("[gateway] Attempting reconnect...");

                match Self::start_connection(&self.config, self.ws_writer.clone(), self.next_id.clone()) {
                    Ok((rx, is_connected)) => {
                        self.event_rx = Some(rx);
                        self.connected = is_connected;
                        if is_connected {
                            self.pending_events.push(GatewayEvent::ConnectionStatus(true));
                            eprintln!("[gateway] Reconnected!");
                        }
                    }
                    Err(e) => {
                        eprintln!("[gateway] Reconnect failed: {}", e);
                    }
                }
            }
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
