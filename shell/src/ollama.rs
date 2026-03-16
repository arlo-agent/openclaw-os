//! Ollama integration — detect, list models, pull models, generate config.
//!
//! All network calls run in spawned threads; communication via mpsc channels
//! (same pattern as gateway.rs).

use serde::{Deserialize, Serialize};
use std::sync::mpsc;

/// Status of the local Ollama installation
#[derive(Debug, Clone, PartialEq)]
pub enum OllamaStatus {
    Unknown,
    Checking,
    Running(String), // version
    NotInstalled,
    NotRunning, // installed but service not up
}

/// A model available in Ollama
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub parameter_size: Option<String>,
    pub quantization: Option<String>,
}

impl OllamaModel {
    pub fn display_size(&self) -> String {
        let gb = self.size as f64 / 1_073_741_824.0;
        if gb >= 1.0 {
            format!("{:.1} GB", gb)
        } else {
            let mb = self.size as f64 / 1_048_576.0;
            format!("{:.0} MB", mb)
        }
    }

    pub fn display_name(&self) -> String {
        if let Some(ref ps) = self.parameter_size {
            format!("{} ({})", self.name, ps)
        } else {
            self.name.clone()
        }
    }
}

/// Events sent from background threads to the main app
#[derive(Debug, Clone)]
pub enum OllamaEvent {
    StatusChecked(OllamaStatus),
    ModelsListed(Vec<OllamaModel>),
    ModelPullProgress {
        model: String,
        percent: f32,
        status: String,
    },
    ModelPullComplete(String),
    ModelPullError {
        model: String,
        error: String,
    },
    Error(String),
}

/// Client for interacting with the local Ollama instance
pub struct OllamaClient {
    base_url: String,
    event_tx: mpsc::Sender<OllamaEvent>,
    event_rx: mpsc::Receiver<OllamaEvent>,
}

impl OllamaClient {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            base_url: "http://127.0.0.1:11434".to_string(),
            event_tx: tx,
            event_rx: rx,
        }
    }

    /// Check if Ollama is running (background thread)
    pub fn check_status(&self) {
        let url = format!("{}/api/version", self.base_url);
        let tx = self.event_tx.clone();

        std::thread::spawn(move || {
            match ureq::get(&url).call() {
                Ok(resp) => {
                    if let Ok(body) = resp.into_string() {
                        // Response is {"version":"0.6.2"}
                        let version = serde_json::from_str::<serde_json::Value>(&body)
                            .ok()
                            .and_then(|v| v.get("version").and_then(|s| s.as_str()).map(String::from))
                            .unwrap_or_else(|| "unknown".to_string());
                        let _ = tx.send(OllamaEvent::StatusChecked(OllamaStatus::Running(version)));
                    } else {
                        let _ = tx.send(OllamaEvent::StatusChecked(OllamaStatus::Running(
                            "unknown".to_string(),
                        )));
                    }
                }
                Err(_) => {
                    // Check if the binary exists
                    let installed = std::process::Command::new("which")
                        .arg("ollama")
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false);

                    let status = if installed {
                        OllamaStatus::NotRunning
                    } else {
                        OllamaStatus::NotInstalled
                    };
                    let _ = tx.send(OllamaEvent::StatusChecked(status));
                }
            }
        });
    }

    /// List locally available models (background thread)
    pub fn list_models(&self) {
        let url = format!("{}/api/tags", self.base_url);
        let tx = self.event_tx.clone();

        std::thread::spawn(move || {
            match ureq::get(&url).call() {
                Ok(resp) => {
                    if let Ok(body) = resp.into_string() {
                        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&body);
                        let models = parsed
                            .ok()
                            .and_then(|v| {
                                v.get("models").and_then(|m| m.as_array()).map(|arr| {
                                    arr.iter()
                                        .filter_map(|item| {
                                            let name =
                                                item.get("name")?.as_str()?.to_string();
                                            let size =
                                                item.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
                                            let parameter_size = item
                                                .get("details")
                                                .and_then(|d| d.get("parameter_size"))
                                                .and_then(|s| s.as_str())
                                                .map(String::from);
                                            let quantization = item
                                                .get("details")
                                                .and_then(|d| d.get("quantization_level"))
                                                .and_then(|s| s.as_str())
                                                .map(String::from);
                                            Some(OllamaModel {
                                                name,
                                                size,
                                                parameter_size,
                                                quantization,
                                            })
                                        })
                                        .collect::<Vec<_>>()
                                })
                            })
                            .unwrap_or_default();
                        let _ = tx.send(OllamaEvent::ModelsListed(models));
                    }
                }
                Err(e) => {
                    let _ = tx.send(OllamaEvent::Error(format!("Failed to list models: {}", e)));
                }
            }
        });
    }

    /// Pull (download) a model with streaming progress (background thread)
    pub fn pull_model(&self, model_name: &str) {
        let url = format!("{}/api/pull", self.base_url);
        let tx = self.event_tx.clone();
        let model = model_name.to_string();

        std::thread::spawn(move || {
            let body = serde_json::json!({
                "name": model,
                "stream": true
            });

            match ureq::post(&url)
                .set("Content-Type", "application/json")
                .send_string(&body.to_string())
            {
                Ok(resp) => {
                    use std::io::BufRead;
                    let reader = std::io::BufReader::new(resp.into_reader());
                    for line in reader.lines() {
                        match line {
                            Ok(text) if !text.is_empty() => {
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                                    let status_str = v
                                        .get("status")
                                        .and_then(|s| s.as_str())
                                        .unwrap_or("")
                                        .to_string();

                                    if status_str == "success" {
                                        let _ = tx.send(OllamaEvent::ModelPullComplete(
                                            model.clone(),
                                        ));
                                        return;
                                    }

                                    let total =
                                        v.get("total").and_then(|t| t.as_f64()).unwrap_or(0.0);
                                    let completed = v
                                        .get("completed")
                                        .and_then(|c| c.as_f64())
                                        .unwrap_or(0.0);
                                    let percent = if total > 0.0 {
                                        (completed / total * 100.0) as f32
                                    } else {
                                        0.0
                                    };

                                    let _ = tx.send(OllamaEvent::ModelPullProgress {
                                        model: model.clone(),
                                        percent,
                                        status: status_str,
                                    });
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(OllamaEvent::ModelPullError {
                                    model: model.clone(),
                                    error: format!("Stream read error: {}", e),
                                });
                                return;
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(OllamaEvent::ModelPullError {
                        model,
                        error: format!("Pull request failed: {}", e),
                    });
                }
            }
        });
    }

    /// Non-blocking drain of pending events
    pub fn drain_events(&self) -> Vec<OllamaEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.event_rx.try_recv() {
            events.push(event);
        }
        events
    }

    /// Recommended models for first-time setup
    pub fn recommended_models() -> Vec<(&'static str, &'static str)> {
        vec![
            ("llama3.3", "Meta's Llama 3.3 8B — great all-rounder"),
            ("mistral", "Mistral 7B — fast and capable"),
            ("deepseek-r1:8b", "DeepSeek R1 8B — strong reasoning"),
            ("phi4-mini", "Microsoft Phi-4 Mini — compact and quick"),
            ("gemma3", "Google Gemma 3 — efficient and accurate"),
        ]
    }
}

/// Generate the openclaw.json config for Ollama as the LLM provider
pub fn generate_ollama_config(model_name: &str) -> serde_json::Value {
    serde_json::json!({
        "llm": {
            "provider": "ollama",
            "model": model_name,
            "baseUrl": "http://127.0.0.1:11434"
        }
    })
}
