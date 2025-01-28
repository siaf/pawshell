//! Configuration management for PetCLI
//!
//! This module handles the application's configuration settings, including:
//! - LLM provider selection and settings
//! - Pet customization options
//! - Command history limits
//!
//! The configuration is stored in TOML format and supports multiple LLM backends.
//! Consider splitting the pet-specific configuration into a separate module if
//! pet customization options grow more complex.

use serde::{Deserialize, Serialize};

/// Supported Language Model providers
#[derive(Debug, Serialize, Deserialize)]
pub enum LLMProvider {
    OpenAI,
    Ollama,
}

/// Main configuration structure for the application
/// 
/// Handles both application-level settings and pet customization.
/// If pet customization options grow, consider moving them to a dedicated
/// PetConfig struct in the pet module.
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