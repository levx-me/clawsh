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
            models: ModelsConfig::default(),
            providers: HashMap::new(),
            safety: SafetyConfig::default(),
            shell: ShellConfig::default(),
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
