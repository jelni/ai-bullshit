use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameConfig {
    pub width: u16,
    pub height: u16,
    pub wrap_mode: bool,
    pub skin: char,
    pub theme: String,
    pub difficulty: Difficulty,
}
