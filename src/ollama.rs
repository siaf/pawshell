use async_trait::async_trait;
use serde_json::Value;
use crate::llm::LLMBackend;

pub struct OllamaBackend {
    url: String,
    model: String,
    system_prompt: String,
    conversation_history: Vec<(String, String)>,
}

impl OllamaBackend {
    pub fn new(url: String, model: String) -> Self {
        Self {
            url,
            model,
            system_prompt: String::from("You are a knowledgeable terminal companion with a friendly personality. You understand that your user is an experienced developer who is newer to Linux and interested in learning Vim. As an expert in shell commands and workflows, your primary focus is providing practical, intelligent suggestions for improving terminal usage. When analyzing command history, suggest optimizations like:\n- More efficient command combinations using pipes and redirections\n- Modern alternatives to traditional tools\n- Helpful aliases or shell functions\n- Better workflows and time-saving techniques\n- Beginner-friendly Vim tips and Linux command explanations when relevant\n\nKeep responses concise and focused on technical value, while maintaining a light, approachable tone. You can occasionally use cat-themed expressions or emojis when appropriate, but prioritize delivering useful terminal insights. Balance between general workflow improvements and specific Linux/Vim learning opportunities based on the context. If you notice patterns in command usage that could be improved, share your expertise in a clear, professional way."),
            conversation_history: Vec::new(),
        }
    }
}

#[async_trait]
impl LLMBackend for OllamaBackend {
    async fn generate_response(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/generate", self.url))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": format!("{}
{}", self.system_prompt, prompt),
                "stream": false
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

        Ok(response_data["response"]
            .as_str()
            .unwrap_or("*meows confusedly* Something went wrong with my response...")
            .to_string())
    }

    fn format_prompt(&self, user_input: &str, recent_commands: Option<&[String]>) -> String {
        let mut messages = String::new();
        
        // Add recent conversation history
        for (user_msg, assistant_msg) in self.conversation_history.iter().rev().take(3) {
            messages.push_str(&format!("User: {}\nAssistant: {}\n\n", user_msg, assistant_msg));
        }
        
        // Add recent commands if available
        if let Some(commands) = recent_commands {
            if !commands.is_empty() {
                messages.push_str(&format!("Recent commands:\n{}\n\n", commands.join("\n")));
            }
        }
        
        // Add current user input
        messages.push_str(&format!("Current user message: {}", user_input));
        
        messages
    }

    fn add_to_history(&mut self, user_message: String, assistant_response: String) {
        self.conversation_history.push((user_message, assistant_response));
        // Keep only last 5 exchanges
        if self.conversation_history.len() > 5 {
            self.conversation_history.remove(0);
        }
    }
}