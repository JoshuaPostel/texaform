use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, strum_macros::EnumIter)]
pub enum Tutorial {
    #[default]
    Start,
    ClickCommandLine,
    PickIron,
    DropIron,
    SelectResearch,
    SelectFab,
    SendRESR,
    ConnectTCP,
    GoodLuck,
    Complete,
}

impl Tutorial {
    pub fn progress(&self) -> usize {
        for (idx, item) in Tutorial::iter().enumerate() {
            if *self == item {
                return idx;
            }
        }
        unreachable!()
    }

    pub fn next(&mut self) {
        let n = self.progress() + 1;
        if let Some(state) = Tutorial::iter().nth(n) {
            *self = state;
        }
    }

    pub fn previous(&mut self) {
        if let Some(n) = self.progress().checked_sub(1)
            && let Some(state) = Tutorial::iter().nth(n)
        {
            *self = state;
        }
    }

    pub fn instructions(&self) -> String {
        match self {
            Tutorial::Start => {
                "Successful deployment of Von Neumann probe 147 on exoplanest TOI-1846 b...
HUD initalized...
Surface view can be moved with arrows key, Page Up/Down, Home/End".to_string()
            },
            Tutorial::ClickCommandLine => {
                //"to control an Agent manually, click on the Command Line in the bottom right corner of the Heads Up Display (HUD)".to_string()
                "to control an Agent manually, select an Agent and click on the Command Line in the bottom right or press [C]
try typing in 'MOVE' and press [ENTER]. to exit editing mode press [CTRL + C]".to_string()
            },
            Tutorial::PickIron => {
                "pick up the IRON (I) by moving a DOG (>) next to it and sending the command 'PICK I' or 'PICK IRON'".to_string()
            },
            Tutorial::DropIron => {
                "navigate DOG next to FABRICATOR (F) and deposit the IRON
the full list of commands for DOG can be found under Menu > Documentation > Dog".to_string()
            },
            Tutorial::SelectResearch => {
                "select the SMELTER research by double clicking on it in the tech tree
Menu > Technology Tree".to_string()
            },
            Tutorial::SelectFab => {
                "select the FABRICATOR by clicking on it in the Agents list or in the map".to_string()
            },
            Tutorial::SendRESR => {
                "send the command 'RESR' to FABRICATOR
this consumes one IRON and progresses the SMELTER research".to_string()
            },
            Tutorial::ConnectTCP => {
                "automate each agent by sending commands to its port (e.g. DOG at 3335) via TCP
see Menu > Documentation > Agents for details".to_string()
            }
            Tutorial::GoodLuck => {
                "research Self-Sufficiency
this is left as an exercise for the reader".to_string()
            }
            Tutorial::Complete => "".to_string(),
        }
    }
}
