use super::{Deserialize, Point, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Laser {
    pub position: Point,
    pub direction: crate::snake::Direction,
    pub player: u8,
}
