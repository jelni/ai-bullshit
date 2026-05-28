use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BountyType {
    EatFood(u32), // target amount
    KillBosses(u32),
    SurviveTime(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bounty {
    pub b_type: BountyType,
    pub target: u32,
    pub progress: u32,
    pub reward_coins: u32,
}

impl Bounty {
    #[must_use]
    pub fn new(b_type: BountyType, reward_coins: u32) -> Self {
        let target = match b_type {
            BountyType::EatFood(t) | BountyType::KillBosses(t) => t,
            BountyType::SurviveTime(t) => u32::try_from(t).unwrap_or(u32::MAX),
        };
        Self {
            b_type,
            target,
            progress: 0,
            reward_coins,
        }
    }

    #[must_use]
    pub const fn is_completed(&self) -> bool {
        self.progress >= self.target
    }
}
