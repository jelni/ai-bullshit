use crate::snake::Point;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Crop {
    pub position: Point,
    pub growth_stage: u8, // 0 = seed, 1 = sprout, 2 = mature
    pub timer: u32,
}
