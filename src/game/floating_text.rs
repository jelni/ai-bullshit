use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct FloatingText {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: crate::color::Color,
    pub lifetime: f32,
    pub max_lifetime: f32,
}
