mod pet;
mod llm;
mod ui;

use crossterm::event::{self, Event, KeyCode};
use std::time::{Duration, Instant};
use dotenv::dotenv;

use crate::pet::PetState;
use crate::llm::{OpenAIBackend, LLMBackend};
use crate::ui::AppUI;

use ratatui::prelude::*;

struct App {
    ui: AppUI,
    state: PetState,
    llm: OpenAIBackend,
}

impl App {
    fn new() -> Self {
        let state: PetState = confy::load("petcli", None).unwrap_or_default();
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not found in environment variables");
        let llm = OpenAIBackend::new(api_key);
        let mut ui = AppUI::new();

        // Load chat history into messages
        for (user_msg, pet_response) in state.chat_history.iter() {
            ui.add_message(format!("You: {}", user_msg));
            ui.add_message(format!("{}: {}", state.name, pet_response));
        }

        Self { ui, state, llm }
    }

    async fn handle_input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.ui.input.is_empty() {
            let user_message = self.ui.input.clone();
            self.ui.add_message(format!("You: {}", user_message));

            // Update mood and interaction time
            self.state.last_interaction = chrono::Utc::now();
            self.state.mood = (self.state.mood + 0.1).min(1.0);

            // Get response from LLM
            let response = match self.llm.generate_response(&user_message).await {
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
    app.ui.render(f, &app.state.name, app.state.mood);
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
