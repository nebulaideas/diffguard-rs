use crate::error::DiffguardError;
use crate::llm::{ChatMessage, ChatRequest, ChatResponse, LlmError};
use reqwest::header::{self, HeaderMap, HeaderValue};

const DEFAULT_BASE_URL: &str = "https://api.deepseek.com";
const DEFAULT_MODEL: &str = "deepseek-v4-flash";
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct DeepSeekClient {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl DeepSeekClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        let api_key = api_key.into();
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
        );
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
            client,
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub async fn chat_completion(
        &self,
        system_prompt: &str,
        user_message: &str,
        temperature: f32,
    ) -> Result<String, DiffguardError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
            temperature,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                let status = e.status().map(|s| s.as_u16()).unwrap_or(0);
                LlmError {
                    provider: "deepseek".to_string(),
                    status,
                    message: e.to_string(),
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(LlmError {
                provider: "deepseek".to_string(),
                status: status.as_u16(),
                message: body,
            }
            .into());
        }

        let chat_response: ChatResponse = response.json().await.map_err(|e| LlmError {
            provider: "deepseek".to_string(),
            status: 0,
            message: format!("Failed to parse response: {}", e),
        })?;

        chat_response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| {
                LlmError {
                    provider: "deepseek".to_string(),
                    status: 0,
                    message: "Empty response from LLM".to_string(),
                }
                .into()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_chat_completion_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{
                    "message": {
                        "content": "This looks good.\n\n[DIFFGUARD_VERDICT_METADATA]\nVerdict: POSITIVE\nCriticalBugs: 0\nSecurityIssues: 0"
                    }
                }]
            })))
            .mount(&mock_server)
            .await;

        let client = DeepSeekClient::new("test-key").with_base_url(mock_server.uri());
        let result = client
            .chat_completion("You are a reviewer.", "diff content", 0.1)
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().contains("POSITIVE"));
    }

    #[tokio::test]
    async fn test_chat_completion_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(429).set_body_string("Rate limited"))
            .mount(&mock_server)
            .await;

        let client = DeepSeekClient::new("test-key").with_base_url(mock_server.uri());
        let result = client
            .chat_completion("You are a reviewer.", "diff content", 0.1)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("429"));
    }
}
