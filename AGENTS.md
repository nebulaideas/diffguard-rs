# diffguard-rs — Agent Guide

> This file describes the current state of the `diffguard-rs` repository for AI coding agents. If you are reading this, you are expected to know nothing about the project beyond what is written here.

---

## Project Overview

**diffguard-rs** is a planned Rust-based AI code review CLI tool. It is designed to fetch Pull Request diffs from GitHub, send them to an LLM provider for review, parse a structured verdict from the response, and submit the review state (`APPROVE`, `REQUEST_CHANGES`, or `COMMENT`) back to GitHub — all in a single execution.

**Current Status:** The repository is in the **planning stage** (Phase 0). There is no source code, no build configuration, and no tests yet. What exists is a comprehensive implementation plan, architectural diagrams, a minimal README, and a MIT license.

- **Repository:** `git@github.com:nebulaideas/diffguard-rs.git`
- **Current Branch:** `foundation-workspace-deepseekmvp` (1 commit ahead of `main`)
- **License:** MIT License (Copyright 2026 Nebula Ideas)
- **Language:** Rust (planned — edition 2021, toolchain 1.82+)

---

## Technology Stack (Planned)

| Layer | Technology |
|---|---|
| Language | Rust (edition 2021, toolchain 1.82+) |
| Build Tool | Cargo (workspace with 3 crates) |
| Async Runtime | Tokio |
| HTTP Client | reqwest |
| CLI Framework | clap (derive macros) |
| Serialization | serde, serde_json |
| Error Handling | thiserror, anyhow |
| Terminal Output | colored |
| Testing | Built-in test framework + wiremock (HTTP mocking) + criterion.rs (benchmarks) |
| Coverage | cargo-tarpaulin |
| Linting | Clippy |
| Formatting | rustfmt |
| Security Auditing | cargo-deny, cargo-audit |

### Planned LLM Providers

| Provider | Phase | Base URL |
|---|---|---|
| DeepSeek | 1 | `https://api.deepseek.com` |
| Kimi (Moonshot AI) | 2 | `https://api.moonshot.ai/v1` |
| Qwen (Alibaba Cloud) | 2 | `https://dashscope-intl.aliyuncs.com/compatible-mode/v1` |
| OpenRouter | 2 | `https://openrouter.ai/api/v1` |
| OpenAI (generic) | 2 | `https://api.openai.com/v1` |

---

## Repository Structure

```
diffguard-rs/
├── .git/                           # Git repository
├── .qodo/                          # Empty IDE directory (qodo.ai)
│   ├── agents/
│   └── workflows/
├── docs/                           # Documentation and diagrams
│   ├── MVP_IMPLEMENTATION_PLAN.md  # 727-line implementation roadmap (6 phases)
│   ├── architecture_diagram.png    # Architecture visualization
│   ├── pipeline_flow.png           # Pipeline flow diagram
│   └── project_structure.png       # Planned project structure diagram
├── .gitignore                      # Rust-focused gitignore
├── LICENSE                         # MIT License
├── README.md                       # 2-line project description
└── AGENTS.md                       # This file
```

**What is Missing (Planned but Not Yet Created):**

- `Cargo.toml` / `Cargo.lock` — No workspace manifest or crate manifests exist
- `crates/` — The three planned crates do not exist yet
- `src/` — No Rust source code exists anywhere in the repository
- `tests/` / `benches/` / `examples/` — No test, benchmark, or example directories
- `.github/workflows/` — No CI/CD configuration
- `deny.toml`, `.rustfmt.toml` — No formatting or audit configuration
- `.reviewer.toml` — No runtime configuration schema implemented

---

## Planned Code Organization

The project is designed as a **3-crate Cargo workspace**:

### `crates/diffguard-core`
Core business logic: diff fetching, verdict parsing, GitHub API interaction, and output formatting.

| Planned File | Responsibility |
|---|---|
| `src/error.rs` | `DiffguardError` enum (`GitHubApi`, `LlmApi`, `VerdictParse`, `Config`, `Io`) |
| `src/diff.rs` | `fetch_pr_diff()` — HTTP GET from GitHub API with diff accept header |
| `src/verdict.rs` | `parse_metadata_block()`, `determine_review_state()`, tag-based fallback |
| `src/github.rs` | `submit_review()`, `dismiss_previous_reviews()`, permission fallback logic |
| `src/output.rs` | `write_artifact()`, `print_colored_report()` |

### `crates/diffguard-llm`
LLM provider abstraction layer.

| Planned File | Responsibility |
|---|---|
| `src/lib.rs` | `LlmProvider` async trait definition |
| `src/types.rs` | `ChatMessage`, `ChatRequest`, `ChatResponse`, `LlmError` |
| `src/deepseek.rs` | DeepSeek provider (Phase 1) |
| `src/kimi.rs` | Kimi/Moonshot AI provider (Phase 2) |
| `src/qwen.rs` | Qwen/Alibaba Cloud provider (Phase 2) |
| `src/openrouter.rs` | OpenRouter gateway provider (Phase 2) |
| `src/openai.rs` | Generic OpenAI-compatible provider (Phase 2) |
| `src/factory.rs` | Provider factory for dynamic dispatch (Phase 2) |

### `crates/diffguard-cli`
CLI entry point, argument parsing, and configuration resolution.

| Planned File | Responsibility |
|---|---|
| `src/cli.rs` | Clap derive struct (`Args`) with flags: `--prompt-file`, `--model`, `--temperature`, `--provider` |
| `src/config.rs` | Environment variable resolution, `.reviewer.toml` parsing |
| `src/main.rs` | Entry point: detect CI vs local mode, orchestrate pipeline |

---

## Planned Build and Test Commands

These commands are specified in `docs/MVP_IMPLEMENTATION_PLAN.md` but **cannot be run yet** because no `Cargo.toml` exists:

```bash
# Development builds
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all
cargo doc --workspace --no-deps --open

# Network-dependent tests
cargo test --workspace -- --ignored

# Coverage and quality
cargo tarpaulin --workspace --out xml
cargo +nightly doc --show-coverage
cargo deny check
cargo audit

# Release build
cargo build --release -p diffguard-cli --target x86_64-unknown-linux-gnu
```

---

## Planned Testing Strategy

| Test Type | Location | Tool | Notes |
|---|---|---|---|
| Unit tests (verdict) | `crates/diffguard-core/tests/verdict_tests.rs` | Built-in | Covers all verdict parsing scenarios |
| Unit tests (diff) | `crates/diffguard-core/tests/diff_tests.rs` | wiremock | Mock HTTP tests for diff fetching |
| Unit tests (provider) | `crates/diffguard-llm/tests/provider_tests.rs` | wiremock | Mock DeepSeek API responses |
| Integration tests | `tests/` | Full pipeline mocks | End-to-end with mock GitHub + mock LLM |
| Doc tests | Inline | `cargo test --doc` | Documentation examples |

### Quality Targets (from Plan)

| Metric | Target | Tool |
|---|---|---|
| Test Coverage | 85%+ | `cargo-tarpaulin` |
| Documentation Coverage | 85%+ | `cargo +nightly doc --show-coverage` |
| Clippy | 0 warnings | `cargo clippy -- -D warnings` |
| Rustfmt | Enforced in CI | `cargo fmt --check` |
| License Audit | 0 conflicts | `cargo-deny` |
| Security Audit | 0 known vulnerabilities | `cargo-audit` |

---

## Planned CI/CD Pipelines

### `.github/workflows/ci.yml` (Phase 1)
- Format check: `cargo fmt --all -- --check`
- Lint: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Tests: `cargo test --workspace`
- Doc tests: `cargo test --doc --workspace`
- Coverage: `cargo tarpaulin --workspace --out xml` → Codecov upload
- Doc coverage: `cargo +nightly doc --show-coverage` (85% threshold)
- Release build smoke test

### `.github/workflows/release.yml` (Phase 1)
- Trigger: Push tags matching `v*`
- Build release binary for `x86_64-unknown-linux-gnu`
- Strip binary for size reduction
- Create GitHub Release with binary asset

### `.github/workflows/docs-deploy.yml` (Phase 3)
- Deploy `cargo doc` output to GitHub Pages

**Current Reality:** No `.github/` directory exists. No CI/CD is configured.

---

## Environment Variables Reference (Planned)

| Variable | Required By | Description |
|---|---|---|
| `DEEPSEEK_API_KEY` | DeepSeek provider | API key from DeepSeek platform |
| `KIMI_API_KEY` | Kimi provider | API key from Moonshot AI platform |
| `DASHSCOPE_API_KEY` | Qwen provider | API key from Alibaba Cloud DashScope |
| `OPENROUTER_API_KEY` | OpenRouter provider | API key from OpenRouter |
| `OPENAI_API_KEY` | OpenAI provider | API key from OpenAI |
| `GITHUB_TOKEN` | GitHub mode | Auto-provided by GitHub Actions |
| `PR_NUMBER` | GitHub mode | Pull request number |
| `REPO_FULL_NAME` | GitHub mode | Repository in `owner/repo` format |
| `GITHUB_ACTIONS` | Auto-detected | Presence indicates CI mode vs local mode |

---

## Planned CLI Flags

| Flag | Short | Default | Description |
|---|---|---|---|
| `--prompt-file` | `-p` | `.github/review-prompt.md` | Path to system prompt markdown file |
| `--model` | `-m` | Provider-specific | LLM model identifier |
| `--temperature` | `-t` | `0.1` | Sampling temperature (0.0 - 2.0) |
| `--provider` | | `deepseek` | LLM provider to use |
| `--config` | `-c` | `.reviewer.toml` | Path to configuration TOML file |
| `--no-cache` | | | Bypass response cache (Phase 3) |

---

## Exit Codes (Planned)

| Code | Meaning |
|---|---|
| `0` | Review completed successfully |
| `1` | Error occurred (API failure, parse error, etc.) |
| `1` | Local mode: review returned `REQUEST_CHANGES` (blocks commit) |

---

## Review State Logic (Planned)

```
if verdict == "NEGATIVE" || security_issues > 0 || critical_bugs > 2:
    state = REQUEST_CHANGES
else if critical_bugs == 0 && security_issues == 0:
    state = APPROVE
else:
    state = COMMENT
```

If `REQUEST_CHANGES` or `APPROVE` fails due to GitHub permissions, the system falls back to `COMMENT` with a `[Bot fallback from {state}]` prefix.

---

## Development Conventions (Planned)

- **Rust Edition:** 2021
- **Minimum Toolchain:** 1.82+
- **Formatting:** Enforced via `rustfmt` in CI (`cargo fmt --check`)
- **Linting:** All Clippy warnings treated as errors (`-D warnings`)
- **Error Handling:** `thiserror` for library crates, `anyhow` for the CLI crate
- **Documentation:** All public API items must have doc comments (`#![deny(missing_docs)]`)
- **Dependencies:** Managed via workspace-level `[workspace.dependencies]` in root `Cargo.toml`
- **Security:** No hardcoded secrets; all authentication via environment variables only

---

## Security Considerations

- **Secrets:** All API keys and tokens are passed exclusively through environment variables. No secrets are committed to the repository.
- **Log Sanitization:** Auth headers must be redacted (`[REDACTED]`) in any logs or debug output.
- **GitHub Token Scope:** The minimum required permissions are used for GitHub API operations.
- **Supply Chain:** `cargo-deny` and `cargo-audit` are planned for license and vulnerability auditing. `Cargo.lock` is committed.

---

## Implementation Roadmap

The full roadmap lives in `docs/MVP_IMPLEMENTATION_PLAN.md` (727 lines). It is organized into 6 phases:

1. **Phase 1: Foundation** — Workspace setup, DeepSeek MVP, core crates, basic CI
2. **Phase 2: Multi-Provider Support** — Kimi, Qwen, OpenRouter, OpenAI, `.reviewer.toml`, local mode
3. **Phase 3: Advanced Features** — Diff chunking, response caching, metrics export, retry logic, circuit breaker
4. **Phase 4: README + Documentation Polish** — Complete README, architecture docs, usage guide
5. **Phase 5: Implementation Guide** — Developer-facing guide for contributors
6. **Phase 6: crates.ai Registration** — Publishing and distribution

**Current branch name (`foundation-workspace-deepseekmvp`) indicates intent to begin Phase 1, but no implementation has started.**

---

## Notes for Agents

- **Do not assume any code exists.** Before modifying anything, verify whether the relevant files/directories actually exist.
- **The plan is authoritative.** When in doubt about intended behavior, refer to `docs/MVP_IMPLEMENTATION_PLAN.md` — it contains exhaustive specifications for every module, trait, function, and test case.
- **License discrepancy:** The `LICENSE` file at the root is MIT. The implementation plan mentions Apache-2.0 in some places. The actual root license (MIT) takes precedence until explicitly changed.
- **No existing AGENTS.md or CONTRIBUTING.md** — This file is the first agent-facing documentation in the repository.
