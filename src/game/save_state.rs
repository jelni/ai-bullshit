use super::{
    Boss, Companion, Deserialize, Difficulty, GameMode, Goblin, HashSet, InGameUpgrade, Laser,
    Meteor, Point, PowerUp, Serialize, Snake, Theme, Weather, default_campaign_level,
    default_lives, default_skin, default_wrap_mode,
};
#[derive(Serialize, Deserialize)]
pub struct SaveState {
    #[serde(default)]
    pub mode: GameMode,
    pub snake: Snake,
    #[serde(default)]
    pub player2: Option<Snake>,
    #[serde(default)]
    pub bots: Vec<Snake>,
    #[serde(default)]
    pub bots_autopilot_paths: Vec<Vec<Point>>,
    pub food: Point,
    pub obstacles: HashSet<Point>,
    pub score: u32,
    #[serde(default)]
    pub bonus_food: Option<(Point, u64)>,
    #[serde(default)]
    pub poison_food: Option<(Point, u64)>,
    #[serde(default)]
    pub power_up: Option<PowerUp>,
    #[serde(default = "default_lives")]
    pub lives: u32,
    #[serde(default)]
    pub difficulty: Difficulty,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default = "default_wrap_mode")]
    pub wrap_mode: bool,
    #[serde(default = "default_skin")]
    pub skin: char,
    #[serde(default)]
    pub auto_pilot: bool,
    #[serde(default)]
    pub used_bot_this_session: bool,
    #[serde(default)]
    pub food_eaten_session: u32,
    #[serde(default = "default_campaign_level")]
    pub campaign_level: u32,
    #[serde(default)]
    pub safe_zone_margin: u16,
    #[serde(default)]
    pub combo: u32,
    #[serde(default)]
    pub last_food_time: Option<u64>,
    #[serde(default)]
    pub lasers: Vec<Laser>,
    #[serde(default)]
    pub boss: Option<Boss>,
    #[serde(default)]
    pub bosses: Vec<Boss>,
    #[serde(default)]
    pub portals: Option<(Point, Point)>,
    #[serde(default)]
    pub weather: Weather,
    #[serde(default)]
    pub lightning_column: Option<u16>,
    #[serde(default)]
    pub mines: HashSet<Point>,
    #[serde(default)]
    pub black_hole: Option<Point>,
    #[serde(default)]
    pub meteors: Vec<Meteor>,
    #[serde(default)]
    pub goblin: Option<Goblin>,
    #[serde(default)]
    pub xp: u32,
    #[serde(default = "default_player_level")]
    pub player_level: u32,
    #[serde(default = "default_xp_to_next_level")]
    pub xp_to_next_level: u32,
    #[serde(default)]
    pub in_game_upgrades: std::collections::HashMap<InGameUpgrade, u32>,
    #[serde(default)]
    pub level_up_options: Vec<InGameUpgrade>,
    #[serde(default)]
    pub level_up_selection: usize,
    #[serde(default)]
    pub companion: Option<Companion>,
    #[serde(default)]
    pub equipment_boxes: Vec<Point>,
    #[serde(default)]
    pub fishing_timer: u32,
    #[serde(default)]
    pub fishing_progress: u32,
    #[serde(default)]
    pub is_fishing: bool,
    #[serde(default)]
    pub eggs_on_board: std::collections::HashMap<crate::snake::Point, crate::game::EggType>,
    #[serde(default)]
    pub paladin_life_timer: u32,
    #[serde(default)]
    pub mana: u32,
    #[serde(default)]
    pub max_mana: u32,
}

#[must_use]
pub const fn default_player_level() -> u32 {
    1
}

#[must_use]
pub const fn default_xp_to_next_level() -> u32 {
    5
}
