use super::{Deserialize, Direction, Point, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LandslideRock {
    pub position: Point,
    pub direction: Direction,
}
