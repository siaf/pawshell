use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub command_history_limit: usize,
    pub pet_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            command_history_limit: 50, // Default to keeping last 5 commands
            pet_name: String::from("Whiskers"), // Default pet name
        }
    }
}