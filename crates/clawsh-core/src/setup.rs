use std::path::PathBuf;

const DEFAULT_MODEL: &str = "qwen2.5:3b";

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("clawsh")
        .join("config.toml")
}

async fn ollama_running() -> bool {
    reqwest::get("http://localhost:11434/api/tags")
        .await
        .is_ok()
}

async fn install_ollama() -> anyhow::Result<()> {
    println!("📦 Installing Ollama...");

    #[cfg(target_os = "macos")]
    {
        // Check if brew is available
        let brew = std::process::Command::new("which").arg("brew").output();
        if brew.map(|o| o.status.success()).unwrap_or(false) {
            let status = std::process::Command::new("brew")
                .args(["install", "ollama"])
                .status()?;
            if !status.success() {
                anyhow::bail!("brew install ollama failed");
            }
        } else {
            // Fallback: official install script
            let status = std::process::Command::new("sh")
                .args(["-c", "curl -fsSL https://ollama.com/install.sh | sh"])
                .status()?;
            if !status.success() {
                anyhow::bail!("Ollama install script failed");
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("sh")
            .args(["-c", "curl -fsSL https://ollama.com/install.sh | sh"])
            .status()?;
        if !status.success() {
            anyhow::bail!("Ollama install script failed");
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        anyhow::bail!("Automatic Ollama install is not supported on this platform. Please install manually: https://ollama.com");
    }

    println!("✓ Ollama installed");
    Ok(())
}

async fn start_ollama() -> anyhow::Result<()> {
    // Start ollama serve in background if not already running
    std::process::Command::new("sh")
        .args(["-c", "ollama serve &>/dev/null &"])
        .spawn()?;

    // Wait up to 5 seconds for it to be ready
    for _ in 0..10 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if ollama_running().await {
            return Ok(());
        }
    }
    anyhow::bail!("Ollama started but not responding")
}

async fn pull_model(model: &str) -> anyhow::Result<()> {
    println!("🤖 Pulling {model} (this may take a few minutes on first run)...");
    let status = std::process::Command::new("ollama")
        .args(["pull", model])
        .status()?;
    if !status.success() {
        anyhow::bail!("ollama pull {model} failed");
    }
    println!("✓ {model} ready");
    Ok(())
}

pub async fn ensure_ready() -> anyhow::Result<()> {
    // Check if ollama binary exists
    let ollama_installed = std::process::Command::new("which")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !ollama_installed {
        install_ollama().await?;
    }

    if !ollama_running().await {
        start_ollama().await?;
    }

    // Check if default model is already pulled
    let output = std::process::Command::new("ollama")
        .args(["list"])
        .output()?;
    let list = String::from_utf8_lossy(&output.stdout);
    if !list.contains(DEFAULT_MODEL.split(':').next().unwrap_or(DEFAULT_MODEL)) {
        pull_model(DEFAULT_MODEL).await?;
    }

    Ok(())
}

pub async fn first_run_setup() -> anyhow::Result<()> {
    println!("Welcome to clawsh! Setting up your AI shell...\n");

    ensure_ready().await?;

    let path = config_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    if !path.exists() {
        let default_config = format!(r#"[models]
default = "{DEFAULT_MODEL}"
classifier = "smollm2:135m"

[providers.ollama]
host = "http://localhost:11434"

[safety]
confirm_dangerous = true
auto_explain_errors = true

[shell]
history_size = 10000
prompt = "{{model}} {{cwd}} >"
"#);
        std::fs::write(&path, default_config)?;
        println!("✓ Config written to {}", path.display());
    }

    println!("\nAll done! Run 'clawsh' to start your AI shell.");
    Ok(())
}
