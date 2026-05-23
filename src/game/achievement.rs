use super::{Deserialize, Serialize};
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Achievement {
    FirstBlood,
    HighScorer,
    Rich,
    BotUser,
    BossSlayer,
    MassiveMultiplayerEnthusiast,
    PoisonEater,
}
