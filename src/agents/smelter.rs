use crate::agents::{Agent, UpdateEnum};
use crate::entities::{EntityContainer, PickResult, Properties};
use crate::surface::grid::Grid;
use crate::surface::{GameState, Power};
use crate::theme::DEFAULT_STYLE;

use ratatui::buffer::Cell;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Style};

use serde::{Deserialize, Serialize};

use ratatui::buffer::Buffer;
use ratatui::widgets::{Block, Gauge, WidgetRef};

impl WidgetRef for Smelter {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let chunks =
            Layout::vertical([Constraint::Max(1), Constraint::Fill(1), Constraint::Max(3)])
                .split(area);
        let top_row = chunks[0];
        let _middle_row = chunks[1];
        let bottom_row = Layout::horizontal([
            Constraint::Length(13),
            Constraint::Fill(1),
            Constraint::Length(13),
        ])
        .split(chunks[2]);

        let bottom_left = bottom_row[0];
        let bottom_center = bottom_row[1];
        let bottom_right = bottom_row[2];

        let style = if self.temprature < self.min_smelt_temp {
            Style::new().fg(Color::Gray).bg(Color::Black)
        } else if self.temprature <= self.max_smelt_temp {
            Style::new().fg(Color::Yellow).bg(Color::Black)
        } else {
            Style::new().fg(Color::Red).bg(Color::Black)
        };

        let percent = 100.min(self.temprature as u16 / (self.max_temp as u16 / 100));

        Gauge::default()
            //.block(Block::bordered().title(format!("Temprature")))
            .gauge_style(style)
            .label(format!("Temprature: {}", self.temprature))
            .percent(percent)
            .render_ref(top_row, buf);

        self.buffer_in.render_ref(bottom_left, buf);
        let hearth_content = self
            .hearth
            .as_ref()
            .map(|p| p.to_string())
            .unwrap_or("Empty".to_string());

        Gauge::default()
            .block(Block::bordered().title("Smelting Progress".to_string()))
            .gauge_style(DEFAULT_STYLE)
            .label(format!("{hearth_content}: {}%", self.progress))
            .percent(self.progress as u16)
            .render_ref(bottom_center, buf);
        self.buffer_out.render_ref(bottom_right, buf);
    }
}

/// operating range (1200, 1400)
/// over 100 will cause damage (not implemented yet)
#[derive(Debug, Serialize, Deserialize)]
pub struct Smelter {
    //C
    pub temprature: usize,
    pub min_smelt_temp: usize,
    pub max_smelt_temp: usize,
    pub max_temp: usize,
    pub powered_on: bool,
    pub progress: u8,
    pub integrity: u8,
    pub hearth: Option<Properties>,
    pub buffer_in: EntityContainer,
    pub buffer_out: EntityContainer,
}

#[typetag::serde]
impl Agent for Smelter {
    fn new() -> Self {
        Self {
            temprature: 60,
            min_smelt_temp: 500,
            max_smelt_temp: 1500,
            max_temp: 2000,
            powered_on: false,
            progress: 0,
            hearth: None,
            integrity: 100,
            buffer_in: EntityContainer::new("BUFFER_IN", 10),
            buffer_out: EntityContainer::new("BUFFER_OUT", 10),
        }
    }

    fn handle_message(
        &mut self,
        _pos: &Position,
        _grid: &mut Grid,
        _game_state: &mut GameState,
        msg: String,
    ) -> UpdateEnum {
        match Self::parse_command(&msg) {
            Ok(command) => self.handle_command(command),
            Err(e) => UpdateEnum::reply(format!("ERRR: {e}")),
        }
    }

    fn tick(&mut self, power: &mut Power) {
        if self.powered_on {
            if power.consume(1_000) && self.temprature < self.max_temp {
                self.temprature += 10;
            } else {
                self.temprature = self.temprature.saturating_sub(1);
            }
        } else {
            self.temprature = self.temprature.saturating_sub(1);
        }
        if let Some(next_in) = self.buffer_in.content.first()
            && self.hearth.is_none()
            && let Some(p) = self.buffer_in.pick(next_in.character()).picked
        {
            self.hearth = Some(p);
        }
        if self.min_smelt_temp <= self.temprature
            && self.temprature <= self.max_smelt_temp
            && let Some(hearth) = self.hearth
        {
            if self.progress < 100 {
                self.progress += 1;
                self.temprature = self.temprature.saturating_sub(5);
            } else {
                if let Some(p) = hearth.smelts_into() {
                    self.buffer_out.place(p);
                }
                self.hearth = None;
                self.progress = 0;
            }
        }
    }

    fn properties(&self) -> Properties {
        Properties::Smelter
    }
    fn render_surface_cell(&self, offset: &Position, cell: &mut Cell) {
        // TODO performance: store this somewhere so we dont recreate for each cell
        let fg = if self.powered_on {
            Color::LightRed
        } else {
            Color::White
        };
        cell.fg = fg;
        cell.bg = Color::DarkGray;
        if offset == &Position::new(0, 0) {
            cell.set_char(self.properties().character());
        } else {
            cell.set_char(self.properties().character().to_ascii_lowercase());
        }
    }
    fn pick(&mut self, c: char) -> PickResult {
        let buffer_out_pr = self.buffer_out.pick(c);
        if buffer_out_pr.picked.is_some() {
            buffer_out_pr
        } else {
            self.buffer_in.pick(c)
        }
    }

    fn placable(&self, _prop: &Properties) -> bool {
        self.buffer_in.placable()
    }
    fn place(&mut self, prop: Properties) {
        self.buffer_in.place(prop);
    }
}

impl Smelter {
    fn handle_command(&mut self, command: Command) -> UpdateEnum {
        match command {
            Command::POWR => {
                self.powered_on = !self.powered_on;
                UpdateEnum::okay()
            }
            Command::STAT => UpdateEnum::reply(Reply::STAT {
                temprature: self.temprature,
                buffer_in: self.buffer_in.content_chars.clone(),
                buffer_out: self.buffer_out.content_chars.clone(),
            }),
        }
    }

    fn parse_command(msg: &str) -> Result<Command, String> {
        match msg {
            "POWR" => Ok(Command::POWR),
            "STAT" => Ok(Command::STAT),
            _ => Err(format!("unknow command: {msg}")),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
enum Command {
    POWR,
    STAT,
}

pub const DOCUMENTATION: &str = "SMELTER

  when kept between 500 and 1500 degrees the smelter will transform raw materials

   input    | output
  ----------+-------------
   IRON     | IRON_PLATE
   COPPER   | COPPER_PLATE
   SILICATE | WAFER
            
  all other entities will be destroyed by the smelter

  upon smelt completion, the finished product is automatically placed into BUFFER_OUT
  and the next item is automatically loaded from from BUFFER_IN


COMMANDS

  POWR
    toggle the smelter on/off, consuming 1Kj when on and increasing the temprature

  STAT
    returns the temprature, entities in BUFFER_IN, and entities in BUFFER_OUT

    Usage:
      STAT  ->  425 _ _     #SMELTER is 425 degrees celsius with no content
      STAT  ->  1000 XXX _  #SMELTER is 1000 degrees celsius with three ROCKs in its input
      STAT  ->  1000 _ PP   #SMELTER is 1000 degrees celsius with two PLATEs in its output
";

#[derive(Debug)]
pub enum Reply {
    ERRR(String),
    STAT {
        temprature: usize,
        buffer_in: Vec<char>,
        buffer_out: Vec<char>,
    },
}

impl std::fmt::Display for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Reply::ERRR(msg) => write!(f, "ERRR {msg}"),
            Reply::STAT {
                temprature,
                buffer_in,
                buffer_out,
            } => {
                tracing::info!("in: {buffer_in:?}");
                tracing::info!("out: {buffer_out:?}");
                write!(f, "STAT ")?;
                write!(f, "{temprature} ")?;
                if buffer_in.iter().all(|c| *c == ' ') {
                    tracing::info!("in empty!");
                    write!(f, "_")?;
                } else {
                    for c in buffer_in {
                        tracing::info!("in not empty: {c}");
                        write!(f, "{c}")?;
                    }
                }
                write!(f, " ")?;
                if buffer_out.iter().all(|c| *c == ' ') {
                    write!(f, "_")?;
                } else {
                    for c in buffer_out {
                        write!(f, "{c}")?;
                    }
                }
                Ok(())
            }
        }
    }
}
