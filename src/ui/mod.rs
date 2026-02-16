use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::editor::Editor;

/// Render the editor state to the terminal.
pub fn draw(frame: &mut Frame, editor: &Editor) {
    let area = frame.area();

    // Split into text area (all but last row) and status bar (last row).
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // text area
            Constraint::Length(1), // status bar
        ])
        .split(area);

    draw_text_area(frame, editor, chunks[0]);
    draw_status_bar(frame, editor, chunks[1]);
}

/// The width of the line number gutter, including the trailing space.
fn gutter_width(line_count: usize) -> u16 {
    let digits = if line_count == 0 {
        1
    } else {
        (line_count as f64).log10().floor() as u16 + 1
    };
    digits + 1 // one space of padding after the number
}

fn draw_text_area(frame: &mut Frame, editor: &Editor, area: Rect) {
    let viewport_height = area.height as usize;
    let gutter_w = gutter_width(editor.buffer.line_count());

    let mut lines: Vec<Line> = Vec::with_capacity(viewport_height);

    for i in 0..viewport_height {
        let file_line = editor.scroll_offset + i;
        if let Some(content) = editor.buffer.line(file_line) {
            let line_num = format!(
                "{:>width$} ",
                file_line + 1,
                width = (gutter_w - 1) as usize
            );
            let spans = vec![
                Span::styled(line_num, Style::default().fg(Color::DarkGray)),
                Span::raw(content),
            ];
            lines.push(Line::from(spans));
        } else {
            // Vim shows '~' for lines past end of file
            let padding = " ".repeat((gutter_w - 1) as usize);
            lines.push(Line::from(vec![
                Span::styled(format!("{padding} "), Style::default().fg(Color::DarkGray)),
                Span::styled("-", Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines).block(Block::default());
    frame.render_widget(paragraph, area);

    // Place the terminal cursor at the editor's cursor position.
    let cursor_x = area.x + gutter_w + editor.cursor_col as u16;
    let cursor_y = area.y + (editor.cursor_row - editor.scroll_offset) as u16;
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn draw_status_bar(frame: &mut Frame, editor: &Editor, area: Rect) {
    let filename = editor.buffer.filename().file_name().map_or_else(
        || "[no name]".to_string(),
        |f| f.to_string_lossy().to_string(),
    );

    let position = format!("{}:{}", editor.cursor_row + 1, editor.cursor_col + 1);

    let mode_str = format!(" {} ", editor.mode);
    let status = format!(" {filename}");
    // Right-align position info
    let spacing_len =
        (area.width as usize).saturating_sub(mode_str.len() + status.len() + position.len() + 1);
    let spacing = " ".repeat(spacing_len);

    let status_line = Line::from(vec![
        Span::styled(
            mode_str,
            Style::default()
                .bg(Color::Blue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{status}{spacing}{position} "),
            Style::default().bg(Color::DarkGray).fg(Color::White),
        ),
    ]);

    let paragraph = Paragraph::new(status_line);
    frame.render_widget(paragraph, area);
}
