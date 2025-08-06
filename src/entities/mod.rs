pub mod shape;

pub mod container;
pub use container::EntityContainer;

use lazy_static::lazy_static;

use crate::AppResult;
use crate::surface::grid::Gent;

use std::collections::HashMap;
use std::str::FromStr;

use ratatui::layout::Position;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros;

use crate::agents::Agent;
use crate::agents::dog::Dog;
use crate::agents::fabricator::Fabricator;
use crate::agents::laser_cutter::LaserCutter;
use crate::agents::smelter::Smelter;
use crate::entities::shape::Shape;
use crate::surface::Power;

// seems like an enum to keep track of all entities
#[derive(
    Debug,
    Hash,
    PartialEq,
    Eq,
    Copy,
    Clone,
    Serialize,
    Deserialize,
    strum_macros::EnumIter,
    strum_macros::EnumString,
    strum_macros::EnumMessage,
    strum_macros::Display,
    strum_macros::AsRefStr,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum Entity {
    // Entities
    #[strum(message = "the symbol of industry")]
    Gear,
    #[strum(message = "for ease of assembling")]
    BarWinding,
    #[strum(message = "something something...")]
    Nut,
    #[strum(message = "it gets everywhere")]
    Silicate,
    #[strum(message = "silicate wafer")]
    Wafer,
    #[strum(message = "raw sulfer")]
    Sulfer,
    #[strum(message = "iron ore")]
    Iron,
    #[strum(message = "iron ore")]
    Copper,
    #[strum(message = "iron plate")]
    IronPlate,
    #[strum(message = "copper plate")]
    CopperPlate,
    #[strum(message = "electric of course")]
    Motor,
    #[strum(message = "whats your voltage?")]
    Battery,
    // Powered Entities
    #[strum(message = "by the power of Ra!")]
    SolarPannel,
    #[strum(message = "just add batteries!")]
    Accumulator,
    // Agents
    #[strum(message = "where the real magic happens")]
    Fabricator,
    #[strum(message = "bark!")]
    Dog,
    #[strum(message = "hot hot hot")]
    Smelter,
    #[strum(message = "if only it were that easy")]
    LaserCutter,
    // do we really want this or just use internal dev?
    #[strum(message = "heads up display")]
    HUD,
    // internal use
    #[strum(message = "null and void")]
    Empty,
}

// seems like a catch all for const/static things
impl Entity {
    // PERF lazy_static this
    pub fn line(self) -> Line<'static> {
        let kind = self.to_string();
        let c = self.character();
        let mut styled_chars = vec![];

        let mut modified_one = false;
        for kind_char in kind.chars() {
            let style = if c == kind_char && !modified_one {
                modified_one = true;
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            let f = Span::styled(kind_char.to_ascii_uppercase().to_string(), style);
            styled_chars.push(f);
        }
        Line::from(styled_chars)
    }
    pub const fn character(&self) -> char {
        match self {
            // Intermediates (technically still entities?)
            Self::Gear => 'G',
            Self::BarWinding => 'B',
            Self::Nut => 'N',
            // Entities
            Self::Silicate => 'L',
            Self::Wafer => 'W',
            Self::Sulfer => 'U',
            Self::Iron => 'I',
            Self::IronPlate => 'R',
            Self::Copper => 'O',
            Self::CopperPlate => 'E',
            Self::Motor => 'M',
            Self::Battery => 'T',
            Self::SolarPannel => 'P',
            Self::Accumulator => 'A',
            // Agents
            Self::Fabricator => 'F',
            Self::Dog => 'D',
            Self::LaserCutter => 'C',
            Self::Smelter => 'S',
            //
            Self::HUD => 'H',
            Self::Empty => '.',
        }
    }

    /// accept the entities char or name
    pub fn from_user_input(text: &str) -> Option<Entity> {
        if text.len() == 1 {
            for entity in Entity::iter() {
                if entity.character().to_string() == text {
                    return Some(entity);
                }
            }
            None
        } else {
            Entity::from_str(text).ok()
        }
    }

    pub const fn fg(&self) -> Color {
        match self {
            Self::Silicate => Color::LightCyan,
            Self::Sulfer => Color::LightYellow,
            Self::Iron => Color::Gray,
            Self::Copper => Color::LightRed,
            _ => Color::White,
        }
    }
    pub const fn bg(&self) -> Color {
        match self {
            Self::SolarPannel => Color::Blue,
            Self::Accumulator => Color::Blue,
            _ => Color::Black,
        }
    }
    // also indicates that something is buildable to DOG
    pub const fn footprint(&self) -> Option<Position> {
        match self {
            Self::Fabricator => Some(Position { x: 3, y: 3 }),
            Self::SolarPannel => Some(Position { x: 4, y: 1 }),
            Self::Accumulator => Some(Position { x: 2, y: 1 }),
            Self::Smelter => Some(Position { x: 3, y: 3 }),
            Self::LaserCutter => Some(Position { x: 6, y: 2 }),
            Self::Dog => Some(Position { x: 1, y: 1 }),
            _ => None,
        }
    }
    pub const fn is_agent(&self) -> bool {
        matches!(
            self,
            Self::Fabricator | Self::Smelter | Self::LaserCutter | Self::Dog
        )
    }
    pub fn create_agent(&self) -> AppResult<Box<dyn Agent>> {
        match self {
            Self::Fabricator => Ok(Box::new(Fabricator::new())),
            Self::Smelter => Ok(Box::new(Smelter::new())),
            Self::LaserCutter => Ok(Box::new(LaserCutter::new())),
            Self::Dog => Ok(Box::new(Dog::new())),
            _ => Err(format!("{self} is not an agent").into()),
        }
    }
    pub fn cost(&self) -> Option<HashMap<Self, u8>> {
        match self {
            Self::Smelter => Some(HashMap::from([
                (Self::IronPlate, 4),
                (Self::CopperPlate, 1),
            ])),
            Self::LaserCutter => Some(HashMap::from([
                (Self::IronPlate, 4),
                (Self::Gear, 2),
                (Self::Motor, 1),
            ])),
            Self::Fabricator => Some(HashMap::from([
                (Self::IronPlate, 4),
                (Self::Gear, 2),
                (Self::Nut, 4),
                (Self::Motor, 2),
            ])),
            // TODO add stator entity to cost
            Self::Motor => Some(HashMap::from([
                (Self::IronPlate, 1),
                (Self::Gear, 1),
                (Self::BarWinding, 3),
            ])),
            Self::Battery => Some(HashMap::from([
                (Self::IronPlate, 1),
                (Self::CopperPlate, 1),
                (Self::Sulfer, 1),
            ])),
            Self::Accumulator => Some(HashMap::from([
                (Self::IronPlate, 2),
                (Self::CopperPlate, 1),
                (Self::Battery, 4),
            ])),
            Self::SolarPannel => Some(HashMap::from([
                (Self::IronPlate, 1),
                (Self::CopperPlate, 2),
                (Self::Wafer, 2),
            ])),
            Self::Dog => Some(HashMap::from([
                (Self::IronPlate, 6),
                (Self::Motor, 5),
                (Self::Battery, 1),
                (Self::SolarPannel, 1),
            ])),
            _ => None,
        }
    }
    pub fn material_and_shape(&self) -> Option<(Entity, shape::Shape)> {
        match self {
            Self::Gear => Some((Entity::IronPlate, Shape::from(shape::GEAR))),
            Self::Nut => Some((Entity::IronPlate, Shape::from(shape::NUT))),
            Self::BarWinding => Some((Entity::CopperPlate, Shape::from(shape::BAR_WINDING))),
            _ => None,
        }
    }

    pub fn on_attach_to_power_grid(&self, power: &mut Power) {
        match self {
            //Self::SolarPannel => power.generation += 400,
            Self::SolarPannel => power.solar_pannel_count += 1,
            Self::Accumulator => power.add_capacity(100_000),
            _ => (),
        }
    }

    pub const fn cuttable(&self) -> bool {
        matches!(self, Self::IronPlate | Self::CopperPlate | Self::Wafer)
    }

    pub fn smelts_into(&self) -> Option<Self> {
        match self {
            Self::Iron => Some(Self::IronPlate),
            Self::Copper => Some(Self::CopperPlate),
            Self::Silicate => Some(Self::Wafer),
            _ => None,
        }
    }
}

pub struct PickResult {
    pub picked: Option<Entity>,
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
    for entity in Entity::iter() {
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
