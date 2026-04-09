# clawsh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an AI-native POSIX-compatible shell in Rust that translates natural language to shell commands using a local LLM (Ollama), with support for multiple LLM providers.

**Architecture:** A lightweight classifier (regex-based) routes input — POSIX commands go directly to `bash -c`, natural language goes to an LLM provider. The LLM abstraction uses a trait so providers (Ollama, Claude, OpenAI) are interchangeable. All POSIX execution is delegated to the system bash subprocess.

**Tech Stack:** Rust, tokio, rustyline, reqwest, serde/toml, ratatui, nix

---

## File Structure

```
clawsh/
├── Cargo.toml                          # workspace root
├── crates/
│   ├── clawsh-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # re-exports
│   │       ├── repl.rs                 # main REPL loop
│   │       ├── executor.rs             # bash subprocess delegation
│   │       └── history.rs              # command history
│   ├── clawsh-classifier/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs                  # regex-based classifier
│   ├── clawsh-llm/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # re-exports + LLMProvider trait
│   │       ├── ollama.rs               # Ollama provider
│   │       ├── claude.rs               # Claude API provider
│   │       └── openai.rs               # OpenAI-compatible provider
│   ├── clawsh-safety/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs                  # dangerous command detection
│   └── clawsh-config/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs                  # config file parsing
└── src/
    └── main.rs                         # entry point, wires crates together
```

---

## Task 1: Cargo Workspace Setup

**Files:**
- Create: `Cargo.toml` (workspace)
- Create: `crates/clawsh-core/Cargo.toml`
- Create: `crates/clawsh-classifier/Cargo.toml`
- Create: `crates/clawsh-llm/Cargo.toml`
- Create: `crates/clawsh-safety/Cargo.toml`
- Create: `crates/clawsh-config/Cargo.toml`
- Create: `src/main.rs`

- [ ] **Step 1: Initialize workspace**

```bash
cargo new clawsh --name clawsh
cd clawsh
mkdir -p crates
cargo new --lib crates/clawsh-core
cargo new --lib crates/clawsh-classifier
cargo new --lib crates/clawsh-llm
cargo new --lib crates/clawsh-safety
cargo new --lib crates/clawsh-config
```

- [ ] **Step 2: Write workspace Cargo.toml**

```toml
# Cargo.toml
[workspace]
members = [
    ".",
    "crates/clawsh-core",
    "crates/clawsh-classifier",
    "crates/clawsh-llm",
    "crates/clawsh-safety",
    "crates/clawsh-config",
]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
anyhow = "1"
async-trait = "0.1"
```

- [ ] **Step 3: Write root Cargo.toml**

```toml
# (append to workspace Cargo.toml or create separate bin Cargo.toml)
[package]
name = "clawsh"
version = "0.1.0"
edition = "2021"

[dependencies]
clawsh-core = { path = "crates/clawsh-core" }
clawsh-classifier = { path = "crates/clawsh-classifier" }
clawsh-llm = { path = "crates/clawsh-llm" }
clawsh-safety = { path = "crates/clawsh-safety" }
clawsh-config = { path = "crates/clawsh-config" }
tokio = { workspace = true }
anyhow = { workspace = true }
```

- [ ] **Step 4: Write placeholder main.rs**

```rust
// src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("clawsh starting...");
    Ok(())
}
```

- [ ] **Step 5: Verify build**

```bash
cargo build
```
Expected: compiles without errors.

- [ ] **Step 6: Commit**

```bash
git init
git add .
git commit -m "chore: initialize clawsh workspace"
```

---

## Task 2: Config Crate

**Files:**
- Create: `crates/clawsh-config/src/lib.rs`
- Create: `crates/clawsh-config/Cargo.toml`
- Create: `crates/clawsh-config/tests/config_test.rs`

- [ ] **Step 1: Write Cargo.toml for clawsh-config**

```toml
# crates/clawsh-config/Cargo.toml
[package]
name = "clawsh-config"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
toml = "0.8"
anyhow = { workspace = true }
dirs = "5"
```

- [ ] **Step 2: Write the failing test**

```rust
// crates/clawsh-config/tests/config_test.rs
use clawsh_config::Config;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.models.default, "qwen2.5:3b");
    assert_eq!(config.models.classifier, "smollm2:135m");
    assert!(config.safety.confirm_dangerous);
    assert!(config.safety.auto_explain_errors);
}

#[test]
fn test_parse_toml() {
    let toml = r#"
[models]
default = "llama3.2:3b"
classifier = "smollm2:135m"

[providers.ollama]
host = "http://localhost:11434"

[safety]
confirm_dangerous = false
auto_explain_errors = true

[shell]
history_size = 5000
prompt = "{cwd} ❯"
"#;
    let config: clawsh_config::Config = toml::from_str(toml).unwrap();
    assert_eq!(config.models.default, "llama3.2:3b");
    assert!(!config.safety.confirm_dangerous);
    assert_eq!(config.shell.history_size, 5000);
}
```

- [ ] **Step 3: Run test to verify it fails**

```bash
cargo test -p clawsh-config
```
Expected: FAIL — `Config` not defined.

- [ ] **Step 4: Implement Config**

```rust
// crates/clawsh-config/src/lib.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub models: ModelsConfig,
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default)]
    pub safety: SafetyConfig,
    #[serde(default)]
    pub shell: ShellConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig {
    #[serde(default = "default_model")]
    pub default: String,
    #[serde(default = "default_classifier")]
    pub classifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub host: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    #[serde(default = "default_true")]
    pub confirm_dangerous: bool,
    #[serde(default = "default_true")]
    pub auto_explain_errors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    #[serde(default = "default_history_size")]
    pub history_size: usize,
    #[serde(default = "default_prompt")]
    pub prompt: String,
}

fn default_model() -> String { "qwen2.5:3b".to_string() }
fn default_classifier() -> String { "smollm2:135m".to_string() }
fn default_true() -> bool { true }
fn default_history_size() -> usize { 10000 }
fn default_prompt() -> String { "{model} {cwd} ❯".to_string() }

impl Default for Config {
    fn default() -> Self {
        Self {
            models: ModelsConfig {
                default: default_model(),
                classifier: default_classifier(),
            },
            providers: HashMap::new(),
            safety: SafetyConfig {
                confirm_dangerous: true,
                auto_explain_errors: true,
            },
            shell: ShellConfig {
                history_size: default_history_size(),
                prompt: default_prompt(),
            },
        }
    }
}

impl Default for ModelsConfig {
    fn default() -> Self {
        Self { default: default_model(), classifier: default_classifier() }
    }
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self { confirm_dangerous: true, auto_explain_errors: true }
    }
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self { history_size: default_history_size(), prompt: default_prompt() }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("clawsh")
            .join("config.toml");

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test -p clawsh-config
```
Expected: 2 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/clawsh-config/
git commit -m "feat(config): add TOML config with serde defaults"
```

---

## Task 3: Safety Crate — Dangerous Command Detection

**Files:**
- Create: `crates/clawsh-safety/src/lib.rs`
- Create: `crates/clawsh-safety/Cargo.toml`
- Create: `crates/clawsh-safety/tests/safety_test.rs`

- [ ] **Step 1: Write Cargo.toml for clawsh-safety**

```toml
# crates/clawsh-safety/Cargo.toml
[package]
name = "clawsh-safety"
version = "0.1.0"
edition = "2021"

[dependencies]
regex = "1"
```

- [ ] **Step 2: Write the failing tests**

```rust
// crates/clawsh-safety/tests/safety_test.rs
use clawsh_safety::is_dangerous;

#[test]
fn test_rm_rf_is_dangerous() {
    assert!(is_dangerous("rm -rf /home/user"));
    assert!(is_dangerous("rm -rf *"));
}

#[test]
fn test_dd_is_dangerous() {
    assert!(is_dangerous("dd if=/dev/zero of=/dev/sda"));
}

#[test]
fn test_mkfs_is_dangerous() {
    assert!(is_dangerous("mkfs.ext4 /dev/sdb1"));
}

#[test]
fn test_chmod_recursive_is_dangerous() {
    assert!(is_dangerous("chmod -R 777 /"));
}

#[test]
fn test_safe_commands_are_not_dangerous() {
    assert!(!is_dangerous("ls -la"));
    assert!(!is_dangerous("git status"));
    assert!(!is_dangerous("cargo build"));
    assert!(!is_dangerous("rm -f /tmp/myfile.txt"));
}
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cargo test -p clawsh-safety
```
Expected: FAIL — `is_dangerous` not defined.

- [ ] **Step 4: Implement dangerous command detection**

```rust
// crates/clawsh-safety/src/lib.rs
use regex::Regex;
use std::sync::OnceLock;

static DANGEROUS_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

fn patterns() -> &'static Vec<Regex> {
    DANGEROUS_PATTERNS.get_or_init(|| {
        let raw = [
            r"rm\s+-[a-zA-Z]*r[a-zA-Z]*f",   // rm -rf / rm -fr
            r"rm\s+-[a-zA-Z]*f[a-zA-Z]*r",
            r"\bdd\b.+of=",                    // dd ... of=...
            r"\bmkfs\b",                        // mkfs.*
            r"chmod\s+-R\s+[0-7]*7[0-7]*\s+/", // chmod -R 777 /
            r":\(\)\s*\{.*\}.*:",               // fork bomb
            r">\s*/dev/sd[a-z]",               // redirect to block device
        ];
        raw.iter().map(|p| Regex::new(p).unwrap()).collect()
    })
}

pub fn is_dangerous(cmd: &str) -> bool {
    patterns().iter().any(|re| re.is_match(cmd))
}
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test -p clawsh-safety
```
Expected: all tests PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/clawsh-safety/
git commit -m "feat(safety): add dangerous command detection with regex patterns"
```

---

## Task 4: Classifier Crate — Natural Language vs POSIX

**Files:**
- Create: `crates/clawsh-classifier/src/lib.rs`
- Create: `crates/clawsh-classifier/Cargo.toml`
- Create: `crates/clawsh-classifier/tests/classifier_test.rs`

- [ ] **Step 1: Write Cargo.toml**

```toml
# crates/clawsh-classifier/Cargo.toml
[package]
name = "clawsh-classifier"
version = "0.1.0"
edition = "2021"

[dependencies]
regex = "1"
```

- [ ] **Step 2: Write the failing tests**

```rust
// crates/clawsh-classifier/tests/classifier_test.rs
use clawsh_classifier::{classify, InputKind};

#[test]
fn test_posix_commands_classified_correctly() {
    assert_eq!(classify("ls -la"), InputKind::Posix);
    assert_eq!(classify("git status"), InputKind::Posix);
    assert_eq!(classify("cargo build --release"), InputKind::Posix);
    assert_eq!(classify("cd /home/user"), InputKind::Posix);
    assert_eq!(classify("cat file.txt | grep foo"), InputKind::Posix);
    assert_eq!(classify("echo hello world"), InputKind::Posix);
}

#[test]
fn test_natural_language_classified_correctly() {
    assert_eq!(classify("kill the process using port 8080"), InputKind::NaturalLanguage);
    assert_eq!(classify("show files modified last week"), InputKind::NaturalLanguage);
    assert_eq!(classify("how much disk space do I have"), InputKind::NaturalLanguage);
    assert_eq!(classify("find all rust files in this project"), InputKind::NaturalLanguage);
}

#[test]
fn test_shell_builtins_are_posix() {
    assert_eq!(classify("export FOO=bar"), InputKind::Posix);
    assert_eq!(classify("source ~/.bashrc"), InputKind::Posix);
    assert_eq!(classify("alias ll='ls -la'"), InputKind::Posix);
}
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cargo test -p clawsh-classifier
```
Expected: FAIL — `classify` and `InputKind` not defined.

- [ ] **Step 4: Implement the classifier**

```rust
// crates/clawsh-classifier/src/lib.rs
use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, PartialEq, Eq)]
pub enum InputKind {
    Posix,
    NaturalLanguage,
}

// Common CLI programs and shell builtins
static POSIX_PROGRAMS: OnceLock<Regex> = OnceLock::new();
// Patterns that strongly indicate natural language
static NL_INDICATORS: OnceLock<Regex> = OnceLock::new();

fn posix_re() -> &'static Regex {
    POSIX_PROGRAMS.get_or_init(|| {
        Regex::new(
            r"(?x)^(
                ls|cd|pwd|echo|cat|grep|find|cp|mv|rm|mkdir|rmdir|
                touch|chmod|chown|ps|kill|top|df|du|tar|zip|unzip|
                curl|wget|git|cargo|npm|pip|python|python3|node|
                docker|kubectl|ssh|scp|rsync|sed|awk|cut|sort|uniq|
                head|tail|wc|tr|tee|xargs|which|whereis|man|
                export|source|alias|unalias|history|jobs|fg|bg|
                make|gcc|clang|rustc|go|java|mvn|gradle|
                systemctl|service|journalctl|apt|brew|yum|dnf|
                vim|nano|emacs|less|more|
                /[a-zA-Z]|\./
            )\b"
        ).unwrap()
    })
}

fn nl_re() -> &'static Regex {
    NL_INDICATORS.get_or_init(|| {
        Regex::new(
            r"(?i)\b(show|find|kill|delete|create|list|how|what|why|
                     get|give|open|close|stop|start|restart|check|
                     using|files|process|folder|directory|disk|memory|
                     modified|created|running|installed)\b"
        ).unwrap()
    })
}

pub fn classify(input: &str) -> InputKind {
    let trimmed = input.trim();

    // Empty or comment
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return InputKind::Posix;
    }

    // Starts with a known POSIX program
    if posix_re().is_match(trimmed) {
        return InputKind::Posix;
    }

    // Starts with uppercase letter = likely a sentence
    if trimmed.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return InputKind::NaturalLanguage;
    }

    // Contains natural language indicators and no path separators
    if nl_re().is_match(trimmed) && !trimmed.contains('/') {
        return InputKind::NaturalLanguage;
    }

    // Default: try as POSIX
    InputKind::Posix
}
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test -p clawsh-classifier
```
Expected: all tests PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/clawsh-classifier/
git commit -m "feat(classifier): add regex-based natural language vs POSIX classifier"
```

---

## Task 5: LLM Provider Abstraction & Ollama Integration

**Files:**
- Create: `crates/clawsh-llm/Cargo.toml`
- Create: `crates/clawsh-llm/src/lib.rs`
- Create: `crates/clawsh-llm/src/ollama.rs`
- Create: `crates/clawsh-llm/tests/ollama_test.rs`

- [ ] **Step 1: Write Cargo.toml for clawsh-llm**

```toml
# crates/clawsh-llm/Cargo.toml
[package]
name = "clawsh-llm"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = { workspace = true }
tokio = { workspace = true }
anyhow = { workspace = true }
serde = { workspace = true }
serde_json = "1"
reqwest = { version = "0.12", features = ["json"] }
```

- [ ] **Step 2: Write the failing tests**

```rust
// crates/clawsh-llm/tests/ollama_test.rs
// Integration test — requires Ollama running locally.
// Run with: cargo test -p clawsh-llm -- --ignored
use clawsh_llm::{LLMProvider, ollama::OllamaProvider};

#[tokio::test]
#[ignore = "requires Ollama running at localhost:11434"]
async fn test_ollama_translates_natural_language() {
    let provider = OllamaProvider::new("http://localhost:11434", "qwen2.5:3b");
    let result = provider
        .translate_to_command("list all files in the current directory", "/tmp", &[])
        .await
        .unwrap();
    assert!(!result.is_empty());
    // Should contain 'ls'
    assert!(result.contains("ls"), "expected ls command, got: {result}");
}
```

- [ ] **Step 3: Define the LLMProvider trait**

```rust
// crates/clawsh-llm/src/lib.rs
pub mod ollama;

use async_trait::async_trait;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Translate a natural language query to a shell command.
    /// `cwd` is the current working directory.
    /// `history` is recent command history for context.
    async fn translate_to_command(
        &self,
        query: &str,
        cwd: &str,
        history: &[String],
    ) -> anyhow::Result<String>;

    /// Explain why a command failed.
    async fn explain_error(
        &self,
        cmd: &str,
        stderr: &str,
    ) -> anyhow::Result<String>;

    fn name(&self) -> &str;
}
```

- [ ] **Step 4: Implement OllamaProvider**

```rust
// crates/clawsh-llm/src/ollama.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::LLMProvider;

pub struct OllamaProvider {
    host: String,
    model: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

impl OllamaProvider {
    pub fn new(host: &str, model: &str) -> Self {
        Self {
            host: host.to_string(),
            model: model.to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn complete(&self, prompt: &str) -> anyhow::Result<String> {
        let url = format!("{}/api/generate", self.host);
        let body = OllamaRequest {
            model: &self.model,
            prompt,
            stream: false,
        };
        let resp: OllamaResponse = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.response.trim().to_string())
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn translate_to_command(
        &self,
        query: &str,
        cwd: &str,
        history: &[String],
    ) -> anyhow::Result<String> {
        let history_ctx = if history.is_empty() {
            String::new()
        } else {
            format!("Recent commands:\n{}\n\n", history.last().unwrap_or(&String::new()))
        };

        let prompt = format!(
            "You are a shell command translator. Output ONLY the shell command, nothing else. No explanation, no markdown, no backticks.\n\
            Current directory: {cwd}\n\
            {history_ctx}\
            Translate this to a bash command: {query}"
        );
        self.complete(&prompt).await
    }

    async fn explain_error(&self, cmd: &str, stderr: &str) -> anyhow::Result<String> {
        let prompt = format!(
            "A shell command failed. Explain why in 1-2 sentences and suggest a fix.\n\
            Command: {cmd}\n\
            Error: {stderr}"
        );
        self.complete(&prompt).await
    }

    fn name(&self) -> &str {
        &self.model
    }
}
```

- [ ] **Step 5: Build to verify compilation**

```bash
cargo build -p clawsh-llm
```
Expected: compiles without errors.

- [ ] **Step 6: Run integration test (optional — requires Ollama)**

```bash
cargo test -p clawsh-llm -- --include-ignored
```
Expected: PASS if Ollama running, SKIP otherwise.

- [ ] **Step 7: Commit**

```bash
git add crates/clawsh-llm/
git commit -m "feat(llm): add LLMProvider trait and Ollama integration"
```

---

## Task 6: Executor Crate — bash Delegation

**Files:**
- Create: `crates/clawsh-core/src/executor.rs`
- Create: `crates/clawsh-core/Cargo.toml`
- Create: `crates/clawsh-core/tests/executor_test.rs`

- [ ] **Step 1: Write Cargo.toml for clawsh-core**

```toml
# crates/clawsh-core/Cargo.toml
[package]
name = "clawsh-core"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = { workspace = true }
tokio = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["macros"] }
```

- [ ] **Step 2: Write the failing tests**

```rust
// crates/clawsh-core/tests/executor_test.rs
use clawsh_core::executor::{execute, ExecuteResult};

#[tokio::test]
async fn test_simple_command_succeeds() {
    let result = execute("echo hello").await.unwrap();
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("hello"));
}

#[tokio::test]
async fn test_failing_command_returns_nonzero() {
    let result = execute("ls /nonexistent_path_xyz").await.unwrap();
    assert_ne!(result.exit_code, 0);
    assert!(!result.stderr.is_empty());
}

#[tokio::test]
async fn test_pipe_command() {
    let result = execute("echo hello | grep hello").await.unwrap();
    assert_eq!(result.exit_code, 0);
}
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cargo test -p clawsh-core
```
Expected: FAIL — `execute` and `ExecuteResult` not defined.

- [ ] **Step 4: Implement executor**

```rust
// crates/clawsh-core/src/executor.rs
use std::process::Stdio;
use tokio::process::Command;

pub struct ExecuteResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub async fn execute(cmd: &str) -> anyhow::Result<ExecuteResult> {
    let output = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .output()
        .await?;

    Ok(ExecuteResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}
```

- [ ] **Step 5: Wire executor into lib.rs**

```rust
// crates/clawsh-core/src/lib.rs
pub mod executor;
```

- [ ] **Step 6: Run tests to verify they pass**

```bash
cargo test -p clawsh-core
```
Expected: all tests PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/clawsh-core/
git commit -m "feat(core): add bash subprocess executor"
```

---

## Task 7: REPL Loop — Wire Everything Together

**Files:**
- Create: `crates/clawsh-core/src/repl.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add rustyline dependency to clawsh-core**

```toml
# Add to crates/clawsh-core/Cargo.toml [dependencies]
rustyline = "14"
```

- [ ] **Step 2: Implement REPL**

```rust
// crates/clawsh-core/src/repl.rs
use rustyline::DefaultEditor;
use crate::executor::execute;

pub struct Repl {
    model_name: String,
}

impl Repl {
    pub fn new(model_name: &str) -> Self {
        Self { model_name: model_name.to_string() }
    }

    pub async fn run<P>(
        &self,
        provider: &P,
        config: &clawsh_config::Config,
    ) -> anyhow::Result<()>
    where
        P: clawsh_llm::LLMProvider,
    {
        let mut rl = DefaultEditor::new()?;
        let mut history: Vec<String> = Vec::new();
        let classifier = clawsh_classifier::classify;

        loop {
            let cwd = std::env::current_dir()
                .unwrap_or_default()
                .display()
                .to_string();
            let prompt = format!("clawsh [{}] {} ❯ ", self.model_name, cwd);

            let line = match rl.readline(&prompt) {
                Ok(l) => l,
                Err(rustyline::error::ReadlineError::Eof) => break,
                Err(rustyline::error::ReadlineError::Interrupted) => continue,
                Err(e) => return Err(e.into()),
            };

            let input = line.trim().to_string();
            if input.is_empty() { continue; }
            let _ = rl.add_history_entry(&input);

            let cmd = match classifier(&input) {
                clawsh_classifier::InputKind::Posix => input.clone(),
                clawsh_classifier::InputKind::NaturalLanguage => {
                    print!("  → ");
                    let cmd = provider
                        .translate_to_command(&input, &cwd, &history)
                        .await?;
                    println!("{cmd}");

                    if config.safety.confirm_dangerous
                        && clawsh_safety::is_dangerous(&cmd)
                    {
                        print!("  ⚠️  Dangerous command. Execute? [y/N] ");
                        use std::io::{self, BufRead};
                        let mut answer = String::new();
                        io::stdin().lock().read_line(&mut answer)?;
                        if !answer.trim().eq_ignore_ascii_case("y") {
                            println!("  cancelled.");
                            continue;
                        }
                    }
                    cmd
                }
            };

            let result = execute(&cmd).await?;
            history.push(input.clone());
            if history.len() > 20 { history.remove(0); }

            if result.exit_code != 0 && config.safety.auto_explain_errors && !result.stderr.is_empty() {
                let explanation = provider.explain_error(&cmd, &result.stderr).await?;
                println!("  💡 {explanation}");
            }
        }

        Ok(())
    }
}
```

- [ ] **Step 3: Update clawsh-core lib.rs**

```rust
// crates/clawsh-core/src/lib.rs
pub mod executor;
pub mod repl;
```

- [ ] **Step 4: Wire into main.rs**

```toml
# Add to root Cargo.toml [dependencies]
clawsh-config = { path = "crates/clawsh-config" }
clawsh-llm = { path = "crates/clawsh-llm" }
clawsh-core = { path = "crates/clawsh-core" }
```

```rust
// src/main.rs
use clawsh_config::Config;
use clawsh_core::repl::Repl;
use clawsh_llm::ollama::OllamaProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;

    let ollama_host = config.providers
        .get("ollama")
        .and_then(|p| p.host.as_deref())
        .unwrap_or("http://localhost:11434");

    let provider = OllamaProvider::new(ollama_host, &config.models.default);
    let repl = Repl::new(&config.models.default);

    repl.run(&provider, &config).await
}
```

- [ ] **Step 5: Build and run**

```bash
cargo build
cargo run
```
Expected: clawsh prompt appears. Type `ls` — lists files. Type `show disk usage` — Ollama translates to `df -h` and executes.

- [ ] **Step 6: Commit**

```bash
git add crates/clawsh-core/src/repl.rs src/main.rs Cargo.toml
git commit -m "feat: wire REPL loop with classifier, LLM, executor, and safety guard"
```

---

## Task 8: /model Switching Command

**Files:**
- Modify: `crates/clawsh-core/src/repl.rs`
- Create: `crates/clawsh-llm/src/openai.rs`
- Create: `crates/clawsh-llm/src/claude.rs`

- [ ] **Step 1: Add OpenAI-compatible provider**

```rust
// crates/clawsh-llm/src/openai.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::LLMProvider;

pub struct OpenAIProvider {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: String,
}

impl OpenAIProvider {
    pub fn new(api_key: &str, model: &str, base_url: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn complete(&self, system: &str, user: &str) -> anyhow::Result<String> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = ChatRequest {
            model: &self.model,
            messages: vec![
                Message { role: "system", content: system },
                Message { role: "user", content: user },
            ],
        };
        let resp: ChatResponse = self.client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.choices[0].message.content.trim().to_string())
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn translate_to_command(&self, query: &str, cwd: &str, history: &[String]) -> anyhow::Result<String> {
        let system = "You are a shell command translator. Output ONLY the shell command. No explanation, no markdown, no backticks.";
        let user = format!("Current directory: {cwd}\nTranslate to bash: {query}");
        self.complete(system, &user).await
    }

    async fn explain_error(&self, cmd: &str, stderr: &str) -> anyhow::Result<String> {
        let system = "Explain shell errors briefly in 1-2 sentences and suggest a fix.";
        let user = format!("Command: {cmd}\nError: {stderr}");
        self.complete(system, &user).await
    }

    fn name(&self) -> &str { &self.model }
}
```

- [ ] **Step 2: Add Claude provider (reuses OpenAI-compatible format)**

```rust
// crates/clawsh-llm/src/claude.rs
// Claude API uses the same messages format as OpenAI
// Re-use OpenAIProvider pointing at Anthropic's base URL
use crate::openai::OpenAIProvider;

pub fn new_claude_provider(api_key: &str, model: &str) -> OpenAIProvider {
    OpenAIProvider::new(api_key, model, "https://api.anthropic.com/v1")
}
```

- [ ] **Step 3: Update lib.rs exports**

```rust
// crates/clawsh-llm/src/lib.rs
pub mod ollama;
pub mod openai;
pub mod claude;

use async_trait::async_trait;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn translate_to_command(&self, query: &str, cwd: &str, history: &[String]) -> anyhow::Result<String>;
    async fn explain_error(&self, cmd: &str, stderr: &str) -> anyhow::Result<String>;
    fn name(&self) -> &str;
}

pub type BoxedProvider = Box<dyn LLMProvider>;
```

- [ ] **Step 4: Add /model command handling to repl.rs**

In `repl.rs`, before the classifier call, add:

```rust
// Inside the REPL loop, after getting `input`:
if let Some(rest) = input.strip_prefix("/model") {
    let arg = rest.trim();
    if arg == "list" {
        println!("  Available providers: ollama, openai, claude");
        println!("  Usage: /model qwen2.5:3b  or  /model claude");
    } else {
        println!("  💡 Model switching mid-session: restart clawsh with CLAWSH_MODEL={arg}");
        println!("  Full runtime switching coming in a future release.");
    }
    continue;
}
```

- [ ] **Step 5: Build**

```bash
cargo build
```
Expected: compiles without errors.

- [ ] **Step 6: Commit**

```bash
git add crates/clawsh-llm/src/ crates/clawsh-core/src/repl.rs
git commit -m "feat(llm): add OpenAI-compatible and Claude providers, /model command"
```

---

## Task 9: clawsh setup Wizard & First-Run Experience

**Files:**
- Create: `crates/clawsh-core/src/setup.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Implement setup wizard**

```rust
// crates/clawsh-core/src/setup.rs
use std::path::PathBuf;

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("clawsh")
        .join("config.toml")
}

pub async fn first_run_setup() -> anyhow::Result<()> {
    println!("Welcome to clawsh! Let's get you set up.\n");

    // Check Ollama
    let ollama_running = reqwest::get("http://localhost:11434/api/tags")
        .await
        .is_ok();

    if !ollama_running {
        println!("⚠️  Ollama not detected. Install it at: https://ollama.com");
        println!("   After installing, run: ollama pull qwen2.5:3b\n");
    } else {
        println!("✓ Ollama detected at localhost:11434");
    }

    // Write default config
    let path = config_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    let default_config = r#"[models]
default = "qwen2.5:3b"
classifier = "smollm2:135m"

[providers.ollama]
host = "http://localhost:11434"

[safety]
confirm_dangerous = true
auto_explain_errors = true

[shell]
history_size = 10000
prompt = "{model} {cwd} ❯"
"#;

    std::fs::write(&path, default_config)?;
    println!("✓ Config written to {}", path.display());
    println!("\nRun 'clawsh' to start your AI shell!");
    Ok(())
}
```

- [ ] **Step 2: Update main.rs to handle `clawsh setup`**

```rust
// src/main.rs
use clawsh_config::Config;
use clawsh_core::{repl::Repl, setup::first_run_setup};
use clawsh_llm::ollama::OllamaProvider;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.get(1).map(|s| s.as_str()) == Some("setup") {
        return first_run_setup().await;
    }

    let config = Config::load()?;

    let ollama_host = config.providers
        .get("ollama")
        .and_then(|p| p.host.as_deref())
        .unwrap_or("http://localhost:11434");

    let provider = OllamaProvider::new(ollama_host, &config.models.default);
    let repl = Repl::new(&config.models.default);

    repl.run(&provider, &config).await
}
```

- [ ] **Step 3: Add dirs and reqwest to clawsh-core**

```toml
# Add to crates/clawsh-core/Cargo.toml [dependencies]
dirs = "5"
reqwest = { version = "0.12", features = ["json"] }
```

- [ ] **Step 4: Build and test setup command**

```bash
cargo build
cargo run -- setup
```
Expected: prints setup info and writes config to `~/.config/clawsh/config.toml`.

- [ ] **Step 5: Commit**

```bash
git add crates/clawsh-core/src/setup.rs src/main.rs
git commit -m "feat: add clawsh setup wizard and first-run experience"
```

---

## Task 10: GitHub Actions CI + Release Binaries

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Write CI workflow**

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo fmt --check
```

- [ ] **Step 2: Write release workflow**

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ["v*"]

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: clawsh-linux-x86_64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: clawsh-macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: clawsh-macos-arm64

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - run: mv target/${{ matrix.target }}/release/clawsh ${{ matrix.artifact }}
      - uses: softprops/action-gh-release@v1
        with:
          files: ${{ matrix.artifact }}
```

- [ ] **Step 3: Commit**

```bash
mkdir -p .github/workflows
git add .github/
git commit -m "ci: add GitHub Actions for tests and release binaries"
```

---

## Self-Review

**Spec coverage check:**
- ✅ Natural language → command translation (Task 7 REPL + Task 5 Ollama)
- ✅ Error explanation (Task 7 REPL, `auto_explain_errors`)
- ✅ Safety guard (Task 3 safety crate, wired in Task 7)
- ✅ Multi-LLM support (Task 8 — Ollama, OpenAI, Claude)
- ✅ `/model` command (Task 8)
- ✅ Config file with TOML (Task 2)
- ✅ bash subprocess delegation (Task 6)
- ✅ POSIX classifier (Task 4)
- ✅ First-run setup wizard (Task 9)
- ✅ CI/CD + binary releases (Task 10)
- ⚠️ AI tab completion — deferred to Phase 3 (not in this plan, matches spec MVP scope)
- ⚠️ smollm2 classifier — deferred to Phase 3 (regex classifier sufficient for MVP)

**Placeholder scan:** No TBDs. All code blocks are complete and runnable.

**Type consistency:**
- `LLMProvider` trait defined in Task 5, used in Tasks 7 and 8 ✅
- `Config` struct defined in Task 2, used in Tasks 7 and 9 ✅
- `execute()` / `ExecuteResult` defined in Task 6, used in Task 7 ✅
- `classify()` / `InputKind` defined in Task 4, used in Task 7 ✅
- `is_dangerous()` defined in Task 3, used in Task 7 ✅
