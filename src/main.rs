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
