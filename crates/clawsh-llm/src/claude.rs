use crate::openai::OpenAIProvider;

pub fn new_claude_provider(api_key: &str, model: &str) -> OpenAIProvider {
    OpenAIProvider::new(api_key, model, "https://api.anthropic.com/v1")
}
