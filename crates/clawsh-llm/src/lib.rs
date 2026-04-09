pub mod ollama;
pub mod openai;
pub mod claude;

use async_trait::async_trait;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn translate_to_command(
        &self,
        query: &str,
        cwd: &str,
        history: &[String],
    ) -> anyhow::Result<String>;

    async fn explain_error(
        &self,
        cmd: &str,
        stderr: &str,
    ) -> anyhow::Result<String>;

    fn name(&self) -> &str;
}

pub type BoxedProvider = Box<dyn LLMProvider>;
