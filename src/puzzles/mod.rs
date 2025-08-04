use serde::{Deserialize, Serialize};
use rand::RngCore;

mod count_groups;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Puzzle {
    CountGroups,
    Trivial,
}

impl Puzzle {
    pub fn generate_prompt_solution_pair(&self, rng: &mut impl RngCore) -> (String, String) {
        match self {
            Puzzle::CountGroups => {
                let mut rp = count_groups::CountGroups::new(rng);
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
