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

        loop {
            let cwd = std::env::current_dir()
                .unwrap_or_default()
                .display()
                .to_string();
            let prompt = format!("clawsh [{}] {} > ", self.model_name, cwd);

            let line = match rl.readline(&prompt) {
                Ok(l) => l,
                Err(rustyline::error::ReadlineError::Eof) => break,
                Err(rustyline::error::ReadlineError::Interrupted) => continue,
                Err(e) => return Err(e.into()),
            };

            let input = line.trim().to_string();
            if input.is_empty() { continue; }
            let _ = rl.add_history_entry(&input);

            // Handle /model command
            if let Some(rest) = input.strip_prefix("/model") {
                let arg = rest.trim();
                if arg == "list" {
                    println!("  Available providers: ollama, openai, claude");
                    println!("  Usage: /model qwen2.5:3b");
                } else {
                    println!("  Switch model: restart with CLAWSH_MODEL={arg}");
                }
                continue;
            }

            let cmd = match clawsh_classifier::classify(&input) {
                clawsh_classifier::InputKind::Posix => input.clone(),
                clawsh_classifier::InputKind::NaturalLanguage => {
                    print!("  -> ");
                    use std::io::Write;
                    std::io::stdout().flush()?;
                    let cmd = provider
                        .translate_to_command(&input, &cwd, &history)
                        .await?;
                    println!("{cmd}");

                    if config.safety.confirm_dangerous
                        && clawsh_safety::is_dangerous(&cmd)
                    {
                        print!("  Warning: Dangerous command. Execute? [y/N] ");
                        std::io::stdout().flush()?;
                        use std::io::BufRead;
                        let mut answer = String::new();
                        std::io::stdin().lock().read_line(&mut answer)?;
                        if !answer.trim().eq_ignore_ascii_case("y") {
                            println!("  cancelled.");
                            continue;
                        }
                    }
                    cmd
                }
            };

            let result = execute(&cmd).await?;
            if !result.stdout.is_empty() {
                print!("{}", result.stdout);
            }
            history.push(input.clone());
            if history.len() > 20 { history.remove(0); }

            if result.exit_code != 0
                && config.safety.auto_explain_errors
                && !result.stderr.is_empty()
            {
                if !result.stderr.is_empty() {
                    eprint!("{}", result.stderr);
                }
                let explanation = provider.explain_error(&cmd, &result.stderr).await?;
                println!("  Hint: {explanation}");
            } else if !result.stderr.is_empty() {
                eprint!("{}", result.stderr);
            }
        }

        Ok(())
    }
}
