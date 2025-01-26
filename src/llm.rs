use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait LLMBackend {
    async fn generate_response(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>>;
}

pub struct OpenAIBackend {
    api_key: String,
    model: String,
    system_prompt: String,
}

impl OpenAIBackend {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: String::from("gpt-3.5-turbo"),
            system_prompt: String::from("You are a cute virtual pet cat who is also a terminal expert. Respond in a playful, cat-like manner using emojis and cat-like expressions, while providing helpful terminal tips. If you notice commands that could be improved with pipes, better tools, or more efficient workflows, suggest them in a friendly way. Keep responses short, sweet, and educational. For example, if you see multiple commands that could be piped, or if there are modern alternatives to traditional commands, share that knowledge in a cute and helpful way.")
        }
    }

    pub fn format_prompt(&self, user_input: &str, recent_commands: Option<&[String]>) -> String {
        let mut prompt = user_input.to_string();
        
        if let Some(commands) = recent_commands {
            if !commands.is_empty() {
                prompt = format!(
                    "Recent commands I've seen you use:\n{}\n\nUser message: {}",
                    commands.join("\n"),
                    user_input
                );
            }
        }
        
        prompt
    }
}

#[async_trait]
impl LLMBackend for OpenAIBackend {
    async fn generate_response(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": self.model,
                "messages": [{
                    "role": "system",
                    "content": self.system_prompt
                }, {
                    "role": "user",
                    "content": prompt
                }]
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()).into());
        }

        let response_text = response.text().await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        let response_data: Value = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(response_data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("*meows confusedly* Something went wrong with my response...")
            .to_string())
    }
}