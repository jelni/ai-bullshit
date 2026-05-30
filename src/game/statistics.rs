use super::{
    Achievement, Bounty, CompanionType, CraftableItem, Deserialize, HeroClass, Resource, Equipment, Serialize,
    Stock, Property, Theme, default_elo, default_unlocked_themes,
};
#[derive(Serialize, Deserialize, Default)]
pub struct Statistics {
    pub games_played: u32,
    pub total_score: u32,
    pub total_food_eaten: u32,
    pub total_time_s: u64,
    #[serde(default)]
    pub coins: u32,
    #[serde(default)]
    pub unlocked_skins: Vec<char>,
    #[serde(default = "default_unlocked_themes")]
    pub unlocked_themes: Vec<Theme>,
    #[serde(default)]
    pub unlocked_achievements: Vec<Achievement>,
    #[serde(default)]
    pub upgrade_powerup_duration: u8,
    #[serde(default)]
    pub upgrade_extra_lives: u8,
    #[serde(default)]
    pub upgrade_laser_capacity: u8,
    #[serde(default)]
    pub upgrade_coin_multiplier: u8,
    #[serde(default = "default_elo")]
    pub player_elo: u32,
    #[serde(default = "default_elo")]
    pub bot_elo: u32,
    #[serde(default)]
    pub inventory: std::collections::HashMap<Resource, u32>,
    #[serde(default)]
    pub crafted_items: std::collections::HashMap<CraftableItem, u32>,
    #[serde(default)]
    pub active_bounty: Option<Bounty>,
    #[serde(default)]
    pub completed_bounties: u32,
    #[serde(default)]
    pub unlocked_companions: Vec<CompanionType>,
    #[serde(default)]
    pub equipped_companion: Option<CompanionType>,
    #[serde(default)]
    pub unlocked_classes: Vec<HeroClass>,
    #[serde(default)]
    pub equipped_class: Option<HeroClass>,
    #[serde(default)]
    pub unlocked_equipment: Vec<Equipment>,
    #[serde(default)]
    pub equipped_gear: Option<Equipment>,
    #[serde(default)]
    pub portfolio: std::collections::HashMap<Stock, u32>,
    #[serde(default)]
    pub stock_prices: std::collections::HashMap<Stock, u32>,
    #[serde(default)]
    pub properties: std::collections::HashMap<Property, u32>,
}
