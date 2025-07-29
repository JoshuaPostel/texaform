use serde::{Deserialize, Serialize};

mod count_groups;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Puzzle {
    CountGroups,
    Trivial,
}

impl Puzzle {
    pub fn generate_prompt_solution_pair(&self) -> (String, String) {
        match self {
            Puzzle::CountGroups => {
                let mut rp = count_groups::CountGroups::new();
                (rp.prompt(), rp.solution())
            }
            Puzzle::Trivial => ("FOO".to_string(), "BAR".to_string()),
        }
    }
    pub fn description(&self) -> &str {
        match self {
            Puzzle::CountGroups => count_groups::DESCRIPTION,
            Puzzle::Trivial => "placeholder while rethinking lab gameplay. answer is always 'BAR'",
        }
    }
}
