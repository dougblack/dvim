use super::Editor;
use crate::mode::Mode;

impl Editor {
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
}

#[cfg(test)]
mod tests {
    use super::super::test_editor;
    use crate::mode::Mode;

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
}
