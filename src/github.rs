use crate::error::DiffguardError;
use crate::retry::with_retry;
use crate::verdict::ReviewState;
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde_json::json;

const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const BOT_SIGNATURE: &str = "<!-- diffguard-bot -->";

async fn submit_review_inner(
    owner: &str,
    repo: &str,
    pr_number: u64,
    state: &ReviewState,
    message: &str,
    token: &str,
) -> Result<(), DiffguardError> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/reviews",
        owner, repo, pr_number
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
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

    let body = json!({
        "body": format!("{}\n\n{}", message, BOT_SIGNATURE),
        "event": state.as_github_state(),
    });

    with_retry(|| async {
        let resp = client
            .post(&url)
            .headers(headers.clone())
            .json(&body)
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
            let body_text = resp.text().await.unwrap_or_default();
            return Err(DiffguardError::GitHubApi {
                status: status.as_u16(),
                message: body_text,
            });
        }

        Ok(())
    })
    .await
}

pub async fn submit_review(
    owner: &str,
    repo: &str,
    pr_number: u64,
    state: ReviewState,
    message: &str,
    token: &str,
) -> Result<(), DiffguardError> {
    let result = submit_review_inner(owner, repo, pr_number, &state, message, token).await;

    match result {
        Ok(()) => Ok(()),
        Err(err) if err.is_permission_denied() && state != ReviewState::Comment => {
            log::warn!(
                "Permission denied for {}. Falling back to COMMENT...",
                state
            );
            let fallback_msg = format!(
                "[Bot fallback from {}]\n\n{}",
                state, message
            );
            submit_review_inner(owner, repo, pr_number, &ReviewState::Comment, &fallback_msg, token)
                .await
        }
        Err(err) => Err(err),
    }
}

pub async fn dismiss_previous_reviews(
    owner: &str,
    repo: &str,
    pr_number: u64,
    token: &str,
) -> Result<(), DiffguardError> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/reviews",
        owner, repo, pr_number
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
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

    let reviews: Vec<serde_json::Value> = with_retry(|| async {
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

        resp.json().await.map_err(|e| DiffguardError::GitHubApi {
            status: 0,
            message: e.to_string(),
        })
    })
    .await?;

    for review in reviews {
        let state = review["state"].as_str().unwrap_or("");
        let body = review["body"].as_str().unwrap_or("");
        let review_id = review["id"].as_u64();

        if state == "CHANGES_REQUESTED" && body.contains(BOT_SIGNATURE) {
            if let Some(id) = review_id {
                let dismiss_url = format!(
                    "https://api.github.com/repos/{}/{}/pulls/{}/reviews/{}/dismissals",
                    owner, repo, pr_number, id
                );

                let dismiss_body = json!({
                    "message": "Outdated — new review submitted",
                });

                let _ = with_retry(|| async {
                    let resp = client
                        .put(&dismiss_url)
                        .headers(headers.clone())
                        .json(&dismiss_body)
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

                    Ok(())
                })
                .await;
            }
        }
    }

    Ok(())
}
