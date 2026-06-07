use crate::verdict::{ReviewState, Verdict};
use colored::Colorize;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct ReviewConfig {
    pub provider: String,
    pub model: String,
    pub temperature: f32,
    pub pr_number: Option<u64>,
    pub diff_size_bytes: usize,
    pub diff_line_count: usize,
}

pub fn write_artifact(
    review: &str,
    verdict: &Verdict,
    state: &ReviewState,
    config: &ReviewConfig,
    path: &str,
) -> std::io::Result<()> {
    let content = format!(
        "diffguard-rs Review Result
==========================
Provider: {}
Model: {}
Temperature: {}
Diff Size: {} lines ({} bytes)
Review State: {}

--- LLM Review ---
{}

--- Parsed Metadata ---
Verdict: {}
CriticalBugs: {}
SecurityIssues: {}
",
        config.provider,
        config.model,
        config.temperature,
        config.diff_line_count,
        config.diff_size_bytes,
        state,
        review,
        verdict.verdict,
        verdict.critical_bugs,
        verdict.security_issues,
    );

    let mut file = std::fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn print_colored_report(review: &str, state: &ReviewState) {
    println!("{}", "diffguard-rs Review".bold().underline());
    println!();

    match state {
        ReviewState::Approve => {
            println!("{}", "✓ State: APPROVE".green().bold());
        }
        ReviewState::RequestChanges => {
            println!("{}", "✗ State: REQUEST_CHANGES".red().bold());
        }
        ReviewState::Comment => {
            println!("{}", "→ State: COMMENT".yellow().bold());
        }
    }

    println!();
    println!("{}", review);
}

pub fn print_colored_summary(
    review: &str,
    verdict: &Verdict,
    state: &ReviewState,
    config: &ReviewConfig,
) {
    print_colored_report(review, state);

    println!();
    println!("{}", "--- Metadata ---".dimmed());
    println!("Provider:    {}", config.provider);
    println!("Model:       {}", config.model);
    println!("Temperature: {}", config.temperature);
    println!("Diff Lines:  {}", config.diff_line_count);
    println!();
    println!("Verdict:         {}", verdict.verdict);
    println!("Critical Bugs:   {}", verdict.critical_bugs);
    println!("Security Issues: {}", verdict.security_issues);
}
