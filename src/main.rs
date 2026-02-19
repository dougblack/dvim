mod buffer;
mod editor;
mod error;
mod mode;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dvim", about = "A vim-like text editor")]
struct Cli {
    /// File to open
    file: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let buffer = buffer::Buffer::from_file(cli.file)?;
    let mut editor = editor::Editor::new(buffer);

    // Set up terminal
    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main event loop
    let result = run_loop(&mut terminal, &mut editor);

    // Teardown — always runs, even if the loop errored
    terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    editor: &mut editor::Editor,
) -> Result<()> {
    while editor.running {
        let viewport_height = terminal.size()?.height.saturating_sub(1) as usize;
        editor.adjust_scroll(viewport_height);

        terminal.draw(|frame| {
            ui::draw(frame, editor);
        })?;

        if let Event::Key(key) = event::read()? {
            editor::handle_key(editor, key, viewport_height)?;
        }
    }
    Ok(())
}
