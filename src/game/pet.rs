use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PetType {
    Dragon,
    Fairy,
    Mimic,
    Turtle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pet {
    pub p_type: PetType,
    pub location: crate::snake::Point,
}
