use super::Editor;
use crate::mode::Mode;

impl Editor {
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

    // ── Character classification helpers ──────────────────────────────

    /// Classify a character into one of three categories used for word motions.
    /// 0 = whitespace, 1 = word (alphanumeric / underscore), 2 = punctuation.
    #[allow(dead_code)]
    fn char_class(c: char) -> u8 {
        if c.is_whitespace() {
            0
        } else if c.is_alphanumeric() || c == '_' {
            1
        } else {
            2
        }
    }

    // ── Word motions ──────────────────────────────────────────────────

    /// Move cursor to the start of the next word (vim `w`).
    #[allow(dead_code)]
    pub fn move_word_forward(&mut self) {
        let max_row = self.max_row();
        let mut row = self.cursor_row;
        let mut col = self.cursor_col;

        let Some(line) = self.buffer.line(row) else {
            return;
        };
        let chars: Vec<char> = line.chars().collect();

        // If the line is empty or we're past the end, jump to the next line.
        if chars.is_empty() || col >= chars.len() {
            if row < max_row {
                self.cursor_row = row + 1;
                self.cursor_col = 0;
                // If the next line is non-empty, find first non-whitespace (or stay at 0).
                if let Some(next_line) = self.buffer.line(row + 1) {
                    let nchars: Vec<char> = next_line.chars().collect();
                    let mut nc = 0;
                    while nc < nchars.len() && nchars[nc].is_whitespace() {
                        nc += 1;
                    }
                    if nc < nchars.len() {
                        self.cursor_col = nc;
                    }
                }
            }
            return;
        }

        // Step 1: skip over the current word (contiguous chars of the same class).
        let start_class = Self::char_class(chars[col]);
        if start_class != 0 {
            // On a word or punctuation — skip the rest of this run.
            while col < chars.len() && Self::char_class(chars[col]) == start_class {
                col += 1;
            }
        }

        // Step 2: skip any whitespace after the word.
        while col < chars.len() && chars[col].is_whitespace() {
            col += 1;
        }

        // If we've reached end of line, move to the start of the next line's first word.
        if col >= chars.len() {
            row += 1;
            if row > max_row {
                // Stay at end of current line.
                self.cursor_col = if chars.is_empty() { 0 } else { chars.len() - 1 };
                return;
            }
            col = 0;
            if let Some(next_line) = self.buffer.line(row) {
                let nchars: Vec<char> = next_line.chars().collect();
                while col < nchars.len() && nchars[col].is_whitespace() {
                    col += 1;
                }
                if col >= nchars.len() {
                    col = 0;
                }
            }
        }

        self.cursor_row = row;
        self.cursor_col = col;
    }

    /// Move cursor to the start of the previous word (vim `b`).
    #[allow(dead_code)]
    pub fn move_word_backward(&mut self) {
        let mut row = self.cursor_row;
        let mut col = self.cursor_col;

        // If at the beginning of a line, move to the end of the previous line.
        if col == 0 {
            if row == 0 {
                return;
            }
            row -= 1;
            let line_len = self.buffer.line_len(row);
            col = if line_len > 0 { line_len - 1 } else { 0 };
        } else {
            col -= 1;
        }

        let Some(line) = self.buffer.line(row) else {
            return;
        };
        let chars: Vec<char> = line.chars().collect();

        if chars.is_empty() {
            self.cursor_row = row;
            self.cursor_col = 0;
            return;
        }

        // Skip whitespace backwards.
        while col > 0 && chars[col].is_whitespace() {
            col -= 1;
        }
        if chars[col].is_whitespace() {
            // Entire prefix is whitespace — go to previous line if possible.
            if row > 0 {
                row -= 1;
                let prev_len = self.buffer.line_len(row);
                col = if prev_len > 0 { prev_len - 1 } else { 0 };
                if let Some(prev_line) = self.buffer.line(row) {
                    let pchars: Vec<char> = prev_line.chars().collect();
                    while col > 0 && pchars[col].is_whitespace() {
                        col -= 1;
                    }
                    // Now back up to the start of this word.
                    let cls = Self::char_class(pchars[col]);
                    while col > 0 && Self::char_class(pchars[col - 1]) == cls {
                        col -= 1;
                    }
                }
            }
            self.cursor_row = row;
            self.cursor_col = col;
            return;
        }

        // Now we're on a word or punctuation char — back up to the start of this run.
        let cls = Self::char_class(chars[col]);
        while col > 0 && Self::char_class(chars[col - 1]) == cls {
            col -= 1;
        }

        self.cursor_row = row;
        self.cursor_col = col;
    }

    /// Move cursor to the end of the current/next word (vim `e`).
    #[allow(dead_code)]
    pub fn move_word_end(&mut self) {
        let max_row = self.max_row();
        let mut row = self.cursor_row;
        let mut col = self.cursor_col;

        let Some(line) = self.buffer.line(row) else {
            return;
        };
        let chars: Vec<char> = line.chars().collect();

        if chars.is_empty() {
            // Empty line — try the next line.
            if row < max_row {
                row += 1;
                col = 0;
            } else {
                return;
            }
        } else {
            // Move at least one character forward.
            col += 1;

            // Skip whitespace.
            while col < chars.len() && chars[col].is_whitespace() {
                col += 1;
            }

            if col < chars.len() {
                // Find the end of this word.
                let cls = Self::char_class(chars[col]);
                while col + 1 < chars.len() && Self::char_class(chars[col + 1]) == cls {
                    col += 1;
                }
                self.cursor_row = row;
                self.cursor_col = col;
                return;
            }

            // Fell off end of line — move to next line.
            if row < max_row {
                row += 1;
                col = 0;
            } else {
                // Stay at end of current line.
                self.cursor_col = chars.len() - 1;
                return;
            }
        }

        // We're now at the start of a new line.
        if let Some(next_line) = self.buffer.line(row) {
            let nchars: Vec<char> = next_line.chars().collect();

            // Skip leading whitespace.
            while col < nchars.len() && nchars[col].is_whitespace() {
                col += 1;
            }

            if col < nchars.len() {
                // Find the end of this word.
                let cls = Self::char_class(nchars[col]);
                while col + 1 < nchars.len() && Self::char_class(nchars[col + 1]) == cls {
                    col += 1;
                }
            } else {
                col = 0;
            }
        }

        self.cursor_row = row;
        self.cursor_col = col;
    }

    // ── Line position motions ─────────────────────────────────────────

    /// Move cursor to column 0 (vim `0`).
    #[allow(dead_code)]
    pub fn goto_line_start(&mut self) {
        self.cursor_col = 0;
    }

    /// Move cursor to last character of line (vim `$`).
    #[allow(dead_code)]
    pub fn goto_line_end(&mut self) {
        let line_len = self.buffer.line_len(self.cursor_row);
        if line_len == 0 {
            self.cursor_col = 0;
        } else {
            self.cursor_col = line_len - 1;
        }
    }

    /// Move cursor to first non-whitespace character on line (vim `^`).
    #[allow(dead_code)]
    pub fn goto_first_non_blank(&mut self) {
        if let Some(line) = self.buffer.line(self.cursor_row) {
            let chars: Vec<char> = line.chars().collect();
            let mut col = 0;
            while col < chars.len() && chars[col].is_whitespace() {
                col += 1;
            }
            // If the whole line is whitespace, go to column 0.
            if col >= chars.len() {
                col = 0;
            }
            self.cursor_col = col;
        } else {
            self.cursor_col = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_editor;

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

    // ── Word motion tests ─────────────────────────────────────────────

    #[test]
    fn move_word_forward_basic() {
        let mut ed = test_editor("hello world\n");
        ed.cursor_col = 0;
        ed.move_word_forward();
        assert_eq!(ed.cursor_col, 6);
        assert_eq!(ed.cursor_row, 0);
    }

    #[test]
    fn move_word_forward_punctuation() {
        let mut ed = test_editor("foo.bar\n");
        ed.cursor_col = 0;
        ed.move_word_forward();
        // "foo" is a word, "." is a separate punctuation word → lands on "."
        assert_eq!(ed.cursor_col, 3);
        assert_eq!(ed.cursor_row, 0);
    }

    #[test]
    fn move_word_forward_end_of_line() {
        let mut ed = test_editor("hello\nworld\n");
        ed.cursor_col = 0;
        // First w: we're on "hello", skip it — no more words on this line → go to next line.
        ed.move_word_forward();
        assert_eq!(ed.cursor_row, 1);
        assert_eq!(ed.cursor_col, 0);
    }

    #[test]
    fn move_word_backward_basic() {
        let mut ed = test_editor("hello world\n");
        ed.cursor_col = 6;
        ed.move_word_backward();
        assert_eq!(ed.cursor_col, 0);
        assert_eq!(ed.cursor_row, 0);
    }

    #[test]
    fn move_word_end_basic() {
        let mut ed = test_editor("hello world\n");
        ed.cursor_col = 0;
        ed.move_word_end();
        assert_eq!(ed.cursor_col, 4);
        assert_eq!(ed.cursor_row, 0);
    }

    // ── Line position motion tests ────────────────────────────────────

    #[test]
    fn test_goto_line_start() {
        let mut ed = test_editor("hello\n");
        ed.cursor_col = 5;
        ed.goto_line_start();
        assert_eq!(ed.cursor_col, 0);
    }

    #[test]
    fn test_goto_line_end() {
        let mut ed = test_editor("hello\n");
        ed.cursor_col = 0;
        ed.goto_line_end();
        assert_eq!(ed.cursor_col, 4);
    }

    #[test]
    fn test_goto_first_non_blank() {
        let mut ed = test_editor("  hello\n");
        ed.cursor_col = 0;
        ed.goto_first_non_blank();
        assert_eq!(ed.cursor_col, 2);
    }
}
