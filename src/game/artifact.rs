use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Artifact {
    CoinAmulet,
    LifeChalice,
    GhostCloak,
    MagnetStone,
    TimeCrystal,
}
