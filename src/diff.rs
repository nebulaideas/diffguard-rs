use crate::error::DiffguardError;
use crate::retry::with_retry;
use reqwest::header::{self, HeaderMap, HeaderValue};

const MAX_DIFF_BYTES: usize = 100 * 1024; // 100KB
const MAX_DIFF_LINES: usize = 1500;
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct DiffResult {
    pub content: String,
    pub size_bytes: usize,
    pub line_count: usize,
}

pub async fn fetch_pr_diff(
    base_url: &str,
    owner: &str,
    repo: &str,
    pr_number: u64,
    token: &str,
) -> Result<DiffResult, DiffguardError> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    let url = format!(
        "{}/repos/{}/{}/pulls/{}",
        base_url, owner, repo, pr_number
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        HeaderValue::from_static("application/vnd.github.v3.diff"),
    );
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static("2022-11-28"),
    );
    headers.insert(
        header::USER_AGENT,
        HeaderValue::from_static("diffguard-rs/0.1.0"),
    );

    let response = with_retry(|| async {
        let resp = client
            .get(&url)
            .headers(headers.clone())
            .send()
            .await
            .map_err(|e| {
                let status = e.status().map(|s| s.as_u16()).unwrap_or(0);
                DiffguardError::GitHubApi {
                    status,
                    message: e.to_string(),
                }
            })?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(DiffguardError::GitHubApi {
                status: status.as_u16(),
                message: body,
            });
        }

        let body = resp.text().await.map_err(|e| DiffguardError::GitHubApi {
            status: 0,
            message: e.to_string(),
        })?;

        Ok(body)
    })
    .await?;

    if response.is_empty() {
        return Err(DiffguardError::EmptyDiff);
    }

    let size_bytes = response.len();
    let line_count = response.lines().count();

    if size_bytes > MAX_DIFF_BYTES || line_count > MAX_DIFF_LINES {
        return Err(DiffguardError::DiffTooLarge {
            size_bytes,
            line_count,
        });
    }

    Ok(DiffResult {
        content: response,
        size_bytes,
        line_count,
    })
}

pub fn fetch_local_diff() -> Result<DiffResult, DiffguardError> {
    let output = std::process::Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .map_err(DiffguardError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DiffguardError::Config(format!(
            "git diff --cached failed: {}",
            stderr
        )));
    }

    let content = String::from_utf8_lossy(&output.stdout).to_string();

    if content.is_empty() {
        return Err(DiffguardError::EmptyDiff);
    }

    let size_bytes = content.len();
    let line_count = content.lines().count();

    if size_bytes > MAX_DIFF_BYTES || line_count > MAX_DIFF_LINES {
        return Err(DiffguardError::DiffTooLarge {
            size_bytes,
            line_count,
        });
    }

    Ok(DiffResult {
        content,
        size_bytes,
        line_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_fetch_pr_diff_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/repos/test-owner/test-repo/pulls/42"))
            .and(header("Accept", "application/vnd.github.v3.diff"))
            .respond_with(ResponseTemplate::new(200).set_body_string("diff content\nline 2"))
            .mount(&mock_server)
            .await;

        let result = fetch_pr_diff(
            &mock_server.uri(),
            "test-owner",
            "test-repo",
            42,
            "test-token",
        )
        .await;

        assert!(result.is_ok());
        let diff = result.unwrap();
        assert_eq!(diff.content, "diff content\nline 2");
        assert_eq!(diff.line_count, 2);
    }

    #[tokio::test]
    async fn test_fetch_pr_diff_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/repos/test-owner/test-repo/pulls/999"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&mock_server)
            .await;

        let result = fetch_pr_diff(
            &mock_server.uri(),
            "test-owner",
            "test-repo",
            999,
            "test-token",
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("404"));
    }
}
