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
        let history_ctx = history.last()
            .map(|h| format!("Recent command: {h}\n\n"))
            .unwrap_or_default();

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

    fn name(&self) -> &str { &self.model }
}
