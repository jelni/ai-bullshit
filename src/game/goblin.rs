use super::{Deserialize, Point, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Goblin {
    pub position: Point,
    pub move_timer: u8,
    pub food_eaten: u8,
}
