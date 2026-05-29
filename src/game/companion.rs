use serde::{Deserialize, Serialize};
use crate::snake::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompanionType {
    Collector,
    Fighter,
    Healer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Companion {
    pub position: Point,
    pub kind: CompanionType,
    pub move_timer: u32,
    pub action_timer: u32,
    pub path: Vec<Point>,
}
