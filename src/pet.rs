use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
pub struct PetState {
    pub name: String,
    pub mood: f32,          // 0.0 to 1.0
    pub last_interaction: DateTime<Utc>,
    pub chat_history: Vec<(String, String)>,  // (user_message, pet_response)
}

pub trait Pet {
    fn update_mood(&mut self);
    fn get_response(&mut self, input: &str) -> String;
    fn get_name(&self) -> &str;
    fn get_mood(&self) -> f32;
    fn get_state(&self) -> &PetState;
    fn get_state_mut(&mut self) -> &mut PetState;
}

impl Default for PetState {
    fn default() -> Self {
        Self {
            name: String::from("Whiskers"),
            mood: 0.8,
            last_interaction: Utc::now(),
            chat_history: Vec::new(),
        }
    }
}