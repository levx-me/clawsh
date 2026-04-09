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
    let toml_str = r#"
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
    let config: clawsh_config::Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.models.default, "llama3.2:3b");
    assert!(!config.safety.confirm_dangerous);
    assert_eq!(config.shell.history_size, 5000);
}
