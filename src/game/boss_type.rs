use super::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
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
}
