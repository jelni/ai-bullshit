use super::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub enum TimeOfDay {
    #[default]
    Day,
    Night,
}
