use clawsh_config::Config;
use clawsh_core::{
    repl::Repl,
    setup::{config_path, ensure_ready, first_run_setup},
};
use clawsh_llm::ollama::OllamaProvider;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.get(1).map(|s| s.as_str()) == Some("setup") {
        return first_run_setup().await;
    }

    // First run: config doesn't exist yet — auto-setup before starting REPL
    if !config_path().exists() {
        first_run_setup().await?;
    } else {
        // Config exists but Ollama might not be running — ensure it's ready
        ensure_ready().await?;
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
