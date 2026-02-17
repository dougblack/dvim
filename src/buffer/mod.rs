use std::io::BufWriter;
use std::path::PathBuf;

use ropey::Rope;

use crate::error::DvimError;

/// A text buffer backed by a rope data structure.
///
/// The rope stores the file contents as a balanced tree of text chunks,
/// giving us O(log n) indexing by line and efficient future insert/delete
/// operations — even on very large files.
pub struct Buffer {
    rope: Rope,
    filename: PathBuf,
}

impl Buffer {
    /// Load a file from disk into a rope-backed buffer.
    pub fn from_file(path: PathBuf) -> Result<Self, DvimError> {
        let rope =
            Rope::from_reader(std::fs::File::open(&path).map_err(|e| DvimError::FileRead {
                path: path.display().to_string(),
                source: e,
            })?)
            .map_err(|e| DvimError::FileRead {
                path: path.display().to_string(),
                source: e,
            })?;

        Ok(Self {
            rope,
            filename: path,
        })
    }

    /// Total number of lines in the buffer.
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// Returns the text of line `idx` (0-indexed), without the trailing newline.
    pub fn line(&self, idx: usize) -> Option<String> {
        if idx >= self.line_count() {
            return None;
        }
        let line = self.rope.line(idx);
        let text = line.to_string();
        // Strip trailing newline characters
        Some(
            text.trim_end_matches('\n')
                .trim_end_matches('\r')
                .to_string(),
        )
    }

    /// Length of line `idx` in characters (excluding trailing newline).
    pub fn line_len(&self, idx: usize) -> usize {
        self.line(idx).map_or(0, |l| l.len())
    }

    pub fn filename(&self) -> &std::path::Path {
        &self.filename
    }

    /// Write the buffer contents back to its file.
    pub fn write(&self) -> Result<(), DvimError> {
        let file = std::fs::File::create(&self.filename).map_err(|e| DvimError::FileWrite {
            path: self.filename.display().to_string(),
            source: e,
        })?;
        self.rope
            .write_to(BufWriter::new(file))
            .map_err(|e| DvimError::FileWrite {
                path: self.filename.display().to_string(),
                source: e,
            })?;
        Ok(())
    }

    // -- Mutation methods for insert mode --

    /// Insert a character at the given (line, col) position.
    pub fn insert_char(&mut self, line: usize, col: usize, ch: char) {
        let char_idx = self.rope.line_to_char(line) + col;
        self.rope.insert_char(char_idx, ch);
    }

    /// Insert a newline at the given (line, col) position, splitting the line.
    pub fn insert_newline(&mut self, line: usize, col: usize) {
        let char_idx = self.rope.line_to_char(line) + col;
        self.rope.insert_char(char_idx, '\n');
    }

    /// Delete the entire line at `line`, including its trailing newline.
    /// Does nothing if it would empty the buffer entirely.
    pub fn delete_line(&mut self, line: usize) {
        let count = self.line_count();
        if line >= count {
            return;
        }
        let start = self.rope.line_to_char(line);
        let end = if line + 1 < count {
            self.rope.line_to_char(line + 1)
        } else {
            self.rope.len_chars()
        };
        // Don't delete if it would remove all content
        if end - start >= self.rope.len_chars() {
            return;
        }
        self.rope.remove(start..end);
    }

    /// Delete the character at (line, col). Does nothing if the line is empty.
    pub fn delete_char_at(&mut self, line: usize, col: usize) {
        let line_len = self.line_len(line);
        if line_len == 0 || col >= line_len {
            return;
        }
        let char_idx = self.rope.line_to_char(line) + col;
        self.rope.remove(char_idx..char_idx + 1);
    }

    /// Delete the character before (line, col). Returns the new cursor (line, col).
    /// At col 0, joins with the previous line. Otherwise deletes the char before cursor.
    pub fn delete_char_back(&mut self, line: usize, col: usize) -> (usize, usize) {
        if col == 0 {
            if line == 0 {
                return (0, 0);
            }
            // Join with previous line: remove the newline at end of previous line
            let prev_line_len = self.line_len(line - 1);
            let char_idx = self.rope.line_to_char(line) - 1;
            self.rope.remove(char_idx..char_idx + 1);
            (line - 1, prev_line_len)
        } else {
            let char_idx = self.rope.line_to_char(line) + col;
            self.rope.remove(char_idx - 1..char_idx);
            (line, col - 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn buffer_from_str(content: &str) -> Buffer {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();
        Buffer::from_file(tmp.path().to_path_buf()).unwrap()
    }

    #[test]
    fn line_count_simple() {
        let buf = buffer_from_str("hello\nworld\n");
        // Ropey counts the trailing empty line, so "hello\nworld\n" has 3 lines
        // (the third being empty after the final newline).
        assert_eq!(buf.line_count(), 3);
    }

    #[test]
    fn line_returns_content_without_newline() {
        let buf = buffer_from_str("first\nsecond\nthird\n");
        assert_eq!(buf.line(0).unwrap(), "first");
        assert_eq!(buf.line(1).unwrap(), "second");
        assert_eq!(buf.line(2).unwrap(), "third");
    }

    #[test]
    fn line_out_of_bounds_returns_none() {
        let buf = buffer_from_str("one\n");
        assert!(buf.line(999).is_none());
    }

    #[test]
    fn line_len_matches_content() {
        let buf = buffer_from_str("abcde\nfg\n");
        assert_eq!(buf.line_len(0), 5);
        assert_eq!(buf.line_len(1), 2);
    }

    #[test]
    fn from_file_nonexistent_returns_error() {
        let result = Buffer::from_file(PathBuf::from("/tmp/dvim_no_such_file_ever"));
        assert!(result.is_err());
    }

    #[test]
    fn insert_char_mid_line() {
        let mut buf = buffer_from_str("hello\n");
        buf.insert_char(0, 2, 'X');
        assert_eq!(buf.line(0).unwrap(), "heXllo");
    }

    #[test]
    fn insert_char_at_start() {
        let mut buf = buffer_from_str("abc\n");
        buf.insert_char(0, 0, 'Z');
        assert_eq!(buf.line(0).unwrap(), "Zabc");
    }

    #[test]
    fn insert_newline_splits_line() {
        let mut buf = buffer_from_str("abcdef\n");
        buf.insert_newline(0, 3);
        assert_eq!(buf.line(0).unwrap(), "abc");
        assert_eq!(buf.line(1).unwrap(), "def");
    }

    #[test]
    fn delete_char_back_mid_line() {
        let mut buf = buffer_from_str("hello\n");
        let (line, col) = buf.delete_char_back(0, 3);
        assert_eq!((line, col), (0, 2));
        assert_eq!(buf.line(0).unwrap(), "helo");
    }

    #[test]
    fn delete_char_back_joins_lines() {
        let mut buf = buffer_from_str("abc\ndef\n");
        let (line, col) = buf.delete_char_back(1, 0);
        assert_eq!((line, col), (0, 3));
        assert_eq!(buf.line(0).unwrap(), "abcdef");
    }

    #[test]
    fn delete_char_back_at_start_of_file() {
        let mut buf = buffer_from_str("hello\n");
        let (line, col) = buf.delete_char_back(0, 0);
        assert_eq!((line, col), (0, 0));
        assert_eq!(buf.line(0).unwrap(), "hello");
    }

    #[test]
    fn delete_line_middle() {
        let mut buf = buffer_from_str("aaa\nbbb\nccc\n");
        buf.delete_line(1);
        assert_eq!(buf.line(0).unwrap(), "aaa");
        assert_eq!(buf.line(1).unwrap(), "ccc");
    }

    #[test]
    fn delete_line_first() {
        let mut buf = buffer_from_str("aaa\nbbb\n");
        buf.delete_line(0);
        assert_eq!(buf.line(0).unwrap(), "bbb");
    }

    #[test]
    fn delete_line_last_real_line() {
        let mut buf = buffer_from_str("aaa\nbbb\n");
        buf.delete_line(1);
        assert_eq!(buf.line(0).unwrap(), "aaa");
    }

    #[test]
    fn delete_line_single_line_does_nothing() {
        let mut buf = buffer_from_str("only\n");
        buf.delete_line(0);
        assert_eq!(buf.line(0).unwrap(), "only");
    }

    #[test]
    fn delete_char_at_mid() {
        let mut buf = buffer_from_str("hello\n");
        buf.delete_char_at(0, 2);
        assert_eq!(buf.line(0).unwrap(), "helo");
    }

    #[test]
    fn delete_char_at_start() {
        let mut buf = buffer_from_str("abc\n");
        buf.delete_char_at(0, 0);
        assert_eq!(buf.line(0).unwrap(), "bc");
    }

    #[test]
    fn delete_char_at_end() {
        let mut buf = buffer_from_str("abc\n");
        buf.delete_char_at(0, 2);
        assert_eq!(buf.line(0).unwrap(), "ab");
    }

    #[test]
    fn delete_char_at_empty_line_does_nothing() {
        let mut buf = buffer_from_str("abc\n\ndef\n");
        let before = buf.line_count();
        buf.delete_char_at(1, 0);
        assert_eq!(buf.line_count(), before);
    }

    #[test]
    fn write_round_trip() {
        let mut buf = buffer_from_str("hello\nworld\n");
        buf.insert_char(0, 5, '!');
        buf.write().unwrap();

        let buf2 = Buffer::from_file(buf.filename.clone()).unwrap();
        assert_eq!(buf2.line(0).unwrap(), "hello!");
        assert_eq!(buf2.line(1).unwrap(), "world");
    }
}
