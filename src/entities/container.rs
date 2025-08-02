use crate::entities::{PickResult, Properties};

use std::collections::HashMap;

use ratatui::layout::{Margin, Rect};

use serde::{Deserialize, Serialize};

use ratatui::buffer::Buffer;
use ratatui::widgets::{Block, BorderType, Paragraph, WidgetRef, Wrap};

// TODO should remove_content have this check?
//fn remove_cost(buffer: &mut Buffer, cost: &Cost) {
//    if contains_cost(buffer, cost) {
//        for (prop, count) in cost.iter() {
//            for _ in 0..*count {
//                let idx = buffer.iter().position(|p| p == prop).expect("checked");
//                buffer.remove(idx);
//            }
//        }
//    }
//}

impl WidgetRef for EntityContainer {
    // TODO move logic to struct state so that it is not calculated every frame
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let inner = area.inner(Margin::new(1, 1));
        let inner_area = inner.area() as usize;
        if inner_area < self.capacity {
            let mut content: String = self
                .content_chars
                .iter()
                .take(inner_area.saturating_sub(2))
                .collect();
            //            // TODO bug when content.len() close to inner_area.area()
            //            let mut content: String = self.content.iter().map(|e| e.character()).collect();
            //            while content.len() <= inner.area() as usize - 3 {
            //                content.push(' ');
            //            }
            content.push('.');
            content.push('.');
            content.push('.');
            let p = Paragraph::new(content)
                .block(
                    Block::bordered()
                        .border_type(BorderType::QuadrantInside)
                        .title(self.title.clone()),
                )
                .wrap(Wrap { trim: false });
            p.render_ref(area, buf);
        } else {
            // TODO clean up the logic of this branch
            let mut height = self.capacity / inner.width as usize;
            if self.capacity % inner.width as usize != 0 {
                height += 1;
            }

            let width = if height == 1 {
                (self.capacity as u16 + 2).max(self.title.len() as u16 + 2)
            } else {
                area.width
            };

            let container_area = Rect {
                x: area.x,
                y: area.y,
                width,
                height: height as u16 + 2,
            };

            let fill_space = if height == 1 {
                (width - 2 - self.capacity as u16).into()
            } else {
                (height * inner.width as usize) - self.capacity
            };
            //            let mut chars: Vec<char> = self.content.iter().map(|e| e.character()).collect();
            //            let empty_space = self.capacity - chars.len();
            //            for _ in 0..empty_space {
            //                chars.push(' ');
            //            }
            let mut chars = self.content_chars.clone();
            chars.extend(vec!['█'; fill_space]);

            let block = Block::bordered()
                .border_type(BorderType::Thick)
                .title(self.title.clone());
            block.render_ref(container_area, buf);

            for (idx, pos) in inner.positions().enumerate() {
                if let Some(cell) = buf.cell_mut(pos)
                    && let Some(foo_char) = chars.get(idx)
                {
                    cell.set_char(*foo_char);
                }
            }
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct EntityContainer {
    pub content: Vec<Properties>,
    pub capacity: usize,
    pub title: String,
    pub content_chars: Vec<char>,
}

impl EntityContainer {
    pub fn new(title: &str, capacity: usize) -> EntityContainer {
        EntityContainer {
            content: vec![],
            capacity,
            title: title.to_string(),
            content_chars: vec![' '; capacity],
        }
    }

    pub fn pick(&mut self, c: char) -> PickResult {
        let prop = match self.content.iter().position(|gent| gent.character() == c) {
            Some(idx) => {
                self.content_chars.remove(idx);
                self.content_chars.push(' ');
                Some(self.content.remove(idx))
            }
            None => None,
        };
        PickResult {
            picked: prop,
            replace: None,
        }
    }

    pub fn placable(&self) -> bool {
        self.content.len() < self.capacity
    }

    pub fn place(&mut self, properties: Properties) {
        if self.placable() {
            self.content_chars[self.content.len()] = properties.character();
            self.content.push(properties);
        }
    }

    pub fn pop(&mut self) -> Option<Properties> {
        match self.content.pop() {
            Some(prop) => {
                self.content_chars[self.content.len()] = ' ';
                Some(prop)
            }
            None => None,
        }
    }

    pub fn remove_entity(&mut self, to_remove: &Properties) -> Result<(), ()> {
        let idx = self.content.iter().position(|p| p == to_remove).ok_or(())?;
        self.content_chars.remove(idx);
        self.content_chars.push(' ');
        self.content.remove(idx);
        Ok(())
    }

    pub fn remove_content(&mut self, to_remove: &HashMap<Properties, u8>) -> Result<(), ()> {
        for (prop, count) in to_remove.iter() {
            for _ in 0..*count {
                self.remove_entity(prop)?;
                //                let idx = self.content.iter().position(|p| p == prop).ok_or(())?;
                //                self.content_chars.remove(idx);
                //                self.content_chars.push(' ');
                //                self.content.remove(idx);
            }
        }
        Ok(())
    }
}
