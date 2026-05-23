use super::{Deserialize, Point, PowerUpType, Serialize, serde_as};
#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct PowerUp {
    pub p_type: PowerUpType,
    pub location: Point,
    pub activation_time: Option<u64>,
}
