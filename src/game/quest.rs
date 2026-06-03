use super::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestType {
    SlayBosses,
    CollectCoins,
    ReachScore,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum QuestStatus {
    Active,
    Completed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Quest {
    pub name: String,
    pub description: String,
    pub q_type: QuestType,
    pub target: u32,
    pub progress: u32,
    pub reward: u32,
    pub status: QuestStatus,
}
