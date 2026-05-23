use super::{Deserialize, Serialize};
#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum PowerUpType {
    SlowDown,
    SpeedBoost,
    Invincibility,
    ExtraLife,
    PassThroughWalls,
    Shrink,
    ClearObstacles,
    ScoreMultiplier,
    Teleport,
    Magnet,
    TimeFreeze,
    Reverse,
    Decoy,
    Emp,
    Nuke,
}
