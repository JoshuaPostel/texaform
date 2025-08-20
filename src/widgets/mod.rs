pub mod button;
pub mod list;
pub mod text_box;

use std::time::{Duration, Instant};

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

#[derive(Debug, Clone)]
pub struct DoubleClickTracker<T: PartialEq> {
    id: Option<T>,
    last_clicked: Instant,
}

impl<T: PartialEq> Default for DoubleClickTracker<T> {
    fn default() -> DoubleClickTracker<T> {
        DoubleClickTracker {
            id: None,
            last_clicked: Instant::now(),
        }
    }
}

impl<T: PartialEq> DoubleClickTracker<T> {
    /// record a click on element `id` and return weather it was a double click
    pub fn clicked(&mut self, id: T) -> bool {
        let was_double_click = if Some(&id) == self.id.as_ref() {
            self.last_clicked.elapsed() < Duration::from_millis(500)
        } else {
            self.id = Some(id);
            false
        };
        self.last_clicked = Instant::now();
        was_double_click
    }
}

