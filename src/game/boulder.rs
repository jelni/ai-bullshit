use super::{Deserialize, Direction, Point, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Boulder {
    pub position: Point,
    pub dir: Direction,
}
