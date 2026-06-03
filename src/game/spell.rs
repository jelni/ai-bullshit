use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpellType {
    Heal,
    Blink,
    Fireball,
    Shield,
}

impl SpellType {
    #[must_use]
    pub const fn cost(self) -> u32 {
        match self {
            Self::Heal => 50,
            Self::Blink => 30,
            Self::Fireball => 40,
            Self::Shield => 60,
        }
    }
}
