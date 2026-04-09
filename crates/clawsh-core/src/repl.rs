use rustyline::DefaultEditor;
use crate::executor::execute;

// ANSI helpers — wrap in \x01..\x02 so rustyline counts display width correctly
macro_rules! ansi {
    ($code:expr, $text:expr) => {
        format!("\x01\x1b[{}m\x02{}\x01\x1b[0m\x02", $code, $text)
    };
}

fn bold(s: &str)    -> String { ansi!("1",     s) }
fn cyan(s: &str)    -> String { ansi!("36",    s) }
fn green(s: &str)   -> String { ansi!("32",    s) }
fn magenta(s: &str) -> String { ansi!("35",    s) }
fn blue(s: &str)    -> String { ansi!("34",    s) }
fn yellow(s: &str)  -> String { ansi!("33",    s) }
fn red(s: &str)     -> String { ansi!("31",    s) }
fn dim(s: &str)     -> String { ansi!("2",     s) }

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

            // clawsh [model] ~/path ❯
            let prompt = format!(
                "{} {} {} {} ",
                bold(&cyan("clawsh")),
                dim(&magenta(&format!("[{}]", self.model_name))),
                green(&cwd),
                blue("❯"),
            );

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
                    use std::io::Write;
                    print!("  \x1b[36m→\x1b[0m ");
                    std::io::stdout().flush()?;
                    let cmd = provider
                        .translate_to_command(&input, &cwd, &history)
                        .await?;
                    println!("\x1b[1m{cmd}\x1b[0m");

                    if config.safety.confirm_dangerous
                        && clawsh_safety::is_dangerous(&cmd)
                    {
                        print!("  \x1b[31m⚠️  Dangerous command. Execute? [y/N]\x1b[0m ");
                        std::io::stdout().flush()?;
                        use std::io::BufRead;
                        let mut answer = String::new();
                        std::io::stdin().lock().read_line(&mut answer)?;
                        if !answer.trim().eq_ignore_ascii_case("y") {
                            println!("  \x1b[2mcancelled.\x1b[0m");
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
                eprint!("{}", result.stderr);
                let explanation = provider.explain_error(&cmd, &result.stderr).await?;
                println!("  \x1b[33m💡 {explanation}\x1b[0m");
            } else if !result.stderr.is_empty() {
                eprint!("{}", result.stderr);
            }
        }

        Ok(())
    }
}
