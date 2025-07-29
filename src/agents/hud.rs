use std::str::FromStr;

use ratatui::layout::Rect;

use serde::{Deserialize, Serialize};

use ratatui::buffer::Buffer;
use ratatui::widgets::WidgetRef;

use crate::surface::Power;
use crate::tech_tree::{Tech, TechKind, TechStatus};

impl WidgetRef for Hud {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {}
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Hud {
    pub agent_idx: usize,
    pub research_idx: usize,
}

impl Hud {
    pub fn parse_command(msg: &str) -> Result<Command, String> {
        match msg {
            "LIST AGNT" => Ok(Command::LIST_AGNT),
            "LIST RESR" => Ok(Command::LIST_RESR),
            "STAT POWR" => Ok(Command::STAT_POWR),
            x if x.starts_with("RESR") => {
                let kind = x.split_whitespace().nth(1).unwrap_or_default();
                if let Ok(tech_kind) = TechKind::from_str(kind)
                {
                    Ok(Command::RESR(tech_kind))
                } else {
                    Err(format!("unknown research {kind}"))
                }
            }
            _ => Err(format!("unknown command: {msg}")),
        }
    }
}

#[allow(non_camel_case_types)]
pub enum Command {
    STAT_POWR,
    LIST_AGNT,
    LIST_RESR,
    RESR(TechKind),
}

pub const DOCUMENTATION: &str = "Heads Up Display (HUD)

  a unique agent (cannot be built) for interacting with TEXAFORM's Heads Up Display

COMMANDS

  STAT POWR  
    return the current state of the power grid

    Usage: 
      STAT POWR  ->  100 200 300000 400000 # production, consumption, stored energy, storage capacity in joules

  LIST topic={AGNT|RESR} n
    cycles through information on the topic

    Usage:

      LIST AGNT  ->  3333 HUD          # HUD agent on port 3333
      LIST AGNT  ->  3334 FABRICATOR   # FABRICATOR agent on port 3334
      LIST AGNT  ->  3335 DOG          # DOG agent on port 3335
      LIST AGNT  ->  3336 DOG          # DOG agent on port 3336
      LIST AGNT  ->  3333 HUD          # list has been exhausted and will restart from the top

      LIST RESR  ->  SMELTER RESEARCHED 2/2  # SMELTER research is complete
      LIST RESR  ->  SOLAR UNLOCKED 1/2    # SOLAR is able to be researched and is 50% complete 
      LIST RESR  ->  BATTERY SELECTED 0/2  # BATTERY is the selected research and is 0% complete
      LIST RESR  ->  DOG LOCKED 0/5        # DOG prerequisite research incomplete
      ...
      LIST RESR  ->  SMELTER 2/2       # list has been exhausted and will restart from the top

  RESR research={SMELTER|LASER_CUTTER|SOLAR|...}
    set the active research

    Usage:

      RESR SOLAR    ->  OKAY
      RESR SMELTER  ->  ERRR already researched
      RESR DOG      ->  ERRR prerequisites not met

";

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Reply {
    ERRR(String),
    LIST_AGNT {
        port: usize,
        kind: String,
    },
    LIST_RSER {
        kind: String,
        status: String,
        numerator: usize,
        denominator: usize,
    },
    STAT_POWR {
        production: usize,
        consumption: usize,
        stored: usize,
        capacity: usize,
    },
    RESR,
}

impl Reply {
    pub fn from_tech(tech: &Tech, status: TechStatus) -> Reply {
        let kind = tech.kind.to_string();
        let status = status.to_string();
        let numerator = tech.progress_numerator as usize;
        let denominator = tech.progress_denominator as usize;
        Reply::LIST_RSER {
            kind,
            status,
            numerator,
            denominator,
        }
    }
}

impl From<&Power> for Reply {
    fn from(power: &Power) -> Reply {
        Reply::STAT_POWR {
            production: power.generation,
            consumption: power.consumption,
            stored: power.stored,
            capacity: power.capacity,
        }
    }
}

impl std::fmt::Display for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Reply::ERRR(msg) => write!(f, "ERRR {msg}"),
            Reply::LIST_AGNT { port, kind } => write!(f, "{port} {kind}"),
            Reply::LIST_RSER {
                kind,
                status,
                numerator,
                denominator,
            } => {
                write!(f, "{kind} {status} {numerator}/{denominator}")
            }
            Reply::STAT_POWR {
                production,
                consumption,
                stored,
                capacity,
            } => {
                write!(f, "{production} {consumption} {stored} {capacity}")
            }
            Reply::RESR => write!(f, "OKAY"),
        }
    }
}
