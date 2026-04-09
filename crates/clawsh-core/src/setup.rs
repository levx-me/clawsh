use std::path::PathBuf;

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("clawsh")
        .join("config.toml")
}

pub async fn first_run_setup() -> anyhow::Result<()> {
    println!("Welcome to clawsh! Let's get you set up.\n");

    let ollama_running = reqwest::get("http://localhost:11434/api/tags")
        .await
        .is_ok();

    if !ollama_running {
        println!("Warning: Ollama not detected. Install it at: https://ollama.com");
        println!("   After installing, run: ollama pull qwen2.5:3b\n");
    } else {
        println!("OK: Ollama detected at localhost:11434");
    }

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
prompt = "{model} {cwd} >"
"#;

    std::fs::write(&path, default_config)?;
    println!("OK: Config written to {}", path.display());
    println!("\nRun 'clawsh' to start your AI shell!");
    Ok(())
}
