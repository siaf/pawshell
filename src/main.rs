mod pet;
mod llm;
mod ui;
mod config;
mod config_path;

use crossterm::event::{self, Event, KeyCode};
use std::time::{Duration, Instant};
use dotenv::dotenv;

use crate::pet::PetState;
use crate::llm::{OpenAIBackend, LLMBackend};
use crate::ui::AppUI;

use ratatui::prelude::*;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use dirs;

struct App {
    ui: AppUI,
    state: PetState,
    llm: OpenAIBackend,
    recent_commands: Vec<String>,
    config: config::Config,
}

impl App {
    fn new() -> Self {
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
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not found in environment variables");
        let llm = OpenAIBackend::new(api_key);
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

    fn load_shell_history(&mut self) {
        if let Some(home_dir) = dirs::home_dir() {
            let history_files = vec![
                home_dir.join(".zsh_history"),
                home_dir.join(".bash_history"),
                home_dir.join(".history"),
            ];

            for history_file in history_files {
                if let Ok(lines) = read_lines(history_file) {
                    for line in lines.flatten() {
                        // Clean the history line (remove timestamps if present)
                        let cmd = clean_history_line(&line);
                        if !cmd.is_empty() {
                            self.recent_commands.push(cmd);
                            if self.recent_commands.len() > self.config.command_history_limit {
                                self.recent_commands.remove(0);
                            }
                        }
                    }
                    break; // Stop after finding first available history file
                }
            }
        }
    }
}

fn clean_history_line(line: &str) -> String {
    // Remove common history file formatting
    // zsh format: : 1234567890:0;command
    // bash format: #1234567890
    // command
    if line.starts_with(':') {
        if let Some(cmd) = line.split(';').last() {
            return cmd.trim().to_string();
        }
    }
    // Remove timestamp prefix if present
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

impl App {
    async fn handle_input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.ui.input.is_empty() {
            let user_message = self.ui.input.clone();
            self.ui.add_message(format!("You: {}", user_message));

            // Track command if it looks like one
            if user_message.starts_with('$') {
                if let Some(cmd) = user_message.strip_prefix('$') {
                    self.recent_commands.push(cmd.trim().to_string());
                    // Keep only last 5 commands
                    if self.recent_commands.len() > 5 {
                        self.recent_commands.remove(0);
                    }
                }
            }

            // Update mood and interaction time
            self.state.last_interaction = chrono::Utc::now();
            self.state.mood = (self.state.mood + 0.1).min(1.0);

            // Get response from LLM
            let response = match self.llm.generate_response(&self.llm.format_prompt(&user_message, Some(&self.recent_commands))).await {
                Ok(response) => response,
                Err(_) => {
                    // Fallback responses
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


    fn update(&mut self) {
        let now = chrono::Utc::now();
        let hours_since_last = (now - self.state.last_interaction).num_hours() as f32;
        self.state.mood = (self.state.mood - (hours_since_last * 0.1)).max(0.1).min(1.0);
    }

    fn save_state(&self) -> Result<(), confy::ConfyError> {
        confy::store("petcli", None, &self.state)
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    app.ui.render(f, &app.state.name, app.state.mood, &app.config.pet_ascii);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Terminal initialization
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => {
                        app.handle_input().await?
                    }
                    KeyCode::Up => {
                        app.ui.scroll_up();
                    }
                    KeyCode::Down => {
                        app.ui.scroll_down();
                    }
                    KeyCode::PageUp => {
                        for _ in 0..5 { app.ui.scroll_up(); }
                    }
                    KeyCode::PageDown => {
                        for _ in 0..5 { app.ui.scroll_down(); }
                    }
                    KeyCode::Char(c) => {
                        app.ui.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.ui.input.pop();
                    }
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.update();
            last_tick = Instant::now();
        }
    }

    // Cleanup
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;

    Ok(())
}
