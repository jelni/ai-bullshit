use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Default, Hash)]
pub enum Planet {
    #[default]
    Earth,
    Moon,
    Mars,
    Jupiter,
}
