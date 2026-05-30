#!/bin/bash
sed -i 's/use super::{Achievement, Deserialize, Serialize, Theme, default_elo, default_unlocked_themes, Resource, CraftableItem, Bounty, CompanionType};/use super::{Achievement, Deserialize, Serialize, Theme, default_elo, default_unlocked_themes, Resource, CraftableItem, Bounty, CompanionType, HeroClass};/' src/game/statistics.rs
sed -i '/pub equipped_companion: Option<CompanionType>,/a\    #[serde(default)]\n    pub unlocked_classes: Vec<HeroClass>,\n    #[serde(default)]\n    pub equipped_class: Option<HeroClass>,' src/game/statistics.rs
