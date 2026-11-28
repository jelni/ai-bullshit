use super::{Deserialize, Point, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comet {
    pub position: Point,
    pub dx: i8,
    pub dy: i8,
}
