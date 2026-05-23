use super::{Deserialize, Serialize};
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub enum Weather {
    #[default]
    Clear,
    Rain,
    Snow,
    Storm,
    Tornado,
}
