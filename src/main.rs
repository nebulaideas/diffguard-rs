use anyhow::Context;
use clap::Parser;
use diffguard::cli::Args;
use diffguard::config::Config;
use diffguard::diff::{fetch_local_diff, fetch_pr_diff};
use diffguard::github::{dismiss_previous_reviews, submit_review};
use diffguard::llm::factory::create_provider;
use diffguard::output::{print_colored_summary, write_artifact, ReviewConfig};
use diffguard::verdict::parse_verdict;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();

    let mut config = Config::from_env().context("Failed to load configuration")?;
    config
        .load_prompt_file(&args.prompt_file)
        .context("Failed to load prompt file")?;
    config.validate_for_ci().context("Configuration validation failed")?;

    log::info!("diffguard-rs starting (provider: {}, model: {})", config.provider, config.model);

    // Fetch diff
    let diff_result = if config.is_ci {
        log::info!("CI mode detected. Fetching PR diff...");
        let owner = config.repo_owner.as_ref().unwrap();
        let repo = config.repo_name.as_ref().unwrap();
        let pr = config.pr_number.unwrap();
        let token = config.github_token.as_ref().unwrap();

        match fetch_pr_diff("https://api.github.com", owner, repo, pr, token).await {
            Ok(diff) => {
                log::info!(
                    "Fetched diff: {} lines ({} bytes)",
                    diff.line_count,
                    diff.size_bytes
                );
                diff
            }
            Err(e) => {
                if let diffguard::error::DiffguardError::DiffTooLarge { size_bytes, line_count } = &e {
                    log::warn!("Diff too large: {} bytes ({} lines). Submitting explanatory comment.", size_bytes, line_count);
                    let msg = format!(
                        "⚠️ **diffguard-rs**: This PR diff exceeds the review size limit ({} lines / {} bytes).\n\n\
                        The diff is too large for an effective AI review. Consider breaking this PR into smaller, focused changes.",
                        line_count, size_bytes
                    );
                    submit_review(owner, repo, pr, diffguard::verdict::ReviewState::Comment, &msg, token).await
                        .context("Failed to submit size-limit comment")?;
                    return Ok(());
                }
                return Err(e).context("Failed to fetch PR diff");
            }
        }
    } else {
        log::info!("Local mode detected. Fetching staged diff...");
        match fetch_local_diff() {
            Ok(diff) => {
                log::info!(
                    "Fetched local diff: {} lines ({} bytes)",
                    diff.line_count,
                    diff.size_bytes
                );
                diff
            }
            Err(e) => {
                if let diffguard::error::DiffguardError::DiffTooLarge { size_bytes, line_count } = &e {
                    eprintln!("⚠️  Diff too large: {} bytes ({} lines). Cannot review.", size_bytes, line_count);
                    return Ok(());
                }
                if let diffguard::error::DiffguardError::EmptyDiff = &e {
                    eprintln!("ℹ️  No staged changes to review.");
                    return Ok(());
                }
                return Err(e).context("Failed to fetch local diff");
            }
        }
    };

    // Call LLM
    log::info!("Calling {} ({})...", config.provider, config.model);
    let provider = create_provider(&config.provider, &config.api_key)
        .context("Failed to create LLM provider")?;

    let llm_response = provider
        .chat_completion(&config.prompt, &diff_result.content, config.temperature)
        .await
        .context("LLM API call failed")?;

    log::info!("Received LLM response ({} chars)", llm_response.len());

    // Parse verdict
    let (verdict, state) = parse_verdict(&llm_response)
        .context("Failed to parse verdict from LLM response")?;

    log::info!(
        "Verdict: {} (CriticalBugs: {}, SecurityIssues: {}) -> State: {}",
        verdict.verdict,
        verdict.critical_bugs,
        verdict.security_issues,
        state
    );

    // Build review config for output
    let review_config = ReviewConfig {
        provider: config.provider.clone(),
        model: config.model.clone(),
        temperature: config.temperature,
        pr_number: config.pr_number,
        diff_size_bytes: diff_result.size_bytes,
        diff_line_count: diff_result.line_count,
    };

    // Write artifact
    write_artifact(&llm_response, &verdict, &state, &review_config, "review-result.txt")
        .context("Failed to write review artifact")?;

    if config.is_ci {
        // Submit review
        let owner = config.repo_owner.as_ref().unwrap();
        let repo = config.repo_name.as_ref().unwrap();
        let pr = config.pr_number.unwrap();
        let token = config.github_token.as_ref().unwrap();

        submit_review(owner, repo, pr, state.clone(), &llm_response, token)
            .await
            .context("Failed to submit review")?;

        log::info!("Review submitted: {}", state);

        // Dismiss previous blockers if new state is non-blocking
        if state != diffguard::verdict::ReviewState::RequestChanges {
            log::info!("Dismissing previous blocker reviews...");
            if let Err(e) = dismiss_previous_reviews(owner, repo, pr, token).await {
                log::warn!("Failed to dismiss previous reviews: {}", e);
            }
        }

        println!("diffguard-rs Review Complete");
        println!("============================");
        println!("Provider:    {}", config.provider);
        println!("Model:       {}", config.model);
        println!("Diff Lines:  {}", diff_result.line_count);
        println!("Verdict:     {}", verdict.verdict);
        println!("State:       {}", state);
    } else {
        // Local mode: print colored output
        print_colored_summary(&llm_response, &verdict, &state, &review_config);

        // Exit with appropriate code
        if state == diffguard::verdict::ReviewState::RequestChanges {
            std::process::exit(2);
        }
    }

    Ok(())
}
