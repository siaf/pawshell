use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, ListState, Paragraph, Wrap};
use ratatui::text::{Line, Span};

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

    pub fn scroll_to_bottom(&mut self) {
        // Calculate total lines including line breaks
        let total_lines = self.messages.iter().map(|msg| {
            let line_count = msg.split('\n').count();
            // Add extra line after pet responses
            if !msg.starts_with("You: ") {
                line_count + 1
            } else {
                line_count
            }
        }).sum();
        
        self.scroll_offset = total_lines;
        self.scroll_state.select(Some(self.scroll_offset));
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            self.scroll_state.select(Some(self.scroll_offset));
        }
    }

    pub fn scroll_down(&mut self) {
        // Calculate total lines same as in scroll_to_bottom
        let total_lines = self.messages.iter().map(|msg| {
            let line_count = msg.split('\n').count();
            if !msg.starts_with("You: ") {
                line_count + 1
            } else {
                line_count
            }
        }).sum();

        if self.scroll_offset < total_lines {
            self.scroll_offset += 1;
            self.scroll_state.select(Some(self.scroll_offset));
        }
    }

    pub fn add_message(&mut self, message: String) {
        const MAX_MESSAGES: usize = 100;
        if self.messages.len() >= MAX_MESSAGES {
            self.messages.remove(0);
        }
        self.messages.push(message);
        self.scroll_to_bottom();
    }

    pub fn render(&mut self, f: &mut Frame, pet_name: &str, pet_mood: f32, pet_ascii: &str) {
        // Add margin around the entire UI
        let main_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Top margin
                Constraint::Min(3),     // Content
                Constraint::Length(1),  // Bottom margin
            ])
            .margin(1)
            .split(f.size())[1];

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),    // Pet ASCII art
                Constraint::Length(1),     // Spacing
                Constraint::Min(5),        // Chat area
                Constraint::Length(1),     // Spacing
                Constraint::Length(3),     // Input box
            ])
            .split(main_area);

        // Pet ASCII art section with modern styling
        let mood_color = match pet_mood {
            m if m > 0.8 => Color::LightGreen,
            m if m > 0.4 => Color::Yellow,
            _ => Color::LightRed,
        };

        let pet_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(mood_color))
            .title(Span::styled(
                format!(" {} (Mood: {:.0}%) ", pet_name, pet_mood * 100.0),
                Style::default().fg(mood_color).bold()
            ))
            .style(Style::default().bg(Color::Reset));
        
        let pet_text = Paragraph::new(pet_ascii)
            .block(pet_block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(mood_color));
        
        f.render_widget(pet_text, chunks[0]);

        // Chat history with modern styling
        let messages_text: Vec<Line> = self.messages.iter().flat_map(|msg| {
            let mut lines = Vec::new();
            if msg.starts_with("You: ") {
                let (prefix, content) = msg.split_at(5);
                for (i, line) in content.split('\n').enumerate() {
                    if i == 0 {
                        lines.push(Line::from(vec![
                            Span::styled(prefix, Style::default().fg(Color::Cyan).bold()),
                            Span::raw(" "),  // Add spacing
                            Span::styled(line, Style::default().fg(Color::White))
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled("     ", Style::default().fg(Color::Cyan)),
                            Span::raw(" "),  // Add spacing
                            Span::styled(line, Style::default().fg(Color::White))
                        ]));
                    }
                }
            } else {
                let (prefix, content) = msg.split_once(": ").unwrap_or((msg, ""));
                for (i, line) in content.split('\n').enumerate() {
                    if i == 0 {
                        lines.push(Line::from(vec![
                            Span::styled(format!("{}: ", prefix), Style::default().fg(mood_color).bold()),
                            Span::raw(" "),  // Add spacing
                            Span::styled(line, Style::default().fg(Color::Gray))
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled("     ", Style::default().fg(mood_color)),
                            Span::raw(" "),  // Add spacing
                            Span::styled(line, Style::default().fg(Color::Gray))
                        ]));
                    }
                }
                lines.push(Line::from(""));
            }
            lines
        }).collect();

        let messages_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(" Chat History ", Style::default().fg(Color::White).bold()));

        let messages_paragraph = Paragraph::new(messages_text)
            .block(messages_block)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset as u16, 0))
            .alignment(Alignment::Left);

        f.render_widget(messages_paragraph, chunks[2]); // Updated index

        // Input box with modern styling
        let input = Paragraph::new(self.input.as_str())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(Span::styled(" Input ", Style::default().fg(Color::Blue).bold())))
            .style(Style::default().fg(Color::White));
            
        f.render_widget(input, chunks[4]); // Updated index
    }
}