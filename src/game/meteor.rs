use super::{Deserialize, Point, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Meteor {
    pub position: Point,
    pub timer: u8,
    #[serde(default)]
    pub vx: i8,
}
