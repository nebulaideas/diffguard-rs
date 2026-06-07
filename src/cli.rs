use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "diffguard")]
#[command(about = "AI-powered code review CLI for GitHub PRs")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Args {
    #[arg(
        short,
        long,
        default_value = ".github/review-prompt.md",
        help = "Path to system prompt markdown file"
    )]
    pub prompt_file: PathBuf,

    #[arg(
        short,
        long,
        default_value = "deepseek-v4-flash",
        help = "LLM model identifier"
    )]
    pub model: String,

    #[arg(
        short,
        long,
        default_value_t = 0.1,
        help = "Sampling temperature (0.0 - 2.0)"
    )]
    pub temperature: f32,

    #[arg(
        long,
        env = "DIFFGUARD_PROVIDER",
        default_value = "deepseek",
        help = "LLM provider to use"
    )]
    pub provider: String,

    #[arg(
        short,
        long,
        default_value = ".reviewer.toml",
        help = "Path to configuration TOML file"
    )]
    pub config: PathBuf,
}
