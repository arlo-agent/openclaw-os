//! Gateway integration — communicates with OpenClaw gateway (real or mock).
//!
//! The WebSocket connection and handshake run in a background thread to avoid
//! blocking the UI. Read and write use separate channels — no shared mutex on
//! the socket, so no deadlock.

use crate::device_identity::DeviceIdentity;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tungstenite::Message as WsMessage;

/// Events produced by the gateway for the app to consume
#[derive(Debug, Clone)]
pub enum GatewayEvent {
    AgentResponse(String),
    Notification {
        channel: String,
        title: String,
        body: String,
    },
    ConnectionStatus(bool),
}

/// Commands sent from the main thread to the WS writer thread
#[derive(Debug)]
enum WsCommand {
    Send(String), // JSON text to send
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

        Self { mock, host, port, token }
    }
}

fn arg_value(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == key).map(|w| w[1].clone())
}

const SESSION_KEY: &str = "agent:main:webchat";

fn detect_platform() -> &'static str {
    if cfg!(target_os = "macos") { "macos" } else { "linux" }
}

/// Gateway client
pub struct Gateway {
    pub config: GatewayConfig,
    pub connected: bool,
    pending_events: Vec<GatewayEvent>,
    last_mock_notification: Option<Instant>,
    mock_notification_index: usize,
    /// Receives events from the WS reader thread
    event_rx: Option<mpsc::Receiver<GatewayEvent>>,
    /// Sends commands to the WS writer thread
    cmd_tx: Option<mpsc::Sender<WsCommand>>,
    next_id: AtomicU64,
    streaming_buffer: std::collections::HashMap<String, String>,
    last_reconnect_attempt: Option<Instant>,
    device_identity: Option<DeviceIdentity>,
}

impl Gateway {
    pub fn new(config: GatewayConfig) -> Self {
        let mut event_rx = None;
        let mut cmd_tx = None;
        let mut connected = false;

        let device_identity = if !config.mock {
            let identity_path = dirs_or_home().join("device-identity.json");
            match DeviceIdentity::load_or_create(&identity_path) {
                Ok(id) => {
                    eprintln!("[gateway] Device identity: {}", id.device_id);
                    Some(id)
                }
                Err(e) => {
                    eprintln!("[gateway] Device identity error: {}", e);
                    None
                }
            }
        } else {
            None
        };

        if !config.mock {
            let (erx, ctx, conn) = Self::spawn_connection(&config, device_identity.as_ref());
            event_rx = Some(erx);
            cmd_tx = ctx;
            connected = conn;
        }

        Self {
            config,
            connected,
            pending_events: Vec::new(),
            last_mock_notification: None,
            mock_notification_index: 0,
            event_rx,
            cmd_tx,
            next_id: AtomicU64::new(2),
            streaming_buffer: std::collections::HashMap::new(),
            last_reconnect_attempt: None,
            device_identity,
        }
    }

    /// Spawn the WS connection in a background thread.
    /// Returns (event_receiver, command_sender, initially_connected).
    fn spawn_connection(
        config: &GatewayConfig,
        device_identity: Option<&DeviceIdentity>,
    ) -> (mpsc::Receiver<GatewayEvent>, Option<mpsc::Sender<WsCommand>>, bool) {
        let (event_tx, event_rx) = mpsc::channel();
        let (cmd_tx, cmd_rx) = mpsc::channel::<WsCommand>();

        let url_str = format!("ws://{}:{}/", config.host, config.port);
        let token = config.token.clone();
        let identity_data = device_identity.map(|id| (
            id.device_id.clone(),
            id.public_key_pem.clone(),
            id.private_key_pem.clone(),
        ));

        // Do the handshake in a thread so the UI doesn't block
        let event_tx_clone = event_tx.clone();

        std::thread::spawn(move || {
            eprintln!("[gateway] Connecting to {}...", url_str);

            let socket = match tungstenite::connect(&url_str) {
                Ok((s, _)) => s,
                Err(e) => {
                    eprintln!("[gateway] Connect failed: {}", e);
                    let _ = event_tx_clone.send(GatewayEvent::ConnectionStatus(false));
                    return;
                }
            };

            eprintln!("[gateway] WebSocket connected, performing handshake...");

            // Clone the underlying TCP stream BEFORE using the socket, so we can
            // split into independent read/write WebSocket handles after handshake.
            let tcp_clone = match socket.get_ref() {
                tungstenite::stream::MaybeTlsStream::Plain(tcp) => {
                    match tcp.try_clone() {
                        Ok(t) => t,
                        Err(e) => {
                            eprintln!("[gateway] Failed to clone TCP stream: {}", e);
                            let _ = event_tx_clone.send(GatewayEvent::ConnectionStatus(false));
                            return;
                        }
                    }
                }
                _ => {
                    eprintln!("[gateway] TLS not supported yet");
                    let _ = event_tx_clone.send(GatewayEvent::ConnectionStatus(false));
                    return;
                }
            };

            // Use `socket` for the handshake (both read and write), then split after
            let mut ws_read = socket; // will become the reader after handshake

            // Step 1: Read challenge (using the single socket for handshake)
            let challenge_nonce = match ws_read.read() {
                Ok(WsMessage::Text(text)) => {
                    if let Ok(v) = serde_json::from_str::<Value>(&text) {
                        if v.get("event").and_then(|e| e.as_str()) == Some("connect.challenge") {
                            eprintln!("[gateway] Received challenge");
                            v.get("payload")
                                .and_then(|p| p.get("nonce"))
                                .and_then(|n| n.as_str())
                                .unwrap_or("")
                                .to_string()
                        } else {
                            eprintln!("[gateway] Unexpected first message");
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                }
                Err(e) => {
                    eprintln!("[gateway] Failed to read challenge: {}", e);
                    let _ = event_tx_clone.send(GatewayEvent::ConnectionStatus(false));
                    return;
                }
                _ => String::new(),
            };

            // Step 2: Build and send connect
            let token_str = token.as_deref().unwrap_or("");
            let auth = if !token_str.is_empty() {
                json!({"token": token_str})
            } else {
                json!({})
            };

            let platform = detect_platform();
            let scopes = ["operator.read", "operator.write", "operator.admin"];

            let device_obj = if let Some((ref dev_id, ref pub_pem, ref priv_pem)) = identity_data {
                if !challenge_nonce.is_empty() {
                    // Reconstruct identity to sign
                    if let Ok(identity) = DeviceIdentity::from_pems(dev_id, pub_pem, priv_pem) {
                        let (sig, pubkey, signed_at) = identity.sign_challenge(
                            &challenge_nonce, token_str, "cli", "cli", "operator", &scopes, platform,
                        );
                        Some(json!({
                            "id": dev_id,
                            "publicKey": pubkey,
                            "signature": sig,
                            "signedAt": signed_at,
                            "nonce": challenge_nonce
                        }))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let mut params = json!({
                "minProtocol": 3,
                "maxProtocol": 3,
                "client": {
                    "id": "cli",
                    "version": "0.1.0",
                    "platform": platform,
                    "mode": "cli"
                },
                "role": "operator",
                "scopes": scopes,
                "caps": [],
                "auth": auth
            });
            if let Some(device) = device_obj {
                params.as_object_mut().unwrap().insert("device".to_string(), device);
            }

            let connect_req = json!({
                "type": "req",
                "id": "1",
                "method": "connect",
                "params": params
            });

            // Send connect via the same socket (still single-threaded here)
            if let Err(e) = ws_read.send(WsMessage::Text(connect_req.to_string().into())) {
                eprintln!("[gateway] Failed to send connect: {}", e);
                let _ = event_tx_clone.send(GatewayEvent::ConnectionStatus(false));
                return;
            }

            // Step 3: Read hello-ok
            match ws_read.read() {
                Ok(WsMessage::Text(text)) => {
                    if let Ok(v) = serde_json::from_str::<Value>(&text) {
                        if v.get("ok").and_then(|o| o.as_bool()) == Some(true) {
                            eprintln!("[gateway] Connected! Handshake complete.");
                            let _ = event_tx_clone.send(GatewayEvent::ConnectionStatus(true));
                        } else {
                            eprintln!("[gateway] Connect rejected: {}", text);
                            let _ = event_tx_clone.send(GatewayEvent::ConnectionStatus(false));
                            return;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[gateway] Failed to read hello-ok: {}", e);
                    let _ = event_tx_clone.send(GatewayEvent::ConnectionStatus(false));
                    return;
                }
                _ => {}
            }

            // Now split: ws_read keeps the original socket for reading,
            // create a new WebSocket wrapper around the cloned TCP stream for writing
            let mut ws_write = tungstenite::WebSocket::from_raw_socket(
                tungstenite::stream::MaybeTlsStream::Plain(tcp_clone),
                tungstenite::protocol::Role::Client,
                None,
            );

            // Spawn writer thread — reads commands from cmd_rx, sends via ws_write
            std::thread::spawn(move || {
                for cmd in cmd_rx {
                    match cmd {
                        WsCommand::Send(text) => {
                            if let Err(e) = ws_write.send(WsMessage::Text(text.into())) {
                                eprintln!("[gateway] Write error: {}", e);
                                break;
                            }
                        }
                    }
                }
                eprintln!("[gateway] Writer thread exiting");
            });

            // Reader loop — reads from ws_read, sends events to main thread
            loop {
                match ws_read.read() {
                    Ok(WsMessage::Text(text)) => {
                        if let Ok(v) = serde_json::from_str::<Value>(&text) {
                            handle_ws_message(&v, &event_tx_clone);
                        }
                    }
                    Ok(WsMessage::Close(_)) => {
                        eprintln!("[gateway] Connection closed by server");
                        let _ = event_tx_clone.send(GatewayEvent::ConnectionStatus(false));
                        break;
                    }
                    Ok(WsMessage::Ping(_data)) => {
                        // tungstenite from_raw_socket doesn't auto-pong — we need to respond
                        // But we can't write from the reader socket (it's split).
                        // Send a command to the writer thread instead.
                        eprintln!("[gateway] Received ping, ignoring (pong via writer not wired)");
                    }
                    Ok(WsMessage::Pong(_)) => {
                        // Pong received — connection is alive
                    }
                    Err(e) => {
                        eprintln!("[gateway] Read error: {}", e);
                        let _ = event_tx_clone.send(GatewayEvent::ConnectionStatus(false));
                        break;
                    }
                    _ => {}
                }
            }
            eprintln!("[gateway] Connection thread exiting");
        });

        // The connection happens async — we start disconnected and get
        // ConnectionStatus(true) once the handshake completes
        (event_rx, Some(cmd_tx), false)
    }

    fn next_request_id(&self) -> String {
        self.next_id.fetch_add(1, Ordering::Relaxed).to_string()
    }

    pub fn send_message(&mut self, message: &str) {
        if self.config.mock {
            let response = self.mock_response(message);
            self.pending_events.push(GatewayEvent::AgentResponse(response));
            return;
        }

        let req_id = self.next_request_id();
        let req = json!({
            "type": "req",
            "id": req_id,
            "method": "chat.send",
            "params": {
                "message": message,
                "sessionKey": SESSION_KEY,
                "idempotencyKey": format!("{}-{}", req_id, std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis())
            }
        });

        if let Some(ref tx) = self.cmd_tx {
            match tx.send(WsCommand::Send(req.to_string())) {
                Ok(_) => eprintln!("[gateway] Sent chat.send (id={})", req_id),
                Err(e) => {
                    eprintln!("[gateway] Send failed (channel closed): {}", e);
                    self.connected = false;
                    self.pending_events.push(GatewayEvent::AgentResponse(
                        "Connection lost. Message not sent.".to_string(),
                    ));
                }
            }
        } else {
            self.pending_events.push(GatewayEvent::AgentResponse(
                "Not connected to gateway.".to_string(),
            ));
        }
    }

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
                        let parts: Vec<&str> = text.splitn(5, '\x00').collect();
                        if parts.len() == 5 {
                            let run_id = parts[2].to_string();
                            let done = parts[3] == "1";
                            let chunk = parts[4];

                            // Each event contains the FULL text, not incremental — replace, don't append
                            if !chunk.is_empty() {
                                self.streaming_buffer.insert(run_id.clone(), chunk.to_string());
                            }

                            if done {
                                let full_text = self.streaming_buffer.remove(&run_id).unwrap_or_default();
                                if !full_text.is_empty() {
                                    self.pending_events.push(GatewayEvent::AgentResponse(full_text));
                                }
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

        // Reconnect if disconnected
        if !self.connected {
            let should_reconnect = match self.last_reconnect_attempt {
                None => true,
                Some(last) => last.elapsed() >= Duration::from_secs(5),
            };

            if should_reconnect {
                self.last_reconnect_attempt = Some(Instant::now());
                eprintln!("[gateway] Reconnecting...");
                let (erx, ctx, _) = Self::spawn_connection(&self.config, self.device_identity.as_ref());
                self.event_rx = Some(erx);
                self.cmd_tx = ctx;
                // connected will be set to true when ConnectionStatus(true) arrives
            }
        }
    }

    pub fn drain_events(&mut self) -> Vec<GatewayEvent> {
        std::mem::take(&mut self.pending_events)
    }

    pub fn is_mock(&self) -> bool {
        self.config.mock
    }

    fn mock_response(&self, message: &str) -> String {
        let lower = message.to_lowercase();
        if lower.contains("hello") || lower.contains("hi") {
            "Hey! I'm the OpenClaw agent running in mock mode.".to_string()
        } else if lower.contains("weather") {
            "14C and cloudy in Dublin.".to_string()
        } else if lower.contains("time") {
            let now = chrono::Local::now();
            format!("It's {}.", now.format("%H:%M on %A, %B %-d"))
        } else {
            format!("Echo: \"{}\". (mock mode)", message)
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

/// Handle a parsed WebSocket JSON message from the reader thread
fn handle_ws_message(v: &Value, tx: &mpsc::Sender<GatewayEvent>) {
    let msg_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match msg_type {
        "event" => {
            let event_name = v.get("event").and_then(|e| e.as_str()).unwrap_or("");
            if event_name == "tick" {
                // Gateway keep-alive tick — no action needed, just ack by not dying
                return;
            }
            if event_name == "chat" {
                if let Some(payload) = v.get("payload") {
                    let run_id = payload.get("runId").and_then(|r| r.as_str()).unwrap_or("unknown").to_string();
                    let state = payload.get("state").and_then(|s| s.as_str()).unwrap_or("");
                    let done = state == "final";

                    // Extract text from message.content[0].text
                    let text = payload
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|item| item.get("text"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string();

                    eprintln!("[gateway] chat event: state={} runId={} text_len={}", state, run_id, text.len());

                    if !text.is_empty() {
                        let _ = tx.send(GatewayEvent::AgentResponse(format!(
                            "\x00STREAM\x00{}\x00{}\x00{}",
                            run_id,
                            if done { "1" } else { "0" },
                            text
                        )));
                    } else if done {
                        // Final event with no text — flush buffer
                        let _ = tx.send(GatewayEvent::AgentResponse(format!(
                            "\x00STREAM\x00{}\x001\x00", run_id
                        )));
                    }
                }
            }
        }
        "res" => {
            if v.get("ok").and_then(|o| o.as_bool()) == Some(true) {
                // Check for in_flight status (session already has an active run)
                let status = v.get("payload")
                    .and_then(|p| p.get("status"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                if status == "in_flight" {
                    eprintln!("[gateway] Request in_flight — session has an active run, waiting...");
                } else if !status.is_empty() {
                    eprintln!("[gateway] Request ok: status={}", status);
                }
            } else if v.get("ok").and_then(|o| o.as_bool()) == Some(false) {
                let err_msg = v.get("error")
                    .and_then(|e| e.as_str().map(|s| s.to_string())
                        .or_else(|| e.get("message").and_then(|m| m.as_str()).map(|s| s.to_string())))
                    .unwrap_or_else(|| format!("{}", v));
                eprintln!("[gateway] Request failed: {}", err_msg);
                // Send error as agent response so user sees it
                let _ = tx.send(GatewayEvent::AgentResponse(format!("Error: {}", err_msg)));
            }
        }
        _ => {}
    }
}

fn dirs_or_home() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".openclaw-os")
}

const MOCK_NOTIFICATIONS: &[(&str, &str, &str)] = &[
    ("Telegram", "Message from Francis", "How's the shell UI coming along?"),
    ("System", "NixOS Update", "Generation 43 built successfully."),
    ("Calendar", "Upcoming Event", "Team standup in 15 minutes."),
    ("GitHub", "PR Review", "New comment on openclaw-os #42."),
];
