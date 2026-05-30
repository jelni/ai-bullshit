use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum HeroClass {
    #[default]
    Warrior,
    Mage,
    Rogue,
    Paladin,
}
