use crossterm::event::{Event, KeyCode};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

const CAT_ASCII: &str = r#"
  /\___/\
 (  o o  )
 (  =^=  )
  (____)"
"#;

pub struct AppUI {
    pub input: String,
    pub messages: Vec<String>,
    pub scroll_state: ListState,
    pub scroll_offset: usize,
}

impl AppUI {
    pub fn new() -> Self {
        let mut scroll_state = ListState::default();
        scroll_state.select(Some(0));
        Self {
            input: String::new(),
            messages: vec!["Welcome back! Type your message and press Enter to chat.".to_string()],
            scroll_state,
            scroll_offset: 0,
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            self.scroll_state.select(Some(self.scroll_offset));
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset < self.messages.len().saturating_sub(1) {
            self.scroll_offset += 1;
            self.scroll_state.select(Some(self.scroll_offset));
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.messages.len().saturating_sub(1);
        self.scroll_state.select(Some(self.scroll_offset));
    }

    pub fn add_message(&mut self, message: String) {
        const MAX_MESSAGES: usize = 100;
        if self.messages.len() >= MAX_MESSAGES {
            self.messages.remove(0);
        }
        self.messages.push(message);
        self.scroll_to_bottom();
    }

    pub fn render(&mut self, f: &mut Frame, pet_name: &str, pet_mood: f32) {
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
            .title(format!("{} (Mood: {:.0}%)", pet_name, pet_mood * 100.0));
        let pet_text = Paragraph::new(CAT_ASCII)
            .block(pet_block)
            .alignment(Alignment::Center);
        f.render_widget(pet_text, chunks[0]);

        // Chat history
        let messages_text = self.messages.join("\n");
        let messages_paragraph = Paragraph::new(messages_text)
            .block(Block::default().borders(Borders::ALL).title("Chat History"))
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset as u16, 0))
            .alignment(Alignment::Left)
            .style(Style::default());

        f.render_widget(messages_paragraph, chunks[1]);

        // Input box
        let input = Paragraph::new(self.input.as_str())
            .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input, chunks[2]);
    }
}