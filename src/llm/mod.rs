use crate::error::DiffguardError;
use serde::{Deserialize, Serialize};

pub mod deepseek;
pub mod factory;

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
pub struct ChatMessageResponse {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, Clone)]
pub enum Provider {
    DeepSeek(deepseek::DeepSeekClient),
}

impl Provider {
    pub async fn chat_completion(
        &self,
        system_prompt: &str,
        user_message: &str,
        temperature: f32,
    ) -> Result<String, DiffguardError> {
        match self {
            Provider::DeepSeek(client) => {
                client
                    .chat_completion(system_prompt, user_message, temperature)
                    .await
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LlmError {
    pub provider: String,
    pub status: u16,
    pub message: String,
}

impl From<LlmError> for DiffguardError {
    fn from(err: LlmError) -> Self {
        DiffguardError::LlmApi {
            provider: err.provider,
            status: err.status,
            message: err.message,
        }
    }
}
