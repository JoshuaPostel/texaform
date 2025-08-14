use crate::widgets::HandleInput;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use serde::{Deserialize, Serialize};

use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Color,
    widgets::{Paragraph, Widget, WidgetRef},
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TextBox {
    content: String,
    character_index: usize,
    clear_on_enter: bool,
}

// could return this as Output
#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    Edit(String),
    Submit(String),
}

impl HandleInput for TextBox {
    // TODO try returing &'a str
    type Output = Action;
    fn handle_key_event(&mut self, event: KeyEvent) -> Option<Action> {
        match event.code {
            KeyCode::Char(c) => {
                self.enter_char(c.to_ascii_uppercase());
                return Some(Action::Edit(self.content.clone()));
            }
            KeyCode::Backspace => {
                if let Some(idx) = self.character_index.checked_sub(1) {
                    self.delete_char(idx);
                    self.move_cursor_left();
                    return Some(Action::Edit(self.content.clone()));
                }
            }
            KeyCode::Delete => {
                self.delete_char(self.character_index);
                return Some(Action::Edit(self.content.clone()));
            }
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            KeyCode::Home => self.character_index = 0,
            KeyCode::End => self.character_index = self.content.len(),
            KeyCode::Enter => {
                let output = if self.clear_on_enter {
                    self.take()
                } else {
                    self.content.clone()
                };
                if !output.is_empty() {
                    return Some(Action::Submit(output));
                }
            }
            _ => (),
        }
        None
    }

    fn handle_mouse_event(&mut self, event: MouseEvent, rel_pos: Position) -> Option<Action> {
        if event.kind == MouseEventKind::Down(MouseButton::Left) {
            if let Some(idx) = rel_pos.x.checked_sub(3) {
                self.character_index = (idx as usize).min(self.content.len());
            } else {
                self.character_index = 0;
            }
        }
        None
    }
}

impl WidgetRef for TextBox {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(format!("> {}", self.content)).render(area, buf);
        for pos in area.positions().take(area.width as usize) {
            if usize::from(pos.x - area.x) == self.character_index + 2
                && let Some(cell) = buf.cell_mut(pos)
            {
                cell.fg = Color::Black;
                cell.bg = Color::Green;
                break;
            }
        }
    }
}

impl TextBox {
    pub fn new() -> TextBox {
        TextBox::default()
    }

    pub fn clear_on_enter(self, clear_on_enter: bool) -> TextBox {
        TextBox {
            clear_on_enter,
            ..self
        }
    }

    pub fn set_content(&mut self, input: String) {
        self.character_index = input.len();
        self.content = input;
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.content.insert(index, new_char);
        self.move_cursor_right();
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.content.chars().count())
    }

    fn byte_index(&mut self) -> usize {
        self.content
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.content.len())
    }

    fn delete_char(&mut self, idx: usize) {
        let before_char_to_delete = self.content.chars().take(idx);
        let after_char_to_delete = self.content.chars().skip(idx + 1);
        self.content = before_char_to_delete.chain(after_char_to_delete).collect();
    }

    fn take(&mut self) -> String {
        let msg = std::mem::take(&mut self.content);
        self.character_index = 0;
        msg.trim().to_string()
    }
}
