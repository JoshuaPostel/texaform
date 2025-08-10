use crate::widgets::HandleInput;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{layout::Position, widgets::Paragraph};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TextBox {
    pub input: String,
    pub character_index: usize,
}

impl HandleInput for TextBox {
    type Output = String;
    fn handle_key_event(&mut self, event: KeyEvent) -> Option<String> {
        match event.code {
            KeyCode::Char(c) => self.enter_char(c.to_ascii_uppercase()),
            KeyCode::Backspace => self.delete_char(),
            // TODO need to render cursor based on character_index if we want this functionality
            // KeyCode::Left => self.move_cursor_left(),
            // KeyCode::Right => self.move_cursor_right(),
            // KeyCode::Home
            // KeyCode::End
            KeyCode::Enter => return Some(self.take()),
            _ => (),
        }
        None
    }

    // TODO set character index based on position
    // fn handle_mouse_event(&mut self, _event: MouseEvent, _rel_pos: Position) -> Option<String>
}

impl TextBox {
    pub fn new(input: String) -> TextBox {
        let character_index = input.len();
        TextBox {
            input,
            character_index,
        }
    }

    pub fn get_paragraph(&self) -> Paragraph<'_> {
        Paragraph::new(self.input.as_str())
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    pub fn byte_index(&mut self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn take(&mut self) -> String {
        let msg = std::mem::take(&mut self.input);
        self.character_index = 0;
        msg.trim().to_string()
    }
}
