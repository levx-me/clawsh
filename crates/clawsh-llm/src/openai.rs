use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::LLMProvider;

pub struct OpenAIProvider {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: String,
}

impl OpenAIProvider {
    pub fn new(api_key: &str, model: &str, base_url: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn complete(&self, system: &str, user: &str) -> anyhow::Result<String> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = ChatRequest {
            model: &self.model,
            messages: vec![
                Message { role: "system", content: system },
                Message { role: "user", content: user },
            ],
        };
        let resp: ChatResponse = self.client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.choices[0].message.content.trim().to_string())
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn translate_to_command(&self, query: &str, cwd: &str, _history: &[String]) -> anyhow::Result<String> {
        let system = "You are a shell command translator. Output ONLY the shell command. No explanation, no markdown, no backticks.";
        let user = format!("Current directory: {cwd}\nTranslate to bash: {query}");
        self.complete(system, &user).await
    }

    async fn explain_error(&self, cmd: &str, stderr: &str) -> anyhow::Result<String> {
        let system = "Explain shell errors briefly in 1-2 sentences and suggest a fix.";
        let user = format!("Command: {cmd}\nError: {stderr}");
        self.complete(system, &user).await
    }

    fn name(&self) -> &str { &self.model }
}
