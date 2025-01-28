//! PetCLI - An interactive terminal pet companion with LLM capabilities
//!
//! This is the main entry point for the PetCLI application. It initializes the terminal
//! interface and starts the main application loop. The application is organized into several
//! modules, each handling specific functionality:
//!
//! - app: Core application logic and state management
//! - pet: Pet state and behavior definitions
//! - llm: Language model interface and implementations
//! - ui: Terminal user interface components
//! - config: Configuration management
//! - config_path: Configuration file path handling
//! - ollama: Ollama LLM backend implementation
//! - terminal: Terminal initialization and event handling

mod pet;
mod llm;
mod ui;
mod config;
mod config_path;
mod ollama;
mod app;
mod terminal;

use dotenv::dotenv;
use crate::app::App;
use crate::terminal::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let mut terminal = Terminal::<CrosstermBackend<io::Stdout>>::init()?;
    let app = App::new();
    terminal.run(app).await?;

    Ok(())
}
