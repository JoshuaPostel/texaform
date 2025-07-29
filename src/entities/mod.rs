mod properties;
pub use properties::Properties;

pub mod shape;

pub mod container;
pub use container::EntityContainer;

use lazy_static::lazy_static;
use strum::IntoEnumIterator;

use std::collections::HashMap;

use crate::surface::grid::Gent;

pub type Cost = HashMap<Properties, u8>;
pub type Buffer = Vec<Properties>;

pub fn contains_cost(buffer: &Buffer, cost: &Cost) -> bool {
    cost.iter()
        .all(|(prop, count)| *count <= buffer.iter().filter(|&p| p == prop).count() as u8)
}

// TODO rename contains intermediate
pub fn contains_entity(buffer: &Buffer, entity: &Properties) -> bool {
    buffer.iter().any(|p| p == entity)
}


pub struct PickResult {
    pub picked: Option<Properties>,
    pub replace: Option<Gent>,
}

impl PickResult {
    pub fn noop() -> PickResult {
        PickResult {
            picked: None,
            replace: None,
        }
    }
}

fn can_this_go_inside_lazy_static() -> String {
    let mut documentation = String::from(
        "ENTITIES

  all entities can be PICKed by DOG and DROPed on the ground or in agent buffers

  FABRICATOR, SMELTER, DOG, and LASER_CUTTER entities become agents when BULT by DOG

  SOLAR_PANNEL and ACCUMULATOR entities become powered structures when BULT by DOG

",
    );
    for entity in Properties::iter() {
        documentation.push_str(entity.as_ref());
        documentation.push('\n');
        documentation.push_str(&format!("  character: {}\n", entity.character()));
        if let Some(fp) = entity.footprint() {
            documentation.push_str(&format!("  footprint: {}x{}\n", fp.x, fp.y));
        }
        if let Some(cost) = entity.cost() {
            documentation.push_str("  cost:\n");
            for (entity, count) in cost.iter() {
                documentation.push_str(&format!("    {entity}: {count}\n"));
            }
        }
        if let Some((entity, shape)) = entity.material_and_shape() {
            documentation.push_str(&format!("  cut from: {entity}\n"));
            documentation.push_str(&format!("  shape: \n{shape}\n"));
        }
        documentation.push('\n');
    }
    documentation
}

lazy_static! {
    pub static ref DOCUMENTATION: String = can_this_go_inside_lazy_static();
}
