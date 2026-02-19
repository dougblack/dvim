use super::Editor;
use crate::mode::Mode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key(
    editor: &mut Editor,
    key: KeyEvent,
    viewport_height: usize,
) -> anyhow::Result<()> {
    match editor.mode {
        Mode::Normal => handle_normal_key(editor, key, viewport_height),
        Mode::Insert => handle_insert_key(editor, key, viewport_height),
        Mode::Command => handle_command_key(editor, key),
    }
    Ok(())
}

fn handle_normal_key(editor: &mut Editor, key: KeyEvent, viewport_height: usize) {
    // Handle 'd' prefix for dd/dw commands
    if editor.pending_d {
        editor.pending_d = false;
        match key.code {
            KeyCode::Char('d') => editor.delete_line(),
            KeyCode::Char('w') => editor.delete_word(),
            _ => {}
        }
        return;
    }

    // Handle 'g' prefix for gg command
    if editor.pending_g {
        editor.pending_g = false;
        if key.code == KeyCode::Char('g') {
            editor.goto_top();
        }
        return;
    }

    match key.code {
        // Command mode
        KeyCode::Char(':') => editor.enter_command_mode(),

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

        // Viewport-relative jumps
        KeyCode::Char('H') => editor.goto_viewport_top(),
        KeyCode::Char('M') => editor.goto_viewport_middle(viewport_height),
        KeyCode::Char('L') => editor.goto_viewport_bottom(viewport_height),

        // Deletion
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            editor.scroll_half_page_down(viewport_height);
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            editor.scroll_half_page_up(viewport_height);
        }

        // Word motions
        KeyCode::Char('w') => editor.move_word_forward(),
        KeyCode::Char('b') => editor.move_word_backward(),
        KeyCode::Char('e') => editor.move_word_end(),

        // Line position motions
        KeyCode::Char('0') => editor.goto_line_start(),
        KeyCode::Char('$') => editor.goto_line_end(),
        KeyCode::Char('^') => editor.goto_first_non_blank(),

        // Normal mode deletion
        KeyCode::Char('d') => editor.pending_d = true,
        KeyCode::Char('D') => editor.delete_to_end_of_line(),
        KeyCode::Char('x') => editor.delete_char_at_cursor(),

        _ => {}
    }
}

fn handle_command_key(editor: &mut Editor, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => editor.exit_command_mode(),
        KeyCode::Enter => {
            // Ignore errors for now — could display in status bar later
            let _ = editor.execute_command();
        }
        KeyCode::Backspace => editor.command_pop(),
        KeyCode::Char(c) => editor.command_push(c),
        _ => {}
    }
}

fn handle_insert_key(editor: &mut Editor, key: KeyEvent, _viewport_height: usize) {
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
