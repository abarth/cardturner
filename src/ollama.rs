use std::io::Write;
use std::time::Instant;

use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, trace};

/// Abstracts a chat-style LLM. Production code uses `OllamaClient`; tests use a mock.
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn chat(&self, system: &str, user: &str) -> Result<String, LlmError>;
}

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("HTTP error talking to Ollama: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Ollama returned status {status}: {body}")]
    BadStatus { status: u16, body: String },
    #[error("Ollama response missing message content")]
    MissingContent,
}

pub struct OllamaClient {
    url: String,
    model: String,
    http: reqwest::Client,
    show_thinking: bool,
    show_content_stream: bool,
}

impl OllamaClient {
    pub fn new(url: String, model: String) -> Self {
        Self {
            url,
            model,
            http: reqwest::Client::new(),
            show_thinking: false,
            show_content_stream: false,
        }
    }

    /// Stream the model's `thinking` content to stderr as it arrives.
    /// Thinking is always *requested* from the model (so reasoning-capable models
    /// always reason); this flag only controls whether we display the transcript.
    /// Non-thinking models simply don't return a thinking field.
    pub fn show_thinking(mut self, on: bool) -> Self {
        self.show_thinking = on;
        self
    }

    /// Stream content tokens to stderr as they arrive (so the user can see the
    /// JSON forming token-by-token). Independent of `show_thinking`.
    pub fn show_content_stream(mut self, on: bool) -> Self {
        self.show_content_stream = on;
        self
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    stream: bool,
    format: &'a str,
    /// Always `true`. For reasoning-capable models this enables the thinking
    /// step (which materially improves answer quality); other models ignore it.
    /// Whether the user *sees* the thinking is controlled by `show_thinking`.
    think: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize, Default)]
struct StreamChunk {
    #[serde(default)]
    message: Option<StreamMessage>,
    #[serde(default)]
    done: bool,
}

#[derive(Debug, Deserialize, Default)]
struct StreamMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    thinking: Option<String>,
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn chat(&self, system: &str, user: &str) -> Result<String, LlmError> {
        let endpoint = format!("{}/api/chat", self.url.trim_end_matches('/'));
        let body = ChatRequest {
            model: &self.model,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: system,
                },
                ChatMessage {
                    role: "user",
                    content: user,
                },
            ],
            stream: true,
            format: "json",
            think: true,
        };

        debug!(model = %self.model, url = %endpoint, "sending streaming chat request to Ollama");
        debug!(
            system_chars = system.len(),
            user_chars = user.len(),
            "request payload sizes"
        );
        trace!(system = %system, user = %user, "full request messages");

        let started = Instant::now();
        let resp = self.http.post(&endpoint).json(&body).send().await?;
        let status = resp.status();
        debug!(
            status = status.as_u16(),
            elapsed_ms = started.elapsed().as_millis() as u64,
            "received HTTP headers from Ollama"
        );
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(LlmError::BadStatus {
                status: status.as_u16(),
                body,
            });
        }

        let mut stream = resp.bytes_stream();
        let mut buffer: Vec<u8> = Vec::new();
        let mut content = String::new();
        let mut thinking = String::new();
        let mut printed_thinking_header = false;
        let mut printed_content_header = false;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            buffer.extend_from_slice(&chunk);

            while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buffer.drain(..=pos).collect();
                let line_str = match std::str::from_utf8(&line[..line.len() - 1]) {
                    Ok(s) => s.trim(),
                    Err(_) => continue,
                };
                if line_str.is_empty() {
                    continue;
                }

                let parsed: StreamChunk = match serde_json::from_str(line_str) {
                    Ok(p) => p,
                    Err(e) => {
                        debug!(error = %e, line = %line_str, "skipping unparseable stream line");
                        continue;
                    }
                };

                if let Some(msg) = parsed.message {
                    if let Some(t) = msg.thinking.filter(|s| !s.is_empty()) {
                        thinking.push_str(&t);
                        if self.show_thinking {
                            if !printed_thinking_header {
                                eprintln!("--- model thinking ---");
                                printed_thinking_header = true;
                            }
                            eprint!("{t}");
                            let _ = std::io::stderr().flush();
                        }
                    }
                    if let Some(c) = msg.content.filter(|s| !s.is_empty()) {
                        content.push_str(&c);
                        if self.show_content_stream {
                            if !printed_content_header {
                                // Close the thinking section's last (possibly partial) line.
                                if printed_thinking_header {
                                    eprintln!();
                                }
                                eprintln!("--- model response ---");
                                printed_content_header = true;
                            }
                            eprint!("{c}");
                            let _ = std::io::stderr().flush();
                        }
                    }
                }

                if parsed.done && (printed_thinking_header || printed_content_header) {
                    eprintln!();
                    eprintln!("----------------------");
                }
            }
        }

        let total_ms = started.elapsed().as_millis() as u64;
        debug!(
            elapsed_ms = total_ms,
            response_chars = content.len(),
            thinking_chars = thinking.len(),
            "got chat response"
        );
        trace!(response = %content, thinking = %thinking, "raw response content");

        if content.is_empty() {
            return Err(LlmError::MissingContent);
        }
        Ok(content)
    }
}
