use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum LLMProvider {
    OpenAI,
    Ollama,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub command_history_limit: usize,
    pub pet_name: String,
    pub pet_ascii: String,
    pub llm_provider: LLMProvider,
    pub ollama_url: String,
    pub ollama_model: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            command_history_limit: 50,
            pet_name: String::from("Whiskers"),
            pet_ascii: String::from(r#"
  /\___/\
 (  o o  )
 (  =^=  )
  (____)
"#),
            llm_provider: LLMProvider::OpenAI,
            ollama_url: String::from("http://localhost:11434"),
            ollama_model: String::from("llama2"),
        }
    }
}