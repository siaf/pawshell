use crossterm::event::{Event, KeyCode};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

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
        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .map(|m| ListItem::new(Line::from(m.as_str())).style(Style::default()))
            .collect();
        let messages_block = Block::default()
            .borders(Borders::ALL)
            .title("Chat History");

        let messages_list = List::new(messages)
            .block(messages_block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .style(Style::default());

        f.render_stateful_widget(messages_list, chunks[1], &mut self.scroll_state);

        // Input box
        let input = Paragraph::new(self.input.as_str())
            .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input, chunks[2]);
    }
}