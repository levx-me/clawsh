# clawsh — AI-Native Shell Design Spec

**Date:** 2026-04-09  
**Status:** Draft

---

## 1. Overview

`clawsh` is an AI-first, POSIX-compatible shell with a built-in local LLM. Users can operate their system using natural language while maintaining full compatibility with existing bash scripts. The name derives from Claude's claw icon. Any LLM provider can be swapped in; the default is a local model requiring no internet connection.

**Implementation language:** Rust  
**Default LLM:** Ollama + qwen2.5:3b (local, CPU-capable)  
**POSIX execution:** delegated to bash subprocess  
**Distribution:** open source

---

## 2. Core Design Principles

- **POSIX commands never touch the LLM** — zero latency, delegated directly to bash
- **Only natural language goes through the LLM** — a lightweight classifier decides first
- **No model lock-in** — any LLM provider can be swapped via config or shell command
- **Safety first** — dangerous commands always require confirmation
- **Local first** — default model runs entirely offline

---

## 3. Architecture

### 3.1 Data Flow

```
User input
     │
     ▼
┌─────────────┐
│  Classifier  │  ← rule-based (regex) + smollm2:135m
└─────────────┘
     │
     ├─── POSIX command ──→ bash -c "cmd"  (immediate execution)
     │
     └─── Natural language ──→ LLM Provider
                                    │
                               command generated
                                    │
                          ┌─────────────────┐
                          │  Confirm UI      │
                          │  > rm -rf /tmp   │
                          │  [y/N]           │
                          └─────────────────┘
                                    │
                                 execute
                                    │
                          error? → LLM explains cause
```

### 3.2 POSIX Execution Strategy

clawsh does not fork bash or implement its own POSIX parser. All POSIX execution is delegated to the system bash.

```rust
Command::new("bash").arg("-c").arg(cmd).spawn()
```

Benefits:
- bash updates are automatically inherited
- User `.bashrc`, aliases, and plugins work as-is
- No GPLv3 license contamination
- No multi-month parser implementation effort

---

## 4. Features

### 4.1 Natural Language Commands
```bash
❯ kill the process using port 8080
  → lsof -ti:8080 | xargs kill -9  [y/N] y
  ✓ done

❯ show files modified in the last week
  → find . -mtime -7 -type f
```

### 4.2 Automatic Error Explanation
When a command fails, the LLM automatically explains the cause and suggests a fix.
```bash
❯ git push origin main
  error: failed to push some refs...
  
  💡 The remote branch has commits not present locally.
     Try: git pull --rebase origin main, then push again.
```

### 4.3 AI Tab Completion
```bash
❯ docker run --[Tab]
  --rm          remove container after exit
  --network     configure networking
  --env-file    load environment variables from file
```

### 4.4 Context Awareness
- Current directory, git status, and recent command history included in LLM context
- Conversational follow-ups supported: "undo that last command"

### 4.5 Safety Guard
- Dangerous patterns auto-detected: `rm -rf`, `dd`, `chmod 777 -R`, `mkfs`, etc.
- Detected commands require explicit confirmation before execution
- Can be disabled in config

### 4.6 Multi-LLM Support
```bash
❯ /model claude        # switch to Claude API
❯ /model qwen2.5:3b    # switch back to local Ollama
❯ /model gpt-4o        # switch to OpenAI
❯ /model list          # list available models
```

Prompt shows current active model:
```
clawsh [qwen2.5:3b] ~/projects ❯
```

---

## 5. Configuration

```toml
# ~/.config/clawsh/config.toml

[models]
default = "qwen2.5:3b"      # command generation
classifier = "smollm2:135m" # classifier (ultra-lightweight)

[providers.ollama]
host = "http://localhost:11434"

[providers.claude]
api_key = "sk-ant-..."       # optional

[providers.openai]
api_key = "sk-..."           # optional

[safety]
confirm_dangerous = true
auto_explain_errors = true

[shell]
history_size = 10000
prompt = "{model} {cwd} ❯"
```

Supported providers:
- Ollama (local)
- Claude API (Anthropic)
- OpenAI API
- Gemini API (Google)
- Any OpenAI-compatible endpoint (LM Studio, llama.cpp server, etc.)

---

## 6. Component Structure

```
clawsh/
├── crates/
│   ├── clawsh-core/       # shell REPL loop, history, bash delegation
│   ├── clawsh-classifier/ # natural language vs POSIX classifier
│   ├── clawsh-llm/        # LLM provider abstraction
│   │   ├── provider.rs    # LLMProvider trait
│   │   ├── ollama.rs
│   │   ├── claude.rs
│   │   ├── openai.rs
│   │   └── gemini.rs
│   ├── clawsh-ui/         # TUI, prompt, autocomplete
│   └── clawsh-safety/     # dangerous command detection
└── src/
    └── main.rs
```

### LLM Abstraction Trait

```rust
#[async_trait]
trait LLMProvider: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String>;
    fn name(&self) -> &str;
}
```

Adding a new provider = implement this trait only.

---

## 7. Distribution

```bash
# cargo
cargo install clawsh

# install script (Linux/macOS)
curl -fsSL https://clawsh.dev/install | sh

# package managers
brew install clawsh
apt install clawsh
```

- GitHub Actions builds Linux/macOS/Windows binaries on every release
- First run detects missing Ollama and guides installation
- `clawsh setup` runs an interactive first-time setup wizard

---

## 8. MVP Scope

**Phase 1 (MVP):**
- [ ] Basic shell REPL (rustyline)
- [ ] Rule-based classifier (regex)
- [ ] Ollama integration + natural language → command translation
- [ ] Confirmation UI
- [ ] bash delegation for execution
- [ ] Basic config file (TOML)

**Phase 2:**
- [ ] Automatic error explanation
- [ ] Claude / OpenAI providers
- [ ] `/model` switching command
- [ ] Safety guard

**Phase 3:**
- [ ] AI tab completion
- [ ] Context awareness (git status, recent history)
- [ ] smollm2 classifier integration
- [ ] Package manager distribution

---

## 9. Tech Stack

| Role | Crate |
|---|---|
| Input handling | `rustyline` |
| TUI | `ratatui` |
| HTTP client | `reqwest` |
| Async runtime | `tokio` |
| Config parsing | `serde` + `toml` |
| Unix bindings | `nix` |
