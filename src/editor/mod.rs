mod command;
mod deletion;
mod insert;
mod keymap;
mod movement;

pub use keymap::handle_key;

use crate::buffer::Buffer;
use crate::mode::Mode;

pub struct Editor {
    pub buffer: Buffer,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub mode: Mode,
    pub running: bool,
    /// Tracks whether the previous key was 'g' (for the gg command).
    pub pending_g: bool,
    /// Tracks whether the previous key was 'd' (for the dd command).
    pub pending_d: bool,
    /// The text being typed in command mode (after ':').
    pub command_buffer: String,
}

impl Editor {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            mode: Mode::Normal,
            running: true,
            pending_g: false,
            pending_d: false,
            command_buffer: String::new(),
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    /// The last valid cursor row (skips the trailing empty line ropey adds).
    pub(crate) fn max_row(&self) -> usize {
        let count = self.buffer.line_count();
        if count == 0 {
            0
        } else {
            count.saturating_sub(2)
        }
    }

    /// Clamp cursor_col so it doesn't extend past the current line length.
    /// In Normal mode the cursor sits on the last char; in Insert mode it can
    /// be one past the end (append position).
    pub(crate) fn clamp_cursor_col(&mut self) {
        let line_len = self.buffer.line_len(self.cursor_row);
        if self.mode == Mode::Insert {
            self.cursor_col = self.cursor_col.min(line_len);
        } else if line_len == 0 {
            self.cursor_col = 0;
        } else {
            self.cursor_col = self.cursor_col.min(line_len - 1);
        }
    }
}

#[cfg(test)]
pub(crate) fn test_editor(content: &str) -> Editor {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(content.as_bytes()).unwrap();
    let buf = Buffer::from_file(tmp.path().to_path_buf()).unwrap();
    Editor::new(buf)
}
