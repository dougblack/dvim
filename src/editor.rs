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

    // -- Cursor movement --

    pub fn move_left(&mut self) {
        self.cursor_col = self.cursor_col.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        let max_row = self.max_row();
        if self.cursor_row < max_row {
            self.cursor_row += 1;
        }
        self.clamp_cursor_col();
    }

    pub fn move_up(&mut self) {
        self.cursor_row = self.cursor_row.saturating_sub(1);
        self.clamp_cursor_col();
    }

    pub fn move_right(&mut self) {
        let line_len = self.buffer.line_len(self.cursor_row);
        let max_col = if self.mode == Mode::Insert {
            line_len
        } else if line_len > 0 {
            line_len - 1
        } else {
            0
        };
        if self.cursor_col < max_col {
            self.cursor_col += 1;
        }
    }

    pub fn goto_top(&mut self) {
        self.cursor_row = 0;
        self.clamp_cursor_col();
    }

    pub fn goto_bottom(&mut self) {
        self.cursor_row = self.max_row();
        self.clamp_cursor_col();
    }

    pub fn goto_viewport_top(&mut self) {
        self.cursor_row = self.scroll_offset;
        self.clamp_cursor_col();
    }

    pub fn goto_viewport_middle(&mut self, viewport_height: usize) {
        let top = self.scroll_offset;
        let bottom = (self.scroll_offset + viewport_height - 1).min(self.max_row());
        self.cursor_row = (top + bottom) / 2;
        self.clamp_cursor_col();
    }

    pub fn goto_viewport_bottom(&mut self, viewport_height: usize) {
        let bottom = self.scroll_offset + viewport_height - 1;
        self.cursor_row = bottom.min(self.max_row());
        self.clamp_cursor_col();
    }

    pub fn scroll_half_page_down(&mut self, viewport_height: usize) {
        let half = viewport_height / 2;
        let max_row = self.max_row();
        self.cursor_row = (self.cursor_row + half).min(max_row);
        self.clamp_cursor_col();
    }

    pub fn scroll_half_page_up(&mut self, viewport_height: usize) {
        let half = viewport_height / 2;
        self.cursor_row = self.cursor_row.saturating_sub(half);
        self.clamp_cursor_col();
    }

    /// Ensure scroll_offset keeps the cursor visible within the viewport.
    pub fn adjust_scroll(&mut self, viewport_height: usize) {
        if self.cursor_row < self.scroll_offset {
            self.scroll_offset = self.cursor_row;
        }
        if self.cursor_row >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor_row - viewport_height + 1;
        }
    }

    // -- Helpers --

    /// The last valid cursor row (skips the trailing empty line ropey adds).
    fn max_row(&self) -> usize {
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
    fn clamp_cursor_col(&mut self) {
        let line_len = self.buffer.line_len(self.cursor_row);
        if self.mode == Mode::Insert {
            self.cursor_col = self.cursor_col.min(line_len);
        } else if line_len == 0 {
            self.cursor_col = 0;
        } else {
            self.cursor_col = self.cursor_col.min(line_len - 1);
        }
    }

    // -- Insert mode --

    pub fn enter_insert_mode(&mut self) {
        self.mode = Mode::Insert;
    }

    pub fn enter_insert_mode_append(&mut self) {
        let line_len = self.buffer.line_len(self.cursor_row);
        if line_len > 0 {
            self.cursor_col += 1;
        }
        self.mode = Mode::Insert;
    }

    pub fn enter_insert_mode_open_below(&mut self) {
        let line_len = self.buffer.line_len(self.cursor_row);
        self.buffer.insert_newline(self.cursor_row, line_len);
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.mode = Mode::Insert;
    }

    pub fn enter_insert_mode_open_above(&mut self) {
        self.buffer.insert_newline(self.cursor_row, 0);
        self.cursor_col = 0;
        self.mode = Mode::Insert;
    }

    pub fn exit_insert_mode(&mut self) {
        self.mode = Mode::Normal;
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
        self.clamp_cursor_col();
    }

    pub fn insert_char(&mut self, ch: char) {
        self.buffer
            .insert_char(self.cursor_row, self.cursor_col, ch);
        self.cursor_col += 1;
    }

    pub fn insert_newline(&mut self) {
        self.buffer.insert_newline(self.cursor_row, self.cursor_col);
        self.cursor_row += 1;
        self.cursor_col = 0;
    }

    pub fn delete_char_back(&mut self) {
        let (new_line, new_col) = self
            .buffer
            .delete_char_back(self.cursor_row, self.cursor_col);
        self.cursor_row = new_line;
        self.cursor_col = new_col;
    }

    // -- Normal mode deletion --

    pub fn delete_line(&mut self) {
        self.buffer.delete_line(self.cursor_row);
        let max = self.max_row();
        if self.cursor_row > max {
            self.cursor_row = max;
        }
        self.clamp_cursor_col();
    }

    pub fn delete_char_at_cursor(&mut self) {
        if self.buffer.line_len(self.cursor_row) == 0 {
            return;
        }
        self.buffer.delete_char_at(self.cursor_row, self.cursor_col);
        self.clamp_cursor_col();
    }

    // -- Command mode --

    pub fn enter_command_mode(&mut self) {
        self.mode = Mode::Command;
        self.command_buffer.clear();
    }

    pub fn exit_command_mode(&mut self) {
        self.mode = Mode::Normal;
        self.command_buffer.clear();
    }

    pub fn command_push(&mut self, ch: char) {
        self.command_buffer.push(ch);
    }

    pub fn command_pop(&mut self) {
        self.command_buffer.pop();
        if self.command_buffer.is_empty() {
            self.exit_command_mode();
        }
    }

    /// Parse and execute the current command buffer. Returns Err on write failures.
    pub fn execute_command(&mut self) -> anyhow::Result<()> {
        let cmd = self.command_buffer.trim().to_string();
        self.exit_command_mode();

        match cmd.as_str() {
            "w" => self.buffer.write()?,
            "q" => self.quit(),
            "wq" => {
                self.buffer.write()?;
                self.quit();
            }
            "w!" => {
                let _ = self.buffer.write();
            }
            "q!" => self.quit(),
            "wq!" => {
                let _ = self.buffer.write();
                self.quit();
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn test_editor(content: &str) -> Editor {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();
        let buf = Buffer::from_file(tmp.path().to_path_buf()).unwrap();
        Editor::new(buf)
    }

    #[test]
    fn cursor_starts_at_origin() {
        let ed = test_editor("hello\nworld\n");
        assert_eq!(ed.cursor_row, 0);
        assert_eq!(ed.cursor_col, 0);
    }

    #[test]
    fn move_down_stops_at_last_line() {
        let mut ed = test_editor("a\nb\nc\n");
        // max_row should be 2 (lines: "a", "b", "c", "" — skip trailing empty)
        for _ in 0..10 {
            ed.move_down();
        }
        assert_eq!(ed.cursor_row, 2);
    }

    #[test]
    fn move_up_stops_at_zero() {
        let mut ed = test_editor("a\nb\n");
        ed.move_up();
        assert_eq!(ed.cursor_row, 0);
    }

    #[test]
    fn cursor_col_clamped_on_shorter_line() {
        let mut ed = test_editor("long line\nhi\n");
        // Move to end of first line
        for _ in 0..20 {
            ed.move_right();
        }
        assert_eq!(ed.cursor_col, 8); // "long line" is 9 chars, max col = 8
        // Move down to "hi" — col should clamp to 1
        ed.move_down();
        assert_eq!(ed.cursor_col, 1);
    }

    #[test]
    fn goto_top_and_bottom() {
        let mut ed = test_editor("a\nb\nc\nd\ne\n");
        ed.goto_bottom();
        assert_eq!(ed.cursor_row, 4);
        ed.goto_top();
        assert_eq!(ed.cursor_row, 0);
    }

    // -- Viewport-relative movement tests --

    #[test]
    fn goto_viewport_top() {
        let mut ed = test_editor("a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n");
        ed.scroll_offset = 3;
        ed.cursor_row = 7;
        ed.goto_viewport_top();
        assert_eq!(ed.cursor_row, 3);
    }

    #[test]
    fn goto_viewport_middle() {
        let mut ed = test_editor("a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n");
        ed.scroll_offset = 0;
        ed.cursor_row = 0;
        // 10 real lines, viewport fits them all: middle of (0..9) = 4
        ed.goto_viewport_middle(10);
        assert_eq!(ed.cursor_row, 4);
    }

    #[test]
    fn goto_viewport_middle_short_file() {
        // 3 real lines in a large viewport: middle of (0..2) = 1
        let mut ed = test_editor("a\nb\nc\n");
        ed.scroll_offset = 0;
        ed.goto_viewport_middle(20);
        assert_eq!(ed.cursor_row, 1);
    }

    #[test]
    fn goto_viewport_bottom() {
        let mut ed = test_editor("a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n");
        ed.scroll_offset = 0;
        ed.cursor_row = 0;
        ed.goto_viewport_bottom(10);
        assert_eq!(ed.cursor_row, 9);
    }

    #[test]
    fn goto_viewport_bottom_clamps_to_max_row() {
        // 3 real lines ("a","b","c") + trailing empty = 4 ropey lines, max_row = 2
        let mut ed = test_editor("a\nb\nc\n");
        ed.scroll_offset = 0;
        ed.goto_viewport_bottom(20);
        assert_eq!(ed.cursor_row, 2);
    }

    // -- Insert mode tests --

    #[test]
    fn enter_insert_mode_keeps_cursor() {
        let mut ed = test_editor("hello\n");
        ed.cursor_col = 2;
        ed.enter_insert_mode();
        assert_eq!(ed.mode, Mode::Insert);
        assert_eq!(ed.cursor_col, 2);
    }

    #[test]
    fn enter_insert_mode_append_moves_right() {
        let mut ed = test_editor("hello\n");
        ed.cursor_col = 2;
        ed.enter_insert_mode_append();
        assert_eq!(ed.mode, Mode::Insert);
        assert_eq!(ed.cursor_col, 3);
    }

    #[test]
    fn enter_insert_mode_open_below() {
        let mut ed = test_editor("abc\ndef\n");
        ed.enter_insert_mode_open_below();
        assert_eq!(ed.mode, Mode::Insert);
        assert_eq!(ed.cursor_row, 1);
        assert_eq!(ed.cursor_col, 0);
        assert_eq!(ed.buffer.line(0).unwrap(), "abc");
        assert_eq!(ed.buffer.line(1).unwrap(), "");
        assert_eq!(ed.buffer.line(2).unwrap(), "def");
    }

    #[test]
    fn enter_insert_mode_open_above() {
        let mut ed = test_editor("abc\ndef\n");
        ed.cursor_row = 1;
        ed.enter_insert_mode_open_above();
        assert_eq!(ed.mode, Mode::Insert);
        assert_eq!(ed.cursor_row, 1);
        assert_eq!(ed.cursor_col, 0);
        assert_eq!(ed.buffer.line(0).unwrap(), "abc");
        assert_eq!(ed.buffer.line(1).unwrap(), "");
        assert_eq!(ed.buffer.line(2).unwrap(), "def");
    }

    #[test]
    fn exit_insert_mode_moves_cursor_left() {
        let mut ed = test_editor("hello\n");
        ed.enter_insert_mode();
        ed.cursor_col = 3;
        ed.exit_insert_mode();
        assert_eq!(ed.mode, Mode::Normal);
        assert_eq!(ed.cursor_col, 2);
    }

    #[test]
    fn exit_insert_mode_at_col_zero_stays() {
        let mut ed = test_editor("hello\n");
        ed.enter_insert_mode();
        ed.cursor_col = 0;
        ed.exit_insert_mode();
        assert_eq!(ed.cursor_col, 0);
    }

    #[test]
    fn insert_char_advances_cursor() {
        let mut ed = test_editor("ab\n");
        ed.enter_insert_mode();
        ed.cursor_col = 1;
        ed.insert_char('X');
        assert_eq!(ed.cursor_col, 2);
        assert_eq!(ed.buffer.line(0).unwrap(), "aXb");
    }

    #[test]
    fn insert_newline_moves_to_next_line() {
        let mut ed = test_editor("abcdef\n");
        ed.enter_insert_mode();
        ed.cursor_col = 3;
        ed.insert_newline();
        assert_eq!(ed.cursor_row, 1);
        assert_eq!(ed.cursor_col, 0);
        assert_eq!(ed.buffer.line(0).unwrap(), "abc");
        assert_eq!(ed.buffer.line(1).unwrap(), "def");
    }

    #[test]
    fn delete_char_back_in_insert_mode() {
        let mut ed = test_editor("hello\n");
        ed.enter_insert_mode();
        ed.cursor_col = 3;
        ed.delete_char_back();
        assert_eq!(ed.cursor_col, 2);
        assert_eq!(ed.buffer.line(0).unwrap(), "helo");
    }

    // -- Command mode tests --

    #[test]
    fn enter_command_mode_sets_mode() {
        let mut ed = test_editor("hello\n");
        ed.enter_command_mode();
        assert_eq!(ed.mode, Mode::Command);
        assert!(ed.command_buffer.is_empty());
    }

    #[test]
    fn command_push_and_pop() {
        let mut ed = test_editor("hello\n");
        ed.enter_command_mode();
        ed.command_push('w');
        assert_eq!(ed.command_buffer, "w");
        ed.command_pop();
        // Popping last char exits command mode
        assert_eq!(ed.mode, Mode::Normal);
        assert!(ed.command_buffer.is_empty());
    }

    #[test]
    fn exit_command_mode_clears_buffer() {
        let mut ed = test_editor("hello\n");
        ed.enter_command_mode();
        ed.command_push('q');
        ed.exit_command_mode();
        assert_eq!(ed.mode, Mode::Normal);
        assert!(ed.command_buffer.is_empty());
    }

    #[test]
    fn execute_q_quits() {
        let mut ed = test_editor("hello\n");
        ed.enter_command_mode();
        ed.command_push('q');
        ed.execute_command().unwrap();
        assert!(!ed.running);
    }

    #[test]
    fn execute_w_writes_file() {
        let mut ed = test_editor("hello\n");
        ed.enter_insert_mode();
        ed.cursor_col = 5;
        ed.insert_char('!');
        ed.exit_insert_mode();

        ed.enter_command_mode();
        ed.command_push('w');
        ed.execute_command().unwrap();

        assert!(ed.running);
        assert_eq!(ed.mode, Mode::Normal);

        // Verify written to disk
        let buf2 = Buffer::from_file(ed.buffer.filename().to_path_buf()).unwrap();
        assert_eq!(buf2.line(0).unwrap(), "hello!");
    }

    #[test]
    fn execute_wq_writes_and_quits() {
        let mut ed = test_editor("hello\n");
        ed.enter_command_mode();
        ed.command_push('w');
        ed.command_push('q');
        ed.execute_command().unwrap();
        assert!(!ed.running);
    }

    // -- Deletion tests --

    #[test]
    fn delete_line_middle() {
        let mut ed = test_editor("aaa\nbbb\nccc\n");
        ed.cursor_row = 1;
        ed.delete_line();
        assert_eq!(ed.cursor_row, 1);
        assert_eq!(ed.buffer.line(0).unwrap(), "aaa");
        assert_eq!(ed.buffer.line(1).unwrap(), "ccc");
    }

    #[test]
    fn delete_line_last_moves_cursor_up() {
        let mut ed = test_editor("aaa\nbbb\n");
        ed.cursor_row = 1;
        ed.delete_line();
        assert_eq!(ed.cursor_row, 0);
        assert_eq!(ed.buffer.line(0).unwrap(), "aaa");
    }

    #[test]
    fn delete_line_single_line_does_nothing() {
        let mut ed = test_editor("only\n");
        ed.delete_line();
        assert_eq!(ed.buffer.line(0).unwrap(), "only");
    }

    #[test]
    fn delete_char_at_cursor_mid() {
        let mut ed = test_editor("hello\n");
        ed.cursor_col = 2;
        ed.delete_char_at_cursor();
        assert_eq!(ed.buffer.line(0).unwrap(), "helo");
        assert_eq!(ed.cursor_col, 2);
    }

    #[test]
    fn delete_char_at_cursor_last_char() {
        let mut ed = test_editor("abc\n");
        ed.cursor_col = 2;
        ed.delete_char_at_cursor();
        assert_eq!(ed.buffer.line(0).unwrap(), "ab");
        assert_eq!(ed.cursor_col, 1); // clamped
    }

    #[test]
    fn delete_char_at_cursor_empty_line_does_nothing() {
        let mut ed = test_editor("abc\n\ndef\n");
        ed.cursor_row = 1;
        ed.delete_char_at_cursor();
        assert_eq!(ed.buffer.line(1).unwrap(), "");
    }

    // -- Force command tests --

    #[test]
    fn execute_q_bang_quits() {
        let mut ed = test_editor("hello\n");
        ed.enter_command_mode();
        ed.command_push('q');
        ed.command_push('!');
        ed.execute_command().unwrap();
        assert!(!ed.running);
    }

    #[test]
    fn execute_w_bang_ignores_errors() {
        let mut ed = test_editor("hello\n");
        ed.enter_command_mode();
        ed.command_push('w');
        ed.command_push('!');
        // Should not return an error even if write succeeds
        ed.execute_command().unwrap();
        assert!(ed.running);
    }

    #[test]
    fn execute_wq_bang_quits() {
        let mut ed = test_editor("hello\n");
        ed.enter_command_mode();
        ed.command_push('w');
        ed.command_push('q');
        ed.command_push('!');
        ed.execute_command().unwrap();
        assert!(!ed.running);
    }

    #[test]
    fn execute_unknown_command_does_nothing() {
        let mut ed = test_editor("hello\n");
        ed.enter_command_mode();
        ed.command_push('x');
        ed.execute_command().unwrap();
        assert!(ed.running);
        assert_eq!(ed.mode, Mode::Normal);
    }
}
