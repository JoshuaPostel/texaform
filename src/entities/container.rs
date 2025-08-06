use crate::AppResult;
use crate::entities::{Entity, PickResult};

use std::collections::HashMap;

use ratatui::layout::{Margin, Rect};

use serde::{Deserialize, Serialize};

use ratatui::buffer::Buffer;
use ratatui::widgets::{Block, BorderType, Paragraph, WidgetRef, Wrap};

// TODO should remove_content have this check?
//fn remove_cost(buffer: &mut Buffer, cost: &Cost) {
//    if contains_cost(buffer, cost) {
//        for (entity, count) in cost.iter() {
//            for _ in 0..*count {
//                let idx = buffer.iter().position(|p| p == entity).expect("checked");
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
    pub content: Vec<Entity>,
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
        let entity = match self.content.iter().position(|gent| gent.character() == c) {
            Some(idx) => {
                self.content_chars.remove(idx);
                self.content_chars.push(' ');
                Some(self.content.remove(idx))
            }
            None => None,
        };
        PickResult {
            picked: entity,
            replace: None,
        }
    }

    pub fn placable(&self) -> bool {
        self.content.len() < self.capacity
    }

    pub fn place(&mut self, entity: Entity) {
        if self.placable() {
            self.content_chars[self.content.len()] = entity.character();
            self.content.push(entity);
        }
    }

    pub fn pop(&mut self) -> Option<Entity> {
        match self.content.pop() {
            Some(entity) => {
                self.content_chars[self.content.len()] = ' ';
                Some(entity)
            }
            None => None,
        }
    }

    pub fn remove_entity(&mut self, to_remove: &Entity) -> AppResult<()> {
        let idx = self
            .content
            .iter()
            .position(|p| p == to_remove)
            .ok_or(format!("did not contain {to_remove}"))?;
        self.content_chars.remove(idx);
        self.content_chars.push(' ');
        self.content.remove(idx);
        Ok(())
    }

    pub fn remove_content(&mut self, to_remove: &HashMap<Entity, u8>) -> AppResult<()> {
        for (entity, count) in to_remove.iter() {
            for _ in 0..*count {
                self.remove_entity(entity)?;
            }
        }
        Ok(())
    }
}
