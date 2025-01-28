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
