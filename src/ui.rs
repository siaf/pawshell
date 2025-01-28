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
            ])
            .split(main_area);

        // Calculate visible lines in chat area
        let chat_height = chunks[2].height as usize;
        
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
        let mut messages_text: Vec<Line> = self.messages.iter().flat_map(|msg| {
            let mut lines = Vec::new();
            // Extract the role and content from the message
            let (role, content) = if msg.starts_with("user:") || msg.starts_with("assistant:") {
                let parts: Vec<&str> = msg.splitn(2, ':').collect();
                (parts[0], parts.get(1).map_or("", |v| v.trim()))
            } else if msg.starts_with("You: ") {
                ("user", &msg[5..])
            } else {
                let parts: Vec<&str> = msg.splitn(2, ':').collect();
                ("assistant", parts.get(1).map_or(msg.as_str(), |v| v.trim()))
            };

            // Format based on the role
            match role {
                "user" => {
                    for (i, line) in content.split('\n').enumerate() {
                        let line = line.trim();
                        if !line.is_empty() {
                            if i == 0 {
                                lines.push(Line::from(vec![
                                    Span::styled("You: ", Style::default().fg(Color::Cyan).bold()),
                                    Span::styled(line, Style::default().fg(Color::White))
                                ]));
                            } else {
                                lines.push(Line::from(vec![
                                    Span::styled("     ", Style::default().fg(Color::Cyan)),
                                    Span::styled(line, Style::default().fg(Color::White))
                                ]));
                            }
                        }
                    }
                },
                _ => {
                    // Clean up content by removing extra whitespace and empty lines
                    let content = content.lines()
                        .map(|line| line.trim())
                        .filter(|line| !line.is_empty())
                        .collect::<Vec<_>>();

                    for (i, line) in content.iter().enumerate() {
                        if i == 0 {
                            lines.push(Line::from(vec![
                                Span::styled(format!("{}: ", pet_name), Style::default().fg(mood_color).bold()),
                                Span::styled(*line, Style::default().fg(Color::Gray))
                            ]));
                        } else {
                            lines.push(Line::from(vec![
                                Span::styled("     ", Style::default().fg(mood_color)),
                                Span::styled(*line, Style::default().fg(Color::Gray))
                            ]));
                        }
                    }
                    lines.push(Line::from(""));
                }
            }
            lines
        }).collect();

        // Add the current input line with cursor before creating the paragraph
        let cursor = "█";
        let input_line = Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Cyan).bold()),
            Span::styled(&self.input, Style::default().fg(Color::White)),
            Span::styled(cursor, Style::default().fg(Color::White).add_modifier(Modifier::SLOW_BLINK))
        ]);
        messages_text.push(input_line);

        let messages_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(" Chat History ", Style::default().fg(Color::White).bold()));

        // Calculate total lines and adjust scroll offset to keep cursor visible
        let total_lines = messages_text.len();
        if total_lines > chat_height.saturating_sub(2) { // Account for borders
            self.scroll_offset = total_lines.saturating_sub(chat_height.saturating_sub(2));
        } else {
            self.scroll_offset = 0;
        }

        let messages_paragraph = Paragraph::new(messages_text)
            .block(messages_block)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset as u16, 0))
            .alignment(Alignment::Left);

        f.render_widget(messages_paragraph, chunks[2]); // Updated index
    }
}