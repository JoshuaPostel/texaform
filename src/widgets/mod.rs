pub mod button;
pub mod list;
pub mod optional_list;
pub mod text_box;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Position;

pub trait HandleInput {
    // in future default associated type might make this cleaner
    // https://rust-lang.github.io/rfcs/2532-associated-type-defaults.html
    type Output;
    fn handle_key_event(&mut self, _event: KeyEvent) -> Option<Self::Output> {
        None
    }
    fn handle_mouse_event(
        &mut self,
        _event: MouseEvent,
        _relative_position: Position,
    ) -> Option<Self::Output> {
        None
    }
}
