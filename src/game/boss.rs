use super::{BossType, Deserialize, Point, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Boss {
    pub position: Point,
    pub health: u32,
    pub max_health: u32,
    pub move_timer: u8,
    #[serde(default)]
    pub shoot_timer: u8,
    #[serde(default)]
    pub kind: BossType,
    #[serde(default)]
    pub state_timer: u8,
}
