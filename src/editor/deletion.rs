use super::Editor;

impl Editor {
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

    pub fn delete_to_end_of_line(&mut self) {
        let line_len = self.buffer.line_len(self.cursor_row);
        if line_len == 0 {
            return;
        }
        let count = line_len - self.cursor_col;
        for _ in 0..count {
            self.buffer.delete_char_at(self.cursor_row, self.cursor_col);
        }
        self.clamp_cursor_col();
    }

    pub fn delete_word(&mut self) {
        let line = match self.buffer.line(self.cursor_row) {
            Some(l) => l,
            None => return,
        };
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        if self.cursor_col >= len {
            return;
        }

        let classify = |c: char| -> u8 {
            if c.is_alphanumeric() || c == '_' {
                0 // word
            } else if c.is_whitespace() {
                2 // whitespace
            } else {
                1 // punctuation
            }
        };

        let start = self.cursor_col;
        let start_class = classify(chars[start]);
        let mut pos = start;

        // Skip over the current class of characters
        while pos < len && classify(chars[pos]) == start_class {
            pos += 1;
        }

        // If the current class was not whitespace, also skip trailing whitespace
        if start_class != 2 {
            while pos < len && chars[pos].is_whitespace() {
                pos += 1;
            }
        }

        let count = pos - start;
        for _ in 0..count {
            self.buffer.delete_char_at(self.cursor_row, self.cursor_col);
        }
        self.clamp_cursor_col();
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_editor;

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

    #[test]
    fn delete_to_end_of_line_mid() {
        let mut ed = test_editor("hello world\n");
        ed.cursor_col = 5;
        ed.delete_to_end_of_line();
        assert_eq!(ed.buffer.line(0).unwrap(), "hello");
    }

    #[test]
    fn delete_to_end_of_line_start() {
        let mut ed = test_editor("hello\n");
        ed.cursor_col = 0;
        ed.delete_to_end_of_line();
        assert_eq!(ed.buffer.line(0).unwrap(), "");
    }

    #[test]
    fn delete_to_end_of_line_empty() {
        let mut ed = test_editor("abc\n\ndef\n");
        ed.cursor_row = 1;
        ed.delete_to_end_of_line();
        assert_eq!(ed.buffer.line(1).unwrap(), "");
    }

    #[test]
    fn delete_word_basic() {
        let mut ed = test_editor("hello world\n");
        ed.cursor_col = 0;
        ed.delete_word();
        assert_eq!(ed.buffer.line(0).unwrap(), "world");
    }

    #[test]
    fn delete_word_punctuation() {
        let mut ed = test_editor("foo.bar\n");
        ed.cursor_col = 0;
        ed.delete_word();
        assert_eq!(ed.buffer.line(0).unwrap(), ".bar");
    }

    #[test]
    fn delete_word_at_end() {
        let mut ed = test_editor("hello\n");
        ed.cursor_col = 3;
        ed.delete_word();
        assert_eq!(ed.buffer.line(0).unwrap(), "hel");
    }
}
