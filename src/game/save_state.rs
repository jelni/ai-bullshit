#![allow(clippy::struct_excessive_bools)]
use super::{
    Boss, Boulder, Companion, Deserialize, Difficulty, Direction, GameMode, Goblin, HashSet,
    InGameUpgrade, Laser, Meteor, Point, PowerUp, Serialize, Snake, Theme, Tournament, Weather,
    default_campaign_level, default_lives, default_skin, default_wrap_mode,
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
    pub boulders: Vec<Boulder>,
    #[serde(default)]
    pub avalanche_rolls: u32,
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
    #[serde(default, with = "super::map_as_pairs")]
    pub eggs_on_board: std::collections::HashMap<crate::snake::Point, crate::game::EggType>,
    #[serde(default)]
    pub paladin_life_timer: u32,
    #[serde(default)]
    pub engineer_turret_timer: u32,
    #[serde(default)]
    pub mana: u32,
    #[serde(default)]
    pub max_mana: u32,
    #[serde(default)]
    pub time_of_day: crate::game::TimeOfDay,
    #[serde(default)]
    pub tick_counter: u32,
    #[serde(default)]
    pub p1_flag: Option<Point>,
    #[serde(default)]
    pub p2_flag: Option<Point>,
    #[serde(default)]
    pub p1_has_flag: bool,
    #[serde(default)]
    pub p2_has_flag: bool,
    #[serde(default)]
    pub p1_score: u32,
    #[serde(default)]
    pub p2_score: u32,
    pub koth_zone: Option<Point>,
    #[serde(default)]
    pub xp_gems: HashSet<Point>,
    #[serde(default, with = "super::map_as_pairs")]
    pub dungeon_grid: std::collections::HashMap<(i32, i32), crate::game::dungeon::DungeonRoom>,
    #[serde(default)]
    pub current_room_coords: (i32, i32),
    #[serde(default, with = "super::map_as_pairs")]
    pub painted_tiles: std::collections::HashMap<Point, u8>,
    #[serde(default)]
    pub match_time: u32,
    #[serde(default)]
    pub snow_tiles: HashSet<Point>,
    #[serde(default)]
    pub sand_tiles: HashSet<Point>,
    #[serde(default)]
    pub sand_drifts: u32,
    #[serde(default)]
    pub storm_strikes: u32,
    #[serde(default)]
    pub quake_count: u32,
    #[serde(default)]
    pub fortress_walls: HashSet<Point>,
    #[serde(default)]
    pub fortress_depth: u16,
    #[serde(default)]
    pub fortress_rings: u32,
    #[serde(default)]
    pub thorn_tiles: HashSet<Point>,
    #[serde(default)]
    pub thorn_blooms: u32,
    #[serde(default, with = "super::map_as_pairs")]
    pub mirage_foods: std::collections::HashMap<Point, u32>,
    #[serde(default)]
    pub mirage_spawns: u32,
    #[serde(default)]
    pub lava_tiles: HashSet<Point>,
    #[serde(default)]
    pub volcano_vents: Vec<Point>,
    #[serde(default)]
    pub volcano_eruptions: u32,
    #[serde(default)]
    pub meteor_showers: u32,
    #[serde(default)]
    pub asteroid_impacts: u32,
    pub tournament: Option<Tournament>,
    pub tornado: Option<Point>,
    #[serde(default)]
    pub tornado_gusts: u32,
    #[serde(default)]
    pub rubble_tiles: HashSet<Point>,
    #[serde(default)]
    pub tremor_count: u32,
    pub meteor_strikes: u32,
    pub seismic_rubble: HashSet<Point>,
    #[serde(default)]
    pub seismic_quakes: u32,
    #[serde(default)]
    pub eclipse_flares: u32,
    #[serde(default)]
    pub eclipse_flare: u32,    #[serde(default)]
    pub tornado_spins: u32,
    pub sand_blasts: u32,
    #[serde(default)]
    pub bomb_carrier: Option<u8>,
    #[serde(default)]
    pub bomb_timer: u32,
    #[serde(default)]
    pub bomb_tags: u32,
    #[serde(default)]

    pub acid_puddles: HashSet<Point>,
    #[serde(default)]
    pub acid_drops: u32,
    #[serde(default)]
    pub puddle_tiles: HashSet<Point>,
    #[serde(default)]
    pub monsoon_downpours: u32,
    #[serde(default)]
    pub singularity_pulses: u32,
    #[serde(default)]
    pub laser_volleys: u32,
    #[serde(default)]
    pub laser_telegraph: Option<(bool, u16)>,
    #[serde(default)]
    pub singularity_drifts: u32,
    #[serde(default)]
    pub singularity_shield: u32,
    #[serde(default)]
    pub fire_tiles: HashSet<Point>,
    #[serde(default, with = "super::map_as_pairs")]
    pub fire_ttl: std::collections::HashMap<Point, u32>,
    #[serde(default)]
    pub ash_tiles: HashSet<Point>,
    #[serde(default)]
    pub wildfire_ignitions: u32,
    pub event_horizon_drifts: u32,
    #[serde(default)]
    pub event_horizon_surges: u32,
    pub infected_ids: Vec<u8>,
    #[serde(default)]
    pub infection_spreads: u32,

    pub inferno_tiles: HashSet<Point>,
    pub inferno_hotspots: Vec<Point>,
    #[serde(default, with = "super::map_as_pairs")]
    pub inferno_burn: std::collections::HashMap<Point, u32>,
    pub inferno_flares: u32,
    pub inferno_burned: u32,

    #[serde(default)]
    pub mine_drops: u32,
    #[serde(default)]
    pub ricochet_bounces: u32,
    pub tsunami_edge: u8,
    #[serde(default)]
    pub tsunami_front: i32,
    #[serde(default)]
    pub tsunami_depth: i32,
    #[serde(default)]
    pub tsunami_warning: u32,
    #[serde(default)]
    pub tsunami_waves: u32,
    #[serde(default)]
    pub warp_targets: Option<(Point, Point)>,
    #[serde(default)]
    pub warp_relocate_countdown: u32,
    #[serde(default)]
    pub warp_relocations: u32,
    #[serde(default)]
    pub gravity_well_surges: u32,
    #[serde(default, with = "super::map_as_pairs")]
    pub belt_tiles: std::collections::HashMap<Point, Direction>,
    #[serde(default)]
    pub belt_shifts: u32,
    pub ice_tiles: HashSet<Point>,
    #[serde(default)]
    pub ice_slips: u32,
    pub singularity_wanders: u32,
    #[serde(default)]
    pub singularity_feeds: u32,
    #[serde(default)]
    pub turret_volleys: u32,
    #[serde(default)]
    pub turret_jammed: u32,
    #[serde(default, with = "super::map_as_pairs")]
    pub blaze_tiles: std::collections::HashMap<Point, u32>,
    #[serde(default)]
    pub blaze_sources: Vec<Point>,
    #[serde(default)]
    pub firestorm_blazes: u32,
    #[serde(default)]
    pub geyser_vents: Vec<Point>,
    #[serde(default, with = "super::map_as_pairs")]
    pub geyser_jets: std::collections::HashMap<Point, u32>,
    #[serde(default)]
    pub geyser_eruptions: u32,
}

#[must_use]
pub const fn default_player_level() -> u32 {
    1
}

#[must_use]
pub const fn default_xp_to_next_level() -> u32 {
    5
}
