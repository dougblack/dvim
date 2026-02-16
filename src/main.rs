mod buffer;
mod editor;
mod error;
mod mode;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rvim", about = "A vim-like text editor")]
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
            handle_key(editor, key, viewport_height);
        }
    }
    Ok(())
}

fn handle_key(editor: &mut editor::Editor, key: KeyEvent, viewport_height: usize) {
    match editor.mode {
        mode::Mode::Normal => handle_normal_key(editor, key, viewport_height),
        mode::Mode::Insert => handle_insert_key(editor, key, viewport_height),
    }
}

fn handle_normal_key(editor: &mut editor::Editor, key: KeyEvent, viewport_height: usize) {
    // Handle 'g' prefix for gg command
    if editor.pending_g {
        editor.pending_g = false;
        if key.code == KeyCode::Char('g') {
            editor.goto_top();
        }
        return;
    }

    match key.code {
        // Quit
        KeyCode::Char('q') => editor.quit(),

        // Enter insert mode
        KeyCode::Char('i') => editor.enter_insert_mode(),
        KeyCode::Char('a') => editor.enter_insert_mode_append(),
        KeyCode::Char('o') => editor.enter_insert_mode_open_below(),
        KeyCode::Char('O') => editor.enter_insert_mode_open_above(),

        // Movement
        KeyCode::Char('h') | KeyCode::Left => editor.move_left(),
        KeyCode::Char('j') | KeyCode::Down => editor.move_down(),
        KeyCode::Char('k') | KeyCode::Up => editor.move_up(),
        KeyCode::Char('l') | KeyCode::Right => editor.move_right(),

        // Jump to top/bottom
        KeyCode::Char('g') => editor.pending_g = true,
        KeyCode::Char('G') => editor.goto_bottom(),

        // Half-page scrolling
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            editor.scroll_half_page_down(viewport_height);
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            editor.scroll_half_page_up(viewport_height);
        }

        _ => {}
    }
}

fn handle_insert_key(editor: &mut editor::Editor, key: KeyEvent, _viewport_height: usize) {
    match key.code {
        KeyCode::Esc => editor.exit_insert_mode(),
        KeyCode::Enter => editor.insert_newline(),
        KeyCode::Backspace => editor.delete_char_back(),

        // Arrow keys still navigate
        KeyCode::Left => editor.move_left(),
        KeyCode::Down => editor.move_down(),
        KeyCode::Up => editor.move_up(),
        KeyCode::Right => editor.move_right(),

        // Printable characters
        KeyCode::Char(c) => editor.insert_char(c),

        _ => {}
    }
}
