//! Pet state and behavior management for PetCLI
//!
//! This module defines the core pet-related functionality, including:
//! - Pet state tracking (mood, interaction history)
//! - Pet behavior traits and implementations
//! - Chat history management
//!
//! Consider splitting this module if pet behaviors become more complex:
//! - Move chat history to a dedicated ChatHistory module
//! - Create separate modules for different pet personalities/behaviors
//! - Add a dedicated module for mood management algorithms

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Represents the current state of the pet, including mood and interaction history
#[derive(Serialize, Deserialize)]
pub struct PetState {
    pub name: String,
    pub mood: f32,          // 0.0 to 1.0
    pub last_interaction: DateTime<Utc>,
    pub chat_history: Vec<(String, String)>,  // (user_message, pet_response)
}

/// Defines the core behavior interface for pets
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