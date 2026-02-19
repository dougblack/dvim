use super::Editor;
use crate::mode::Mode;

impl Editor {
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

        // Try to parse as a line number (e.g. `:123` jumps to line 123)
        if let Ok(n) = cmd.parse::<usize>() {
            let target = if n == 0 {
                0
            } else {
                (n - 1).min(self.max_row())
            };
            self.cursor_row = target;
            self.clamp_cursor_col();
            return Ok(());
        }

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
    use super::super::test_editor;
    use crate::buffer::Buffer;
    use crate::mode::Mode;

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

    #[test]
    fn execute_goto_line() {
        let mut ed = test_editor("one\ntwo\nthree\nfour\nfive\n");
        ed.enter_command_mode();
        ed.command_push('3');
        ed.execute_command().unwrap();
        assert_eq!(ed.cursor_row, 2);
    }

    #[test]
    fn execute_goto_line_one() {
        let mut ed = test_editor("one\ntwo\nthree\nfour\nfive\n");
        ed.cursor_row = 3;
        ed.enter_command_mode();
        ed.command_push('1');
        ed.execute_command().unwrap();
        assert_eq!(ed.cursor_row, 0);
    }

    #[test]
    fn execute_goto_line_beyond_end() {
        let mut ed = test_editor("one\ntwo\nthree\nfour\nfive\n");
        ed.enter_command_mode();
        ed.command_push('9');
        ed.command_push('9');
        ed.command_push('9');
        ed.execute_command().unwrap();
        assert_eq!(ed.cursor_row, ed.max_row());
    }

    #[test]
    fn execute_goto_line_zero() {
        let mut ed = test_editor("one\ntwo\nthree\nfour\nfive\n");
        ed.cursor_row = 3;
        ed.enter_command_mode();
        ed.command_push('0');
        ed.execute_command().unwrap();
        assert_eq!(ed.cursor_row, 0);
    }
}
