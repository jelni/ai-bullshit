use crate::snake::Point;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Turret {
    pub position: Point,
    pub shoot_timer: u32,
}
