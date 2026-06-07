use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiffguardError {
    #[error("GitHub API error: {status} - {message}")]
    GitHubApi { status: u16, message: String },

    #[error("LLM API error ({provider}): {status} - {message}")]
    LlmApi {
        provider: String,
        status: u16,
        message: String,
    },

    #[error("Failed to parse verdict: {0}")]
    VerdictParse(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Diff too large: {size_bytes} bytes ({line_count} lines). Maximum is 100KB or 1500 lines.")]
    DiffTooLarge { size_bytes: usize, line_count: usize },

    #[error("No diff content found")]
    EmptyDiff,

    #[error("Permission denied for review state {state}: {message}")]
    PermissionDenied { state: String, message: String },
}

impl DiffguardError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            DiffguardError::GitHubApi { status: 429 | 502 | 503 | 504, .. }
                | DiffguardError::LlmApi {
                    status: 429 | 502 | 503 | 504,
                    ..
                }
        )
    }

    pub fn is_permission_denied(&self) -> bool {
        matches!(
            self,
            DiffguardError::GitHubApi { status: 403, .. }
                | DiffguardError::PermissionDenied { .. }
        )
    }
}
