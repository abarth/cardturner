use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
}

impl OllamaClient {
    pub fn new(url: String, model: String) -> Self {
        Self {
            url,
            model,
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    stream: bool,
    format: &'a str,
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: Option<ChatResponseMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    #[allow(dead_code)]
    role: Option<String>,
    content: String,
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
            stream: false,
            format: "json",
        };
        let resp = self.http.post(&endpoint).json(&body).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(LlmError::BadStatus {
                status: status.as_u16(),
                body,
            });
        }
        let parsed: ChatResponse = resp.json().await?;
        parsed
            .message
            .map(|m| m.content)
            .ok_or(LlmError::MissingContent)
    }
}
