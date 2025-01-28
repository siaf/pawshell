//! Terminal interface and event handling for PetCLI
//!
//! This module manages the terminal interface using the crossterm and ratatui libraries.
//! It handles:
//! - Terminal initialization and cleanup
//! - Event processing (keyboard input)
//! - UI rendering and update loop
//! - Terminal state management
//!
//! Consider splitting the event handling logic into a separate module if the
//! input handling becomes more complex.

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::execute;
use ratatui::prelude::*;
use std::io;
use std::time::{Duration, Instant};

use crate::app::App;

/// Terminal wrapper that manages the terminal interface and event loop
pub struct Terminal<B: Backend + io::Write> {
    terminal: ratatui::Terminal<B>,
}

impl<B: Backend + io::Write> Terminal<B> {
    pub fn new(backend: B) -> io::Result<Self> {
        let terminal = ratatui::Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn init() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        Terminal::new(backend)
    }

    pub async fn run(&mut self, mut app: App) -> io::Result<()> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(100);

        loop {
            let terminal = &mut self.terminal;
            terminal.draw(|f| {
                app.ui.render(f, &app.state.name, app.state.mood, &app.config.pet_ascii);
            })?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Enter => {
                            if let Err(e) = app.handle_input().await {
                                eprintln!("Error handling input: {}", e);
                            }
                        }
                        KeyCode::Up => app.ui.scroll_up(),
                        KeyCode::Down => app.ui.scroll_down(),
                        KeyCode::PageUp => {
                            for _ in 0..5 { app.ui.scroll_up(); }
                        }
                        KeyCode::PageDown => {
                            for _ in 0..5 { app.ui.scroll_down(); }
                        }
                        KeyCode::Char(c) => app.ui.input.push(c),
                        KeyCode::Backspace => { app.ui.input.pop(); }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                app.update();
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    // Remove unused ui method
    fn ui(&mut self, f: &mut Frame, app: &mut App) {
        app.ui.render(f, &app.state.name, app.state.mood, &app.config.pet_ascii);
    }
}

impl<B: Backend + io::Write> Drop for Terminal<B> {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        );
    }
}