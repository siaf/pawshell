//! Core application logic and state management for PetCLI
//!
//! This module contains the central App struct and its implementation, managing the overall
//! application state, user interactions, and integrations with other components. The module
//! currently handles multiple responsibilities that could potentially be split into separate
//! modules for better organization:
//!
//! Potential refactoring suggestions:
//! 1. Command History Management: Move shell history loading and processing into a separate module
//! 2. State Management: Create a dedicated module for handling pet state and persistence
//! 3. Input Handler: Separate command processing logic into its own module
//! 4. LLM Integration: Move LLM initialization and interaction logic to a dedicated module

use chrono::Utc;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;

use crate::pet::PetState;
use crate::llm::{LLMBackend, OpenAIBackend};
use crate::ollama::OllamaBackend;
use crate::config::LLMProvider;
use crate::ui::AppUI;
use crate::config;
use crate::config_path;

/// The main application struct that coordinates all components and manages the application state.
/// 
/// This struct is responsible for:
/// - Initializing and managing the UI
/// - Maintaining the pet's state
/// - Managing LLM interactions
/// - Processing user input and commands
/// - Handling shell history
pub struct App {
    pub ui: AppUI,
    pub state: PetState,
    llm: Box<dyn LLMBackend>,
    pub recent_commands: Vec<String>,
    pub config: config::Config,
}

impl App {
    pub fn new() -> Self {
        config_path::ensure_config_dir().expect("Failed to create config directory");
        let config_path = config_path::get_config_file_path(None);
        let config: config::Config = if config_path.exists() {
            std::fs::read_to_string(&config_path)
                .and_then(|content| toml::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
                .unwrap_or_default()
        } else {
            let default_config = config::Config::default();
            let toml = toml::to_string(&default_config).expect("Failed to serialize config");
            std::fs::write(&config_path, toml).expect("Failed to write default config");
            default_config
        };
        let mut state: PetState = confy::load("petcli", None).unwrap_or_default();
        state.name = config.pet_name.clone();

        let llm: Box<dyn LLMBackend> = match config.llm_provider {
            LLMProvider::OpenAI => {
                let api_key = std::env::var("OPENAI_API_KEY")
                    .expect("OPENAI_API_KEY not found in environment variables");
                Box::new(OpenAIBackend::new(api_key))
            },
            LLMProvider::Ollama => {
                Box::new(OllamaBackend::new(
                    config.ollama_url.clone(),
                    config.ollama_model.clone(),
                ))
            }
        };

        let mut ui = AppUI::new();

        // Load chat history into messages
        for (user_msg, pet_response) in state.chat_history.iter() {
            ui.add_message(format!("You: {}", user_msg));
            ui.add_message(format!("{}: {}", state.name, pet_response));
        }

        let mut app = Self { ui, state, llm, recent_commands: Vec::new(), config };
        app.load_shell_history();
        app
    }

    pub fn load_shell_history(&mut self) {
        if let Some(home_dir) = dirs::home_dir() {
            let history_files = vec![
                home_dir.join(".zsh_history"),
                home_dir.join(".bash_history"),
                home_dir.join(".history"),
            ];

            for history_file in history_files {
                if let Ok(lines) = read_lines(history_file) {
                    for line in lines.flatten() {
                        let cmd = clean_history_line(&line);
                        if !cmd.is_empty() {
                            self.recent_commands.push(cmd);
                            if self.recent_commands.len() > self.config.command_history_limit {
                                self.recent_commands.remove(0);
                            }
                        }
                    }
                    break;
                }
            }
        }
    }

    pub async fn handle_input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.ui.input.is_empty() {
            let user_message = self.ui.input.clone();
            
            if user_message.trim() == "/exit" {
                self.ui.add_message(format!("{}: Goodbye! Take care! 👋", self.state.name));
                self.save_state()?;
                return Ok(());
            }
            
            self.ui.add_message(format!("You: {}", user_message));

            if user_message.starts_with('$') {
                if let Some(cmd) = user_message.strip_prefix('$') {
                    self.recent_commands.push(cmd.trim().to_string());
                    if self.recent_commands.len() > 5 {
                        self.recent_commands.remove(0);
                    }
                }
            }

            if user_message.starts_with('/') {
                match user_message.trim() {
                    "/stats" => {
                        let stats = format!("Current Stats:\nMood: {:.0}%\nLast Interaction: {}\nChat History: {} messages",
                            self.state.mood * 100.0,
                            self.state.last_interaction.format("%Y-%m-%d %H:%M:%S UTC"),
                            self.state.chat_history.len());
                        self.ui.add_message(format!("{}: {}", self.state.name, stats));
                        self.ui.input.clear();
                        return Ok(());
                    },
                    "/clear" => {
                        self.ui.messages.clear();
                        self.ui.add_message("Chat window cleared.".to_string());
                        self.ui.input.clear();
                        return Ok(());
                    },
                    "/purge" => {
                        self.state.chat_history.clear();
                        self.ui.messages.clear();
                        self.ui.add_message("Chat history has been purged from disk.".to_string());
                        self.save_state()?;
                        self.ui.input.clear();
                        return Ok(());
                    },
                    "/help" => {
                        let help = "Available Commands:\n\
                        /stats - Display current pet statistics\n\
                        /clear - Clear chat window\n\
                        /purge - Remove all chat history\n\
                        /help  - Show this help message\n\
                        /exit  - Exit the application";
                        self.ui.add_message(format!("{}: {}", self.state.name, help));
                        self.ui.input.clear();
                        return Ok(());
                    },
                    "/exit" => {
                        self.ui.add_message(format!("{}: Goodbye! Take care! 👋", self.state.name));
                        self.save_state()?;
                        return Ok(());
                    },
                    _ => {}
                }
            }

            self.state.last_interaction = Utc::now();
            self.state.mood = (self.state.mood + 0.1).min(1.0);

            let response = match self.llm.generate_response(&self.llm.format_prompt(&user_message, Some(&self.recent_commands))).await {
                Ok(response) => {
                    self.llm.add_to_history(user_message.clone(), response.clone());
                    response
                }
                Err(_) => {
                    if user_message.to_lowercase().contains("treat") {
                        self.state.mood = (self.state.mood + 0.2).min(1.0);
                        "*purrs happily* Thank you for the treat! 😊".to_string()
                    } else if user_message.to_lowercase().contains("play") {
                        self.state.mood = (self.state.mood + 0.15).min(1.0);
                        "*bounces around excitedly* I love to play! 🐱".to_string()
                    } else if self.state.mood > 0.8 {
                        "*purrs contentedly* 😊".to_string()
                    } else if self.state.mood > 0.4 {
                        "*looks at you curiously* Meow?".to_string()
                    } else {
                        "*seems a bit distant* ...".to_string()
                    }
                }
            };

            self.ui.add_message(format!("{}: {}", self.state.name, response));
            self.state.chat_history.push((user_message, response));
            self.ui.input.clear();
            self.save_state()?;
        }
        Ok(())
    }

    pub fn update(&mut self) {
        let now = Utc::now();
        let hours_since_last = (now - self.state.last_interaction).num_hours() as f32;
        self.state.mood = (self.state.mood - (hours_since_last * 0.1)).max(0.1).min(1.0);
    }

    pub fn save_state(&self) -> Result<(), confy::ConfyError> {
        confy::store("petcli", None, &self.state)
    }
}

fn clean_history_line(line: &str) -> String {
    if line.starts_with(':') {
        if let Some(cmd) = line.split(';').last() {
            return cmd.trim().to_string();
        }
    }
    if let Some(cmd) = line.split_whitespace().last() {
        cmd.trim().to_string()
    } else {
        line.trim().to_string()
    }
}

fn read_lines(filename: PathBuf) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}