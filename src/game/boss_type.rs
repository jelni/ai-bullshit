use super::{Deserialize, Serialize};
use std::hash::Hash;
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default, Hash)]
pub enum BossType {
    #[default]
    Shooter,
    Charger,
    Spawner,
    Teleporter,
    Splitter,
    Trapper,
    Necromancer,
    ShadowClone,
    Mimic,
    Puffer,
    Juggernaut,
    Dragon,
    Mage,
    Gorgon,
    VampireLord,
}
