use crate::error::DiffguardError;
use crate::llm::{deepseek::DeepSeekClient, Provider};

pub fn create_provider(provider_name: &str, api_key: &str) -> Result<Provider, DiffguardError> {
    match provider_name {
        "deepseek" => Ok(Provider::DeepSeek(DeepSeekClient::new(api_key))),
        other => Err(DiffguardError::Config(format!(
            "Unknown provider: {}. Supported: deepseek",
            other
        ))),
    }
}
