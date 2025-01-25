use crossterm::event::{self, Event, KeyCode};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use reqwest;
use dotenv::dotenv;

// ASCII art for the cat
const CAT_ASCII: &str = r#"
  /\___/\
 (  o o  )
 (  =^=  )
  (____)"
"#;

#[derive(Serialize, Deserialize)]
struct PetState {
    name: String,
    mood: f32,          // 0.0 to 1.0
    last_interaction: chrono::DateTime<chrono::Utc>,
    chat_history: Vec<(String, String)>,  // (user_message, pet_response)
}

impl PetState {
    async fn get_chatgpt_response(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| "OPENAI_API_KEY not found in environment variables")?;
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "gpt-3.5-turbo",
                "messages": [{
                    "role": "system",
                    "content": "You are a cute virtual pet cat. Respond in a playful, cat-like manner using emojis and cat-like expressions. Keep responses short and sweet."
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
    
        let response_data: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse response: {}", e))?;
    
        Ok(response_data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("*meows confusedly* Something went wrong with my response...")
            .to_string())
    }

    fn update_mood(&mut self) {
        let now = chrono::Utc::now();
        let hours_since_last = (now - self.last_interaction).num_hours() as f32;
        self.mood = (self.mood - (hours_since_last * 0.1)).max(0.1).min(1.0);
    }

    async fn get_response(&mut self, input: &str) -> String {
        self.last_interaction = chrono::Utc::now();
        self.mood = (self.mood + 0.1).min(1.0);

        match Self::get_chatgpt_response(input).await {
            Ok(response) => response,
            Err(_) => {
                // Fallback to basic responses if ChatGPT fails
                if input.to_lowercase().contains("treat") {
                    self.mood = (self.mood + 0.2).min(1.0);
                    "*purrs happily* Thank you for the treat! 😊".to_string()
                } else if input.to_lowercase().contains("play") {
                    self.mood = (self.mood + 0.15).min(1.0);
                    "*bounces around excitedly* I love to play! 🐱".to_string()
                } else if self.mood > 0.8 {
                    "*purrs contentedly* 😊".to_string()
                } else if self.mood > 0.4 {
                    "*looks at you curiously* Meow?".to_string()
                } else {
                    "*seems a bit distant* ...".to_string()
                }
            }
        }
    }
}

impl Default for PetState {
    fn default() -> Self {
        Self {
            name: String::from("Whiskers"),
            mood: 0.8,
            last_interaction: chrono::Utc::now(),
            chat_history: Vec::new(),
        }
    }
}

struct App {
    state: PetState,
    input: String,
    messages: Vec<String>,
    scroll_state: ListState,
    scroll_offset: usize,
}

impl App {
    fn new() -> Self {
        let state: PetState = confy::load("petcli", None).unwrap_or_default();
        let mut messages = vec!["Welcome back! Type your message and press Enter to chat.".to_string()];
        // Load chat history into messages
        for (user_msg, pet_response) in state.chat_history.iter() {
            messages.push(format!("You: {}", user_msg));
            messages.push(format!("{}: {}", state.name, pet_response));
        }
        let mut scroll_state = ListState::default();
        scroll_state.select(Some(0));
        Self {
            state,
            input: String::new(),
            messages,
            scroll_state,
            scroll_offset: 0,
        }
    }

    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            self.scroll_state.select(Some(self.scroll_offset));
        }
    }

    fn scroll_down(&mut self) {
        if self.scroll_offset < self.messages.len().saturating_sub(1) {
            self.scroll_offset += 1;
            self.scroll_state.select(Some(self.scroll_offset));
        }
    }

    fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.messages.len().saturating_sub(1);
        self.scroll_state.select(Some(self.scroll_offset));
    }



    async fn handle_user_input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.input.is_empty() {
            let user_message = self.input.clone();
            self.add_message(format!("You: {}", user_message));
            let response = self.state.get_response(&user_message).await;
            self.add_message(format!("{}: {}", self.state.name, response));
            self.state.chat_history.push((user_message, response));
            self.input.clear();
            self.save_state()?;
        }
        Ok(())
    }



    fn add_message(&mut self, message: String) {
        const MAX_MESSAGES: usize = 100;
        if self.messages.len() >= MAX_MESSAGES {
            self.messages.remove(0);
        }
        self.messages.push(message);
        self.scroll_to_bottom();
    }
}

impl App {
    fn update(&mut self) {
        self.state.update_mood();
    }

    fn save_state(&self) -> Result<(), confy::ConfyError> {
        confy::store("petcli", None, &self.state)
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),  // Pet ASCII art
            Constraint::Min(5),     // Chat area
            Constraint::Length(3),   // Input box
        ])
        .split(f.size());

    // Pet ASCII art section
    let pet_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("{}  (Mood: {:.0}%)", app.state.name, app.state.mood * 100.0));
    let pet_text = Paragraph::new(CAT_ASCII)
        .block(pet_block)
        .alignment(Alignment::Center);
    f.render_widget(pet_text, chunks[0]);

    // Chat history
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .map(|m| ListItem::new(Line::from(m.as_str())).style(Style::default()))
        .collect();
    let messages_block = Block::default()
        .borders(Borders::ALL)
        .title("Chat History");

    let messages_area = chunks[1];
    let messages_list = List::new(messages)
        .block(messages_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .style(Style::default());

    app.scroll_state.select(Some(app.messages.len().saturating_sub(1)));
    f.render_stateful_widget(messages_list, messages_area, &mut app.scroll_state);

    // Input box
    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[2]);
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
                        if !app.input.is_empty() {
                            app.add_message(format!("You: {}", app.input));
                            let response = app.state.get_response(&app.input).await;
                            app.add_message(format!("{}: {}", app.state.name, response));
                            app.state.chat_history.push((app.input.clone(), response.clone()));
                            app.input.clear();
                            app.save_state()?;
                        }
                    }
                    KeyCode::Up => {
                        app.scroll_up();
                    }
                    KeyCode::Down => {
                        app.scroll_down();
                    }
                    KeyCode::PageUp => {
                        for _ in 0..5 { app.scroll_up(); }
                    }
                    KeyCode::PageDown => {
                        for _ in 0..5 { app.scroll_down(); }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
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
