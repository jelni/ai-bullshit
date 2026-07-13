#![allow(clippy::missing_panics_doc, clippy::match_like_matches_macro)]
#![allow(
    clippy::useless_let_if_seq,
    clippy::unnecessary_lazy_evaluations,
    clippy::match_wildcard_for_single_variants,
    clippy::collection_is_never_read,
    clippy::if_same_then_else,
    clippy::match_same_arms
)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::similar_names
)]
use super::{
    AStarState, Achievement, Boss, BossType, Companion, CompanionType, Difficulty, Direction,
    Duration, File, FloatingText, GameMode, GameState, Goblin, HashSet, HistoryState,
    InGameUpgrade, Instant, Laser, Meteor, Particle, Planet, Point, PowerUp, PowerUpType, Read,
    Resource, Rng, SaveState, SeedableRng, Snake, Statistics, Theme, Turret, Weather, Write, beep,
    default_unlocked_themes, fs, io,
};
#[expect(clippy::struct_excessive_bools, reason = "Game struct naturally has many bools")]
pub struct Game {
    pub width: u16,
    pub height: u16,
    pub wrap_mode: bool,
    pub snake: Snake,
    pub food: Point,
    pub bonus_food: Option<(Point, Instant)>,
    pub merchant: Option<Point>,
    pub poison_food: Option<(Point, Instant)>,
    pub power_up: Option<PowerUp>,
    pub decoy: Option<(Point, u64)>,
    pub obstacles: HashSet<Point>,
    pub score: u32,
    pub high_score: u32,
    pub high_scores: Vec<(String, u32)>,
    pub state: GameState,
    pub rng: rand::rngs::StdRng,
    pub just_died: bool,
    pub skin: char,
    pub theme: Theme,
    pub lives: u32,
    pub menu_selection: usize,
    pub settings_selection: usize,
    pub nft_selection: usize,
    pub skill_tree_selection: usize,
    pub stats: Statistics,
    pub start_time: Instant,
    pub death_message: String,
    pub difficulty: Difficulty,
    pub player_name: String,
    pub previous_state: Option<GameState>,
    pub auto_pilot: bool,
    pub used_bot_this_session: bool,
    pub autopilot_path: Vec<Point>,
    pub p2_autopilot_path: Vec<Point>,
    pub food_eaten_session: u32,
    pub mode: GameMode,
    pub player2: Option<Snake>,
    pub bots: Vec<Snake>,
    pub bots_autopilot_paths: Vec<Vec<Point>>,
    pub campaign_level: u32,
    pub safe_zone_margin: u16,
    pub last_shrink_time: Instant,
    pub last_obstacle_spawn_time: Instant,
    pub history: std::collections::VecDeque<HistoryState>,
    pub editor_cursor: Option<Point>,
    pub particles: Vec<Particle>,
    pub combo: u32,
    pub last_food_time: Option<Instant>,
    pub chat_log: std::collections::VecDeque<(String, crate::color::Color)>,
    pub last_chat_time: Option<Instant>,
    pub lasers: Vec<Laser>,
    pub bosses: Vec<Boss>,
    pub portals: Option<(Point, Point)>,
    pub weather: Weather,
    pub lightning_column: Option<u16>,
    pub mines: HashSet<Point>,
    pub black_hole: Option<Point>,
    pub meteors: Vec<Meteor>,
    pub goblin: Option<Goblin>,
    pub ghost_moves: std::collections::VecDeque<crate::snake::Direction>,
    pub current_replay: Vec<crate::snake::Direction>,
    pub ghost_snake: Option<Snake>,
    pub is_sprinting: bool,
    pub xp: u32,
    pub player_level: u32,
    pub xp_to_next_level: u32,
    pub in_game_upgrades: std::collections::HashMap<InGameUpgrade, u32>,
    pub level_up_options: Vec<InGameUpgrade>,
    pub level_up_selection: usize,
    pub turrets: Vec<Turret>,
    pub resources: std::collections::HashMap<Point, Resource>,
    pub companion: Option<Companion>,
    pub crops: Vec<crate::game::Crop>,
    pub equipment_boxes: Vec<Point>,
    pub last_real_estate_tick: Option<Instant>,
    pub last_bank_tick: Option<Instant>,
    pub fishing_timer: u32,
    pub fishing_progress: u32,
    pub is_fishing: bool,
    pub eggs_on_board: std::collections::HashMap<Point, crate::game::EggType>,
    pub paladin_life_timer: u32,
    pub current_planet: Planet,
    pub floating_texts: Vec<FloatingText>,
    pub mana: u32,
    pub max_mana: u32,
    pub time_of_day: crate::game::TimeOfDay,
    pub tick_counter: u32,
    pub p1_flag: Option<Point>,
    pub p2_flag: Option<Point>,
    pub p1_has_flag: bool,
    pub p2_has_flag: bool,
    pub p1_score: u32,
    pub p2_score: u32,
    pub koth_zone: Option<Point>,
    pub xp_gems: HashSet<Point>,
    pub flow_field: Option<std::collections::HashMap<Point, crate::snake::Direction>>,
    pub flow_field_targets: Vec<Point>,
    pub dungeon_grid: std::collections::HashMap<(i32, i32), crate::game::dungeon::DungeonRoom>,
    pub current_room_coords: (i32, i32),
}
impl Game {
    pub fn spawn_turret(&mut self) {
        self.turrets.push(Turret {
            position: self.snake.head(),
            shoot_timer: 0,
        });
    }

    pub fn cast_spell(&mut self, spell: super::SpellType) {
        match spell {
            super::SpellType::Heal => {
                self.lives += 1;
                crate::game::beep();
            },
            super::SpellType::Blink => {
                let current_head = self.snake.head();
                let dir = self.snake.direction;
                let mut next_pos = current_head;
                for _ in 0..3 {
                    let tentative = Self::calculate_next_head_dir(next_pos, dir);
                    let margin = if self.mode == GameMode::BattleRoyale {
                        self.safe_zone_margin
                    } else {
                        0
                    };
                    if tentative.x > margin
                        && tentative.x < self.width - 1 - margin
                        && tentative.y > margin
                        && tentative.y < self.height - 1 - margin
                        && !self.obstacles.contains(&tentative)
                    {
                        next_pos = tentative;
                    } else {
                        break;
                    }
                }
                if next_pos != current_head {
                    self.snake.move_to(next_pos, false);
                    crate::game::beep();
                }
            },
            super::SpellType::Fireball => {
                let current_head = self.snake.head();
                let dir = self.snake.direction;
                let laser_pos = Self::calculate_next_head_dir(current_head, dir);
                let margin = if self.mode == GameMode::BattleRoyale {
                    self.safe_zone_margin
                } else {
                    0
                };
                if laser_pos.x > margin
                    && laser_pos.x < self.width - 1 - margin
                    && laser_pos.y > margin
                    && laser_pos.y < self.height - 1 - margin
                {
                    self.lasers.push(Laser {
                        position: laser_pos,
                        direction: dir,
                        player: 1, // acts like player 1 laser, but could add explosion modifier if we wanted
                    });
                    crate::game::beep();
                }
            },
            super::SpellType::Shield => {
                self.power_up = Some(PowerUp {
                    p_type: PowerUpType::Invincibility,
                    location: Point {
                        x: 0,
                        y: 0,
                    },
                    activation_time: Some(
                        web_time::SystemTime::now()
                            .duration_since(web_time::SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    ),
                });
                crate::game::beep();
            },
        }
    }

    #[must_use]
    pub fn powerup_duration(&self) -> u64 {
        5 + u64::from(self.stats.upgrade_powerup_duration)
    }
    #[must_use]
    pub fn is_reverse_active(&self) -> bool {
        self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::Reverse
                && p.activation_time.is_some_and(|t| {
                    web_time::SystemTime::now()
                        .duration_since(web_time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .saturating_sub(t)
                        < self.powerup_duration()
                })
        })
    }
    #[doc = " # Panics"]
    #[doc = ""]
    #[expect(
        clippy::too_many_lines,
        reason = "Game struct naturally requires many lines for initialization"
    )]
    #[doc = " Panics if the board is completely full and there's no room for food."]
    #[must_use]
    pub fn new(
        width: u16,
        height: u16,
        wrap_mode: bool,
        skin: char,
        theme: Theme,
        difficulty: Difficulty,
    ) -> Self {
        let mut rng = rand::rngs::StdRng::from_entropy();
        let start_x = width / 2;
        let start_y = height / 2;
        let snake = Snake::new(Point {
            x: start_x,
            y: start_y,
        });
        let obs_count = match difficulty {
            Difficulty::Easy => 1,
            Difficulty::Normal => 3,
            Difficulty::Hard => 5,
            Difficulty::Insane => 10,
            Difficulty::GodMode => 20,
        };
        let avoid = |p: &Point| p.x == start_x && p.y == start_y - 1;
        let obstacles =
            Self::generate_obstacles(width, height, &snake, avoid, &mut rng, obs_count, 0);
        let avoid_food = |p: &Point| obstacles.contains(p);
        let food = Self::get_random_empty_point(width, height, &snake, avoid_food, &mut rng, 0)
            .expect("Board cannot be full on start");
        if std::path::Path::new("highscore.txt").exists()
            && !std::path::Path::new("highscore_normal.txt").exists()
        {
            let _ = std::fs::rename("highscore.txt", "highscore_normal.txt");
        }
        let mode = GameMode::SinglePlayer;
        let high_scores =
            Self::load_high_scores_from_file(&Self::get_high_score_filename(difficulty, mode));
        let high_score = high_scores.first().map_or(0, |(_, s)| *s);
        let stats = Self::load_stats();
        let initial_lives = if skin == '💎' {
            3 + u32::from(stats.upgrade_extra_lives) + 1
        } else {
            3 + u32::from(stats.upgrade_extra_lives)
        };
        let initial_power_up = if skin == '🚀' {
            Some(PowerUp {
                p_type: PowerUpType::SpeedBoost,
                location: Point {
                    x: 0,
                    y: 0,
                },
                activation_time: Some(
                    web_time::SystemTime::now()
                        .duration_since(web_time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                ),
            })
        } else {
            None
        };
        Self {
            width,
            height,
            wrap_mode,
            snake,
            food,
            bonus_food: None,
            merchant: None,
            poison_food: None,
            power_up: initial_power_up,
            decoy: None,
            obstacles,
            score: 0,
            high_score,
            high_scores,
            state: GameState::Menu,
            rng,
            just_died: false,
            skin,
            theme,
            lives: initial_lives,
            menu_selection: 0,
            settings_selection: 0,
            nft_selection: 0,
            skill_tree_selection: 0,
            stats,
            start_time: web_time::Instant::now(),
            death_message: String::new(),
            difficulty,
            player_name: String::new(),
            previous_state: None,
            auto_pilot: false,
            used_bot_this_session: false,
            autopilot_path: Vec::new(),
            p2_autopilot_path: Vec::new(),
            food_eaten_session: 0,
            mode,
            player2: None,
            bots: Vec::new(),
            bots_autopilot_paths: Vec::new(),
            campaign_level: 1,
            safe_zone_margin: 0,
            last_shrink_time: web_time::Instant::now(),
            last_obstacle_spawn_time: web_time::Instant::now(),
            history: std::collections::VecDeque::new(),
            editor_cursor: None,
            particles: Vec::new(),
            combo: 0,
            last_food_time: None,
            chat_log: std::collections::VecDeque::new(),
            last_chat_time: None,
            lasers: Vec::new(),
            bosses: Vec::new(),
            portals: None,
            weather: Weather::Clear,
            lightning_column: None,
            mines: HashSet::new(),
            black_hole: None,
            meteors: Vec::new(),
            goblin: None,
            ghost_moves: std::collections::VecDeque::new(),
            current_replay: Vec::new(),
            ghost_snake: None,
            is_sprinting: false,
            xp: 0,
            player_level: 1,
            xp_to_next_level: 5,
            in_game_upgrades: std::collections::HashMap::new(),
            level_up_options: Vec::new(),
            level_up_selection: 0,
            turrets: Vec::new(),
            resources: std::collections::HashMap::new(),
            companion: None,
            crops: Vec::new(),
            equipment_boxes: Vec::new(),
            last_real_estate_tick: Some(Instant::now()),
            last_bank_tick: None,
            fishing_timer: 0,
            fishing_progress: 0,
            is_fishing: false,
            eggs_on_board: std::collections::HashMap::new(),
            paladin_life_timer: 0,
            current_planet: Planet::Earth,
            floating_texts: Vec::new(),
            mana: 100,
            max_mana: 100,
            time_of_day: crate::game::TimeOfDay::Day,
            tick_counter: 0,
            p1_flag: None,
            p2_flag: None,
            p1_has_flag: false,
            p2_has_flag: false,
            p1_score: 0,
            p2_score: 0,
            koth_zone: None,
            xp_gems: HashSet::new(),
            flow_field: None,
            flow_field_targets: Vec::new(),
            dungeon_grid: std::collections::HashMap::new(),
            current_room_coords: (0, 0),
        }
    }
    #[must_use]
    pub fn get_high_score_filename(difficulty: Difficulty, mode: GameMode) -> String {
        if mode == GameMode::DailyChallenge {
            "highscore_daily.txt".to_string()
        } else if mode == GameMode::WeeklyChallenge {
            "highscore_weekly.txt".to_string()
        } else if mode == GameMode::MonthlyChallenge {
            "highscore_monthly.txt".to_string()
        } else if mode == GameMode::YearlyChallenge {
            "highscore_yearly.txt".to_string()
        } else if mode == GameMode::DecadeChallenge {
            "highscore_decade.txt".to_string()
        } else if mode == GameMode::CenturyChallenge {
            "highscore_century.txt".to_string()
        } else if mode == GameMode::MillenniumChallenge {
            "highscore_millennium.txt".to_string()
        } else if mode == GameMode::EonChallenge {
            "highscore_eon.txt".to_string()
        } else {
            format!("highscore_{difficulty:?}.txt").to_lowercase()
        }
    }
    #[must_use]
    pub fn load_high_scores_from_file(path: &str) -> Vec<(String, u32)> {
        let mut content = String::new();
        File::open(path).and_then(|f| f.take(1024 * 1024).read_to_string(&mut content)).map_or_else(
            |_| Vec::new(),
            |_| {
                content
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2
                            && let Some(score_str) = parts.last()
                        {
                            let name = parts[..parts.len() - 1].join(" ");
                            if let Ok(score) = score_str.parse::<u32>() {
                                return Some((name, score));
                            }
                        }
                        None
                    })
                    .collect()
            },
        )
    }
    fn load_stats() -> Statistics {
        Self::load_stats_from_file("stats.json")
    }
    fn load_stats_from_file(path: &str) -> Statistics {
        let mut stats: Statistics = File::open(path)
            .ok()
            .and_then(|f| serde_json::from_reader(f.take(1024 * 1024)).ok())
            .unwrap_or_default();
        if stats.unlocked_skins.is_empty() {
            stats.unlocked_skins = vec!['█', 'O', '@', '#', '*'];
        }
        if stats.unlocked_themes.is_empty() {
            stats.unlocked_themes = default_unlocked_themes();
        }
        stats
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn atomic_write(path: &str, content: impl AsRef<[u8]>) -> io::Result<()> {
        let mut rng = rand::rngs::StdRng::from_entropy();
        let suffix: u32 = rng.r#gen();
        let tmp_path = format!("{path}.{suffix}.tmp");
        let mut options = fs::File::options();
        options.write(true).create_new(true);
        #[cfg(all(unix, feature = "cli"))]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
        }
        let mut file = options.open(&tmp_path)?;
        file.write_all(content.as_ref())?;
        file.sync_all()?;
        fs::rename(tmp_path, path)
    }
    pub fn save_stats(&self) {
        self.save_stats_to_file("stats.json");
    }
    pub fn update_elo(&mut self, player_won: bool, draw: bool) {
        let p_elo = f64::from(self.stats.player_elo);
        let b_elo = f64::from(self.stats.bot_elo);
        let expected_p = 1.0 / (1.0 + 10.0_f64.powf((b_elo - p_elo) / 400.0));
        let expected_b = 1.0 / (1.0 + 10.0_f64.powf((p_elo - b_elo) / 400.0));
        let (score_p, score_b) = if draw {
            (0.5, 0.5)
        } else if player_won {
            (1.0, 0.0)
        } else {
            (0.0, 1.0)
        };
        let k = 32.0;
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let new_p_elo = (p_elo + k * (score_p - expected_p)).max(0.0).round() as u32;
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let new_bot_elo = (b_elo + k * (score_b - expected_b)).max(0.0).round() as u32;
        self.stats.player_elo = new_p_elo;
        self.stats.bot_elo = new_bot_elo;
    }
    #[cfg(target_arch = "wasm32")]
    pub fn save_stats_to_file(&self, _path: &str) {}
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_stats_to_file(&self, path: &str) {
        if let Ok(json) = serde_json::to_string(&self.stats) {
            let _ = Self::atomic_write(path, json);
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn save_high_score(&mut self, _name: String, _score: u32) {}
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_high_score(&mut self, name: String, score: u32) {
        let filename = Self::get_high_score_filename(self.difficulty, self.mode);
        self.save_high_score_to_file(&filename, name, score);
    }
    pub fn update_high_scores(&mut self) {
        self.high_scores = Self::load_high_scores_from_file(&Self::get_high_score_filename(
            self.difficulty,
            self.mode,
        ));
        self.high_score = self.high_scores.first().map_or(0, |(_, s)| *s);
    }
    #[cfg(target_arch = "wasm32")]
    pub fn save_high_score_to_file(&mut self, _path: &str, _name: String, _score: u32) {}
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_high_score_to_file(&mut self, path: &str, name: String, score: u32) {
        if let Some(pos) = self.high_scores.iter().position(|(n, _)| n == &name) {
            if self.high_scores[pos].1 < score {
                self.high_scores[pos].1 = score;
            }
        } else {
            self.high_scores.push((name, score));
        }
        self.high_scores.sort_unstable_by_key(|b| std::cmp::Reverse(b.1));
        self.high_scores.truncate(5);
        let content =
            self.high_scores.iter().map(|(n, s)| format!("{n} {s}")).collect::<Vec<_>>().join("\n");
        let _ = Self::atomic_write(path, content);
    }
    pub fn save_game(&self) {
        self.save_game_to_file("savegame.json");
    }
    #[cfg(target_arch = "wasm32")]
    pub fn save_custom_level(&self) {}
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_custom_level(&self) {
        if let Ok(json) = serde_json::to_string(&self.obstacles) {
            let _ = Self::atomic_write("custom_level.json", json);
        }
    }
    #[must_use]
    pub fn load_custom_level() -> HashSet<Point> {
        File::open("custom_level.json")
            .ok()
            .and_then(|f| serde_json::from_reader(f.take(1024 * 1024)).ok())
            .unwrap_or_default()
    }
    #[cfg(target_arch = "wasm32")]
    pub fn save_game_to_file(&self, _path: &str) {}
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_game_to_file(&self, path: &str) {
        let state = SaveState {
            mode: self.mode,
            snake: Snake {
                body: self.snake.body.clone(),
                body_map: self.snake.body_map.clone(),
                direction: self.snake.direction,
                direction_queue: self.snake.direction_queue.clone(),
            },
            player2: self.player2.clone(),
            bots: self.bots.clone(),
            bots_autopilot_paths: self.bots_autopilot_paths.clone(),
            food: self.food,
            obstacles: self.obstacles.clone(),
            score: self.score,
            bonus_food: self.bonus_food.map(|(p, t)| (p, t.elapsed().as_secs())),
            poison_food: self.poison_food.map(|(p, t)| (p, t.elapsed().as_secs())),
            power_up: self.power_up.clone(),
            lives: self.lives,
            difficulty: self.difficulty,
            theme: self.theme,
            wrap_mode: self.wrap_mode,
            skin: self.skin,
            auto_pilot: self.auto_pilot,
            used_bot_this_session: self.used_bot_this_session,
            food_eaten_session: self.food_eaten_session,
            campaign_level: self.campaign_level,
            safe_zone_margin: self.safe_zone_margin,
            combo: self.combo,
            last_food_time: self.last_food_time.map(|t| t.elapsed().as_secs()),
            lasers: self.lasers.clone(),
            boss: None,
            bosses: self.bosses.clone(),
            portals: self.portals,
            weather: self.weather,
            lightning_column: self.lightning_column,
            mines: self.mines.clone(),
            black_hole: self.black_hole,
            meteors: self.meteors.clone(),
            goblin: self.goblin,
            xp: self.xp,
            player_level: self.player_level,
            xp_to_next_level: self.xp_to_next_level,
            in_game_upgrades: self.in_game_upgrades.clone(),
            level_up_options: self.level_up_options.clone(),
            level_up_selection: self.level_up_selection,
            companion: self.companion.clone(),
            equipment_boxes: self.equipment_boxes.clone(),
            fishing_timer: self.fishing_timer,
            fishing_progress: self.fishing_progress,
            is_fishing: self.is_fishing,
            eggs_on_board: self.eggs_on_board.clone(),
            paladin_life_timer: self.paladin_life_timer,
            mana: self.mana,
            max_mana: self.max_mana,
            time_of_day: self.time_of_day,
            tick_counter: self.tick_counter,
            p1_flag: self.p1_flag,
            p2_flag: self.p2_flag,
            p1_has_flag: self.p1_has_flag,
            p2_has_flag: self.p2_has_flag,
            p1_score: self.p1_score,
            p2_score: self.p2_score,
            koth_zone: self.koth_zone,
            xp_gems: self.xp_gems.clone(),
            dungeon_grid: self.dungeon_grid.clone(),
            current_room_coords: self.current_room_coords,
        };
        if let Ok(json) = serde_json::to_string(&state) {
            let _ = Self::atomic_write(path, json);
        }
    }
    pub fn load_game(&mut self) -> bool {
        self.load_game_from_file("savegame.json")
    }
    #[expect(clippy::too_many_lines, reason = "Loading game requires assigning many fields")]
    pub(crate) fn load_game_from_file(&mut self, path: &str) -> bool {
        File::open(path)
            .ok()
            .and_then(|f| serde_json::from_reader::<_, SaveState>(f.take(1024 * 1024)).ok())
            .is_some_and(|mut state| {
                let valid_point =
                    |p: &Point| p.x > 0 && p.x < self.width - 1 && p.y > 0 && p.y < self.height - 1;
                if !state.snake.body.iter().all(valid_point) {
                    return false;
                }
                if !valid_point(&state.food) {
                    return false;
                }
                if !state.obstacles.iter().all(valid_point) {
                    return false;
                }
                if let Some((bp, _)) = &state.bonus_food
                    && !valid_point(bp)
                {
                    return false;
                }
                if let Some((pp, _)) = &state.poison_food
                    && !valid_point(pp)
                {
                    return false;
                }
                if let Some(pu) = &state.power_up
                    && !valid_point(&pu.location)
                {
                    return false;
                }
                if let Some(p2) = &state.player2
                    && !p2.body.iter().all(valid_point)
                {
                    return false;
                }
                state.snake.rebuild_map();
                if let Some(p2) = &mut state.player2 {
                    p2.rebuild_map();
                }
                self.mode = state.mode;
                self.snake = state.snake;
                self.player2 = state.player2;
                self.bots = state.bots;
                self.bots_autopilot_paths = state.bots_autopilot_paths;
                self.food = state.food;
                self.obstacles = state.obstacles;
                self.score = state.score;
                self.bonus_food = state.bonus_food.and_then(|(p, elapsed)| {
                    web_time::Instant::now()
                        .checked_sub(Duration::from_secs(elapsed))
                        .map(|t| (p, t))
                });
                self.poison_food = state.poison_food.and_then(|(p, elapsed)| {
                    web_time::Instant::now()
                        .checked_sub(Duration::from_secs(elapsed))
                        .map(|t| (p, t))
                });
                self.lives = state.lives;
                self.power_up = state.power_up;
                self.difficulty = state.difficulty;
                self.theme = state.theme;
                self.wrap_mode = state.wrap_mode;
                self.skin = state.skin;
                self.auto_pilot = state.auto_pilot;
                self.used_bot_this_session = state.used_bot_this_session;
                self.food_eaten_session = state.food_eaten_session;
                self.campaign_level = state.campaign_level;
                self.safe_zone_margin = state.safe_zone_margin;
                self.last_shrink_time = web_time::Instant::now();
                self.last_obstacle_spawn_time = web_time::Instant::now();
                self.combo = state.combo;
                self.last_food_time = state.last_food_time.and_then(|elapsed| {
                    web_time::Instant::now().checked_sub(Duration::from_secs(elapsed))
                });
                self.lasers = state.lasers;
                if let Some(boss) = state.boss {
                    self.bosses = vec![boss];
                } else {
                    self.bosses = state.bosses;
                }
                self.portals = state.portals;
                self.weather = state.weather;
                self.lightning_column = state.lightning_column;
                self.mines = state.mines;
                self.black_hole = state.black_hole;
                self.meteors = state.meteors;
                self.goblin = state.goblin;
                self.xp = state.xp;
                self.player_level = state.player_level;
                self.xp_to_next_level = state.xp_to_next_level;
                self.in_game_upgrades = state.in_game_upgrades;
                self.level_up_options = state.level_up_options;
                self.level_up_selection = state.level_up_selection;
                self.companion = state.companion;
                self.crops = Vec::new(); // Or state.crops if we added it to SaveState, but we can just initialize empty for now
                self.equipment_boxes = state.equipment_boxes;
                self.last_real_estate_tick = Some(Instant::now());
                self.fishing_timer = state.fishing_timer;
                self.fishing_progress = state.fishing_progress;
                self.is_fishing = state.is_fishing;
                self.eggs_on_board = state.eggs_on_board;
                self.paladin_life_timer = state.paladin_life_timer;
                self.mana = state.mana;
                self.max_mana = state.max_mana;
                self.time_of_day = state.time_of_day;
                self.tick_counter = state.tick_counter;
                self.p1_flag = state.p1_flag;
                self.p2_flag = state.p2_flag;
                self.p1_has_flag = state.p1_has_flag;
                self.p2_has_flag = state.p2_has_flag;
                self.p1_score = state.p1_score;
                self.p2_score = state.p2_score;
                self.koth_zone = state.koth_zone;
                self.xp_gems = state.xp_gems;
                self.dungeon_grid = state.dungeon_grid;
                self.current_room_coords = state.current_room_coords;
                self.flow_field = None;
                self.flow_field_targets = Vec::new();
                self.ghost_moves = std::collections::VecDeque::new();
                self.current_replay = Vec::new();
                self.ghost_snake = None;
                self.is_sprinting = false;
                self.state = GameState::Paused;
                self.start_time = web_time::Instant::now();
                self.floating_texts = Vec::new();
                self.update_high_scores();
                self.history.clear();
                true
            })
    }
    fn get_random_empty_point(
        width: u16,
        height: u16,
        snake: &Snake,
        avoid: impl Fn(&Point) -> bool,
        rng: &mut rand::rngs::StdRng,
        margin: u16,
    ) -> Option<Point> {
        let mut i = 0;
        loop {
            let min_x = 1 + margin;
            let max_x = (width - 1).saturating_sub(margin).max(min_x + 1);
            let min_y = 1 + margin;
            let max_y = (height - 1).saturating_sub(margin).max(min_y + 1);
            if min_x >= max_x || min_y >= max_y {
                return None;
            }
            let x = rng.gen_range(min_x..max_x);
            let y = rng.gen_range(min_y..max_y);
            let p = Point {
                x,
                y,
            };
            if !snake.body_map.contains_key(&p) && !avoid(&p) {
                return Some(p);
            }
            i += 1;
            if i >= 100 {
                let mut empty = Vec::new();
                for y_ in min_y..max_y {
                    for x_ in min_x..max_x {
                        let p_ = Point {
                            x: x_,
                            y: y_,
                        };
                        if !snake.body_map.contains_key(&p_) && !avoid(&p_) {
                            empty.push(p_);
                        }
                    }
                }
                if !empty.is_empty() {
                    let idx = rng.gen_range(0..empty.len());
                    return Some(empty[idx]);
                }
                return None;
            }
        }
    }
    fn manage_turrets(&mut self) {
        let mut new_lasers = Vec::new();
        for turret in &mut self.turrets {
            turret.shoot_timer += 1;
            if turret.shoot_timer >= 10 {
                let mut target_dir = None;
                let mut min_dist = u32::MAX;

                let mut possible_targets = Vec::new();
                for boss in &self.bosses {
                    possible_targets.push(boss.position);
                }
                if let Some(goblin) = &self.goblin {
                    possible_targets.push(goblin.position);
                }
                for meteor in &self.meteors {
                    possible_targets.push(meteor.position);
                }

                for target in possible_targets {
                    if target.x == turret.position.x || target.y == turret.position.y {
                        let dist = u32::from(turret.position.x.abs_diff(target.x))
                            + u32::from(turret.position.y.abs_diff(target.y));
                        if dist < min_dist {
                            min_dist = dist;
                            if target.x == turret.position.x && target.y < turret.position.y {
                                target_dir = Some(Direction::Up);
                            } else if target.x == turret.position.x && target.y > turret.position.y
                            {
                                target_dir = Some(Direction::Down);
                            } else if target.y == turret.position.y && target.x < turret.position.x
                            {
                                target_dir = Some(Direction::Left);
                            } else if target.y == turret.position.y && target.x > turret.position.x
                            {
                                target_dir = Some(Direction::Right);
                            }
                        }
                    }
                }

                if let Some(dir) = target_dir {
                    turret.shoot_timer = 0;
                    new_lasers.push(Laser {
                        position: turret.position,
                        direction: dir,
                        player: 1, // Considered player 1's laser
                    });
                }
            }
        }
        self.lasers.extend(new_lasers);
    }

    #[expect(
        clippy::too_many_lines,
        reason = "Game loop inherently requires handling multiple states and events"
    )]
    fn manage_goblin(&mut self) {
        let spawn_chance = if self.mode == GameMode::SnakeSurvivor {
            0.05 // Spawns frequently in Survivor
        } else if self.time_of_day == crate::game::TimeOfDay::Night {
            0.02
        } else {
            0.005
        };
        let can_spawn = if self.mode == GameMode::SnakeSurvivor {
            self.goblin.is_none() && self.rng.gen_bool(spawn_chance) // Ignore bosses being empty in survivor
        } else {
            self.goblin.is_none() && self.rng.gen_bool(spawn_chance) && self.bosses.is_empty()
        };

        if can_spawn {
            let margin = if self.mode == GameMode::BattleRoyale {
                self.safe_zone_margin
            } else {
                0
            };
            let avoid = |p: &Point| {
                self.obstacles.contains(p)
                    || *p == self.food
                    || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                    || self.poison_food.is_some_and(|(pp, _)| *p == pp)
                    || self.power_up.as_ref().is_some_and(|pu| *p == pu.location)
                    || self.mines.contains(p)
                    || (self.portals.is_some()
                        && (p == &self.portals.unwrap().0 || p == &self.portals.unwrap().1))
                    || self.snake.body_map.contains_key(p)
                    || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p))
            };
            if let Some(spawn_pos) = Self::get_random_empty_point(
                self.width,
                self.height,
                &self.snake,
                avoid,
                &mut self.rng,
                margin,
            ) {
                self.goblin = Some(Goblin {
                    position: spawn_pos,
                    move_timer: 0,
                    food_eaten: 0,
                });
                self.spawn_particles(
                    f32::from(spawn_pos.x),
                    f32::from(spawn_pos.y),
                    20,
                    crate::color::Color::Yellow,
                    'G',
                );
            }
        }
        let mut despawn = false;
        let mut new_food_needed = false;
        let mut particle_spawns = Vec::new();
        if let Some(mut goblin) = self.goblin {
            goblin.move_timer += 1;
            if goblin.move_timer >= 2 {
                goblin.move_timer = 0;
                let target = self.food;
                if let Some(dir) = self.bot_smart_pathfind(goblin.position, target, 3) {
                    let next_pos = Self::calculate_next_head_dir(goblin.position, dir);
                    let margin = if self.mode == GameMode::BattleRoyale {
                        self.safe_zone_margin
                    } else {
                        0
                    };
                    if next_pos.x > margin
                        && next_pos.x < self.width - 1 - margin
                        && next_pos.y > margin
                        && next_pos.y < self.height - 1 - margin
                        && !self.obstacles.contains(&next_pos)
                    {
                        goblin.position = next_pos;
                        if goblin.position == self.food {
                            goblin.food_eaten += 1;
                            new_food_needed = true;
                            particle_spawns.push((
                                f32::from(goblin.position.x),
                                f32::from(goblin.position.y),
                                15,
                                crate::color::Color::Green,
                                '-',
                            ));
                            if goblin.food_eaten >= 3 {
                                despawn = true;
                                particle_spawns.push((
                                    f32::from(goblin.position.x),
                                    f32::from(goblin.position.y),
                                    30,
                                    crate::color::Color::White,
                                    '\\',
                                ));
                            }
                        }
                    }
                }
            }
            self.goblin = Some(goblin);
        }
        for (x, y, count, color, char) in particle_spawns {
            self.spawn_particles(x, y, count, color, char);
        }
        if new_food_needed {
            let margin = if self.mode == GameMode::BattleRoyale {
                self.safe_zone_margin
            } else {
                0
            };
            let avoid_food = |p: &Point| {
                self.obstacles.contains(p)
                    || self.snake.body_map.contains_key(p)
                    || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p))
            };
            if let Some(new_food) = Self::get_random_empty_point(
                self.width,
                self.height,
                &self.snake,
                avoid_food,
                &mut self.rng,
                margin,
            ) {
                self.food = new_food;
            }
        }
        if despawn {
            self.goblin = None;
        }
    }
    fn manage_meteors(&mut self) {
        if self.rng.gen_bool(0.01) {
            let margin = if self.mode == GameMode::BattleRoyale {
                self.safe_zone_margin
            } else {
                0
            };
            let min_x = margin + 1;
            let max_x = (self.width - 1).saturating_sub(margin).max(min_x);
            if max_x > min_x {
                let spawn_x = self.rng.gen_range(min_x..max_x);
                self.meteors.push(Meteor {
                    position: Point {
                        x: spawn_x,
                        y: margin + 1,
                    },
                    timer: 0,
                });
            }
        }
        let mut meteors_to_keep = Vec::new();
        let margin = if self.mode == GameMode::BattleRoyale {
            self.safe_zone_margin
        } else {
            0
        };
        for mut meteor in std::mem::take(&mut self.meteors) {
            meteor.timer += 1;
            if meteor.timer >= 2 {
                meteor.timer = 0;
                meteor.position.y += 1;
            }
            if meteor.position.y >= self.height - 1 - margin {
                self.spawn_particles(
                    f32::from(meteor.position.x),
                    f32::from(meteor.position.y),
                    10,
                    crate::color::Color::Red,
                    '*',
                );
                continue;
            }
            if self.obstacles.contains(&meteor.position) {
                self.obstacles.remove(&meteor.position);
                self.spawn_particles(
                    f32::from(meteor.position.x),
                    f32::from(meteor.position.y),
                    20,
                    crate::color::Color::Red,
                    'X',
                );
                crate::game::beep();
                continue;
            }
            if self.mines.contains(&meteor.position) {
                self.mines.remove(&meteor.position);
                self.spawn_particles(
                    f32::from(meteor.position.x),
                    f32::from(meteor.position.y),
                    40,
                    crate::color::Color::Red,
                    'X',
                );
                beep();
                continue;
            }
            meteors_to_keep.push(meteor);
        }
        self.meteors = meteors_to_keep;
    }
    fn manage_black_hole(&mut self) {
        let spawn_chance = 0.002;
        if self.black_hole.is_none() && self.rng.gen_bool(spawn_chance) {
            let avoid = |p: &Point| {
                self.obstacles.contains(p)
                    || *p == self.food
                    || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                    || self.poison_food.is_some_and(|(pp, _)| *p == pp)
                    || self.power_up.as_ref().is_some_and(|pu| *p == pu.location)
                    || self.mines.contains(p)
                    || (self.portals.is_some()
                        && (p == &self.portals.unwrap().0 || p == &self.portals.unwrap().1))
                    || self.snake.body_map.contains_key(p)
                    || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p))
            };
            if let Some(bh) = Self::get_random_empty_point(
                self.width,
                self.height,
                &self.snake,
                avoid,
                &mut self.rng,
                self.safe_zone_margin + 2,
            ) {
                self.black_hole = Some(bh);
                self.spawn_particles(
                    f32::from(bh.x),
                    f32::from(bh.y),
                    30,
                    crate::color::Color::DarkGrey,
                    'O',
                );
            }
        } else if self.black_hole.is_some() && self.rng.gen_bool(0.01) {
            self.black_hole = None;
        }
        if let Some(bh) = self.black_hole {
            if self.rng.gen_bool(0.2) {
                let mut p = self.food;
                self.pull_point_towards_black_hole(&mut p, bh);
                self.food = p;
            }
            if let Some((mut bp, time)) = self.bonus_food {
                if self.rng.gen_bool(0.2) {
                    self.pull_point_towards_black_hole(&mut bp, bh);
                }
                self.bonus_food = Some((bp, time));
            }
            if let Some((mut pp, time)) = self.poison_food {
                if self.rng.gen_bool(0.2) {
                    self.pull_point_towards_black_hole(&mut pp, bh);
                }
                self.poison_food = Some((pp, time));
            }
            if let Some(mut pu) = self.power_up.clone()
                && self.rng.gen_bool(0.2)
            {
                self.pull_point_towards_black_hole(&mut pu.location, bh);
                self.power_up = Some(pu);
            }
            let mut to_remove = Vec::new();
            for obs in &self.obstacles {
                let dist = bh.x.abs_diff(obs.x).saturating_add(bh.y.abs_diff(obs.y));
                if dist <= 1 {
                    to_remove.push(*obs);
                }
            }
            for obs in to_remove {
                self.obstacles.remove(&obs);
            }
            let mut mines_to_remove = Vec::new();
            for mine in &self.mines {
                let dist = bh.x.abs_diff(mine.x).saturating_add(bh.y.abs_diff(mine.y));
                if dist <= 1 {
                    mines_to_remove.push(*mine);
                }
            }
            for mine in mines_to_remove {
                self.mines.remove(&mine);
            }
        }
    }
    fn pull_point_towards_black_hole(&self, point: &mut Point, bh: Point) {
        let dx = i32::from(bh.x) - i32::from(point.x);
        let dy = i32::from(bh.y) - i32::from(point.y);
        if dx == 0 && dy == 0 {
            return;
        }
        let mut next_p = *point;
        if dx.abs() > dy.abs() {
            if dx > 0 {
                next_p.x += 1;
            } else {
                next_p.x -= 1;
            }
        } else {
            if dy > 0 {
                next_p.y += 1;
            } else {
                next_p.y -= 1;
            }
        }
        if next_p.x > 0
            && next_p.x < self.width - 1
            && next_p.y > 0
            && next_p.y < self.height - 1
            && !self.snake.body_map.contains_key(&next_p)
            && !self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(&next_p))
        {
            *point = next_p;
        }
    }
    fn manage_mines(&mut self) {
        let spawn_chance = 0.005;
        if self.rng.gen_bool(spawn_chance) && self.mines.len() < 5 {
            let avoid = |p: &Point| {
                self.obstacles.contains(p)
                    || *p == self.food
                    || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                    || self.poison_food.is_some_and(|(pp, _)| *p == pp)
                    || self.power_up.as_ref().is_some_and(|pu| *p == pu.location)
                    || self.mines.contains(p)
                    || (self.portals.is_some()
                        && (p == &self.portals.unwrap().0 || p == &self.portals.unwrap().1))
            };
            if let Some(mine) = Self::get_random_empty_point(
                self.width,
                self.height,
                &self.snake,
                avoid,
                &mut self.rng,
                self.safe_zone_margin,
            ) {
                self.mines.insert(mine);
            }
        }
    }
    fn manage_poison_food(&mut self) {
        let spawn_chance = 0.015;
        if let Some((_, spawn_time)) = self.poison_food {
            if spawn_time.elapsed() > Duration::from_secs(8) {
                self.poison_food = None;
            }
        } else if self.rng.gen_bool(spawn_chance) {
            let avoid = |p: &Point| {
                self.obstacles.contains(p)
                    || *p == self.food
                    || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                    || self.power_up.as_ref().is_some_and(|pu| *p == pu.location)
            };
            if let Some(poison) = Self::get_random_empty_point(
                self.width,
                self.height,
                &self.snake,
                avoid,
                &mut self.rng,
                self.safe_zone_margin,
            ) {
                self.poison_food = Some((poison, web_time::Instant::now()));
            }
        }
    }
    fn generate_obstacles(
        width: u16,
        height: u16,
        snake: &Snake,
        avoid: impl Fn(&Point) -> bool,
        rng: &mut rand::rngs::StdRng,
        count: usize,
        margin: u16,
    ) -> HashSet<Point> {
        let mut obstacles = HashSet::new();
        for _ in 0..count {
            let current_avoid = |p: &Point| avoid(p) || obstacles.contains(p);
            if let Some(p) =
                Self::get_random_empty_point(width, height, snake, current_avoid, rng, margin)
            {
                obstacles.insert(p);
            }
        }
        obstacles
    }
    #[must_use]
    pub fn generate_dungeon_obstacles(
        width: u16,
        height: u16,
        rng: &mut rand::rngs::StdRng,
    ) -> HashSet<Point> {
        let mut obstacles = HashSet::new();
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                obstacles.insert(Point {
                    x,
                    y,
                });
            }
        }
        let mut rooms: Vec<(u16, u16, u16, u16)> = Vec::new();
        let num_rooms = rng.gen_range(3..=6);
        let start_x = width / 2;
        let start_y = height / 2;
        let center_room_w = rng.gen_range(3..=5);
        let center_room_h = rng.gen_range(3..=5);
        let center_room_x = start_x.saturating_sub(center_room_w / 2).max(1);
        let center_room_y = start_y.saturating_sub(center_room_h / 2).max(1);
        rooms.push((center_room_x, center_room_y, center_room_w, center_room_h));
        for _ in 1..num_rooms {
            let w = rng.gen_range(3..=7);
            let h = rng.gen_range(3..=7);
            let x = rng.gen_range(2..width.saturating_sub(w + 1).max(3));
            let y = rng.gen_range(2..height.saturating_sub(h + 1).max(3));
            let mut overlap = false;
            for &(rx, ry, rw, rh) in &rooms {
                if x < rx + rw && x + w > rx && y < ry + rh && y + h > ry {
                    overlap = true;
                    break;
                }
            }
            if !overlap {
                rooms.push((x, y, w, h));
            }
        }
        for &(rx, ry, rw, rh) in &rooms {
            for y in ry..ry + rh {
                for x in rx..rx + rw {
                    obstacles.remove(&Point {
                        x,
                        y,
                    });
                }
            }
        }
        for i in 0..rooms.len() - 1 {
            let (r1_x, r1_y, r1_w, r1_h) = rooms[i];
            let (r2_x, r2_y, r2_w, r2_h) = rooms[i + 1];
            let c1_x = r1_x + r1_w / 2;
            let c1_y = r1_y + r1_h / 2;
            let c2_x = r2_x + r2_w / 2;
            let c2_y = r2_y + r2_h / 2;
            let mut x = c1_x;
            let mut y = c1_y;
            while x != c2_x {
                obstacles.remove(&Point {
                    x,
                    y,
                });
                if x < c2_x {
                    x += 1;
                } else {
                    x -= 1;
                }
            }
            while y != c2_y {
                obstacles.remove(&Point {
                    x,
                    y,
                });
                if y < c2_y {
                    y += 1;
                } else {
                    y -= 1;
                }
            }
        }
        for dy in -2..=2 {
            for dx in -2..=2 {
                let cx = i32::from(start_x) + dx;
                let cy = i32::from(start_y) + dy;
                if cx > 0 && cx < i32::from(width - 1) && cy > 0 && cy < i32::from(height - 1) {
                    obstacles.remove(&Point {
                        x: u16::try_from(cx).unwrap_or(0),
                        y: u16::try_from(cy).unwrap_or(0),
                    });
                }
            }
        }
        obstacles
    }
    pub fn shift_timers(&mut self, delta: Duration) {
        if let Some(new_time) = self.start_time.checked_add(delta) {
            self.start_time = new_time;
        }
        if let Some((pos, spawn_time)) = self.bonus_food
            && let Some(new_time) = spawn_time.checked_add(delta)
        {
            self.bonus_food = Some((pos, new_time));
        }
        if let Some((pos, spawn_time)) = self.poison_food
            && let Some(new_time) = spawn_time.checked_add(delta)
        {
            self.poison_food = Some((pos, new_time));
        }
        if let Some(power_up) = &mut self.power_up
            && let Some(activation_time) = power_up.activation_time
            && let Some(new_time) = activation_time.checked_add(delta.as_secs())
        {
            power_up.activation_time = Some(new_time);
        }
        if let Some(new_time) = self.last_shrink_time.checked_add(delta) {
            self.last_shrink_time = new_time;
        }
        if let Some(new_time) = self.last_obstacle_spawn_time.checked_add(delta) {
            self.last_obstacle_spawn_time = new_time;
        }
        if let Some(last_food) = self.last_food_time
            && let Some(new_time) = last_food.checked_add(delta)
        {
            self.last_food_time = Some(new_time);
        }
    }
    #[must_use]
    pub fn generate_maze_obstacles(
        width: u16,
        height: u16,
        rng: &mut rand::rngs::StdRng,
    ) -> HashSet<Point> {
        let mut obstacles = HashSet::new();
        let max_x = width.saturating_sub(2);
        let max_y = height.saturating_sub(2);
        for y in 1..=max_y {
            for x in 1..=max_x {
                obstacles.insert(Point {
                    x,
                    y,
                });
            }
        }
        let start_x = width / 2;
        let start_y = height / 2;
        let start_maze_x = (rng.gen_range(1..=std::cmp::max(1, max_x / 2)) * 2) - 1;
        let start_maze_y = (rng.gen_range(1..=std::cmp::max(1, max_y / 2)) * 2) - 1;
        let mut stack = vec![Point {
            x: start_maze_x,
            y: start_maze_y,
        }];
        obstacles.remove(&Point {
            x: start_maze_x,
            y: start_maze_y,
        });
        while let Some(current) = stack.last().copied() {
            let mut neighbors = Vec::new();
            let dirs = [(0, -2), (0, 2), (-2, 0), (2, 0)];
            for (dx, dy) in dirs {
                let nx = i32::from(current.x) + dx;
                let ny = i32::from(current.y) + dy;
                if nx > 0 && nx <= i32::from(max_x) && ny > 0 && ny <= i32::from(max_y) {
                    let next_p = Point {
                        x: u16::try_from(nx).unwrap_or(0),
                        y: u16::try_from(ny).unwrap_or(0),
                    };
                    if obstacles.contains(&next_p) {
                        neighbors.push((
                            next_p,
                            Point {
                                x: u16::try_from(i32::from(current.x) + dx / 2).unwrap_or(0),
                                y: u16::try_from(i32::from(current.y) + dy / 2).unwrap_or(0),
                            },
                        ));
                    }
                }
            }
            if neighbors.is_empty() {
                stack.pop();
            } else {
                let (next_p, wall_p) = neighbors[rng.gen_range(0..neighbors.len())];
                obstacles.remove(&wall_p);
                obstacles.remove(&next_p);
                stack.push(next_p);
            }
        }
        for dy in -2..=2 {
            for dx in -2..=2 {
                let cx = i32::from(start_x) + dx;
                let cy = i32::from(start_y) + dy;
                if cx > 0 && cx <= i32::from(max_x) && cy > 0 && cy <= i32::from(max_y) {
                    obstacles.remove(&Point {
                        x: u16::try_from(cx).unwrap_or(0),
                        y: u16::try_from(cy).unwrap_or(0),
                    });
                }
            }
        }
        obstacles
    }
    pub fn rise_flood(&mut self) {
        if self.mode != GameMode::Flood {
            return;
        }
        let mut highest_flood_y = self.height;
        for y in (0..self.height).rev() {
            let mut fully_flooded = true;
            for x in 0..self.width {
                if !self.obstacles.contains(&Point {
                    x,
                    y,
                }) {
                    fully_flooded = false;
                    break;
                }
            }
            if fully_flooded {
                highest_flood_y = y;
            } else {
                break;
            }
        }
        let new_flood_y = highest_flood_y.saturating_sub(1);
        if new_flood_y > 0 {
            for x in 0..self.width {
                self.obstacles.insert(Point {
                    x,
                    y: new_flood_y,
                });
            }
        }
    }
    pub fn evolve_game_of_life(&mut self) {
        if self.mode != GameMode::Evolution {
            return;
        }
        let mut next_obstacles = HashSet::new();
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let p = Point {
                    x,
                    y,
                };
                let mut safe = false;
                let h1 = self.snake.head();
                if (i32::from(x) - i32::from(h1.x)).abs() <= 2
                    && (i32::from(y) - i32::from(h1.y)).abs() <= 2
                {
                    safe = true;
                }
                if let Some(p2) = &self.player2 {
                    let h2 = p2.head();
                    if (i32::from(x) - i32::from(h2.x)).abs() <= 2
                        && (i32::from(y) - i32::from(h2.y)).abs() <= 2
                    {
                        safe = true;
                    }
                }
                if safe {
                    continue;
                }
                let mut neighbors = 0;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = i32::from(x) + dx;
                        let ny = i32::from(y) + dy;
                        if self.obstacles.contains(&Point {
                            x: u16::try_from(nx).unwrap_or(0),
                            y: u16::try_from(ny).unwrap_or(0),
                        }) {
                            neighbors += 1;
                        }
                    }
                }
                let is_alive = self.obstacles.contains(&p);
                if neighbors == 3 || (is_alive && neighbors == 2) {
                    next_obstacles.insert(p);
                }
            }
        }
        let avoid = |p: &Point| {
            self.snake.body_map.contains_key(p)
                || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p))
                || *p == self.food
                || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                || self.poison_food.is_some_and(|(pp, _)| *p == pp)
                || self.power_up.as_ref().is_some_and(|pu| *p == pu.location)
        };
        next_obstacles.retain(|p| !avoid(p));
        self.obstacles = next_obstacles;
    }
    #[must_use]
    pub fn generate_cave_obstacles(
        width: u16,
        height: u16,
        rng: &mut rand::rngs::StdRng,
    ) -> HashSet<Point> {
        let mut grid = vec![vec![false; width as usize]; height as usize];
        let fill_probability = 0.45;
        for y in 0..height {
            for x in 0..width {
                if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                    grid[y as usize][x as usize] = true;
                } else {
                    grid[y as usize][x as usize] = rng.gen_bool(fill_probability);
                }
            }
        }
        let iterations = 4;
        for _ in 0..iterations {
            let mut next_grid = grid.clone();
            for y in 1..(height - 1) {
                for x in 1..(width - 1) {
                    let mut neighbor_walls = 0;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = i32::from(x) + dx;
                            let ny = i32::from(y) + dy;
                            if grid[usize::try_from(ny).unwrap_or(0)]
                                [usize::try_from(nx).unwrap_or(0)]
                            {
                                neighbor_walls += 1;
                            }
                        }
                    }
                    if grid[y as usize][x as usize] {
                        next_grid[y as usize][x as usize] = neighbor_walls >= 4;
                    } else {
                        next_grid[y as usize][x as usize] = neighbor_walls >= 5;
                    }
                }
            }
            grid = next_grid;
        }
        let start_x = width / 2;
        let start_y = height / 2;
        for dy in -3..=3 {
            for dx in -3..=3 {
                let cx = i32::from(start_x) + dx;
                let cy = i32::from(start_y) + dy;
                if cx > 0 && cx < i32::from(width - 1) && cy > 0 && cy < i32::from(height - 1) {
                    grid[usize::try_from(cy).unwrap_or(0)][usize::try_from(cx).unwrap_or(0)] =
                        false;
                }
            }
        }
        let mut obstacles = HashSet::new();
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                if grid[y as usize][x as usize] {
                    obstacles.insert(Point {
                        x,
                        y,
                    });
                }
            }
        }
        obstacles
    }
    #[must_use]
    pub fn generate_campaign_obstacles(&self) -> HashSet<Point> {
        crate::game::get_campaign_map(self.campaign_level, self.width, self.height)
    }
    fn load_ghost_replay() -> Option<std::collections::VecDeque<crate::snake::Direction>> {
        File::open("ghost.json").ok().and_then(|f| {
            let mut content = String::new();
            f.take(10 * 1024 * 1024).read_to_string(&mut content).ok()?;
            serde_json::from_str(&content).ok()
        })
    }
    #[expect(clippy::too_many_lines, reason = "Game reset handles logic for different game modes")]
    #[doc = " # Panics"]
    #[doc = ""]
    #[doc = " Panics if the board is completely full and there's no room for food upon reset."]
    pub fn reset(&mut self) {
        // Initialize quests if none are active and none are completed (a simple way to populate initial quests)
        if self.stats.active_quests.is_empty() && self.stats.completed_quests.is_empty() {
            self.stats.active_quests.push(crate::game::Quest {
                name: "First Blood".to_string(),
                description: "Defeat 1 Boss".to_string(),
                q_type: crate::game::QuestType::SlayBosses,
                target: 1,
                progress: 0,
                reward: 500,
                status: crate::game::QuestStatus::Active,
            });
            self.stats.active_quests.push(crate::game::Quest {
                name: "High Roller".to_string(),
                description: "Reach 500 Score".to_string(),
                q_type: crate::game::QuestType::ReachScore,
                target: 500,
                progress: 0,
                reward: 1000,
                status: crate::game::QuestStatus::Active,
            });
            self.stats.active_quests.push(crate::game::Quest {
                name: "Coin Collector".to_string(),
                description: "Collect 1000 Coins".to_string(),
                q_type: crate::game::QuestType::CollectCoins,
                target: 1000,
                progress: 0,
                reward: 250,
                status: crate::game::QuestStatus::Active,
            });
        }
        let start_x = self.width / 2;
        let start_y = self.height / 2;
        if self.mode == GameMode::Campaign || self.mode == GameMode::BossRush {
            self.campaign_level = 1;
        }
        match self.mode {
            GameMode::SinglePlayer
            | GameMode::Campaign
            | GameMode::TimeAttack
            | GameMode::Speedrun
            | GameMode::Survival
            | GameMode::Zen
            | GameMode::Maze
            | GameMode::Cave
            | GameMode::Dungeon
            | GameMode::CustomLevel
            | GameMode::PacMan
            | GameMode::DailyChallenge
            | GameMode::WeeklyChallenge
            | GameMode::MonthlyChallenge
            | GameMode::YearlyChallenge
            | GameMode::DecadeChallenge
            | GameMode::CenturyChallenge
            | GameMode::MillenniumChallenge
            | GameMode::EonChallenge
            | GameMode::FogOfWar
            | GameMode::Evolution
            | GameMode::BossRush
            | GameMode::MassiveMultiplayer
            | GameMode::Mirror
            | GameMode::Flood
            | GameMode::Vampire
            | GameMode::Gravity
            | GameMode::Zombie
            | GameMode::Farmstead
            | GameMode::BulletHell
            | GameMode::SnakeSurvivor
            | GameMode::KingOfTheHill
            | GameMode::Dodgeball
            | GameMode::DungeonCrawler
            | GameMode::Chaos
            | GameMode::Miner => {
                self.snake = Snake::new(Point {
                    x: start_x,
                    y: start_y,
                });
                self.player2 = None;
            },
            GameMode::LocalMultiplayer
            | GameMode::OnlineMultiplayer
            | GameMode::Tournament
            | GameMode::PlayerVsBot
            | GameMode::BotVsBot
            | GameMode::BattleRoyale
            | GameMode::Tron
            | GameMode::CaptureTheFlag => {
                self.snake = Snake::new(Point {
                    x: start_x - 5,
                    y: start_y,
                });
                self.player2 = Some(Snake::new(Point {
                    x: start_x + 5,
                    y: start_y,
                }));
            },
        }
        let obs_count = if self.mode == GameMode::Zen
            || self.mode == GameMode::Maze
            || self.mode == GameMode::Cave
            || self.mode == GameMode::Dungeon
        {
            0
        } else {
            match self.difficulty {
                Difficulty::Easy => 1,
                Difficulty::Normal => 3,
                Difficulty::Hard => 5,
                Difficulty::Insane => 10,
                Difficulty::GodMode => 20,
            }
        };
        let avoid = |p: &Point| {
            if self.mode == GameMode::SinglePlayer
                || self.mode == GameMode::Campaign
                || self.mode == GameMode::TimeAttack
                || self.mode == GameMode::Speedrun
                || self.mode == GameMode::Survival
                || self.mode == GameMode::Zen
                || self.mode == GameMode::Maze
                || self.mode == GameMode::Cave
                || self.mode == GameMode::Dungeon
                || self.mode == GameMode::CustomLevel
                || self.mode == GameMode::PacMan
                || self.mode == GameMode::DailyChallenge
                || self.mode == GameMode::FogOfWar
                || self.mode == GameMode::Evolution
                || self.mode == GameMode::BossRush
                || self.mode == GameMode::MassiveMultiplayer
                || self.mode == GameMode::Mirror
                || self.mode == GameMode::Flood
                || self.mode == GameMode::Vampire
                || self.mode == GameMode::Gravity
                || self.mode == GameMode::KingOfTheHill
            {
                p.x == start_x && p.y == start_y - 1
            } else {
                (p.x == start_x + 5 || p.x == start_x - 5) && p.y == start_y - 1
            }
        };
        let empty_snake = Snake::new(Point {
            x: 1,
            y: 1,
        });
        let ref_snake = if self.mode == GameMode::SinglePlayer
            || self.mode == GameMode::Campaign
            || self.mode == GameMode::TimeAttack
            || self.mode == GameMode::Speedrun
            || self.mode == GameMode::Survival
            || self.mode == GameMode::Zen
            || self.mode == GameMode::Maze
            || self.mode == GameMode::Cave
            || self.mode == GameMode::Dungeon
            || self.mode == GameMode::CustomLevel
            || self.mode == GameMode::PacMan
            || self.mode == GameMode::DailyChallenge
            || self.mode == GameMode::FogOfWar
            || self.mode == GameMode::Evolution
            || self.mode == GameMode::BossRush
            || self.mode == GameMode::MassiveMultiplayer
            || self.mode == GameMode::Mirror
            || self.mode == GameMode::Flood
            || self.mode == GameMode::Vampire
            || self.mode == GameMode::Gravity
        {
            &self.snake
        } else {
            &empty_snake
        };
        if self.mode == GameMode::DailyChallenge {
            let days_since_epoch = web_time::SystemTime::now()
                .duration_since(web_time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                / 86400;
            self.rng = rand::rngs::StdRng::seed_from_u64(days_since_epoch);
        } else if self.mode == GameMode::WeeklyChallenge {
            let weeks_since_epoch = web_time::SystemTime::now()
                .duration_since(web_time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                / (86400 * 7);
            self.rng = rand::rngs::StdRng::seed_from_u64(weeks_since_epoch);
        } else if self.mode == GameMode::MonthlyChallenge {
            let months_since_epoch = web_time::SystemTime::now()
                .duration_since(web_time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                / (86400 * 30);
            self.rng = rand::rngs::StdRng::seed_from_u64(months_since_epoch);
        } else if self.mode == GameMode::YearlyChallenge {
            let years_since_epoch = web_time::SystemTime::now()
                .duration_since(web_time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                / (86400 * 365);
            self.rng = rand::rngs::StdRng::seed_from_u64(years_since_epoch);
        } else if self.mode == GameMode::DecadeChallenge {
            let decades_since_epoch = web_time::SystemTime::now()
                .duration_since(web_time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                / (86400 * 3650);
            self.rng = rand::rngs::StdRng::seed_from_u64(decades_since_epoch);
        } else if self.mode == GameMode::CenturyChallenge {
            let centuries_since_epoch = web_time::SystemTime::now()
                .duration_since(web_time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                / (86400 * 36500);
            self.rng = rand::rngs::StdRng::seed_from_u64(centuries_since_epoch);
        } else if self.mode == GameMode::MillenniumChallenge {
            let millennia_since_epoch = web_time::SystemTime::now()
                .duration_since(web_time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                / (86400 * 365_000);
            self.rng = rand::rngs::StdRng::seed_from_u64(millennia_since_epoch);
        } else if self.mode == GameMode::EonChallenge {
            let eons_since_epoch = web_time::SystemTime::now()
                .duration_since(web_time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                / (86400 * 365_000_000);
            self.rng = rand::rngs::StdRng::seed_from_u64(eons_since_epoch);
        } else {
            self.rng = rand::rngs::StdRng::from_entropy();
        }
        self.current_replay.clear();
        if self.mode == GameMode::Speedrun
            && let Some(ghost) = Self::load_ghost_replay()
        {
            self.ghost_moves = ghost;
            self.ghost_snake = Some(Snake::new(Point {
                x: start_x,
                y: start_y,
            }));
        } else if self.mode != GameMode::Speedrun {
            self.ghost_moves.clear();
            self.ghost_snake = None;
        }
        self.is_sprinting = false;

        if self.mode == GameMode::DungeonCrawler && self.dungeon_grid.is_empty() {
            self.dungeon_grid =
                crate::game::dungeon::generate_dungeon(self.campaign_level, &mut self.rng);
            self.current_room_coords = (0, 0);
        }

        if self.mode == GameMode::CustomLevel {
            self.obstacles = Self::load_custom_level();
            let body_map = self.snake.body_map.clone();
            self.obstacles.retain(|p| !body_map.contains_key(p));
        } else if self.mode == GameMode::Campaign {
            self.obstacles = self.generate_campaign_obstacles();
            let body_map = self.snake.body_map.clone();
            self.obstacles.retain(|p| !body_map.contains_key(p));
        } else if self.mode == GameMode::Maze {
            self.obstacles = Self::generate_maze_obstacles(self.width, self.height, &mut self.rng);
            let body_map = self.snake.body_map.clone();
            self.obstacles.retain(|p| !body_map.contains_key(p));
        } else if self.mode == GameMode::Cave {
            self.obstacles = Self::generate_cave_obstacles(self.width, self.height, &mut self.rng);
            let body_map = self.snake.body_map.clone();
            self.obstacles.retain(|p| !body_map.contains_key(p));
        } else if self.mode == GameMode::Dungeon {
            self.obstacles =
                Self::generate_dungeon_obstacles(self.width, self.height, &mut self.rng);
            let body_map = self.snake.body_map.clone();
            self.obstacles.retain(|p| !body_map.contains_key(p));
        } else if self.mode == GameMode::PacMan {
            self.obstacles = Self::generate_maze_obstacles(self.width, self.height, &mut self.rng);
            let body_map = self.snake.body_map.clone();
            self.obstacles.retain(|p| !body_map.contains_key(p));
            for y in 1..self.height - 1 {
                for x in 1..self.width - 1 {
                    let p = Point {
                        x,
                        y,
                    };
                    if !self.obstacles.contains(&p)
                        && !body_map.contains_key(&p)
                        && p != self.snake.head()
                    {
                        self.crops.push(crate::game::Crop {
                            position: p,
                            growth_stage: 2,
                            timer: 0,
                        });
                    }
                }
            }
        } else if self.mode == GameMode::Evolution {
            let fill_probability = 0.2;
            for y in 1..self.height - 1 {
                for x in 1..self.width - 1 {
                    if self.rng.gen_bool(fill_probability) {
                        self.obstacles.insert(Point {
                            x,
                            y,
                        });
                    }
                }
            }
            let body_map = self.snake.body_map.clone();
            self.obstacles.retain(|p| !body_map.contains_key(p));
        } else if self.mode == GameMode::Miner {
            let mut obstacles = HashSet::new();
            for y in 1..self.height - 1 {
                for x in 1..self.width - 1 {
                    if self.rng.gen_bool(0.7) {
                        obstacles.insert(Point {
                            x,
                            y,
                        });
                    }
                }
            }
            self.obstacles = obstacles;
            let body_map = self.snake.body_map.clone();
            self.obstacles.retain(|p| !body_map.contains_key(p));
            if let Some(p2) = &self.player2 {
                let p2_body_map = p2.body_map.clone();
                self.obstacles.retain(|p| !p2_body_map.contains_key(p));
            }
        } else {
            let mut obstacles = HashSet::new();
            for _ in 0..obs_count {
                let current_avoid = |p: &Point| {
                    avoid(p)
                        || obstacles.contains(p)
                        || self.snake.body_map.contains_key(p)
                        || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p))
                };
                if let Some(p) = Self::get_random_empty_point(
                    self.width,
                    self.height,
                    ref_snake,
                    current_avoid,
                    &mut self.rng,
                    0,
                ) {
                    obstacles.insert(p);
                }
            }
            self.obstacles = obstacles;
        }
        let avoid_food = |p: &Point| {
            self.obstacles.contains(p)
                || self.snake.body_map.contains_key(p)
                || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p))
        };
        self.food = Self::get_random_empty_point(
            self.width,
            self.height,
            ref_snake,
            avoid_food,
            &mut self.rng,
            0,
        )
        .expect("Board cannot be full on reset");
        self.bonus_food = None;
        self.poison_food = None;
        if self.skin == '🚀' {
            self.power_up = Some(PowerUp {
                p_type: PowerUpType::SpeedBoost,
                location: Point {
                    x: 0,
                    y: 0,
                },
                activation_time: Some(
                    web_time::SystemTime::now()
                        .duration_since(web_time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                ),
            });
        } else {
            self.power_up = None;
            if self.stats.equipped_class == Some(crate::game::HeroClass::Mage) {
                self.power_up = Some(PowerUp {
                    p_type: PowerUpType::TimeFreeze,
                    location: Point {
                        x: 0,
                        y: 0,
                    },
                    activation_time: Some(
                        web_time::SystemTime::now()
                            .duration_since(web_time::SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    ),
                });
            }
        }
        self.decoy = None;
        self.score = 0;
        self.paladin_life_timer = 0;
        self.lives = if self.stats.equipped_class == Some(crate::game::HeroClass::Warrior) {
            3 + u32::from(self.stats.upgrade_extra_lives)
        } else if self.skin == '💎' {
            3 + u32::from(self.stats.upgrade_extra_lives) + 1
        } else {
            3 + u32::from(self.stats.upgrade_extra_lives)
        };
        if self.stats.unlocked_artifacts.contains(&crate::game::Artifact::LifeChalice) {
            self.lives += 1;
        }
        self.state = GameState::Playing;
        self.just_died = false;
        self.start_time = web_time::Instant::now();
        self.floating_texts = Vec::new();
        self.food_eaten_session = 0;
        self.auto_pilot = false;
        self.used_bot_this_session = false;
        self.safe_zone_margin = 0;
        self.last_shrink_time = web_time::Instant::now();
        self.last_obstacle_spawn_time = web_time::Instant::now();
        self.history.clear();
        self.particles.clear();
        self.floating_texts.clear();
        self.combo = 0;
        self.last_food_time = None;
        self.chat_log.clear();
        if self.mode == GameMode::MassiveMultiplayer {
            self.auto_pilot = true;
            self.chat_log.push_back((
                "SYSTEM: Simulating 100 bots entering the arena...".to_string(),
                crate::color::Color::Magenta,
            ));
        }
        self.last_chat_time = None;
        self.bosses.clear();
        self.resources.clear();
        self.eggs_on_board.clear();
        self.mana = 100;
        self.max_mana = 100;
        self.time_of_day = crate::game::TimeOfDay::Day;
        self.tick_counter = 0;
        self.portals = None;
        self.weather = Weather::Clear;
        if self.current_planet == Planet::Mars {
            self.weather = Weather::Sandstorm;
        }
        self.lightning_column = None;
        self.mines = HashSet::new();
        self.black_hole = None;
        self.meteors.clear();
        self.goblin = None;
        self.xp = 0;
        self.player_level = 1;
        self.xp_to_next_level = 5;
        self.p1_has_flag = false;
        self.p2_has_flag = false;
        self.xp_gems.clear();
        self.flow_field = None;
        self.flow_field_targets.clear();

        if self.mode == GameMode::CaptureTheFlag {
            self.p1_flag = Some(Point {
                x: 2,
                y: start_y,
            });
            self.p2_flag = Some(Point {
                x: self.width.saturating_sub(3),
                y: start_y,
            });
        } else {
            self.p1_flag = None;
            self.p2_flag = None;
        }

        if self.mode == GameMode::KingOfTheHill {
            self.koth_zone = Some(Point {
                x: start_x,
                y: start_y,
            });
            self.p1_score = 0;
            self.p2_score = 0;
        } else {
            self.koth_zone = None;
        }

        // Score is kept across respawns, but reset on full game reset
        self.p1_score = 0;
        self.p2_score = 0;
        self.in_game_upgrades.clear();
        self.level_up_options.clear();
        self.level_up_selection = 0;
        self.turrets.clear();
        self.bots.clear();
        self.bots_autopilot_paths.clear();

        if let Some(companion_type) = self.stats.equipped_companion {
            self.companion = Some(Companion {
                position: Point {
                    x: start_x.saturating_sub(1).max(1),
                    y: start_y.saturating_sub(1).max(1),
                },
                kind: companion_type,
                move_timer: 0,
                action_timer: 0,
                path: Vec::new(),
            });
        } else {
            self.companion = None;
        }

        self.crops.clear();
        if self.mode == GameMode::MassiveMultiplayer
            || self.mode == GameMode::Tron
            || self.mode == GameMode::Zombie
            || self.mode == GameMode::KingOfTheHill
        {
            let margin = self.safe_zone_margin;
            let count = if self.mode == GameMode::MassiveMultiplayer {
                50
            } else if self.mode == GameMode::Tron {
                3
            } else if self.mode == GameMode::KingOfTheHill {
                3
            } else {
                1
            };
            for _ in 0..count {
                let avoid = |p: &Point| {
                    self.obstacles.contains(p)
                        || self.snake.body_map.contains_key(p)
                        || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p))
                        || self.bots.iter().any(|b| b.body_map.contains_key(p))
                };
                if let Some(pos) = Self::get_random_empty_point(
                    self.width,
                    self.height,
                    &self.snake, // dummy for this call since we pass avoid logic
                    avoid,
                    &mut self.rng,
                    margin,
                ) {
                    self.bots.push(Snake::new(pos));
                    self.bots_autopilot_paths.push(Vec::new());
                }
            }
        }
    }
    fn respawn(&mut self) {
        let start_x = self.width / 2;
        let start_y = self.height / 2;
        match self.mode {
            GameMode::SinglePlayer
            | GameMode::DungeonCrawler
            | GameMode::Campaign
            | GameMode::TimeAttack
            | GameMode::Speedrun
            | GameMode::Survival
            | GameMode::Zen
            | GameMode::Maze
            | GameMode::Cave
            | GameMode::Dungeon
            | GameMode::CustomLevel
            | GameMode::PacMan
            | GameMode::DailyChallenge
            | GameMode::WeeklyChallenge
            | GameMode::MonthlyChallenge
            | GameMode::YearlyChallenge
            | GameMode::DecadeChallenge
            | GameMode::CenturyChallenge
            | GameMode::MillenniumChallenge
            | GameMode::EonChallenge
            | GameMode::FogOfWar
            | GameMode::Evolution
            | GameMode::BossRush
            | GameMode::MassiveMultiplayer
            | GameMode::Mirror
            | GameMode::Flood
            | GameMode::Vampire
            | GameMode::Gravity
            | GameMode::Zombie
            | GameMode::Farmstead
            | GameMode::BulletHell
            | GameMode::SnakeSurvivor
            | GameMode::KingOfTheHill
            | GameMode::Dodgeball
            | GameMode::Chaos
            | GameMode::Miner => {
                self.snake = Snake::new(Point {
                    x: start_x,
                    y: start_y,
                });
                self.player2 = None;
                self.obstacles.retain(|p| {
                    !(p.x == start_x && (p.y >= start_y.saturating_sub(1) && p.y <= start_y + 2))
                });
            },
            GameMode::LocalMultiplayer
            | GameMode::OnlineMultiplayer
            | GameMode::Tournament
            | GameMode::PlayerVsBot
            | GameMode::BotVsBot
            | GameMode::BattleRoyale
            | GameMode::Tron
            | GameMode::CaptureTheFlag => {
                self.snake = Snake::new(Point {
                    x: start_x - 5,
                    y: start_y,
                });
                self.player2 = Some(Snake::new(Point {
                    x: start_x + 5,
                    y: start_y,
                }));
                self.obstacles.retain(|p| {
                    !((p.x == start_x - 5
                        && (p.y >= start_y.saturating_sub(1) && p.y <= start_y + 2))
                        || (p.x == start_x + 5
                            && (p.y >= start_y.saturating_sub(1) && p.y <= start_y + 2)))
                });

                if self.mode == GameMode::CaptureTheFlag {
                    self.p1_flag = Some(Point {
                        x: 2,
                        y: start_y,
                    });
                    self.p2_flag = Some(Point {
                        x: self.width.saturating_sub(3),
                        y: start_y,
                    });
                    self.p1_has_flag = false;
                    self.p2_has_flag = false;
                }
            },
        }
        self.bots.clear();
        self.bots_autopilot_paths.clear();
        self.safe_zone_margin = 0;
        self.last_shrink_time = web_time::Instant::now();
        self.last_obstacle_spawn_time = web_time::Instant::now();
    }
    /// Shoots a laser for the specified player.
    ///
    /// # Panics
    /// Panics if the `Multishot` upgrade is in `in_game_upgrades` but its level value cannot be retrieved.
    pub fn shoot_laser(&mut self, player: u8) {
        let active_lasers = self.lasers.iter().filter(|l| l.player == player).count();
        let mut max_lasers = 3 + usize::from(self.stats.upgrade_laser_capacity);
        if player == 1 && self.skin == '👾' {
            max_lasers += 5;
        }
        if active_lasers >= max_lasers {
            return;
        }
        let (head, dir) = if player == 1 {
            (self.snake.head(), self.snake.direction)
        } else if player == 2 {
            if let Some(p2) = &self.player2 {
                (p2.head(), p2.direction)
            } else {
                return;
            }
        } else {
            return;
        };
        let laser_pos = Self::calculate_next_head_dir(head, dir);
        let margin = if self.mode == GameMode::BattleRoyale {
            self.safe_zone_margin
        } else {
            0
        };
        let mut spawn_positions = vec![laser_pos];

        if player == 1 && self.in_game_upgrades.contains_key(&InGameUpgrade::Multishot) {
            let multishot_level = *self.in_game_upgrades.get(&InGameUpgrade::Multishot).unwrap();

            for i in 1..=(i32::try_from(multishot_level).unwrap_or(1)) {
                match dir {
                    Direction::Up | Direction::Down => {
                        let mut p1 = laser_pos;
                        p1.x = (i32::from(p1.x) + i).try_into().unwrap_or(p1.x);
                        spawn_positions.push(p1);

                        let mut p2 = laser_pos;
                        p2.x = (i32::from(p2.x) - i).try_into().unwrap_or(p2.x);
                        spawn_positions.push(p2);
                    },
                    Direction::Left | Direction::Right => {
                        let mut p1 = laser_pos;
                        p1.y = (i32::from(p1.y) + i).try_into().unwrap_or(p1.y);
                        spawn_positions.push(p1);

                        let mut p2 = laser_pos;
                        p2.y = (i32::from(p2.y) - i).try_into().unwrap_or(p2.y);
                        spawn_positions.push(p2);
                    },
                }
            }
        }

        let mut played_beep = false;
        for pos in spawn_positions {
            if pos.x > margin
                && pos.x < self.width - 1 - margin
                && pos.y > margin
                && pos.y < self.height - 1 - margin
            {
                self.lasers.push(Laser {
                    position: pos,
                    direction: dir,
                    player,
                });
                if !played_beep {
                    beep();
                    played_beep = true;
                }
            }
        }
    }
    pub fn handle_input(&mut self, dir: Direction, player: u8) {
        if player == 1 {
            if self.snake.direction_queue.len() >= 2 {
                return;
            }
            let current_dir =
                self.snake.direction_queue.back().copied().unwrap_or(self.snake.direction);
            let is_opposite = matches!(
                (current_dir, dir),
                (Direction::Up, Direction::Down)
                    | (Direction::Down, Direction::Up)
                    | (Direction::Left, Direction::Right)
                    | (Direction::Right, Direction::Left)
            );
            if !is_opposite && dir != current_dir {
                self.snake.direction_queue.push_back(dir);
            }
        } else if player == 2
            && let Some(p2) = &mut self.player2
        {
            if p2.direction_queue.len() >= 2 {
                return;
            }
            let current_dir = p2.direction_queue.back().copied().unwrap_or(p2.direction);
            let is_opposite = matches!(
                (current_dir, dir),
                (Direction::Up, Direction::Down)
                    | (Direction::Down, Direction::Up)
                    | (Direction::Left, Direction::Right)
                    | (Direction::Right, Direction::Left)
            );
            if !is_opposite && dir != current_dir {
                p2.direction_queue.push_back(dir);
            }
        }
    }
    #[must_use]
    pub fn should_bot_shoot(&self, player: u8) -> bool {
        let (head, dir) = if player == 1 {
            (
                self.snake.head(),
                self.snake.direction_queue.back().copied().unwrap_or(self.snake.direction),
            )
        } else if player == 2 {
            if let Some(p2) = &self.player2 {
                (p2.head(), p2.direction_queue.back().copied().unwrap_or(p2.direction))
            } else {
                return false;
            }
        } else {
            return false;
        };
        let margin = if self.mode == GameMode::BattleRoyale {
            self.safe_zone_margin
        } else {
            0
        };
        let mut current_pos = Self::calculate_next_head_dir(head, dir);
        let mut steps = 0;
        while current_pos.x > margin
            && current_pos.x < self.width - 1 - margin
            && current_pos.y > margin
            && current_pos.y < self.height - 1 - margin
        {
            steps += 1;
            if let Some(final_p) = self.get_final_p(current_pos) {
                current_pos = final_p;
            }
            for boss in &self.bosses {
                if boss.position == current_pos {
                    return true;
                }
            }
            if let Some(goblin) = &self.goblin
                && goblin.position == current_pos
            {
                return true;
            }
            if player == 1 && self.snake.body_map.contains_key(&current_pos) {
                return false;
            } else if player == 2
                && let Some(p2) = &self.player2
                && p2.body_map.contains_key(&current_pos)
            {
                return false;
            }
            if player == 1
                && let Some(p2) = &self.player2
                && p2.body_map.contains_key(&current_pos)
            {
                return true;
            } else if player == 2 && self.snake.body_map.contains_key(&current_pos) {
                return true;
            }
            if self.obstacles.contains(&current_pos) {
                return steps <= 5;
            }
            current_pos = Self::calculate_next_head_dir(current_pos, dir);
        }
        false
    }
    fn handle_autopilot_moves(&mut self) {
        let delay_bot = self.weather == Weather::Snow && self.rng.gen_bool(0.2);
        if !delay_bot {
            if (self.auto_pilot || self.mode == GameMode::BotVsBot)
                && self.snake.direction_queue.is_empty()
            {
                if let Some(dir) = self.calculate_autopilot_move() {
                    self.snake.direction_queue.push_back(dir);
                }
                if self.should_bot_shoot(1) {
                    self.shoot_laser(1);
                }
            }
            if self.mode == GameMode::PlayerVsBot || self.mode == GameMode::BotVsBot {
                let is_empty =
                    self.player2.as_ref().is_some_and(|p2| p2.direction_queue.is_empty());
                if is_empty {
                    if let Some(dir) = self.calculate_p2_autopilot_move()
                        && let Some(p2) = &mut self.player2
                    {
                        p2.direction_queue.push_back(dir);
                    }
                    if self.should_bot_shoot(2) {
                        self.shoot_laser(2);
                    }
                }
            }
            if self.mode == GameMode::MassiveMultiplayer
                || self.mode == GameMode::Tron
                || self.mode == GameMode::Zombie
                || self.mode == GameMode::KingOfTheHill
            {
                for i in 0..self.bots.len() {
                    if self.bots[i].direction_queue.is_empty() {
                        let start = self.bots[i].head();
                        let current_dir = self.bots[i].direction;

                        // Use flow field for MassiveMultiplayer and Zombie to save performance
                        if (self.mode == GameMode::MassiveMultiplayer
                            || self.mode == GameMode::Zombie)
                            && let Some(flow_field) = &self.flow_field
                            && let Some(&dir) = flow_field.get(&start)
                        {
                            let is_opp = match (current_dir, dir) {
                                (Direction::Up, Direction::Down)
                                | (Direction::Down, Direction::Up)
                                | (Direction::Left, Direction::Right)
                                | (Direction::Right, Direction::Left) => true,
                                _ => false,
                            };

                            let next_head = Self::calculate_next_head_dir(start, dir);
                            if !is_opp
                                && let Some(final_p) = self.get_final_p(next_head)
                                && self.is_safe_final_p(final_p, 1, 4)
                            {
                                self.bots_autopilot_paths[i].clear();
                                self.bots[i].direction_queue.push_back(dir);
                                continue;
                            }
                        }

                        let mut targets = if self.mode == GameMode::Zombie {
                            vec![self.snake.head()]
                        } else {
                            vec![self.food]
                        };

                        if self.mode != GameMode::Zombie {
                            if let Some((bf_p, _)) = self.bonus_food {
                                targets.push(bf_p);
                            }
                            if let Some(pu) = &self.power_up
                                && pu.activation_time.is_none()
                            {
                                targets.push(pu.location);
                            }
                            if let Some(goblin) = &self.goblin {
                                targets.push(goblin.position);
                            }
                            if self.mode == GameMode::KingOfTheHill
                                && let Some(koth_pos) = self.koth_zone
                            {
                                targets.insert(0, koth_pos);
                            }
                        }
                        if let Some((dir, path)) =
                            self.astar_search(start, current_dir, &targets, 4)
                        {
                            self.bots_autopilot_paths[i] = path;
                            self.bots[i].direction_queue.push_back(dir);
                        } else if let Some(dir) = self.flood_fill_fallback(start, current_dir, 4) {
                            self.bots_autopilot_paths[i].clear();
                            self.bots[i].direction_queue.push_back(dir);
                        }
                    }
                }
            }
        }
    }
    pub(crate) fn calculate_final_heads(&self) -> (Point, Option<Point>, bool, bool) {
        let head1 = self.snake.head();
        let next_head1 = Self::calculate_next_head_dir(head1, self.snake.direction);
        let next_head2_opt = self.player2.as_ref().map(|p2| {
            let head2 = p2.head();
            Self::calculate_next_head_dir(head2, p2.direction)
        });
        let can_pass_through_walls = self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::PassThroughWalls
                && p.activation_time.is_some_and(|t| {
                    web_time::SystemTime::now()
                        .duration_since(web_time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .saturating_sub(t)
                        < self.powerup_duration()
                })
        }) || self.stats.equipped_vehicle
            == Some(crate::game::Vehicle::Spaceship);
        let mut hit_wall1 = false;
        let final_head1 = if self.portals.is_some_and(|(p1, _)| p1 == next_head1) {
            self.portals.unwrap().1
        } else if self.portals.is_some_and(|(_, p2)| p2 == next_head1) {
            self.portals.unwrap().0
        } else if (self.wrap_mode || can_pass_through_walls || self.mode == GameMode::Zen)
            && self.mode != GameMode::BattleRoyale
        {
            self.calculate_wrapped_head(next_head1)
        } else {
            let margin = if self.mode == GameMode::BattleRoyale {
                self.safe_zone_margin
            } else {
                0
            };
            if next_head1.x <= margin
                || next_head1.x >= self.width - 1 - margin
                || next_head1.y <= margin
                || next_head1.y >= self.height - 1 - margin
            {
                hit_wall1 = true;
            }
            next_head1
        };
        let mut hit_wall2 = false;
        let final_head2_opt = next_head2_opt.map(|next_head2| {
            if self.portals.is_some_and(|(p1, _)| p1 == next_head2) {
                self.portals.unwrap().1
            } else if self.portals.is_some_and(|(_, p2)| p2 == next_head2) {
                self.portals.unwrap().0
            } else if (self.wrap_mode || can_pass_through_walls || self.mode == GameMode::Zen)
                && self.mode != GameMode::BattleRoyale
            {
                self.calculate_wrapped_head(next_head2)
            } else {
                let margin = if self.mode == GameMode::BattleRoyale {
                    self.safe_zone_margin
                } else {
                    0
                };
                if next_head2.x <= margin
                    || next_head2.x >= self.width - 1 - margin
                    || next_head2.y <= margin
                    || next_head2.y >= self.height - 1 - margin
                {
                    hit_wall2 = true;
                }
                next_head2
            }
        });
        (final_head1, final_head2_opt, hit_wall1, hit_wall2)
    }
    pub fn rewind_time(&mut self) {
        if let Some(state) = self.history.pop_back() {
            self.snake = state.snake;
            self.player2 = state.player2;
            self.bots = state.bots;
            self.bots_autopilot_paths = state.bots_autopilot_paths;
            self.food = state.food;
            self.obstacles = state.obstacles;
            self.score = state.score;
            self.bonus_food = state.bonus_food;
            self.poison_food = state.poison_food;
            self.power_up = state.power_up;
            self.lives = state.lives;
            self.food_eaten_session = state.food_eaten_session;
            self.campaign_level = state.campaign_level;
            self.safe_zone_margin = state.safe_zone_margin;
            self.last_shrink_time = state.last_shrink_time;
            self.last_obstacle_spawn_time = state.last_obstacle_spawn_time;
            self.combo = state.combo;
            self.last_food_time = state.last_food_time;
            self.lasers = state.lasers;
            self.bosses = state.bosses;
            self.portals = state.portals;
            self.weather = state.weather;
            self.lightning_column = state.lightning_column;
            self.mines = state.mines;
            self.black_hole = state.black_hole;
            self.meteors = state.meteors;
            self.goblin = state.goblin;
            self.xp = state.xp;
            self.player_level = state.player_level;
            self.xp_to_next_level = state.xp_to_next_level;
            self.in_game_upgrades = state.in_game_upgrades;
            self.level_up_options = state.level_up_options;
            self.level_up_selection = state.level_up_selection;
            self.companion = state.companion;
            self.equipment_boxes = state.equipment_boxes;
            self.last_real_estate_tick = Some(Instant::now());
            self.fishing_timer = state.fishing_timer;
            self.fishing_progress = state.fishing_progress;
            self.is_fishing = state.is_fishing;
            self.eggs_on_board = state.eggs_on_board;
            self.mana = state.mana;
            self.max_mana = state.max_mana;
            self.time_of_day = state.time_of_day;
            self.tick_counter = state.tick_counter;
            self.p1_flag = state.p1_flag;
            self.p2_flag = state.p2_flag;
            self.p1_has_flag = state.p1_has_flag;
            self.p2_has_flag = state.p2_has_flag;
            self.p1_score = state.p1_score;
            self.p2_score = state.p2_score;
            self.koth_zone = state.koth_zone;
            self.xp_gems = state.xp_gems;
        }
    }
    pub fn gain_xp(&mut self, amount: u32) {
        self.stats.battle_pass_xp += amount;
        self.xp += amount;
        if self.xp >= self.xp_to_next_level {
            self.xp -= self.xp_to_next_level;
            self.player_level += 1;

            self.xp_to_next_level = (self.xp_to_next_level * 3) / 2;

            self.generate_level_up_options();
            self.level_up_selection = 0;
            self.state = GameState::LevelUp;
        }
    }

    fn generate_level_up_options(&mut self) {
        let all_options = vec![
            InGameUpgrade::Multishot,
            InGameUpgrade::Piercing,
            InGameUpgrade::ExplosiveLasers,
            InGameUpgrade::LaserSpeed,
            InGameUpgrade::HomingLasers,
            InGameUpgrade::DoubleCoins,
            InGameUpgrade::Magnet,
            InGameUpgrade::Turret,
        ];

        self.level_up_options.clear();
        let mut available = all_options;
        for _ in 0..3 {
            if available.is_empty() {
                break;
            }
            let idx = self.rng.gen_range(0..available.len());
            self.level_up_options.push(available.remove(idx));
        }
    }

    pub fn save_history_state(&mut self) {
        let state = HistoryState {
            snake: self.snake.clone(),
            player2: self.player2.clone(),
            bots: self.bots.clone(),
            bots_autopilot_paths: self.bots_autopilot_paths.clone(),
            food: self.food,
            obstacles: self.obstacles.clone(),
            score: self.score,
            bonus_food: self.bonus_food,
            poison_food: self.poison_food,
            power_up: self.power_up.clone(),
            lives: self.lives,
            food_eaten_session: self.food_eaten_session,
            campaign_level: self.campaign_level,
            safe_zone_margin: self.safe_zone_margin,
            last_shrink_time: self.last_shrink_time,
            last_obstacle_spawn_time: self.last_obstacle_spawn_time,
            combo: self.combo,
            last_food_time: self.last_food_time,
            lasers: self.lasers.clone(),
            bosses: self.bosses.clone(),
            portals: self.portals,
            weather: self.weather,
            lightning_column: self.lightning_column,
            mines: self.mines.clone(),
            black_hole: self.black_hole,
            meteors: self.meteors.clone(),
            goblin: self.goblin,
            xp: self.xp,
            player_level: self.player_level,
            xp_to_next_level: self.xp_to_next_level,
            in_game_upgrades: self.in_game_upgrades.clone(),
            level_up_options: self.level_up_options.clone(),
            level_up_selection: self.level_up_selection,
            companion: self.companion.clone(),
            equipment_boxes: self.equipment_boxes.clone(),
            fishing_timer: self.fishing_timer,
            fishing_progress: self.fishing_progress,
            is_fishing: self.is_fishing,
            eggs_on_board: self.eggs_on_board.clone(),
            mana: self.mana,
            max_mana: self.max_mana,
            time_of_day: self.time_of_day,
            tick_counter: self.tick_counter,
            p1_flag: self.p1_flag,
            p2_flag: self.p2_flag,
            p1_has_flag: self.p1_has_flag,
            p2_has_flag: self.p2_has_flag,
            p1_score: self.p1_score,
            p2_score: self.p2_score,
            koth_zone: self.koth_zone,
            xp_gems: self.xp_gems.clone(),
        };
        self.history.push_back(state);
        if self.history.len() > 50 {
            self.history.pop_front();
        }
    }
    pub fn spawn_floating_text(
        &mut self,
        x: f32,
        y: f32,
        text: String,
        color: crate::color::Color,
    ) {
        self.floating_texts.push(FloatingText {
            x,
            y,
            text,
            color,
            lifetime: 20.0,
            max_lifetime: 20.0,
        });
    }

    pub fn spawn_particles(
        &mut self,
        x: f32,
        y: f32,
        count: usize,
        color: crate::color::Color,
        symbol: char,
    ) {
        for _ in 0..count {
            let angle = self.rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = self.rng.gen_range(0.2..1.5);
            let lifetime = self.rng.gen_range(5.0..15.0);
            self.particles.push(Particle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                lifetime,
                max_lifetime: lifetime,
                symbol,
                color,
            });
        }
    }
    pub fn apply_gravity(&mut self) {
        if self.mode != GameMode::Gravity && self.mode != GameMode::Chaos {
            return;
        }
        if self.rng.gen_bool(0.2) {
            let next_p = Self::calculate_next_head_dir(self.snake.head(), Direction::Down);
            let margin = 0;
            if next_p.x > margin
                && next_p.x < self.width - 1 - margin
                && next_p.y > margin
                && next_p.y < self.height - 1 - margin
                && !self.obstacles.contains(&next_p)
                && !self.snake.body_map.contains_key(&next_p)
                && self.snake.direction != Direction::Up
            {
                self.snake.direction_queue.push_front(Direction::Down);
            }
        }
    }
    pub fn apply_magnet(&mut self) {
        let has_magnet_powerup = self.power_up.as_ref().is_some_and(|pu| {
            pu.p_type == PowerUpType::Magnet
                && pu.activation_time.is_some_and(|t| {
                    web_time::SystemTime::now()
                        .duration_since(web_time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .saturating_sub(t)
                        < self.powerup_duration()
                })
        });

        let has_passive_magnet = self.in_game_upgrades.contains_key(&InGameUpgrade::Magnet);
        let has_ring_magnet = self.stats.equipped_gear == Some(crate::game::Equipment::MagnetRing);

        if (has_magnet_powerup || has_passive_magnet || has_ring_magnet) && self.rng.gen_bool(0.25)
        {
            let head = self.snake.head();

            // Move food
            let mut best_dist = u16::MAX;
            let mut best_pos = None;
            let current_dist =
                self.food.x.abs_diff(head.x).saturating_add(self.food.y.abs_diff(head.y));
            let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
            for &d in &dirs {
                let next_p = Self::calculate_next_head_dir(self.food, d);
                let margin = if self.mode == GameMode::BattleRoyale {
                    self.safe_zone_margin
                } else {
                    0
                };
                if next_p.x <= margin
                    || next_p.x >= self.width - 1 - margin
                    || next_p.y <= margin
                    || next_p.y >= self.height - 1 - margin
                {
                    continue;
                }
                if self.obstacles.contains(&next_p) || self.snake.body_map.contains_key(&next_p) {
                    continue;
                }
                if let Some(p2) = &self.player2
                    && p2.body_map.contains_key(&next_p)
                {
                    continue;
                }
                let dist = next_p.x.abs_diff(head.x).saturating_add(next_p.y.abs_diff(head.y));
                if dist < current_dist && dist < best_dist {
                    best_dist = dist;
                    best_pos = Some(next_p);
                }
            }
            if let Some(new_food_pos) = best_pos {
                self.food = new_food_pos;
            }

            // Move xp gems
            let mut new_gems = HashSet::new();
            for gem in &self.xp_gems {
                let mut best_gem_dist = u16::MAX;
                let mut best_gem_pos = *gem;
                let current_gem_dist =
                    gem.x.abs_diff(head.x).saturating_add(gem.y.abs_diff(head.y));

                // Only attract gems if they are somewhat close (e.g. within 15 units)
                if current_gem_dist < 15 {
                    for &d in &dirs {
                        let next_p = Self::calculate_next_head_dir(*gem, d);
                        let margin = if self.mode == GameMode::BattleRoyale {
                            self.safe_zone_margin
                        } else {
                            0
                        };
                        if next_p.x <= margin
                            || next_p.x >= self.width - 1 - margin
                            || next_p.y <= margin
                            || next_p.y >= self.height - 1 - margin
                        {
                            continue;
                        }
                        if self.obstacles.contains(&next_p)
                            || self.snake.body_map.contains_key(&next_p)
                        {
                            continue;
                        }
                        if let Some(p2) = &self.player2
                            && p2.body_map.contains_key(&next_p)
                        {
                            continue;
                        }
                        let dist =
                            next_p.x.abs_diff(head.x).saturating_add(next_p.y.abs_diff(head.y));
                        if dist < current_gem_dist && dist < best_gem_dist {
                            best_gem_dist = dist;
                            best_gem_pos = next_p;
                        }
                    }
                }
                new_gems.insert(best_gem_pos);
            }
            self.xp_gems = new_gems;
        }
    }
    fn manage_crops(&mut self) {
        if self.mode == GameMode::Farmstead {
            for crop in &mut self.crops {
                crop.timer += 1;
                if crop.growth_stage == 0 && crop.timer > 30 {
                    crop.growth_stage = 1;
                    crop.timer = 0;
                } else if crop.growth_stage == 1 && crop.timer > 50 {
                    crop.growth_stage = 2;
                    crop.timer = 0;
                }
            }
        }
    }

    #[expect(clippy::too_many_lines, reason = "manage_companion naturally requires many lines")]
    fn manage_companion(&mut self) {
        let mut spawn_lasers = Vec::new();
        let margin = if self.mode == GameMode::BattleRoyale {
            self.safe_zone_margin
        } else {
            0
        };

        if let Some(mut comp) = self.companion.take() {
            comp.move_timer += 1;
            comp.action_timer += 1;

            if comp.move_timer >= 2 {
                comp.move_timer = 0;
                let target = match comp.kind {
                    CompanionType::Collector => {
                        let mut targets = vec![self.food];
                        if let Some((bp, _)) = self.bonus_food {
                            targets.push(bp);
                        }
                        for p in self.resources.keys() {
                            targets.push(*p);
                        }
                        // Find closest
                        let mut best_target = self.food;
                        let mut min_dist = u16::MAX;
                        for t in targets {
                            let dist = comp
                                .position
                                .x
                                .abs_diff(t.x)
                                .saturating_add(comp.position.y.abs_diff(t.y));
                            if dist < min_dist {
                                min_dist = dist;
                                best_target = t;
                            }
                        }
                        best_target
                    },
                    CompanionType::Fighter | CompanionType::Healer | CompanionType::Sniper => {
                        self.snake.head()
                    },
                };

                // We use astar_pathfind directly for companions so they don't get restricted by 'neck' turns
                if let Some(dir) = self.bot_smart_pathfind(comp.position, target, 3) {
                    let next_pos = Self::calculate_next_head_dir(comp.position, dir);
                    if next_pos.x > margin
                        && next_pos.x < self.width - 1 - margin
                        && next_pos.y > margin
                        && next_pos.y < self.height - 1 - margin
                        && !self.obstacles.contains(&next_pos)
                    {
                        comp.position = next_pos;
                    }
                }
            }

            match comp.kind {
                CompanionType::Collector => {
                    if comp.position == self.food {
                        self.process_food_collision(comp.position, false);
                    }
                    if let Some((bp, _)) = self.bonus_food
                        && comp.position == bp
                    {
                        self.check_bonus_food_collision(comp.position, false);
                    }
                    if self.resources.contains_key(&comp.position) {
                        self.process_resource_collision(comp.position);
                    }
                },
                CompanionType::Fighter => {
                    if comp.action_timer >= 15 {
                        comp.action_timer = 0;
                        if let Some(boss) = self.bosses.first() {
                            let dx = i32::from(boss.position.x) - i32::from(comp.position.x);
                            let dy = i32::from(boss.position.y) - i32::from(comp.position.y);
                            let dir = if dx.abs() > dy.abs() {
                                if dx > 0 {
                                    Direction::Right
                                } else {
                                    Direction::Left
                                }
                            } else if dy > 0 {
                                Direction::Down
                            } else {
                                Direction::Up
                            };
                            let laser_pos = Self::calculate_next_head_dir(comp.position, dir);
                            if laser_pos.x > margin
                                && laser_pos.x < self.width - 1 - margin
                                && laser_pos.y > margin
                                && laser_pos.y < self.height - 1 - margin
                            {
                                spawn_lasers.push(Laser {
                                    position: laser_pos,
                                    direction: dir,
                                    player: 1, // acts as player 1 laser
                                });
                            }
                        }
                    }
                },
                CompanionType::Healer => {
                    if comp.action_timer >= 100 {
                        comp.action_timer = 0;
                        if self.lives < 3 {
                            self.lives += 1;
                            self.spawn_particles(
                                f32::from(comp.position.x),
                                f32::from(comp.position.y),
                                20,
                                crate::color::Color::Magenta,
                                '♥',
                            );
                        }
                    }
                },
                CompanionType::Sniper => {
                    if comp.action_timer >= 30 {
                        comp.action_timer = 0;
                        if let Some(boss) = self.bosses.first() {
                            let dx = i32::from(boss.position.x) - i32::from(comp.position.x);
                            let dy = i32::from(boss.position.y) - i32::from(comp.position.y);
                            if dx == 0 || dy == 0 {
                                let dir = if dx > 0 {
                                    Direction::Right
                                } else if dx < 0 {
                                    Direction::Left
                                } else if dy > 0 {
                                    Direction::Down
                                } else {
                                    Direction::Up
                                };
                                let laser_pos = Self::calculate_next_head_dir(comp.position, dir);
                                if laser_pos.x > margin
                                    && laser_pos.x < self.width - 1 - margin
                                    && laser_pos.y > margin
                                    && laser_pos.y < self.height - 1 - margin
                                {
                                    spawn_lasers.push(Laser {
                                        position: laser_pos,
                                        direction: dir,
                                        player: 1,
                                    });
                                }
                            }
                        }
                    }
                },
            }

            self.companion = Some(comp);
        }

        self.lasers.extend(spawn_lasers);
    }

    pub fn update(&mut self) {
        if let Some(last_tick) = self.last_bank_tick {
            if last_tick.elapsed() >= web_time::Duration::from_secs(10) {
                // Add 5% interest
                let interest = (self.stats.bank_balance / 20).max(1);
                if self.stats.bank_balance > 0 {
                    self.stats.bank_balance = self.stats.bank_balance.saturating_add(interest);
                }
                self.last_bank_tick = Some(web_time::Instant::now());
            }
        } else {
            self.last_bank_tick = Some(web_time::Instant::now());
        }

        if let Some(last_tick) = self.last_real_estate_tick {
            if last_tick.elapsed() >= web_time::Duration::from_secs(1) {
                let mut total_income = 0;
                for (prop, count) in &self.stats.properties {
                    total_income += prop.income_per_second() * count;
                }
                self.stats.coins += total_income;
                self.last_real_estate_tick = Some(web_time::Instant::now());
            }
        } else {
            self.last_real_estate_tick = Some(web_time::Instant::now());
        }
        if self.state != GameState::Playing {
            return;
        }
        self.update_tick();
        self.update_bounty_progress(crate::game::BountyType::SurviveTime(0), 1);
        if self.is_sprinting && self.state == GameState::Playing {
            self.update_tick();
        }
        self.update_quest_progress(crate::game::QuestType::ReachScore, self.score);

        if self.mana < self.max_mana && self.rng.gen_bool(0.1) {
            self.mana += 1;
        }

        self.update_stock_market();

        if self.mode == GameMode::DungeonCrawler && self.is_door(self.snake.head()) {
            let head = self.snake.head();
            let mut new_head = head;

            if head.y == 0 {
                // North door
                self.current_room_coords.1 -= 1;
                new_head.y = self.height - 2;
            } else if head.y == self.height - 1 {
                // South door
                self.current_room_coords.1 += 1;
                new_head.y = 1;
            } else if head.x == 0 {
                // West door
                self.current_room_coords.0 -= 1;
                new_head.x = self.width - 2;
            } else if head.x == self.width - 1 {
                // East door
                self.current_room_coords.0 += 1;
                new_head.x = 1;
            }

            self.load_dungeon_room();
            self.snake.move_to(new_head, false);
            // Coil up
            self.snake.body.truncate(1);
            self.snake.rebuild_map();
        }

        if self.mode == GameMode::DungeonCrawler
            && let Some(room) = self.dungeon_grid.get_mut(&self.current_room_coords)
            && !room.cleared
            && self.bosses.is_empty()
        {
            room.cleared = true;
            crate::game::beep(); // Play sound

            // Remove door blockers
            let to_remove: Vec<Point> = self
                .obstacles
                .iter()
                .copied()
                .filter(|p| p.x == 0 || p.x == self.width - 1 || p.y == 0 || p.y == self.height - 1)
                .collect();

            for p in to_remove {
                self.obstacles.remove(&p);
            }
        }

        if self.mode == GameMode::MassiveMultiplayer || self.mode == GameMode::Zombie {
            let targets = if self.mode == GameMode::Zombie {
                vec![self.snake.head()]
            } else {
                let mut t = vec![self.food];
                if let Some((bp, _)) = self.bonus_food {
                    t.push(bp);
                }
                if let Some(pu) = &self.power_up
                    && pu.activation_time.is_none()
                {
                    t.push(pu.location);
                }
                if let Some(goblin) = &self.goblin {
                    t.push(goblin.position);
                }
                if self.mode == GameMode::KingOfTheHill
                    && let Some(koth_pos) = self.koth_zone
                {
                    t.push(koth_pos);
                }
                t
            };

            let mut needs_update = self.flow_field.is_none() || self.flow_field_targets != targets;
            if self.mode == GameMode::Zombie
                && !self.tick_counter.is_multiple_of(5)
                && self.flow_field.is_some()
            {
                needs_update = false; // throttle update for zombie mode
            }
            if needs_update {
                self.flow_field = Some(crate::game::generate_flow_field(self, &targets));
                self.flow_field_targets = targets;
            }
        }
    }

    fn handle_survivor_auto_fire(&mut self) {
        if self.mode != GameMode::SnakeSurvivor {
            return;
        }
        let laser_speed_level =
            self.in_game_upgrades.get(&InGameUpgrade::LaserSpeed).copied().unwrap_or(0);
        let fire_rate = 15u32.saturating_sub(laser_speed_level * 2).max(5);

        if self.tick_counter.is_multiple_of(fire_rate) {
            let mut nearest_dist = u32::MAX;
            let mut target_dir = None;
            let head = self.snake.head();

            // Find nearest enemy
            for boss in &self.bosses {
                let dist = u32::from(head.x.abs_diff(boss.position.x))
                    + u32::from(head.y.abs_diff(boss.position.y));
                if dist < nearest_dist {
                    nearest_dist = dist;
                    if boss.position.x == head.x && boss.position.y < head.y {
                        target_dir = Some(Direction::Up);
                    } else if boss.position.x == head.x && boss.position.y > head.y {
                        target_dir = Some(Direction::Down);
                    } else if boss.position.y == head.y && boss.position.x < head.x {
                        target_dir = Some(Direction::Left);
                    } else if boss.position.y == head.y && boss.position.x > head.x {
                        target_dir = Some(Direction::Right);
                    } else {
                        // Diagonal approximation: shoot along the longer axis distance
                        let dx = i32::from(boss.position.x) - i32::from(head.x);
                        let dy = i32::from(boss.position.y) - i32::from(head.y);
                        if dx.abs() > dy.abs() {
                            target_dir = Some(if dx > 0 {
                                Direction::Right
                            } else {
                                Direction::Left
                            });
                        } else {
                            target_dir = Some(if dy > 0 {
                                Direction::Down
                            } else {
                                Direction::Up
                            });
                        }
                    }
                }
            }
            if let Some(goblin) = &self.goblin {
                let dist = u32::from(head.x.abs_diff(goblin.position.x))
                    + u32::from(head.y.abs_diff(goblin.position.y));
                if dist < nearest_dist {
                    let dx = i32::from(goblin.position.x) - i32::from(head.x);
                    let dy = i32::from(goblin.position.y) - i32::from(head.y);
                    if dx.abs() > dy.abs() {
                        target_dir = Some(if dx > 0 {
                            Direction::Right
                        } else {
                            Direction::Left
                        });
                    } else {
                        target_dir = Some(if dy > 0 {
                            Direction::Down
                        } else {
                            Direction::Up
                        });
                    }
                }
            }

            if let Some(dir) = target_dir {
                // Ensure we don't shoot more lasers than we're allowed
                let active_lasers = self.lasers.iter().filter(|l| l.player == 1).count();
                let mut max_lasers = 3 + usize::from(self.stats.upgrade_laser_capacity);
                if self.skin == '👾' {
                    max_lasers += 5;
                }

                // Allow at least 1 laser to always fire in survivor mode if capacity is reached
                if active_lasers < max_lasers || active_lasers < 10 {
                    let laser_pos = Self::calculate_next_head_dir(head, dir);
                    let margin = if self.mode == GameMode::BattleRoyale {
                        self.safe_zone_margin
                    } else {
                        0
                    };

                    if laser_pos.x > margin
                        && laser_pos.x < self.width - 1 - margin
                        && laser_pos.y > margin
                        && laser_pos.y < self.height - 1 - margin
                    {
                        self.lasers.push(Laser {
                            position: laser_pos,
                            direction: dir,
                            player: 1,
                        });
                    }
                }
            }
        }
    }

    fn update_stock_market(&mut self) {
        if self.rng.gen_bool(0.01) {
            let stocks = [
                crate::game::Stock::SnakeCorp,
                crate::game::Stock::GoblinInc,
                crate::game::Stock::BossDynamics,
                crate::game::Stock::LaserTech,
            ];
            let stock = stocks[self.rng.gen_range(0..stocks.len())];
            let current_price = self.stats.stock_prices.get(&stock).copied().unwrap_or(100);

            let volatility = self.rng.gen_range(-10..=10);
            let mut new_price = i32::try_from(current_price).unwrap_or(100) + volatility;

            // Random market events
            let event = self.rng.gen_range(0..100);
            if event < 5 {
                // Crash
                new_price /= 2;
            } else if event < 10 {
                // Moon
                new_price *= 2;
            }

            let new_price = u32::try_from(new_price.clamp(5, 2000)).unwrap_or(5);
            self.stats.stock_prices.insert(stock, new_price);
        }
    }

    #[must_use]
    pub fn is_door(&self, p: Point) -> bool {
        if self.mode != GameMode::DungeonCrawler || self.dungeon_grid.is_empty() {
            return false;
        }
        if let Some(room) = self.dungeon_grid.get(&self.current_room_coords) {
            let is_north = p.x == self.width / 2 && p.y == 0 && room.north_door;
            let is_south = p.x == self.width / 2 && p.y == self.height - 1 && room.south_door;
            let is_west = p.x == 0 && p.y == self.height / 2 && room.west_door;
            let is_east = p.x == self.width - 1 && p.y == self.height / 2 && room.east_door;

            return is_north || is_south || is_west || is_east;
        }
        false
    }

    pub fn load_dungeon_room(&mut self) {
        self.obstacles.clear();
        self.bosses.clear();
        self.resources.clear();
        self.equipment_boxes.clear();

        let room = self.dungeon_grid.get(&self.current_room_coords).unwrap().clone();

        // Generate walls with holes for doors
        for y in 0..self.height {
            for x in 0..self.width {
                if x == 0 || x == self.width - 1 || y == 0 || y == self.height - 1 {
                    let is_door = (x == self.width / 2 && y == 0 && room.north_door)
                        || (x == self.width / 2 && y == self.height - 1 && room.south_door)
                        || (x == 0 && y == self.height / 2 && room.west_door)
                        || (x == self.width - 1 && y == self.height / 2 && room.east_door);

                    let is_door_padding = (y == 0
                        && room.north_door
                        && (x == self.width / 2 - 1 || x == self.width / 2 + 1))
                        || (y == self.height - 1
                            && room.south_door
                            && (x == self.width / 2 - 1 || x == self.width / 2 + 1))
                        || (x == 0
                            && room.west_door
                            && (y == self.height / 2 - 1 || y == self.height / 2 + 1))
                        || (x == self.width - 1
                            && room.east_door
                            && (y == self.height / 2 - 1 || y == self.height / 2 + 1));

                    if !is_door && !is_door_padding {
                        self.obstacles.insert(Point {
                            x,
                            y,
                        });
                    }
                }
            }
        }

        // Add inner obstacles and entities based on room type and if it's cleared
        if !room.cleared {
            let center = Point {
                x: self.width / 2,
                y: self.height / 2,
            };
            match room.r_type {
                crate::game::dungeon::DungeonRoomType::Normal => {
                    use rand::Rng;
                    for _ in 0..self.rng.gen_range(1..=3) {
                        if let Some(pos) = Self::get_random_empty_point(
                            self.width,
                            self.height,
                            &self.snake,
                            |p| self.obstacles.contains(p) || self.snake.body_map.contains_key(p),
                            &mut self.rng,
                            2,
                        ) {
                            self.bosses.push(Boss {
                                position: pos,
                                health: 5,
                                max_health: 5,
                                move_timer: 0,
                                shoot_timer: 0,
                                kind: crate::game::BossType::Shooter,
                                state_timer: 0,
                            });
                        }
                    }
                },
                crate::game::dungeon::DungeonRoomType::Boss => {
                    self.bosses.push(Boss {
                        position: center,
                        health: 30,
                        max_health: 30,
                        move_timer: 0,
                        shoot_timer: 0,
                        kind: crate::game::BossType::Juggernaut,
                        state_timer: 0,
                    });
                },
                crate::game::dungeon::DungeonRoomType::Treasure => {
                    self.equipment_boxes.push(center);
                },
                _ => {},
            }
        }

        // Close doors if there are enemies
        if !self.bosses.is_empty() {
            for y in 0..self.height {
                for x in 0..self.width {
                    if x == 0 || x == self.width - 1 || y == 0 || y == self.height - 1 {
                        self.obstacles.insert(Point {
                            x,
                            y,
                        });
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn get_boss_path(
        &self,
        start: Point,
        target: Point,
        boss_kind: BossType,
    ) -> Option<Direction> {
        let mut open_set = std::collections::BinaryHeap::new();
        let mut g_score = std::collections::HashMap::new();
        let mut came_from = std::collections::HashMap::new();
        let mut first_step = std::collections::HashMap::new();
        let mut tie_breaker_counter = 0u64;

        g_score.insert(start, 0u16);

        let calc_dist = |p1: Point, p2: Point| -> u16 {
            let mut dx = p1.x.abs_diff(p2.x);
            let mut dy = p1.y.abs_diff(p2.y);
            if (self.wrap_mode || self.mode == GameMode::Zen) && self.mode != GameMode::BattleRoyale
            {
                dx = std::cmp::min(dx, self.width.saturating_sub(2).saturating_sub(dx));
                dy = std::cmp::min(dy, self.height.saturating_sub(2).saturating_sub(dy));
            }
            dx.saturating_add(dy)
        };

        let heuristic = |p: Point| -> u16 {
            let mut penalty = 0u16;
            for l in &self.lasers {
                let d = calc_dist(p, l.position);
                if d < 4 {
                    penalty = penalty.saturating_add((4 - d) * 5);
                }
            }
            for m in &self.mines {
                let d = calc_dist(p, *m);
                if d < 4 {
                    penalty = penalty.saturating_add((4 - d) * 10);
                }
            }
            for t in &self.turrets {
                let d = calc_dist(p, t.position);
                if d < 4 {
                    penalty = penalty.saturating_add((4 - d) * 10);
                }
            }
            if let Some(bh) = self.black_hole {
                let d = calc_dist(p, bh);
                if d < 5 {
                    penalty = penalty.saturating_add((5 - d) * 10);
                }
            }
            if let Some(col) = self.lightning_column {
                let dx = p.x.abs_diff(col);
                if dx < 3 {
                    penalty = penalty.saturating_add((3 - dx) * 50);
                }
            }
            for m in &self.meteors {
                let dx = p.x.abs_diff(m.position.x);
                if dx < 2 && p.y >= m.position.y {
                    let dy = p.y.abs_diff(m.position.y);
                    if dy < 10 {
                        penalty = penalty.saturating_add((10 - dy) * 5);
                    }
                }
            }
            if let Some((pf_p, _)) = self.poison_food {
                let d = p.x.abs_diff(pf_p.x) + p.y.abs_diff(pf_p.y);
                if d < 4 {
                    penalty = penalty.saturating_add((4 - d) * 10);
                }
            }
            for boss in &self.bosses {
                let d = calc_dist(p, boss.position);
                if d < 5 {
                    penalty = penalty.saturating_add((5 - d) * 15);
                }
            }
            if self.snake.body_map.contains_key(&p) {
                penalty = penalty.saturating_add(100);
            }
            if self.player2.as_ref().is_some_and(|snake2| snake2.body_map.contains_key(&p)) {
                penalty = penalty.saturating_add(100);
            }
            for bot in &self.bots {
                if bot.body_map.contains_key(&p) {
                    penalty = penalty.saturating_add(100);
                }
            }

            let dist_direct = calc_dist(p, target);
            let base_dist = if let Some((portal1, portal2)) = self.portals {
                let dist_via_portal1 =
                    calc_dist(p, portal1).saturating_add(calc_dist(portal2, target));
                let dist_via_portal2 =
                    calc_dist(p, portal2).saturating_add(calc_dist(portal1, target));
                std::cmp::min(dist_direct, std::cmp::min(dist_via_portal1, dist_via_portal2))
            } else {
                dist_direct
            };
            base_dist.saturating_add(penalty)
        };

        tie_breaker_counter += 1;
        open_set.push(AStarState {
            f_score: heuristic(start),
            tie_breaker: tie_breaker_counter,
            position: start,
        });

        let mut iterations = 0;
        while let Some(AStarState {
            position: current,
            ..
        }) = open_set.pop()
        {
            iterations += 1;
            if iterations > 1000 {
                break;
            }
            if current == target {
                return first_step.get(&current).copied();
            }

            let current_g = *g_score.get(&current).unwrap_or(&u16::MAX);

            let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
            for &d in &dirs {
                let next_p = Self::calculate_next_head_dir(current, d);

                let margin = if self.mode == GameMode::BattleRoyale {
                    self.safe_zone_margin
                } else {
                    0
                };

                // When moving we need to get the final point (which resolves portals)
                let mut final_p = self.get_final_p(next_p);

                if final_p.is_none()
                    && (self.wrap_mode || self.mode == GameMode::Zen)
                    && self.mode != GameMode::BattleRoyale
                {
                    final_p = Some(self.calculate_wrapped_head(next_p));
                }

                if let Some(final_p) = final_p
                    && final_p.x >= margin
                    && final_p.x <= self.width.saturating_sub(1).saturating_sub(margin)
                    && final_p.y >= margin
                    && final_p.y <= self.height.saturating_sub(1).saturating_sub(margin)
                {
                    let mut can_move = true;
                    if final_p != target && self.snake.body_map.contains_key(&final_p) {
                        can_move = false;
                    } else if self.obstacles.contains(&final_p)
                        && boss_kind != BossType::Charger
                        && boss_kind != BossType::Juggernaut
                        && boss_kind != BossType::Phantom
                    {
                        can_move = false;
                    } else if self.mines.contains(&final_p) {
                        can_move = false;
                    } else if self.lasers.iter().any(|l| l.position == final_p) {
                        can_move = false;
                    } else if self.lightning_column == Some(final_p.x) {
                        can_move = false;
                    } else if self.meteors.iter().any(|m| m.position == final_p) {
                        can_move = false;
                    } else if self.black_hole == Some(final_p) {
                        can_move = false;
                    } else if self.poison_food.is_some_and(|(pf, _)| pf == final_p) {
                        can_move = false;
                    }

                    if can_move {
                        if final_p == target {
                            return first_step.get(&current).copied().or(Some(d));
                        }
                        let tentative_g = current_g.saturating_add(1);
                        if tentative_g < *g_score.get(&final_p).unwrap_or(&u16::MAX) {
                            came_from.insert(final_p, current);
                            g_score.insert(final_p, tentative_g);

                            if current == start {
                                first_step.insert(final_p, d);
                            } else if let Some(&f_step) = first_step.get(&current) {
                                first_step.insert(final_p, f_step);
                            }

                            tie_breaker_counter += 1;
                            open_set.push(AStarState {
                                f_score: tentative_g.saturating_add(heuristic(final_p)),
                                tie_breaker: tie_breaker_counter,
                                position: final_p,
                            });
                        }
                    }
                }
            }
        }
        None
    }

    #[expect(
        clippy::too_many_lines,
        reason = "Game loop inherently requires handling multiple states and events"
    )]
    fn update_tick(&mut self) {
        self.tick_counter = self.tick_counter.saturating_add(1);
        if self.tick_counter >= 250 {
            self.tick_counter = 0;
            self.time_of_day = match self.time_of_day {
                crate::game::TimeOfDay::Day => crate::game::TimeOfDay::Night,
                crate::game::TimeOfDay::Night => crate::game::TimeOfDay::Day,
            };
        }

        if self.mode == GameMode::Chaos {
            if self.tick_counter.is_multiple_of(100) {
                let weather_types =
                    [Weather::Clear, Weather::Rain, Weather::Snow, Weather::Sandstorm];
                self.weather = weather_types[self.rng.gen_range(0..weather_types.len())];
            }
            if self.tick_counter.is_multiple_of(500) {
                let mut p = self.snake.head();
                p.x = p.x.saturating_add(5);
                self.bosses.push(Boss {
                    position: p,
                    health: 10,
                    max_health: 10,
                    move_timer: 0,
                    shoot_timer: 0,
                    kind: BossType::Shooter,
                    state_timer: 0,
                });
            }
        }

        self.save_history_state();

        if self.stats.equipped_class == Some(crate::game::HeroClass::Paladin) {
            self.paladin_life_timer += 1;
            if self.paladin_life_timer >= 200 {
                self.paladin_life_timer = 0;
                self.lives += 1;
                self.chat_log.push_back((
                    "SYSTEM: Paladin generated an extra life!".to_string(),
                    crate::color::Color::Yellow,
                ));
                crate::game::beep();
            }
        }

        if self.mode == GameMode::Vampire {
            if let Some(last_food) = self.last_food_time {
                if last_food.elapsed() >= web_time::Duration::from_secs(15) {
                    self.handle_death("Vampire starvation!");
                    if self.state == GameState::Playing {
                        self.last_food_time = Some(web_time::Instant::now());
                    }
                    return;
                }
            } else {
                self.last_food_time = Some(web_time::Instant::now());
            }
        }
        let is_time_frozen = self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::TimeFreeze
                && p.activation_time.is_some_and(|t| {
                    web_time::SystemTime::now()
                        .duration_since(web_time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .saturating_sub(t)
                        < self.powerup_duration()
                })
        });
        if let Some((_, activation_time)) = self.decoy {
            let elapsed = web_time::SystemTime::now()
                .duration_since(web_time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .saturating_sub(activation_time);
            if elapsed >= self.powerup_duration() {
                self.decoy = None;
            }
        }
        let max_bosses = if self.mode == GameMode::BossRush {
            1 + (self.campaign_level / 3)
        } else if self.mode == GameMode::SnakeSurvivor {
            3 + (self.player_level / 2) // More bosses spawn as you level up
        } else {
            1
        };
        let should_spawn_boss = if self.mode == GameMode::BossRush {
            self.bosses.len() < usize::try_from(max_bosses).unwrap_or(1)
        } else if self.mode == GameMode::SnakeSurvivor {
            self.bosses.len() < usize::try_from(max_bosses).unwrap_or(1) && self.rng.gen_bool(0.02)
        } else {
            (self.mode == GameMode::SinglePlayer
                || self.mode == GameMode::DailyChallenge
                || self.mode == GameMode::WeeklyChallenge
                || self.mode == GameMode::MonthlyChallenge
                || self.mode == GameMode::YearlyChallenge
                || self.mode == GameMode::DecadeChallenge
                || self.mode == GameMode::CenturyChallenge
                || self.mode == GameMode::MillenniumChallenge
                || self.mode == GameMode::EonChallenge)
                && self.bosses.is_empty()
                && self.rng.gen_bool(0.005)
        };
        if should_spawn_boss {
            let margin = self.safe_zone_margin;
            let avoid =
                |p: &Point| self.obstacles.contains(p) || self.snake.body_map.contains_key(p);
            if let Some(pos) = Self::get_random_empty_point(
                self.width,
                self.height,
                &self.snake,
                avoid,
                &mut self.rng,
                margin,
            ) {
                let boss_health = if self.mode == GameMode::BossRush {
                    10 + self.campaign_level * 5
                } else if self.mode == GameMode::Campaign {
                    10 + self.campaign_level * 2
                } else {
                    10 + (self.bosses.len() as u32) * 2
                };
                let kind = if self.mode == GameMode::SnakeSurvivor {
                    match self.rng.gen_range(0..3) {
                        0 => BossType::Charger, // Mostly chargers/zombies
                        1 => BossType::Charger,
                        _ => BossType::Shooter,
                    }
                } else {
                    match self.rng.gen_range(0..16) {
                        0 => BossType::Shooter,
                        1 => BossType::Charger,
                        2 => BossType::Spawner,
                        3 => BossType::Teleporter,
                        4 => BossType::Splitter,
                        5 => BossType::Necromancer,
                        6 => BossType::Trapper,
                        7 => BossType::Puffer,
                        8 => BossType::Juggernaut,
                        9 => BossType::Dragon,
                        10 => BossType::Mage,
                        11 => BossType::Gorgon,
                        12 => BossType::VampireLord,
                        13 => BossType::Kraken,
                        14 => BossType::Phantom,
                        15 => BossType::Alchemist,
                        _ => BossType::Mimic,
                    }
                };
                self.bosses.push(Boss {
                    position: pos,
                    health: boss_health,
                    max_health: boss_health,
                    move_timer: 0,
                    shoot_timer: 0,
                    kind,
                    state_timer: 0,
                });
                self.spawn_particles(
                    f32::from(pos.x),
                    f32::from(pos.y),
                    30,
                    crate::color::Color::Magenta,
                    'B',
                );
            }
        }
        let mut next_bosses = Vec::new();
        let mut new_lasers = Vec::new();
        for mut boss in std::mem::take(&mut self.bosses) {
            if !is_time_frozen {
                if boss.state_timer > 0 {
                    boss.state_timer -= 1;
                } else {
                    let mut move_threshold = if self.mode == GameMode::BossRush {
                        std::cmp::max(
                            1,
                            3_u8.saturating_sub(
                                u8::try_from(self.campaign_level).unwrap_or(255) / 5,
                            ),
                        )
                    } else {
                        2
                    };
                    if boss.kind == BossType::Charger || boss.kind == BossType::Juggernaut {
                        move_threshold = std::cmp::max(1, move_threshold / 2);
                    }
                    if boss.health <= boss.max_health / 2 {
                        move_threshold = std::cmp::max(1, move_threshold / 2);
                    }
                    boss.move_timer += 1;
                    if boss.move_timer >= move_threshold {
                        boss.move_timer = 0;
                        let target_pos = if let Some((decoy_pos, _)) = self.decoy {
                            decoy_pos
                        } else {
                            self.snake.head()
                        };
                        let dir_opt = self.get_boss_path(boss.position, target_pos, boss.kind);
                        let dir_opt = if dir_opt.is_none()
                            && boss.kind != BossType::Phantom
                            && boss.kind != BossType::Juggernaut
                            && boss.kind != BossType::Charger
                        {
                            self.bot_smart_pathfind(boss.position, target_pos, 3)
                        } else {
                            dir_opt
                        };
                        if let Some(dir) = dir_opt {
                            let raw_next_pos = Self::calculate_next_head_dir(boss.position, dir);
                            let next_pos = if self.portals.is_some_and(|(p1, _)| p1 == raw_next_pos)
                            {
                                self.portals.unwrap().1
                            } else if self.portals.is_some_and(|(_, p2)| p2 == raw_next_pos) {
                                self.portals.unwrap().0
                            } else {
                                raw_next_pos
                            };
                            let margin = if self.mode == GameMode::BattleRoyale {
                                self.safe_zone_margin
                            } else {
                                0
                            };
                            if next_pos.x > margin
                                && next_pos.x < self.width - 1 - margin
                                && next_pos.y > margin
                                && next_pos.y < self.height - 1 - margin
                            {
                                if self.obstacles.contains(&next_pos) {
                                    if boss.kind == BossType::Charger {
                                        self.obstacles.remove(&next_pos);
                                        boss.position = next_pos;
                                        boss.state_timer = 15;
                                        self.spawn_particles(
                                            f32::from(next_pos.x),
                                            f32::from(next_pos.y),
                                            20,
                                            crate::color::Color::Red,
                                            'X',
                                        );
                                        beep();
                                    } else if boss.kind == BossType::Juggernaut {
                                        self.obstacles.remove(&next_pos);
                                        boss.position = next_pos;
                                        self.spawn_particles(
                                            f32::from(next_pos.x),
                                            f32::from(next_pos.y),
                                            20,
                                            crate::color::Color::DarkGrey,
                                            '*',
                                        );
                                        beep();
                                    } else if boss.kind == BossType::Phantom {
                                        boss.position = next_pos;
                                    }
                                } else {
                                    let old_pos = boss.position;
                                    boss.position = next_pos;
                                    if boss.kind == BossType::Trapper {
                                        self.obstacles.insert(old_pos);
                                    }
                                }
                            }
                        }
                    }
                    if boss.kind == BossType::Dragon {
                        let mut shoot_threshold = if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                10,
                                20_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255),
                                ),
                            )
                        } else {
                            20
                        };
                        if boss.health <= boss.max_health / 2 {
                            shoot_threshold = std::cmp::max(2, shoot_threshold / 2);
                        }
                        boss.shoot_timer += 1;
                        if boss.shoot_timer >= shoot_threshold {
                            boss.shoot_timer = 0;
                            let head = self.snake.head();
                            let dx = i32::from(head.x) - i32::from(boss.position.x);
                            let dy = i32::from(head.y) - i32::from(boss.position.y);
                            let dir = if dx.abs() > dy.abs() {
                                if dx > 0 {
                                    crate::snake::Direction::Right
                                } else {
                                    crate::snake::Direction::Left
                                }
                            } else {
                                if dy > 0 {
                                    crate::snake::Direction::Down
                                } else {
                                    crate::snake::Direction::Up
                                }
                            };
                            let laser_pos = Self::calculate_next_head_dir(boss.position, dir);
                            let margin = if self.mode == GameMode::BattleRoyale {
                                self.safe_zone_margin
                            } else {
                                0
                            };

                            let mut spawn_positions = vec![laser_pos];
                            match dir {
                                Direction::Up | Direction::Down => {
                                    let mut p1 = laser_pos;
                                    p1.x = (i32::from(p1.x) + 1).try_into().unwrap_or(p1.x);
                                    spawn_positions.push(p1);

                                    let mut p2 = laser_pos;
                                    p2.x = (i32::from(p2.x) - 1).try_into().unwrap_or(p2.x);
                                    spawn_positions.push(p2);
                                },
                                Direction::Left | Direction::Right => {
                                    let mut p1 = laser_pos;
                                    p1.y = (i32::from(p1.y) + 1).try_into().unwrap_or(p1.y);
                                    spawn_positions.push(p1);

                                    let mut p2 = laser_pos;
                                    p2.y = (i32::from(p2.y) - 1).try_into().unwrap_or(p2.y);
                                    spawn_positions.push(p2);
                                },
                            }

                            for pos in spawn_positions {
                                if pos.x > margin
                                    && pos.x < self.width - 1 - margin
                                    && pos.y > margin
                                    && pos.y < self.height - 1 - margin
                                {
                                    new_lasers.push(Laser {
                                        position: pos,
                                        direction: dir,
                                        player: 3,
                                    });
                                }
                            }
                            beep();
                        }
                    } else if boss.kind == BossType::Mage {
                        let mut shoot_threshold = if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                15,
                                30_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255),
                                ),
                            )
                        } else {
                            30
                        };
                        if boss.health <= boss.max_health / 2 {
                            shoot_threshold = std::cmp::max(5, shoot_threshold / 2);
                        }
                        boss.shoot_timer += 1;
                        if boss.shoot_timer >= shoot_threshold {
                            boss.shoot_timer = 0;
                            let target_pos = if let Some((decoy_pos, _)) = self.decoy {
                                decoy_pos
                            } else {
                                self.snake.head()
                            };

                            self.meteors.push(Meteor {
                                position: target_pos,
                                timer: 15, // Delay before meteor hits
                            });

                            self.power_up = Some(PowerUp {
                                p_type: PowerUpType::TimeFreeze,
                                location: Point {
                                    x: 0,
                                    y: 0,
                                },
                                activation_time: Some(
                                    web_time::SystemTime::now()
                                        .duration_since(web_time::SystemTime::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs(),
                                ),
                            });
                            beep();
                        }
                    } else if boss.kind == BossType::Shooter {
                        let mut shoot_threshold = if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                5,
                                15_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255),
                                ),
                            )
                        } else {
                            15
                        };
                        if boss.health <= boss.max_health / 2 {
                            shoot_threshold = std::cmp::max(1, shoot_threshold / 2);
                        }
                        boss.shoot_timer += 1;
                        if boss.shoot_timer >= shoot_threshold {
                            boss.shoot_timer = 0;
                            let head = self.snake.head();
                            let dx = i32::from(head.x) - i32::from(boss.position.x);
                            let dy = i32::from(head.y) - i32::from(boss.position.y);
                            let dir = if dx.abs() > dy.abs() {
                                if dx > 0 {
                                    crate::snake::Direction::Right
                                } else {
                                    crate::snake::Direction::Left
                                }
                            } else {
                                if dy > 0 {
                                    crate::snake::Direction::Down
                                } else {
                                    crate::snake::Direction::Up
                                }
                            };
                            let laser_pos = Self::calculate_next_head_dir(boss.position, dir);
                            let margin = if self.mode == GameMode::BattleRoyale {
                                self.safe_zone_margin
                            } else {
                                0
                            };
                            if laser_pos.x > margin
                                && laser_pos.x < self.width - 1 - margin
                                && laser_pos.y > margin
                                && laser_pos.y < self.height - 1 - margin
                            {
                                new_lasers.push(Laser {
                                    position: laser_pos,
                                    direction: dir,
                                    player: 3,
                                });
                                beep();
                            }
                        }
                    } else if boss.kind == BossType::Spawner {
                        let mut spawn_threshold = if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                10,
                                30_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255),
                                ),
                            )
                        } else {
                            30
                        };
                        if boss.health <= boss.max_health / 2 {
                            spawn_threshold = std::cmp::max(5, spawn_threshold / 2);
                        }
                        boss.shoot_timer += 1;
                        if boss.shoot_timer >= spawn_threshold {
                            boss.shoot_timer = 0;
                            if self.mines.len() < 10 {
                                self.mines.insert(boss.position);
                            }
                        }
                    } else if boss.kind == BossType::Necromancer {
                        let mut spawn_threshold = if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                15,
                                45_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255),
                                ),
                            )
                        } else {
                            45
                        };
                        if boss.health <= boss.max_health / 2 {
                            spawn_threshold = std::cmp::max(10, spawn_threshold / 2);
                        }
                        boss.shoot_timer += 1;
                        if boss.shoot_timer >= spawn_threshold {
                            boss.shoot_timer = 0;
                            if self.goblin.is_none() {
                                self.goblin = Some(Goblin {
                                    position: boss.position,
                                    move_timer: 0,
                                    food_eaten: 0,
                                });
                                self.spawn_particles(
                                    f32::from(boss.position.x),
                                    f32::from(boss.position.y),
                                    20,
                                    crate::color::Color::Green,
                                    'G',
                                );
                                beep();
                            }
                        }
                    } else if boss.kind == BossType::ShadowClone {
                        boss.move_timer += 1;
                        if boss.move_timer >= 1 {
                            boss.move_timer = 0;
                            let dx = match self.snake.head().x.cmp(&boss.position.x) {
                                std::cmp::Ordering::Greater => 1,
                                std::cmp::Ordering::Less => -1,
                                std::cmp::Ordering::Equal => 0,
                            };
                            let dy = match self.snake.head().y.cmp(&boss.position.y) {
                                std::cmp::Ordering::Greater => 1,
                                std::cmp::Ordering::Less => -1,
                                std::cmp::Ordering::Equal => 0,
                            };
                            let mut new_pos = boss.position;
                            if dx != 0 && self.rng.gen_bool(0.5) {
                                new_pos.x =
                                    (i32::from(new_pos.x) + dx).try_into().unwrap_or(new_pos.x);
                            } else if dy != 0 {
                                new_pos.y =
                                    (i32::from(new_pos.y) + dy).try_into().unwrap_or(new_pos.y);
                            } else if dx != 0 {
                                new_pos.x =
                                    (i32::from(new_pos.x) + dx).try_into().unwrap_or(new_pos.x);
                            }
                            if new_pos.x > 0
                                && new_pos.x < self.width - 1
                                && new_pos.y > 0
                                && new_pos.y < self.height - 1
                            {
                                boss.position = new_pos;
                            }
                        }
                    } else if boss.kind == BossType::Teleporter {
                        let mut teleport_threshold = if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                10,
                                30_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255),
                                ),
                            )
                        } else {
                            30
                        };
                        if boss.health <= boss.max_health / 2 {
                            teleport_threshold = std::cmp::max(5, teleport_threshold / 2);
                        }
                        boss.move_timer += 1;
                        if boss.move_timer >= teleport_threshold {
                            boss.move_timer = 0;
                            let margin = self.safe_zone_margin;
                            let avoid = |p: &Point| {
                                self.obstacles.contains(p) || self.snake.body_map.contains_key(p)
                            };
                            if let Some(pos) = Self::get_random_empty_point(
                                self.width,
                                self.height,
                                &self.snake,
                                avoid,
                                &mut self.rng,
                                margin,
                            ) {
                                self.spawn_particles(
                                    f32::from(boss.position.x),
                                    f32::from(boss.position.y),
                                    20,
                                    crate::color::Color::Magenta,
                                    '*',
                                );
                                boss.position = pos;
                                boss.state_timer = 15;
                                self.spawn_particles(
                                    f32::from(boss.position.x),
                                    f32::from(boss.position.y),
                                    30,
                                    crate::color::Color::Magenta,
                                    'B',
                                );
                                beep();
                            }
                        }
                    } else if boss.kind == BossType::Puffer {
                        let mut move_threshold = if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                2,
                                4_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255) / 5,
                                ),
                            )
                        } else {
                            3
                        };
                        if boss.health <= boss.max_health / 2 {
                            move_threshold = std::cmp::max(1, move_threshold / 2);
                        }
                        boss.move_timer += 1;
                        if boss.move_timer >= move_threshold {
                            boss.move_timer = 0;
                            let target_pos = if let Some((decoy_pos, _)) = self.decoy {
                                decoy_pos
                            } else {
                                self.snake.head()
                            };
                            if let Some(dir) = self.bot_smart_pathfind(boss.position, target_pos, 3)
                            {
                                let next_pos = Self::calculate_next_head_dir(boss.position, dir);
                                let margin = if self.mode == GameMode::BattleRoyale {
                                    self.safe_zone_margin
                                } else {
                                    0
                                };
                                if next_pos.x > margin
                                    && next_pos.x < self.width - 1 - margin
                                    && next_pos.y > margin
                                    && next_pos.y < self.height - 1 - margin
                                    && !self.obstacles.contains(&next_pos)
                                {
                                    boss.position = next_pos;
                                }
                            }
                        }

                        let mut shoot_threshold = if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                10,
                                30_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255),
                                ),
                            )
                        } else {
                            30
                        };
                        if boss.health <= boss.max_health / 2 {
                            shoot_threshold = std::cmp::max(5, shoot_threshold / 2);
                        }
                        boss.shoot_timer += 1;
                        if boss.shoot_timer >= shoot_threshold {
                            boss.shoot_timer = 0;
                            let margin = if self.mode == GameMode::BattleRoyale {
                                self.safe_zone_margin
                            } else {
                                0
                            };
                            let dirs =
                                [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
                            for &dir in &dirs {
                                let laser_pos = Self::calculate_next_head_dir(boss.position, dir);
                                if laser_pos.x > margin
                                    && laser_pos.x < self.width - 1 - margin
                                    && laser_pos.y > margin
                                    && laser_pos.y < self.height - 1 - margin
                                {
                                    new_lasers.push(Laser {
                                        position: laser_pos,
                                        direction: dir,
                                        player: 3, // 3 means boss/neutral laser
                                    });
                                }
                            }
                            beep();
                        }
                    } else if boss.kind == BossType::Gorgon {
                        let mut move_threshold = if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                2,
                                4_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255) / 5,
                                ),
                            )
                        } else {
                            3
                        };
                        if boss.health <= boss.max_health / 2 {
                            move_threshold = std::cmp::max(1, move_threshold / 2);
                        }
                        boss.move_timer += 1;
                        if boss.move_timer >= move_threshold {
                            boss.move_timer = 0;
                            let target_pos = if let Some((decoy_pos, _)) = self.decoy {
                                decoy_pos
                            } else {
                                self.snake.head()
                            };
                            if let Some(dir) = self.bot_smart_pathfind(boss.position, target_pos, 3)
                            {
                                let next_pos = Self::calculate_next_head_dir(boss.position, dir);
                                let margin = if self.mode == GameMode::BattleRoyale {
                                    self.safe_zone_margin
                                } else {
                                    0
                                };
                                if next_pos.x > margin
                                    && next_pos.x < self.width - 1 - margin
                                    && next_pos.y > margin
                                    && next_pos.y < self.height - 1 - margin
                                    && !self.obstacles.contains(&next_pos)
                                {
                                    boss.position = next_pos;
                                }
                            }
                        }

                        let mut shoot_threshold = if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                15,
                                45_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255),
                                ),
                            )
                        } else {
                            45
                        };
                        if boss.health <= boss.max_health / 2 {
                            shoot_threshold = std::cmp::max(10, shoot_threshold / 2);
                        }
                        boss.shoot_timer += 1;
                        if boss.shoot_timer >= shoot_threshold {
                            boss.shoot_timer = 0;

                            // Turn current food into stone!
                            self.obstacles.insert(self.food);
                            let margin = if self.mode == GameMode::BattleRoyale {
                                self.safe_zone_margin
                            } else {
                                0
                            };
                            let avoid_food = |p: &Point| {
                                self.obstacles.contains(p)
                                    || self.snake.body_map.contains_key(p)
                                    || self
                                        .player2
                                        .as_ref()
                                        .is_some_and(|p2| p2.body_map.contains_key(p))
                            };
                            if let Some(new_food) = Self::get_random_empty_point(
                                self.width,
                                self.height,
                                &self.snake,
                                avoid_food,
                                &mut self.rng,
                                margin,
                            ) {
                                self.food = new_food;
                            }
                            beep();
                        }
                    } else if boss.kind == BossType::Mimic {
                        let target_pos = if let Some((decoy_pos, _)) = self.decoy {
                            decoy_pos
                        } else {
                            self.snake.head()
                        };
                        let dist_x = i32::from(target_pos.x).abs_diff(i32::from(boss.position.x));
                        let dist_y = i32::from(target_pos.y).abs_diff(i32::from(boss.position.y));
                        if dist_x + dist_y <= 3 {
                            let mut move_threshold = if self.mode == GameMode::BossRush {
                                std::cmp::max(
                                    1,
                                    2_u8.saturating_sub(
                                        u8::try_from(self.campaign_level).unwrap_or(255) / 5,
                                    ),
                                )
                            } else {
                                1
                            };
                            if boss.health <= boss.max_health / 2 {
                                move_threshold = std::cmp::max(1, move_threshold / 2);
                            }
                            boss.move_timer += 1;
                            if boss.move_timer >= move_threshold {
                                boss.move_timer = 0;
                                if let Some(dir) =
                                    self.bot_smart_pathfind(boss.position, target_pos, 3)
                                {
                                    let next_pos =
                                        Self::calculate_next_head_dir(boss.position, dir);
                                    let margin = if self.mode == GameMode::BattleRoyale {
                                        self.safe_zone_margin
                                    } else {
                                        0
                                    };
                                    if next_pos.x > margin
                                        && next_pos.x < self.width - 1 - margin
                                        && next_pos.y > margin
                                        && next_pos.y < self.height - 1 - margin
                                        && !self.obstacles.contains(&next_pos)
                                    {
                                        boss.position = next_pos;
                                    }
                                }
                            }
                        } else {
                            boss.move_timer = 0;
                        }
                    } else if boss.kind == BossType::Kraken {
                        let move_threshold = if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                1,
                                3_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255) / 5,
                                ),
                            )
                        } else {
                            2
                        };
                        boss.move_timer += 1;
                        if boss.move_timer >= move_threshold {
                            boss.move_timer = 0;
                            let target_pos = if let Some((decoy_pos, _)) = self.decoy {
                                decoy_pos
                            } else {
                                self.snake.head()
                            };

                            // Kraken pulls the snake towards itself
                            let head = self.snake.head();
                            let dx = i32::from(boss.position.x) - i32::from(head.x);
                            let dy = i32::from(boss.position.y) - i32::from(head.y);

                            let mut pull_dir = None;
                            if dx.abs() > dy.abs() {
                                if dx > 0 {
                                    pull_dir = Some(crate::snake::Direction::Right);
                                } else if dx < 0 {
                                    pull_dir = Some(crate::snake::Direction::Left);
                                }
                            } else {
                                if dy > 0 {
                                    pull_dir = Some(crate::snake::Direction::Down);
                                } else if dy < 0 {
                                    pull_dir = Some(crate::snake::Direction::Up);
                                }
                            }

                            if let Some(dir) = pull_dir {
                                // Simulate snake being pulled 1 tile towards Kraken occasionally
                                if self.rng.gen_bool(0.1) || cfg!(test) {
                                    // For Kraken, we force the queue bypassing handle_input limits
                                    self.snake.direction_queue.push_back(dir);
                                }
                            }

                            // Move towards target
                            let mut next_pos = boss.position;
                            if let Some(dir) = self.bot_smart_pathfind(boss.position, target_pos, 3)
                            {
                                next_pos = Self::calculate_next_head_dir(boss.position, dir);
                            }

                            let kraken_margin = if self.mode == GameMode::BattleRoyale {
                                self.safe_zone_margin
                            } else {
                                0
                            };

                            if next_pos.x > kraken_margin
                                && next_pos.x < self.width - 1 - kraken_margin
                                && next_pos.y > kraken_margin
                                && next_pos.y < self.height - 1 - kraken_margin
                            {
                                boss.position = next_pos;
                            }
                        }
                    } else if boss.kind == BossType::Alchemist {
                        let mut move_threshold = if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                1,
                                3_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255) / 5,
                                ),
                            )
                        } else {
                            3
                        };
                        if boss.health <= boss.max_health / 2 {
                            move_threshold = std::cmp::max(1, move_threshold / 2);
                        }
                        boss.move_timer += 1;
                        if boss.move_timer >= move_threshold {
                            boss.move_timer = 0;
                            let target_pos = if let Some((decoy_pos, _)) = self.decoy {
                                decoy_pos
                            } else {
                                self.snake.head()
                            };
                            if let Some(dir) = self.bot_smart_pathfind(boss.position, target_pos, 3)
                            {
                                let next_pos = Self::calculate_next_head_dir(boss.position, dir);
                                let margin = if self.mode == GameMode::BattleRoyale {
                                    self.safe_zone_margin
                                } else {
                                    0
                                };
                                if next_pos.x > margin
                                    && next_pos.x < self.width - 1 - margin
                                    && next_pos.y > margin
                                    && next_pos.y < self.height - 1 - margin
                                    && !self.obstacles.contains(&next_pos)
                                {
                                    boss.position = next_pos;
                                }
                            }
                        }

                        let mut drop_threshold = if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                10,
                                30_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255),
                                ),
                            )
                        } else {
                            20
                        };
                        if boss.health <= boss.max_health / 2 {
                            drop_threshold = std::cmp::max(5, drop_threshold / 2);
                        }
                        boss.shoot_timer += 1;
                        if boss.shoot_timer >= drop_threshold {
                            boss.shoot_timer = 0;
                            if self.poison_food.is_none() {
                                let avoid = |p: &Point| {
                                    self.obstacles.contains(p)
                                        || *p == self.food
                                        || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                                        || self
                                            .power_up
                                            .as_ref()
                                            .is_some_and(|pu| *p == pu.location)
                                };
                                if let Some(poison) = Self::get_random_empty_point(
                                    self.width,
                                    self.height,
                                    &self.snake,
                                    avoid,
                                    &mut self.rng,
                                    self.safe_zone_margin,
                                ) {
                                    self.poison_food = Some((poison, web_time::Instant::now()));
                                    beep();
                                }
                            }
                        }
                    } else if boss.kind == BossType::VampireLord {
                        let mut move_threshold = if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                1,
                                3_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255) / 5,
                                ),
                            )
                        } else {
                            2
                        };
                        if boss.health <= boss.max_health / 2 {
                            move_threshold = std::cmp::max(1, move_threshold / 2);
                        }
                        boss.move_timer += 1;
                        if boss.move_timer >= move_threshold {
                            boss.move_timer = 0;
                            let target_pos = if let Some((decoy_pos, _)) = self.decoy {
                                decoy_pos
                            } else {
                                self.snake.head()
                            };
                            let dist = i32::from(target_pos.x).abs_diff(i32::from(boss.position.x))
                                + i32::from(target_pos.y).abs_diff(i32::from(boss.position.y));

                            // Steal life if adjacent
                            if dist <= 2 {
                                if self.lives >= 1 {
                                    self.lives = self.lives.saturating_sub(1);
                                    boss.health = std::cmp::min(boss.max_health, boss.health + 5);
                                    self.spawn_particles(
                                        f32::from(boss.position.x),
                                        f32::from(boss.position.y),
                                        20,
                                        crate::color::Color::Red,
                                        '+',
                                    );
                                    beep();

                                    // Teleport away after stealing life
                                    let margin = self.safe_zone_margin;
                                    let avoid = |p: &Point| {
                                        self.obstacles.contains(p)
                                            || self.snake.body_map.contains_key(p)
                                    };
                                    if let Some(pos) = Self::get_random_empty_point(
                                        self.width,
                                        self.height,
                                        &self.snake,
                                        avoid,
                                        &mut self.rng,
                                        margin,
                                    ) {
                                        self.spawn_particles(
                                            f32::from(boss.position.x),
                                            f32::from(boss.position.y),
                                            20,
                                            crate::color::Color::Magenta,
                                            '*',
                                        );
                                        boss.position = pos;
                                        boss.state_timer = 15;
                                    }
                                }
                            } else if let Some(dir) =
                                self.bot_smart_pathfind(boss.position, target_pos, 3)
                            {
                                let next_pos = Self::calculate_next_head_dir(boss.position, dir);
                                let margin = if self.mode == GameMode::BattleRoyale {
                                    self.safe_zone_margin
                                } else {
                                    0
                                };
                                if next_pos.x > margin
                                    && next_pos.x < self.width - 1 - margin
                                    && next_pos.y > margin
                                    && next_pos.y < self.height - 1 - margin
                                    && !self.obstacles.contains(&next_pos)
                                {
                                    boss.position = next_pos;
                                }
                            }
                        }
                    }
                }
            }
            next_bosses.push(boss);
        }
        self.bosses = next_bosses;
        self.lasers.extend(new_lasers);
        self.lightning_column = None;
        if self.weather == Weather::Sandstorm && self.rng.gen_bool(0.1) {
            let margin = if self.mode == GameMode::BattleRoyale {
                self.safe_zone_margin
            } else {
                0
            };
            let mut new_food = self.food;
            let dir = self.rng.gen_range(0..4);
            match dir {
                0 => new_food.y = new_food.y.saturating_sub(1),
                1 => new_food.y = new_food.y.saturating_add(1),
                2 => new_food.x = new_food.x.saturating_sub(1),
                _ => new_food.x = new_food.x.saturating_add(1),
            }
            let min_x = margin;
            let max_x = (self.width - 1).saturating_sub(margin).max(min_x);
            let min_y = margin;
            let max_y = (self.height - 1).saturating_sub(margin).max(min_y);

            if new_food.x > min_x
                && new_food.x < max_x
                && new_food.y > min_y
                && new_food.y < max_y
                && !self.obstacles.contains(&new_food)
            {
                self.food = new_food;
            }
        }
        if self.weather == Weather::Earthquake && self.rng.gen_bool(0.05) {
            if !self.obstacles.is_empty() && self.rng.gen_bool(0.5) {
                if let Some(obs) = self.obstacles.iter().next().copied() {
                    self.obstacles.remove(&obs);
                }
            } else {
                let margin = if self.mode == GameMode::BattleRoyale {
                    self.safe_zone_margin
                } else {
                    0
                };
                let avoid = |p: &Point| self.obstacles.contains(p);
                if let Some(pos) = Self::get_random_empty_point(
                    self.width,
                    self.height,
                    &self.snake,
                    avoid,
                    &mut self.rng,
                    margin,
                ) {
                    self.obstacles.insert(pos);
                }
            }
        }
        if self.weather == Weather::Tornado && self.rng.gen_bool(0.05) {
            let margin = if self.mode == GameMode::BattleRoyale {
                self.safe_zone_margin
            } else {
                0
            };
            if self.rng.gen_bool(0.5) {
                let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
                let dir = dirs[self.rng.gen_range(0..4)];
                let next_p = Self::calculate_next_head_dir(self.food, dir);
                if next_p.x > margin
                    && next_p.x < self.width - 1 - margin
                    && next_p.y > margin
                    && next_p.y < self.height - 1 - margin
                {
                    let avoid = |p: &Point| {
                        self.obstacles.contains(p)
                            || self.snake.body_map.contains_key(p)
                            || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p))
                            || self.bots.iter().any(|b| b.body_map.contains_key(p))
                            || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                            || self.power_up.as_ref().is_some_and(|pu| *p == pu.location)
                    };
                    if !avoid(&next_p) {
                        self.food = next_p;
                    }
                }
            }
        }
        if self.rng.gen_bool(0.002) {
            self.weather = match self.rng.gen_range(0..7) {
                0 => Weather::Clear,
                1 => Weather::Rain,
                2 => Weather::Snow,
                3 => Weather::Storm,
                4 => Weather::Tornado,
                5 => Weather::Sandstorm,
                _ => Weather::Earthquake,
            };
        }
        if self.weather == Weather::Storm && self.rng.gen_bool(0.02) {
            let margin = if self.mode == GameMode::BattleRoyale {
                self.safe_zone_margin
            } else {
                0
            };
            let min_x = margin + 1;
            let max_x = (self.width - 1).saturating_sub(margin).max(min_x);
            if max_x > min_x {
                let strike_x = self.rng.gen_range(min_x..max_x);
                self.lightning_column = Some(strike_x);
                self.obstacles.retain(|p| p.x != strike_x);
                let mut next_bosses = Vec::new();
                let mut new_lasers = Vec::new();
                for mut boss in std::mem::take(&mut self.bosses) {
                    if boss.position.x == strike_x {
                        boss.health = boss.health.saturating_sub(5);
                        if boss.health == 0 {
                            *self.stats.bestiary.entry(boss.kind).or_insert(0) += 1;
                            self.update_quest_progress(crate::game::QuestType::SlayBosses, 1);
                            if self.rng.gen_bool(0.2) {
                                self.equipment_boxes.push(boss.position);
                            }
                            if self.stats.equipped_class
                                == Some(crate::game::HeroClass::Necromancer)
                            {
                                self.companion = Some(Companion {
                                    position: boss.position,
                                    kind: crate::game::CompanionType::Fighter,
                                    move_timer: 0,
                                    action_timer: 0,
                                    path: Vec::new(),
                                });
                                crate::game::beep();
                            }

                            if self.mode == GameMode::SnakeSurvivor {
                                self.xp_gems.insert(boss.position);
                            }

                            if boss.kind == BossType::Splitter && boss.max_health > 5 {
                                let half_max = boss.max_health / 2;
                                let child1_pos = Point {
                                    x: boss.position.x.saturating_sub(1).max(1),
                                    y: boss.position.y,
                                };
                                let child2_pos = Point {
                                    x: (boss.position.x + 1).min(self.width - 2),
                                    y: boss.position.y,
                                };
                                next_bosses.push(Boss {
                                    position: child1_pos,
                                    health: half_max,
                                    max_health: half_max,
                                    move_timer: 0,
                                    shoot_timer: 0,
                                    kind: BossType::Splitter,
                                    state_timer: 0,
                                });
                                next_bosses.push(Boss {
                                    position: child2_pos,
                                    health: half_max,
                                    max_health: half_max,
                                    move_timer: 0,
                                    shoot_timer: 0,
                                    kind: BossType::Splitter,
                                    state_timer: 0,
                                });
                                self.spawn_particles(
                                    f32::from(strike_x),
                                    f32::from(boss.position.y),
                                    30,
                                    crate::color::Color::Magenta,
                                    's',
                                );
                            } else {
                                self.update_bounty_progress(
                                    crate::game::BountyType::KillBosses(0),
                                    1,
                                );
                                if self.mode == GameMode::BossRush {
                                    self.score += 1000 * self.campaign_level;
                                    self.campaign_level += 1;
                                } else {
                                    self.score += 100;
                                }
                                if self.stats.faction.is_some() {
                                    self.stats.faction_rep += 100;
                                }
                                self.spawn_particles(
                                    f32::from(strike_x),
                                    f32::from(boss.position.y),
                                    30,
                                    crate::color::Color::Magenta,
                                    'B',
                                );
                                let boss_pos = boss.position;
                                let margin = if self.mode == GameMode::BattleRoyale {
                                    self.safe_zone_margin
                                } else {
                                    0
                                };
                                for &dir in &[
                                    Direction::Up,
                                    Direction::Down,
                                    Direction::Left,
                                    Direction::Right,
                                ] {
                                    let laser_pos = Self::calculate_next_head_dir(boss_pos, dir);
                                    if laser_pos.x > margin
                                        && laser_pos.x < self.width - 1 - margin
                                        && laser_pos.y > margin
                                        && laser_pos.y < self.height - 1 - margin
                                    {
                                        new_lasers.push(Laser {
                                            position: laser_pos,
                                            direction: dir,
                                            player: 3,
                                        });
                                    }
                                }
                            }
                        } else {
                            self.spawn_particles(
                                f32::from(strike_x),
                                f32::from(boss.position.y),
                                10,
                                crate::color::Color::Yellow,
                                '*',
                            );
                            next_bosses.push(boss);
                        }
                    } else {
                        next_bosses.push(boss);
                    }
                }
                self.bosses = next_bosses;
                self.lasers.extend(new_lasers);
                for y in (margin + 1)..self.height.saturating_sub(margin).saturating_sub(1) {
                    if y % 3 == 0 {
                        self.spawn_particles(
                            f32::from(strike_x),
                            f32::from(y),
                            2,
                            crate::color::Color::Cyan,
                            '!',
                        );
                    }
                }
                crate::game::beep();
            }
        }
        let chat_interval = if self.mode == GameMode::SinglePlayer
            || self.mode == GameMode::DailyChallenge
            || self.mode == GameMode::WeeklyChallenge
            || self.mode == GameMode::MonthlyChallenge
            || self.mode == GameMode::YearlyChallenge
            || self.mode == GameMode::DecadeChallenge
            || self.mode == GameMode::CenturyChallenge
            || self.mode == GameMode::MillenniumChallenge
            || self.mode == GameMode::EonChallenge
        {
            Duration::from_secs(3)
        } else {
            Duration::from_millis(500)
        };
        if self.last_chat_time.is_none_or(|t| t.elapsed() >= chat_interval)
            && self.rng.gen_bool(0.3)
        {
            let messages = [
                "PogChamp",
                "Bot is insane!",
                "Drop skins!",
                "HODL",
                "LUL",
                "EZ",
                "F",
                "What a play",
                "200 IQ",
                "nerf snake pls",
                "kappa",
                "monkaS",
                "sheeeesh",
            ];
            let users = [
                "SnakeMaster99",
                "xX_Slayer_Xx",
                "CryptoBro",
                "GamerGirl",
                "BotSpectator",
                "Noob123",
                "ProPlayer",
                "Admin",
                "Mod",
                "VIP_User",
            ];
            let colors = [
                crate::color::Color::Red,
                crate::color::Color::Green,
                crate::color::Color::Yellow,
                crate::color::Color::Blue,
                crate::color::Color::Magenta,
                crate::color::Color::Cyan,
                crate::color::Color::White,
            ];
            let msg = messages[self.rng.gen_range(0..messages.len())];
            let user = users[self.rng.gen_range(0..users.len())];
            let color = colors[self.rng.gen_range(0..colors.len())];
            let chat_line = format!("{user}: {msg}");
            self.chat_log.push_back((chat_line, color));
            if self.chat_log.len() > 10 {
                self.chat_log.pop_front();
            }
            self.last_chat_time = Some(web_time::Instant::now());
        }
        for p in &mut self.particles {
            p.x += p.vx;
            p.y += p.vy;
            p.lifetime -= 1.0;
        }
        self.particles.retain(|p| p.lifetime > 0.0);
        for t in &mut self.floating_texts {
            t.y -= 0.1;
            t.lifetime -= 1.0;
        }
        self.floating_texts.retain(|t| t.lifetime > 0.0);
        let is_invincible = self.mode == GameMode::Zen
            || self.power_up.as_ref().is_some_and(|p| {
                p.p_type == PowerUpType::Invincibility
                    && p.activation_time.is_some_and(|t| {
                        web_time::SystemTime::now()
                            .duration_since(web_time::SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                            .saturating_sub(t)
                            < self.powerup_duration()
                    })
            });
        let mut p1_dead = false;
        let mut p2_dead = false;
        let mut lasers_to_keep = Vec::new();
        let margin = if self.mode == GameMode::BattleRoyale {
            self.safe_zone_margin
        } else {
            0
        };
        for mut laser in std::mem::take(&mut self.lasers) {
            let mut destroyed = false;
            let base_loops = if is_time_frozen {
                1
            } else {
                2
            };

            let extra_loops = if laser.player == 1 {
                self.in_game_upgrades.get(&InGameUpgrade::LaserSpeed).copied().unwrap_or(0)
            } else {
                0
            };

            let loops = base_loops + extra_loops;

            let is_piercing =
                laser.player == 1 && self.in_game_upgrades.contains_key(&InGameUpgrade::Piercing);

            for _ in 0..loops {
                if !is_time_frozen {
                    if laser.player == 1
                        && self.in_game_upgrades.contains_key(&InGameUpgrade::HomingLasers)
                        && !self.bosses.is_empty()
                    {
                        let mut best_dist = u16::MAX;
                        let mut closest_boss = None;
                        for boss in &self.bosses {
                            let dist = laser.position.x.abs_diff(boss.position.x)
                                + laser.position.y.abs_diff(boss.position.y);
                            if dist < best_dist {
                                best_dist = dist;
                                closest_boss = Some(boss.position);
                            }
                        }

                        if let Some(target) = closest_boss
                            && best_dist <= 5
                        {
                            // Homing range
                            let dx = i32::from(target.x) - i32::from(laser.position.x);
                            let dy = i32::from(target.y) - i32::from(laser.position.y);
                            if dx.abs() > dy.abs() {
                                if dx > 0 {
                                    laser.direction = Direction::Right;
                                } else {
                                    laser.direction = Direction::Left;
                                }
                            } else if dy != 0 {
                                if dy > 0 {
                                    laser.direction = Direction::Down;
                                } else {
                                    laser.direction = Direction::Up;
                                }
                            }
                        }
                    }
                    let mut next_pos =
                        Self::calculate_next_head_dir(laser.position, laser.direction);
                    if let Some((p1, p2)) = self.portals {
                        if next_pos == p1 {
                            next_pos = p2;
                        } else if next_pos == p2 {
                            next_pos = p1;
                        }
                    }
                    laser.position = next_pos;
                }
                if self.mode == GameMode::Dodgeball {
                    if laser.position.x <= margin {
                        if laser.direction == Direction::Left {
                            laser.direction = Direction::Right;
                        }
                        laser.position.x = margin + 1;
                    } else if laser.position.x >= self.width - 1 - margin {
                        if laser.direction == Direction::Right {
                            laser.direction = Direction::Left;
                        }
                        laser.position.x = self.width - 2 - margin;
                    }
                    if laser.position.y <= margin {
                        if laser.direction == Direction::Up {
                            laser.direction = Direction::Down;
                        }
                        laser.position.y = margin + 1;
                    } else if laser.position.y >= self.height - 1 - margin {
                        if laser.direction == Direction::Down {
                            laser.direction = Direction::Up;
                        }
                        laser.position.y = self.height - 2 - margin;
                    }
                }

                if self.mode != GameMode::Dodgeball
                    && (laser.position.x <= margin
                        || laser.position.x >= self.width - 1 - margin
                        || laser.position.y <= margin
                        || laser.position.y >= self.height - 1 - margin)
                {
                    destroyed = true;
                    break;
                }
                if self.mode != GameMode::Dodgeball && self.obstacles.contains(&laser.position) {
                    self.obstacles.remove(&laser.position);
                    if !is_piercing {
                        destroyed = true;
                    }
                    self.spawn_particles(
                        f32::from(laser.position.x),
                        f32::from(laser.position.y),
                        10,
                        crate::color::Color::Red,
                        'x',
                    );
                    if destroyed {
                        break;
                    }
                }
                let mut hit_boss_idx = None;
                let mut damage_text = None;
                for (i, boss) in self.bosses.iter_mut().enumerate() {
                    if boss.position == laser.position {
                        let mut damage = 1;
                        if laser.player == 1
                            && self.stats.faction == Some(crate::game::Faction::CrimsonVipers)
                        {
                            damage += 1 + (self.stats.faction_rep / 5000);
                        }
                        boss.health = boss.health.saturating_sub(damage);
                        damage_text = Some((
                            f32::from(laser.position.x),
                            f32::from(laser.position.y),
                            format!("-{damage}"),
                            crate::color::Color::Red,
                        ));
                        if !is_piercing {
                            destroyed = true;
                        }
                        hit_boss_idx = Some(i);
                        break;
                    }
                }
                if let Some((x, y, text, color)) = damage_text {
                    self.spawn_floating_text(x, y, text, color);
                }
                if self.goblin.as_ref().is_some_and(|goblin| laser.position == goblin.position) {
                    let gob_pos = self.goblin.as_ref().unwrap().position;
                    self.goblin = None;
                    if laser.player != 1 {
                        continue;
                    }
                    if !is_piercing {
                        destroyed = true;
                    }
                    let multiplier = if self.skin == '₿' {
                        2
                    } else {
                        1
                    };
                    self.score += 500;
                    self.stats.total_score += 500;
                    self.stats.coins += 500 * multiplier;
                    self.spawn_floating_text(
                        f32::from(laser.position.x),
                        f32::from(laser.position.y),
                        "+500".to_string(),
                        crate::color::Color::Yellow,
                    );
                    self.spawn_particles(
                        f32::from(laser.position.x),
                        f32::from(laser.position.y),
                        50,
                        crate::color::Color::Yellow,
                        '$',
                    );
                    if self.mode == GameMode::SnakeSurvivor {
                        self.xp_gems.insert(gob_pos);
                    }
                    beep();
                    if destroyed {
                        break;
                    }
                }
                if let Some(i) = hit_boss_idx {
                    let boss_pos = self.bosses[i].position;
                    let boss_health = self.bosses[i].health;
                    if boss_health == 0 {
                        *self.stats.bestiary.entry(self.bosses[i].kind).or_insert(0) += 1;
                        self.update_quest_progress(crate::game::QuestType::SlayBosses, 1);
                        let dead_boss = self.bosses.remove(i);

                        if self.rng.gen_bool(0.2) {
                            self.equipment_boxes.push(dead_boss.position);
                        }

                        if self.stats.equipped_class == Some(crate::game::HeroClass::Necromancer) {
                            self.companion = Some(Companion {
                                position: dead_boss.position,
                                kind: crate::game::CompanionType::Fighter,
                                move_timer: 0,
                                action_timer: 0,
                                path: Vec::new(),
                            });
                            crate::game::beep();
                        }

                        if self.mode == GameMode::SnakeSurvivor {
                            self.xp_gems.insert(dead_boss.position);
                        }

                        if dead_boss.kind == BossType::Splitter && dead_boss.max_health > 5 {
                            let half_max = dead_boss.max_health / 2;
                            let child1_pos = Point {
                                x: dead_boss.position.x.saturating_sub(1).max(1),
                                y: dead_boss.position.y,
                            };
                            let child2_pos = Point {
                                x: (dead_boss.position.x + 1).min(self.width - 2),
                                y: dead_boss.position.y,
                            };
                            self.bosses.push(Boss {
                                position: child1_pos,
                                health: half_max,
                                max_health: half_max,
                                move_timer: 0,
                                shoot_timer: 0,
                                kind: BossType::Splitter,
                                state_timer: 0,
                            });
                            self.bosses.push(Boss {
                                position: child2_pos,
                                health: half_max,
                                max_health: half_max,
                                move_timer: 0,
                                shoot_timer: 0,
                                kind: BossType::Splitter,
                                state_timer: 0,
                            });
                            self.spawn_particles(
                                f32::from(laser.position.x),
                                f32::from(laser.position.y),
                                30,
                                crate::color::Color::Magenta,
                                's',
                            );
                        } else {
                            self.update_bounty_progress(crate::game::BountyType::KillBosses(0), 1);
                            if self.mode == GameMode::BossRush {
                                self.score += 1000 * self.campaign_level;
                                self.campaign_level += 1;
                            } else {
                                self.score += 100;
                            }
                            if self.stats.faction.is_some() {
                                self.stats.faction_rep += 100;
                            }
                            self.spawn_particles(
                                f32::from(laser.position.x),
                                f32::from(laser.position.y),
                                30,
                                crate::color::Color::Magenta,
                                'B',
                            );
                            let margin = if self.mode == GameMode::BattleRoyale {
                                self.safe_zone_margin
                            } else {
                                0
                            };
                            for &dir in
                                &[Direction::Up, Direction::Down, Direction::Left, Direction::Right]
                            {
                                let laser_pos = Self::calculate_next_head_dir(boss_pos, dir);
                                if laser_pos.x > margin
                                    && laser_pos.x < self.width - 1 - margin
                                    && laser_pos.y > margin
                                    && laser_pos.y < self.height - 1 - margin
                                {
                                    lasers_to_keep.push(Laser {
                                        position: laser_pos,
                                        direction: dir,
                                        player: 3,
                                    });
                                }
                            }
                        }
                    } else {
                        self.spawn_particles(
                            f32::from(laser.position.x),
                            f32::from(laser.position.y),
                            5,
                            crate::color::Color::Magenta,
                            '*',
                        );
                    }
                    if destroyed {
                        break;
                    }
                }
                if laser.player != 1 && self.snake.body_map.contains_key(&laser.position) {
                    if !is_invincible {
                        p1_dead = true;
                    }
                    destroyed = true;
                    self.spawn_particles(
                        f32::from(laser.position.x),
                        f32::from(laser.position.y),
                        10,
                        crate::color::Color::Red,
                        'x',
                    );
                    break;
                }
                if let Some(p2) = &self.player2
                    && laser.player != 2
                    && p2.body_map.contains_key(&laser.position)
                {
                    if !is_invincible {
                        p2_dead = true;
                    }
                    destroyed = true;
                    break;
                }
            }
            if destroyed
                && laser.player == 1
                && self.in_game_upgrades.contains_key(&InGameUpgrade::ExplosiveLasers)
            {
                // Explosive Lasers AOE effect
                let radius = 2; // 5x5 area centered on laser.position
                let mut boss_hits = std::collections::HashSet::new();
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let nx = i32::from(laser.position.x) + dx;
                        let ny = i32::from(laser.position.y) + dy;
                        if nx > 0
                            && nx < i32::from(self.width - 1)
                            && ny > 0
                            && ny < i32::from(self.height - 1)
                        {
                            let p = Point {
                                x: u16::try_from(nx).unwrap_or(0),
                                y: u16::try_from(ny).unwrap_or(0),
                            };
                            if self.obstacles.contains(&p) {
                                self.obstacles.remove(&p);
                            }
                            // We do not want to iterate and remove bosses directly while in this loop if they are already handled,
                            // but dealing a damage point is easy:
                            for (b_idx, boss) in self.bosses.iter_mut().enumerate() {
                                if boss.position == p {
                                    boss_hits.insert(b_idx);
                                }
                            }
                        }
                    }
                }
                let mut spawn_texts = Vec::new();
                for &b_idx in &boss_hits {
                    if let Some(b) = self.bosses.get_mut(b_idx) {
                        b.health = b.health.saturating_sub(2); // AOE deals extra damage
                        spawn_texts.push((
                            f32::from(laser.position.x),
                            f32::from(laser.position.y),
                            "-2".to_string(),
                            crate::color::Color::Red,
                        ));
                    }
                }
                for (x, y, text, color) in spawn_texts {
                    self.spawn_floating_text(x, y, text, color);
                }
                // Handle boss deaths from AOE
                let mut next_bosses = Vec::new();
                for boss in std::mem::take(&mut self.bosses) {
                    if boss.health == 0 {
                        *self.stats.bestiary.entry(boss.kind).or_insert(0) += 1;
                        self.update_quest_progress(crate::game::QuestType::SlayBosses, 1);
                        if self.rng.gen_bool(0.2) {
                            self.equipment_boxes.push(boss.position);
                        }
                        if self.stats.equipped_class == Some(crate::game::HeroClass::Necromancer) {
                            self.companion = Some(Companion {
                                position: boss.position,
                                kind: crate::game::CompanionType::Fighter,
                                move_timer: 0,
                                action_timer: 0,
                                path: Vec::new(),
                            });
                            crate::game::beep();
                        }
                        self.update_bounty_progress(crate::game::BountyType::KillBosses(0), 1);
                        self.score += 100;
                        if self.stats.faction.is_some() {
                            self.stats.faction_rep += 100;
                        }
                        self.spawn_particles(
                            f32::from(boss.position.x),
                            f32::from(boss.position.y),
                            30,
                            crate::color::Color::Magenta,
                            'B',
                        );
                        self.lasers.push(Laser {
                            position: boss.position,
                            direction: Direction::Up,
                            player: 3,
                        });
                        self.lasers.push(Laser {
                            position: boss.position,
                            direction: Direction::Down,
                            player: 3,
                        });
                        self.lasers.push(Laser {
                            position: boss.position,
                            direction: Direction::Left,
                            player: 3,
                        });
                        self.lasers.push(Laser {
                            position: boss.position,
                            direction: Direction::Right,
                            player: 3,
                        });
                    } else {
                        next_bosses.push(boss);
                    }
                }
                self.bosses = next_bosses;

                self.spawn_particles(
                    f32::from(laser.position.x),
                    f32::from(laser.position.y),
                    40,
                    crate::color::Color::Red,
                    '*',
                );
                beep();
            }

            if !destroyed {
                lasers_to_keep.push(laser);
            }
        }
        self.lasers = lasers_to_keep;
        if self.mode == GameMode::TimeAttack && self.start_time.elapsed() >= Duration::from_secs(60)
        {
            self.handle_death("Time's up!");
            return;
        }
        if self.mode == GameMode::Speedrun && self.food_eaten_session >= 50 {
            self.handle_win();
            return;
        }
        if self.mode == GameMode::BattleRoyale
            && self.last_shrink_time.elapsed() >= Duration::from_secs(10)
        {
            let max_margin = (self.width.min(self.height) / 2).saturating_sub(2);
            if self.safe_zone_margin < max_margin {
                self.safe_zone_margin += 1;
                self.last_shrink_time = web_time::Instant::now();
                if self.food.x <= self.safe_zone_margin
                    || self.food.x >= self.width - 1 - self.safe_zone_margin
                    || self.food.y <= self.safe_zone_margin
                    || self.food.y >= self.height - 1 - self.safe_zone_margin
                {
                    let avoid_food = |p: &Point| {
                        self.obstacles.contains(p)
                            || self.snake.body_map.contains_key(p)
                            || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p))
                    };
                    if let Some(new_food) = Self::get_random_empty_point(
                        self.width,
                        self.height,
                        &self.snake,
                        avoid_food,
                        &mut self.rng,
                        self.safe_zone_margin,
                    ) {
                        self.food = new_food;
                    }
                }
                if let Some((bp, _)) = self.bonus_food
                    && (bp.x <= self.safe_zone_margin
                        || bp.x >= self.width - 1 - self.safe_zone_margin
                        || bp.y <= self.safe_zone_margin
                        || bp.y >= self.height - 1 - self.safe_zone_margin)
                {
                    self.bonus_food = None;
                }
                if let Some(pu) = &self.power_up
                    && (pu.location.x <= self.safe_zone_margin
                        || pu.location.x >= self.width - 1 - self.safe_zone_margin
                        || pu.location.y <= self.safe_zone_margin
                        || pu.location.y >= self.height - 1 - self.safe_zone_margin)
                {
                    self.power_up = None;
                    if self.stats.equipped_class == Some(crate::game::HeroClass::Mage) {
                        self.power_up = Some(PowerUp {
                            p_type: PowerUpType::TimeFreeze,
                            location: Point {
                                x: 0,
                                y: 0,
                            },
                            activation_time: Some(
                                web_time::SystemTime::now()
                                    .duration_since(web_time::SystemTime::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs(),
                            ),
                        });
                    }
                }
                crate::game::beep();
            }
        }
        if self.mode == GameMode::Survival
            && self.last_obstacle_spawn_time.elapsed() >= Duration::from_secs(3)
        {
            self.last_obstacle_spawn_time = web_time::Instant::now();
            let avoid = |p: &Point| {
                self.obstacles.contains(p)
                    || *p == self.food
                    || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                    || self.power_up.as_ref().is_some_and(|pu| pu.location == *p)
                    || self.snake.body_map.contains_key(p)
                    || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p))
            };
            if let Some(new_obstacle) = Self::get_random_empty_point(
                self.width,
                self.height,
                &self.snake,
                avoid,
                &mut self.rng,
                self.safe_zone_margin,
            ) {
                self.obstacles.insert(new_obstacle);
            }
        }
        if self.mode == GameMode::Evolution
            && self.last_obstacle_spawn_time.elapsed() >= Duration::from_secs(2)
        {
            self.last_obstacle_spawn_time = web_time::Instant::now();
            self.evolve_game_of_life();
        }
        if self.mode == GameMode::Flood
            && self.last_obstacle_spawn_time.elapsed() >= Duration::from_secs(5)
        {
            self.last_obstacle_spawn_time = web_time::Instant::now();
            self.rise_flood();
        }
        if self.mode == GameMode::BulletHell {
            // Give score based on survival time (10 per second via checking elapsed ticks roughly)
            if self.tick_counter.is_multiple_of(10) {
                self.score += 1;
            }

            // Spawn lasers from random edges
            let difficulty_factor = self.start_time.elapsed().as_secs();
            // Max out difficulty multiplier at 100 seconds
            let difficulty_multiplier = std::cmp::min(100, difficulty_factor) / 10 + 1;

            // In BulletHell we spawn multiple lasers per tick depending on survival time
            let spawn_chance = 0.05 * (difficulty_multiplier as f64);

            if self.rng.gen_bool(spawn_chance) {
                let margin = self.safe_zone_margin;

                // Randomly pick an edge (0: Top, 1: Bottom, 2: Left, 3: Right)
                let edge = self.rng.gen_range(0..4);
                let (spawn_pos, dir) = match edge {
                    0 => {
                        // Top edge, moving Down
                        let x = self.rng.gen_range((margin + 1)..(self.width - 1 - margin));
                        (
                            Point {
                                x,
                                y: margin + 1,
                            },
                            Direction::Down,
                        )
                    },
                    1 => {
                        // Bottom edge, moving Up
                        let x = self.rng.gen_range((margin + 1)..(self.width - 1 - margin));
                        (
                            Point {
                                x,
                                y: self.height - 2 - margin,
                            },
                            Direction::Up,
                        )
                    },
                    2 => {
                        // Left edge, moving Right
                        let y = self.rng.gen_range((margin + 1)..(self.height - 1 - margin));
                        (
                            Point {
                                x: margin + 1,
                                y,
                            },
                            Direction::Right,
                        )
                    },
                    _ => {
                        // Right edge, moving Left
                        let y = self.rng.gen_range((margin + 1)..(self.height - 1 - margin));
                        (
                            Point {
                                x: self.width - 2 - margin,
                                y,
                            },
                            Direction::Left,
                        )
                    },
                };

                // Spawn laser (player 3 acts as hostile environmental laser)
                self.lasers.push(Laser {
                    position: spawn_pos,
                    direction: dir,
                    player: 3,
                });
            }
        }
        self.handle_survivor_auto_fire();

        self.handle_autopilot_moves();
        if let Some(dir) = self.snake.direction_queue.pop_front() {
            self.snake.direction = dir;
        }
        if self.mode == GameMode::Speedrun {
            self.current_replay.push(self.snake.direction);
        }
        if let Some(p2) = &mut self.player2
            && let Some(dir) = p2.direction_queue.pop_front()
        {
            p2.direction = dir;
        }
        self.manage_resources();
        self.manage_merchant();
        self.manage_bonus_food();
        self.manage_poison_food();
        self.manage_power_ups();
        self.manage_portals();
        self.manage_mines();
        self.manage_black_hole();
        self.manage_meteors();
        self.manage_goblin();
        self.manage_turrets();
        self.manage_companion();
        self.manage_crops();
        self.manage_eggs();
        self.apply_magnet();
        self.apply_gravity();

        // Handle bots movement and check collisions with walls/boundaries
        let mut final_bot_heads = Vec::new();
        let can_pass_through_walls = self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::PassThroughWalls
                && p.activation_time.is_some_and(|t| {
                    web_time::SystemTime::now()
                        .duration_since(web_time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .saturating_sub(t)
                        < self.powerup_duration()
                })
        }) || self.stats.equipped_vehicle
            == Some(crate::game::Vehicle::Spaceship);

        for i in 0..self.bots.len() {
            if let Some(dir) = self.bots[i].direction_queue.pop_front() {
                self.bots[i].direction = dir;
            }
            let next_head =
                Self::calculate_next_head_dir(self.bots[i].head(), self.bots[i].direction);
            let mut hit_wall = false;
            let final_head = if self.portals.is_some_and(|(p1, _)| p1 == next_head) {
                self.portals.unwrap().1
            } else if self.portals.is_some_and(|(_, p2)| p2 == next_head) {
                self.portals.unwrap().0
            } else if (self.wrap_mode || can_pass_through_walls || self.mode == GameMode::Zen)
                && self.mode != GameMode::BattleRoyale
            {
                self.calculate_wrapped_head(next_head)
            } else {
                let margin = if self.mode == GameMode::BattleRoyale {
                    self.safe_zone_margin
                } else {
                    0
                };
                if next_head.x <= margin
                    || next_head.x >= self.width - 1 - margin
                    || next_head.y <= margin
                    || next_head.y >= self.height - 1 - margin
                {
                    hit_wall = true;
                }
                next_head
            };
            final_bot_heads.push((i, final_head, hit_wall));
        }

        let (final_head1, final_head2_opt, hit_wall1, hit_wall2) = self.calculate_final_heads();
        let hit_obstacle1 = self.obstacles.contains(&final_head1);
        let hit_obstacle2 = final_head2_opt.is_some_and(|fh2| self.obstacles.contains(&fh2));
        let out_of_bounds1 = if self.mode == GameMode::BattleRoyale {
            final_head1.x <= self.safe_zone_margin
                || final_head1.x >= self.width - 1 - self.safe_zone_margin
                || final_head1.y <= self.safe_zone_margin
                || final_head1.y >= self.height - 1 - self.safe_zone_margin
        } else {
            false
        };
        let out_of_bounds2 = if self.mode == GameMode::BattleRoyale {
            final_head2_opt.is_some_and(|fh2| {
                fh2.x <= self.safe_zone_margin
                    || fh2.x >= self.width - 1 - self.safe_zone_margin
                    || fh2.y <= self.safe_zone_margin
                    || fh2.y >= self.height - 1 - self.safe_zone_margin
            })
        } else {
            false
        };
        let hit_boss1 = self
            .bosses
            .iter()
            .any(|b| b.position == final_head1 || self.snake.body_map.contains_key(&b.position));
        let hit_laser1 = self.lasers.iter().any(|l| l.player != 1 && l.position == final_head1);
        let hit_laser2 = final_head2_opt
            .is_some_and(|fh2| self.lasers.iter().any(|l| l.player != 2 && l.position == fh2));
        if hit_wall1 || out_of_bounds1 {
            p1_dead = true;
        }
        if hit_obstacle1 && !is_invincible {
            if self.mode == GameMode::Miner {
                self.obstacles.remove(&final_head1);
                self.spawn_particles(
                    f32::from(final_head1.x),
                    f32::from(final_head1.y),
                    10,
                    crate::color::Color::DarkGrey,
                    '*',
                );
                crate::game::beep();

                // Chance to drop resource or coins
                let rand_val = self.rng.gen_range(0..100);
                if rand_val < 10 {
                    let res = match self.rng.gen_range(0..100) {
                        0..=40 => Resource::Wood,
                        41..=70 => Resource::Iron,
                        71..=90 => Resource::Gold,
                        _ => Resource::Diamond,
                    };
                    self.resources.insert(final_head1, res);
                } else if rand_val < 20 {
                    self.stats.coins += 5;
                    self.spawn_floating_text(
                        f32::from(final_head1.x),
                        f32::from(final_head1.y),
                        "+5".to_string(),
                        crate::color::Color::Yellow,
                    );
                } else if rand_val < 25 {
                    self.food = final_head1;
                }
            } else if self.skin == '🦍' {
                self.obstacles.remove(&final_head1);
                self.spawn_particles(
                    f32::from(final_head1.x),
                    f32::from(final_head1.y),
                    20,
                    crate::color::Color::Red,
                    'X',
                );
                crate::game::beep();
            } else if self.stats.equipped_vehicle == Some(crate::game::Vehicle::Tank) {
                self.obstacles.remove(&final_head1);
                self.spawn_particles(
                    f32::from(final_head1.x),
                    f32::from(final_head1.y),
                    30,
                    crate::color::Color::DarkGrey,
                    '*',
                );
                self.stats.equipped_vehicle = None; // Unequip to balance, like a 1-time shield
                crate::game::beep();
            } else if self.stats.equipped_gear == Some(crate::game::Equipment::HeavyArmor) {
                self.obstacles.remove(&final_head1);
                self.spawn_particles(
                    f32::from(final_head1.x),
                    f32::from(final_head1.y),
                    20,
                    crate::color::Color::Red,
                    'X',
                );
                self.stats.equipped_gear = None;
                crate::game::beep();
            } else {
                p1_dead = true;
            }
        }
        if hit_boss1 && !is_invincible {
            if self.stats.equipped_gear == Some(crate::game::Equipment::SpikedHelmet) {
                // Deal 5 damage to boss
                let mut boss_died = false;
                for boss in &mut self.bosses {
                    if boss.position == final_head1
                        || self.snake.body_map.contains_key(&boss.position)
                    {
                        boss.health = boss.health.saturating_sub(5);
                        if boss.health == 0 {
                            boss_died = true;
                        }
                    }
                }
                if boss_died {
                    let mut next_bosses = Vec::new();
                    for boss in std::mem::take(&mut self.bosses) {
                        if boss.health == 0 {
                            *self.stats.bestiary.entry(boss.kind).or_insert(0) += 1;
                            self.update_quest_progress(crate::game::QuestType::SlayBosses, 1);
                            if self.rng.gen_bool(0.2) {
                                self.equipment_boxes.push(boss.position);
                            }
                            self.update_bounty_progress(crate::game::BountyType::KillBosses(0), 1);
                            self.score += 100;
                            if self.stats.faction.is_some() {
                                self.stats.faction_rep += 100;
                            }
                            self.spawn_particles(
                                f32::from(boss.position.x),
                                f32::from(boss.position.y),
                                30,
                                crate::color::Color::Magenta,
                                'B',
                            );
                        } else {
                            next_bosses.push(boss);
                        }
                    }
                    self.bosses = next_bosses;
                }
                crate::game::beep();
            } else {
                p1_dead = true;
            }
        }
        if hit_laser1 && !is_invincible {
            p1_dead = true;
        }
        if self.lightning_column.is_some_and(|col| final_head1.x == col) && !is_invincible {
            p1_dead = true;
        }
        if hit_wall2 || out_of_bounds2 {
            p2_dead = true;
        }
        if hit_obstacle2 && !is_invincible {
            if self.mode == GameMode::Miner {
                if let Some(fh2) = final_head2_opt {
                    self.obstacles.remove(&fh2);
                    self.spawn_particles(
                        f32::from(fh2.x),
                        f32::from(fh2.y),
                        10,
                        crate::color::Color::DarkGrey,
                        '*',
                    );
                    crate::game::beep();

                    // Chance to drop resource or coins
                    let rand_val = self.rng.gen_range(0..100);
                    if rand_val < 10 {
                        let res = match self.rng.gen_range(0..100) {
                            0..=40 => Resource::Wood,
                            41..=70 => Resource::Iron,
                            71..=90 => Resource::Gold,
                            _ => Resource::Diamond,
                        };
                        self.resources.insert(fh2, res);
                    } else if rand_val < 20 {
                        self.stats.coins += 5;
                        self.spawn_floating_text(
                            f32::from(fh2.x),
                            f32::from(fh2.y),
                            "+5".to_string(),
                            crate::color::Color::Yellow,
                        );
                    } else if rand_val < 25 {
                        self.food = fh2;
                    }
                }
            } else {
                p2_dead = true;
            }
        }
        if hit_laser2 && !is_invincible {
            p2_dead = true;
        }
        if final_head2_opt
            .is_some_and(|head| self.lightning_column.is_some_and(|col| head.x == col))
            && !is_invincible
        {
            p2_dead = true;
        }
        if let Some(final_head2) = final_head2_opt
            && final_head1 == final_head2
        {
            p1_dead = true;
            p2_dead = true;
        }
        let mut exploded_mines = Vec::new();
        let hit_goblin1 = self.goblin.is_some_and(|g| final_head1 == g.position);
        let hit_goblin2 =
            final_head2_opt.is_some_and(|fh2| self.goblin.is_some_and(|g| fh2 == g.position));
        if hit_goblin1 || hit_goblin2 {
            let gob_pos = self.goblin.unwrap().position;
            self.goblin = None;
            let multiplier = if self.skin == '₿' {
                2
            } else {
                1
            };
            self.score += 500;
            self.stats.total_score += 500;
            self.stats.coins += 500 * multiplier;
            self.spawn_floating_text(
                f32::from(final_head1.x),
                f32::from(final_head1.y),
                "+500".to_string(),
                crate::color::Color::Yellow,
            );
            let spawn_x = if hit_goblin1 {
                f32::from(final_head1.x)
            } else {
                final_head2_opt.map_or(0.0, |fh| f32::from(fh.x))
            };
            let spawn_y = if hit_goblin1 {
                f32::from(final_head1.y)
            } else {
                final_head2_opt.map_or(0.0, |fh| f32::from(fh.y))
            };
            self.spawn_particles(spawn_x, spawn_y, 50, crate::color::Color::Yellow, '$');
            if self.mode == GameMode::SnakeSurvivor {
                self.xp_gems.insert(gob_pos);
            }
            beep();
        }
        let hit_mine1 = self.mines.contains(&final_head1);
        let hit_mine2 = final_head2_opt.is_some_and(|fh2| self.mines.contains(&fh2));
        if hit_mine1 && !is_invincible {
            exploded_mines.push(final_head1);
            p1_dead = true;
        }
        if hit_mine2 && !is_invincible {
            if let Some(fh2) = final_head2_opt {
                exploded_mines.push(fh2);
            }
            p2_dead = true;
        }
        let hit_black_hole1 = self.black_hole.is_some_and(|bh| final_head1 == bh);
        let hit_black_hole2 =
            final_head2_opt.is_some_and(|fh2| self.black_hole.is_some_and(|bh| fh2 == bh));
        let mut hit_meteor1 = false;
        let mut hit_meteor2 = false;
        for meteor in &self.meteors {
            if meteor.position == final_head1 || self.snake.body_map.contains_key(&meteor.position)
            {
                hit_meteor1 = true;
            }
            if let Some(fh2) = final_head2_opt
                && (meteor.position == fh2
                    || self
                        .player2
                        .as_ref()
                        .is_some_and(|p2| p2.body_map.contains_key(&meteor.position)))
            {
                hit_meteor2 = true;
            }
        }
        if hit_meteor1 && !is_invincible {
            p1_dead = true;
            self.spawn_particles(
                f32::from(final_head1.x),
                f32::from(final_head1.y),
                30,
                crate::color::Color::DarkYellow,
                '*',
            );
        }
        if hit_meteor2 && !is_invincible {
            p2_dead = true;
            if let Some(fh2) = final_head2_opt {
                self.spawn_particles(
                    f32::from(fh2.x),
                    f32::from(fh2.y),
                    30,
                    crate::color::Color::DarkYellow,
                    '*',
                );
            }
        }
        if hit_black_hole1 && !is_invincible {
            p1_dead = true;
            if let Some(bh) = self.black_hole {
                self.spawn_particles(
                    f32::from(bh.x),
                    f32::from(bh.y),
                    30,
                    crate::color::Color::DarkGrey,
                    'O',
                );
            }
        }
        if hit_black_hole2 && !is_invincible {
            p2_dead = true;
            if let Some(bh) = self.black_hole {
                self.spawn_particles(
                    f32::from(bh.x),
                    f32::from(bh.y),
                    30,
                    crate::color::Color::DarkGrey,
                    'O',
                );
            }
        }
        for mine in exploded_mines {
            self.mines.remove(&mine);
            self.spawn_particles(
                f32::from(mine.x),
                f32::from(mine.y),
                40,
                crate::color::Color::Red,
                'X',
            );
            beep();
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let cx = i32::from(mine.x) + dx;
                    let cy = i32::from(mine.y) + dy;
                    if cx > 0
                        && cx < i32::from(self.width - 1)
                        && cy > 0
                        && cy < i32::from(self.height - 1)
                    {
                        let p = Point {
                            x: u16::try_from(cx).unwrap_or(0),
                            y: u16::try_from(cy).unwrap_or(0),
                        };
                        self.obstacles.remove(&p);
                        self.mines.remove(&p);
                        let mut next_bosses = Vec::new();
                        let mut new_lasers = Vec::new();
                        for mut boss in std::mem::take(&mut self.bosses) {
                            if boss.position == p {
                                boss.health = boss.health.saturating_sub(5);
                                if boss.health == 0 {
                                    *self.stats.bestiary.entry(boss.kind).or_insert(0) += 1;
                                    self.update_quest_progress(
                                        crate::game::QuestType::SlayBosses,
                                        1,
                                    );
                                    if self.rng.gen_bool(0.2) {
                                        self.equipment_boxes.push(boss.position);
                                    }
                                    if self.stats.equipped_class
                                        == Some(crate::game::HeroClass::Necromancer)
                                    {
                                        self.companion = Some(Companion {
                                            position: boss.position,
                                            kind: crate::game::CompanionType::Fighter,
                                            move_timer: 0,
                                            action_timer: 0,
                                            path: Vec::new(),
                                        });
                                        crate::game::beep();
                                    }

                                    if self.mode == GameMode::SnakeSurvivor {
                                        self.xp_gems.insert(boss.position);
                                    }

                                    if boss.kind == BossType::Splitter && boss.max_health > 5 {
                                        let half_max = boss.max_health / 2;
                                        let child1_pos = Point {
                                            x: boss.position.x.saturating_sub(1).max(1),
                                            y: boss.position.y,
                                        };
                                        let child2_pos = Point {
                                            x: (boss.position.x + 1).min(self.width - 2),
                                            y: boss.position.y,
                                        };
                                        next_bosses.push(Boss {
                                            position: child1_pos,
                                            health: half_max,
                                            max_health: half_max,
                                            move_timer: 0,
                                            shoot_timer: 0,
                                            kind: BossType::Splitter,
                                            state_timer: 0,
                                        });
                                        next_bosses.push(Boss {
                                            position: child2_pos,
                                            health: half_max,
                                            max_health: half_max,
                                            move_timer: 0,
                                            shoot_timer: 0,
                                            kind: BossType::Splitter,
                                            state_timer: 0,
                                        });
                                    } else {
                                        self.update_bounty_progress(
                                            crate::game::BountyType::KillBosses(0),
                                            1,
                                        );
                                        if self.mode == GameMode::BossRush {
                                            self.score += 1000 * self.campaign_level;
                                            self.campaign_level += 1;
                                        } else {
                                            self.score += 100;
                                        }
                                        if self.stats.faction.is_some() {
                                            self.stats.faction_rep += 100;
                                        }
                                        let boss_pos = boss.position;
                                        let margin = if self.mode == GameMode::BattleRoyale {
                                            self.safe_zone_margin
                                        } else {
                                            0
                                        };
                                        for &dir in &[
                                            Direction::Up,
                                            Direction::Down,
                                            Direction::Left,
                                            Direction::Right,
                                        ] {
                                            let laser_pos =
                                                Self::calculate_next_head_dir(boss_pos, dir);
                                            if laser_pos.x > margin
                                                && laser_pos.x < self.width - 1 - margin
                                                && laser_pos.y > margin
                                                && laser_pos.y < self.height - 1 - margin
                                            {
                                                new_lasers.push(Laser {
                                                    position: laser_pos,
                                                    direction: dir,
                                                    player: 3,
                                                });
                                            }
                                        }
                                    }
                                } else {
                                    next_bosses.push(boss);
                                }
                            } else {
                                next_bosses.push(boss);
                            }
                        }
                        self.bosses = next_bosses;
                        self.lasers.extend(new_lasers);
                    }
                }
            }
        }
        let old_food_eaten_session = self.food_eaten_session;
        let is_multiplier = self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::ScoreMultiplier
                && p.activation_time.is_some_and(|t| {
                    web_time::SystemTime::now()
                        .duration_since(web_time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .saturating_sub(t)
                        < self.powerup_duration()
                })
        });
        let mut p1_grow = self.check_bonus_food_collision(final_head1, is_multiplier)
            || self.check_crop_collision(final_head1, is_multiplier)
            || self.mode == GameMode::Tron;

        let mut p2_grow = final_head2_opt.is_some_and(|fh2| {
            self.check_bonus_food_collision(fh2, is_multiplier)
                || self.check_crop_collision(fh2, is_multiplier)
        }) || self.mode == GameMode::Tron;
        self.check_poison_food_collision(final_head1, 1);
        if let Some(final_head2) = final_head2_opt {
            self.check_poison_food_collision(final_head2, 2);
        }
        if final_head1 == self.food {
            p1_grow = true;
            if !self.process_food_collision(final_head1, is_multiplier) {
                self.snake.move_to(final_head1, p1_grow);
                self.handle_win();
                return;
            }
        } else if let Some(final_head2) = final_head2_opt
            && final_head2 == self.food
        {
            p2_grow = true;
            if !self.process_food_collision(final_head2, is_multiplier) {
                if let Some(p2) = &mut self.player2 {
                    p2.move_to(final_head2, p2_grow);
                }
                self.handle_win();
                return;
            }
        }
        let mut bots_to_remove = std::collections::HashSet::new();
        let mut bots_grow = vec![self.mode == GameMode::Tron; self.bots.len()];
        let bots_len_start = self.bots.len();
        for (i, final_head, hit_wall) in &final_bot_heads {
            if *hit_wall {
                bots_to_remove.insert(*i);
                continue;
            }
            if self.obstacles.contains(final_head) {
                bots_to_remove.insert(*i);
                continue;
            }
            if self.bosses.iter().any(|b| b.position == *final_head) {
                bots_to_remove.insert(*i);
                continue;
            }
            if self.mines.contains(final_head) {
                bots_to_remove.insert(*i);
                continue;
            }
            if self.black_hole.is_some_and(|bh| bh == *final_head) {
                bots_to_remove.insert(*i);
                continue;
            }
            if self.lasers.iter().any(|l| l.position == *final_head) {
                bots_to_remove.insert(*i);
                continue;
            }
            // Check body collisions for bot
            if self.snake.body_map.contains_key(final_head) {
                bots_to_remove.insert(*i);
                continue;
            }
            if let Some(p2) = &self.player2
                && p2.body_map.contains_key(final_head)
            {
                bots_to_remove.insert(*i);
                continue;
            }
            for j in 0..self.bots.len() {
                if *i != j && self.bots[j].body_map.contains_key(final_head) {
                    bots_to_remove.insert(*i);
                    break;
                }
            }

            // Food logic for bot
            if *final_head == self.food {
                bots_grow[*i] = true;
                if let Some(new_food) = Self::get_random_empty_point(
                    self.width,
                    self.height,
                    &self.snake,
                    |p| {
                        self.obstacles.contains(p)
                            || self.snake.body_map.contains_key(p)
                            || self.bots.iter().any(|b| b.body_map.contains_key(p))
                    },
                    &mut self.rng,
                    self.safe_zone_margin,
                ) {
                    self.food = new_food;
                }
            }
            if self.bonus_food.is_some_and(|(bp, _)| *final_head == bp) {
                bots_grow[*i] = true;
                self.bonus_food = None;
            }
            if self.poison_food.is_some_and(|(pp, _)| *final_head == pp) {
                self.poison_food = None;
            }
        }

        let (body_p1_dead, body_p2_dead) = self.check_body_collisions(
            final_head1,
            final_head2_opt,
            is_invincible,
            p1_grow,
            p2_grow,
        );
        if body_p1_dead {
            p1_dead = true;
        }
        if body_p2_dead {
            p2_dead = true;
        }
        if p1_dead && p2_dead {
            if self.mode == GameMode::PlayerVsBot {
                self.update_elo(false, true);
                self.save_stats();
            }
            if self.mode == GameMode::CaptureTheFlag {
                self.p1_has_flag = false;
                self.p2_has_flag = false;
                self.p1_flag = Some(Point {
                    x: 2,
                    y: self.height / 2,
                });
                self.p2_flag = Some(Point {
                    x: self.width.saturating_sub(3),
                    y: self.height / 2,
                });
                self.respawn();
                return;
            }
            self.handle_death("Draw! Both snakes died!");
            return;
        } else if p1_dead {
            if self.mode == GameMode::CaptureTheFlag {
                self.p1_has_flag = false;
                self.p2_flag = Some(Point {
                    x: self.width.saturating_sub(3),
                    y: self.height / 2,
                });
                self.respawn();
                return;
            }
            if self.mode == GameMode::SinglePlayer
                || self.mode == GameMode::TimeAttack
                || self.mode == GameMode::Speedrun
                || self.mode == GameMode::Survival
                || self.mode == GameMode::DailyChallenge
                || self.mode == GameMode::WeeklyChallenge
                || self.mode == GameMode::BossRush
                || self.mode == GameMode::Vampire
                || self.mode == GameMode::Gravity
                || self.mode == GameMode::KingOfTheHill
            {
                self.handle_death("You Died!");
            } else {
                if self.mode == GameMode::PlayerVsBot {
                    self.update_elo(false, false);
                    self.save_stats();
                }
                self.handle_death("Player 2 Wins!");
            }
            return;
        } else if p2_dead {
            if self.mode == GameMode::CaptureTheFlag {
                self.p2_has_flag = false;
                self.p1_flag = Some(Point {
                    x: 2,
                    y: self.height / 2,
                });
                self.respawn();
                return;
            }
            if self.mode == GameMode::PlayerVsBot {
                self.update_elo(true, false);
                self.save_stats();
            }
            self.handle_death("Player 1 Wins!");
            return;
        }

        if self.mode == GameMode::CaptureTheFlag {
            let p1_base = Point {
                x: 2,
                y: self.height / 2,
            };
            let p2_base = Point {
                x: self.width.saturating_sub(3),
                y: self.height / 2,
            };

            // Player 1 logic
            if !self.p1_has_flag && Some(final_head1) == self.p2_flag {
                self.p1_has_flag = true;
                self.p2_flag = None;
                beep();
            } else if self.p1_has_flag && final_head1 == p1_base {
                self.p1_score += 1;
                self.p1_has_flag = false;
                self.p2_flag = Some(p2_base);
                beep();
            }

            // Player 2 logic
            if let Some(fh2) = final_head2_opt {
                if !self.p2_has_flag && Some(fh2) == self.p1_flag {
                    self.p2_has_flag = true;
                    self.p1_flag = None;
                    beep();
                } else if self.p2_has_flag && fh2 == p2_base {
                    self.p2_score += 1;
                    self.p2_has_flag = false;
                    self.p1_flag = Some(p1_base);
                    beep();
                }
            }
        }

        if self.mode == GameMode::KingOfTheHill
            && let Some(koth_pos) = self.koth_zone
        {
            let mut p1_in_zone = false;
            let mut p2_in_zone = false;

            if final_head1.x >= koth_pos.x.saturating_sub(1)
                && final_head1.x <= koth_pos.x + 1
                && final_head1.y >= koth_pos.y.saturating_sub(1)
                && final_head1.y <= koth_pos.y + 1
            {
                p1_in_zone = true;
            }
            if let Some(fh2) = final_head2_opt
                && fh2.x >= koth_pos.x.saturating_sub(1)
                && fh2.x <= koth_pos.x + 1
                && fh2.y >= koth_pos.y.saturating_sub(1)
                && fh2.y <= koth_pos.y + 1
            {
                p2_in_zone = true;
            }

            if p1_in_zone && !p2_in_zone {
                self.score += 1;
                self.p1_score += 1;
            } else if p2_in_zone && !p1_in_zone {
                self.p2_score += 1;
            }

            let mut bot_in_zone = false;
            for bot in &self.bots {
                let bh = bot.head();
                if bh.x >= koth_pos.x.saturating_sub(1)
                    && bh.x <= koth_pos.x + 1
                    && bh.y >= koth_pos.y.saturating_sub(1)
                    && bh.y <= koth_pos.y + 1
                {
                    bot_in_zone = true;
                    break;
                }
            }

            // In KingOfTheHill, bots in the zone act as blockers preventing players from scoring
            // Let's negate player scores if bots are also in the zone
            if bot_in_zone {
                if p1_in_zone && !p2_in_zone {
                    self.score = self.score.saturating_sub(1);
                    self.p1_score = self.p1_score.saturating_sub(1);
                } else if p2_in_zone && !p1_in_zone {
                    self.p2_score = self.p2_score.saturating_sub(1);
                }
            }

            if self.tick_counter.is_multiple_of(50) {
                let margin = 2;
                if let Some(new_zone) = Self::get_random_empty_point(
                    self.width,
                    self.height,
                    &self.snake,
                    |_| false,
                    &mut self.rng,
                    margin,
                ) {
                    self.koth_zone = Some(new_zone);
                }
            }
        }

        self.process_power_up_collision(final_head1);

        if let Some(final_head2) = final_head2_opt {
            self.process_power_up_collision(final_head2);
        }
        self.process_resource_collision(final_head1);
        if let Some(final_head2) = final_head2_opt {
            self.process_resource_collision(final_head2);
        }
        self.process_egg_collision(final_head1);
        if let Some(final_head2) = final_head2_opt {
            self.process_egg_collision(final_head2);
        }

        if self.mode == GameMode::SnakeSurvivor {
            if self.xp_gems.contains(&final_head1) {
                self.xp_gems.remove(&final_head1);
                self.gain_xp(10);
                self.spawn_particles(
                    f32::from(final_head1.x),
                    f32::from(final_head1.y),
                    10,
                    crate::color::Color::Cyan,
                    '+',
                );
                crate::game::beep();
            }
            if let Some(final_head2) = final_head2_opt
                && self.xp_gems.contains(&final_head2)
            {
                self.xp_gems.remove(&final_head2);
                self.gain_xp(10);
                self.spawn_particles(
                    f32::from(final_head2.x),
                    f32::from(final_head2.y),
                    10,
                    crate::color::Color::Cyan,
                    '+',
                );
                crate::game::beep();
            }
        }

        if let Some((_, mut timer)) = self.stats.incubator
            && timer > 0
        {
            timer -= 1;
            self.stats.incubator.as_mut().unwrap().1 = timer;
            if timer == 0 {
                self.stats.incubator = None;
                let possible_companions = [
                    crate::game::CompanionType::Collector,
                    crate::game::CompanionType::Fighter,
                    crate::game::CompanionType::Healer,
                    crate::game::CompanionType::Sniper,
                ];
                let comp = possible_companions[self.rng.gen_range(0..possible_companions.len())];
                if self.stats.unlocked_companions.contains(&comp) {
                    self.stats.coins += 500;
                    self.chat_log.push_back((
                        "SYSTEM: Your egg hatched a duplicate companion. (+500 Coins)".to_string(),
                        crate::color::Color::Yellow,
                    ));
                } else {
                    self.stats.unlocked_companions.push(comp);
                    self.chat_log.push_back((
                        format!("SYSTEM: Your egg hatched a {comp:?}!"),
                        crate::color::Color::Yellow,
                    ));
                }
                crate::game::beep();
            }
        }
        self.process_equipment_box_collision(final_head1);
        if let Some(final_head2) = final_head2_opt {
            self.process_equipment_box_collision(final_head2);
        }

        if self.merchant.is_some_and(|m| m == final_head1) {
            self.state = GameState::MerchantShop;
            self.merchant = None;
            beep();
        } else if let Some(final_head2) = final_head2_opt
            && self.merchant.is_some_and(|m| m == final_head2)
        {
            self.state = GameState::MerchantShop;
            self.merchant = None;
            beep();
        }

        self.add_obstacles_if_needed(old_food_eaten_session, final_head1);

        if self.stats.equipped_vehicle == Some(crate::game::Vehicle::Car)
            && self.rng.gen_bool(0.05)
            && let Some(tail) = self.snake.body.back()
        {
            self.obstacles.insert(*tail);
        }

        self.snake.move_to(final_head1, p1_grow);
        if let Some(final_head2) = final_head2_opt
            && let Some(p2) = &mut self.player2
        {
            p2.move_to(final_head2, p2_grow);
        }

        let old_bots = std::mem::take(&mut self.bots);
        let mut old_paths = std::mem::take(&mut self.bots_autopilot_paths);
        let mut alive_bots = Vec::new();
        let mut alive_paths = Vec::new();
        for (i, bot) in old_bots.into_iter().enumerate() {
            if !bots_to_remove.contains(&i) {
                let mut b = bot;
                // Since eating food may spawn new bots during update_tick and process_food_collision,
                // final_bot_heads may not cover the newly added bots at the end of the array.
                // We only move bots that existed at the start of the tick.
                if i < bots_len_start
                    && let Some(pos) = final_bot_heads.iter().position(|&(idx, _, _)| idx == i)
                {
                    let mut final_head = final_bot_heads[pos].1;
                    if let Some(wrapped) = self.get_final_p(final_head) {
                        final_head = wrapped;
                    }
                    b.move_to(final_head, bots_grow[i]);
                }
                alive_bots.push(b);
                alive_paths.push(std::mem::take(&mut old_paths[i]));
            }
        }
        self.bots = alive_bots;
        self.bots_autopilot_paths = alive_paths;

        if let Some(mut ghost) = self.ghost_snake.take() {
            if let Some(ghost_dir) = self.ghost_moves.pop_front() {
                ghost.direction = ghost_dir;
                let mut next_ghost_head = Self::calculate_next_head_dir(ghost.head(), ghost_dir);
                if let Some(final_ghost_head) = self.get_final_p(next_ghost_head) {
                    next_ghost_head = final_ghost_head;
                }
                ghost.move_to(next_ghost_head, false);
            }
            self.ghost_snake = Some(ghost);
        }
    }
    fn check_body_collisions(
        &self,
        final_head1: Point,
        final_head2_opt: Option<Point>,
        is_invincible: bool,
        p1_grow: bool,
        p2_grow: bool,
    ) -> (bool, bool) {
        let mut p1_dead = false;
        let mut p2_dead = false;
        if self.snake.body_map.contains_key(&final_head1) && !is_invincible {
            let is_tail = self.snake.body.back().is_some_and(|tail| final_head1 == *tail);
            if !p1_grow && is_tail {
            } else {
                p1_dead = true;
            }
        }
        if let Some(final_head2) = final_head2_opt
            && let Some(p2) = &self.player2
            && p2.body_map.contains_key(&final_head2)
            && !is_invincible
        {
            let is_tail = p2.body.back().is_some_and(|tail| final_head2 == *tail);
            if !p2_grow && is_tail {
            } else {
                p2_dead = true;
            }
        }
        if let Some(final_head2) = final_head2_opt {
            if self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(&final_head1))
                && !is_invincible
            {
                let is_tail = self
                    .player2
                    .as_ref()
                    .unwrap()
                    .body
                    .back()
                    .is_some_and(|tail| final_head1 == *tail);
                if !p2_grow && is_tail {
                } else {
                    p1_dead = true;
                }
            }
            if self.snake.body_map.contains_key(&final_head2) && !is_invincible {
                let is_tail = self.snake.body.back().is_some_and(|tail| final_head2 == *tail);
                if !p1_grow && is_tail {
                } else {
                    p2_dead = true;
                }
            }
        }

        // Player hitting bots
        if !is_invincible {
            for bot in &self.bots {
                if bot.body_map.contains_key(&final_head1) {
                    p1_dead = true;
                }
                if let Some(final_head2) = final_head2_opt
                    && bot.body_map.contains_key(&final_head2)
                {
                    p2_dead = true;
                }
            }
        }

        (p1_dead, p2_dead)
    }
    #[expect(clippy::too_many_lines, reason = "Handling many powerup types")]
    fn process_power_up_collision(&mut self, final_head: Point) {
        let hit_power_up = if let Some(p) = self.power_up.as_ref()
            && final_head == p.location
        {
            true
        } else {
            false
        };
        if hit_power_up {
            self.spawn_particles(
                f32::from(final_head.x),
                f32::from(final_head.y),
                20,
                crate::color::Color::Yellow,
                '*',
            );
        }
        if let Some(p) = self.power_up.as_mut()
            && final_head == p.location
        {
            if p.p_type == PowerUpType::ExtraLife {
                self.lives += 1;
            } else if p.p_type == PowerUpType::Shrink {
                self.snake.shrink_tail();
            } else if p.p_type == PowerUpType::ClearObstacles {
                self.obstacles.clear();
            } else if p.p_type == PowerUpType::Decoy {
                self.decoy = Some((
                    self.snake.head(),
                    web_time::SystemTime::now()
                        .duration_since(web_time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                ));
            } else if p.p_type == PowerUpType::Emp {
                self.mines.clear();
                self.lasers.clear();
                for boss in &mut self.bosses {
                    boss.state_timer = 30;
                }
            } else if p.p_type == PowerUpType::Nuke {
                for boss in std::mem::take(&mut self.bosses) {
                    *self.stats.bestiary.entry(boss.kind).or_insert(0) += 1;
                    self.update_quest_progress(crate::game::QuestType::SlayBosses, 1);
                    if self.rng.gen_bool(0.2) {
                        self.equipment_boxes.push(boss.position);
                    }
                    if self.stats.equipped_class == Some(crate::game::HeroClass::Necromancer) {
                        self.companion = Some(Companion {
                            position: boss.position,
                            kind: crate::game::CompanionType::Fighter,
                            move_timer: 0,
                            action_timer: 0,
                            path: Vec::new(),
                        });
                        crate::game::beep();
                    }
                    self.update_bounty_progress(crate::game::BountyType::KillBosses(0), 1);
                    self.score += 100;
                    if self.stats.faction.is_some() {
                        self.stats.faction_rep += 100;
                    }
                }
                self.bosses.clear();
                self.resources.clear();
                self.meteors.clear();
                self.mines.clear();
                self.lasers.clear();
                self.obstacles.clear();
                self.poison_food = None;
                self.bonus_food = None;
            } else if p.p_type == PowerUpType::Teleport {
                let avoid = |pt: &Point| {
                    self.obstacles.contains(pt)
                        || *pt == self.food
                        || self.bonus_food.is_some_and(|(bp, _)| *pt == bp)
                };
                if let Some(new_pos) = Self::get_random_empty_point(
                    self.width,
                    self.height,
                    &self.snake,
                    avoid,
                    &mut self.rng,
                    self.safe_zone_margin,
                ) {
                    let old_head = self.snake.head();
                    let dx = i32::from(new_pos.x) - i32::from(old_head.x);
                    let dy = i32::from(new_pos.y) - i32::from(old_head.y);
                    for part in &mut self.snake.body {
                        let new_x = i32::from(part.x) + dx;
                        let new_y = i32::from(part.y) + dy;
                        let inner_width = i32::from(self.width) - 2;
                        let inner_height = i32::from(self.height) - 2;
                        let mut nx = (new_x - 1) % inner_width;
                        if nx < 0 {
                            nx += inner_width;
                        }
                        let mut ny = (new_y - 1) % inner_height;
                        if ny < 0 {
                            ny += inner_height;
                        }
                        part.x = u16::try_from(nx + 1).unwrap_or(1);
                        part.y = u16::try_from(ny + 1).unwrap_or(1);
                    }
                    self.snake.rebuild_map();
                }
            } else {
                p.activation_time = Some(
                    web_time::SystemTime::now()
                        .duration_since(web_time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                );
            }
            beep();
        }
        if let Some(p) = self.power_up.as_ref()
            && (p.p_type == PowerUpType::ExtraLife
                || p.p_type == PowerUpType::Shrink
                || p.p_type == PowerUpType::ClearObstacles
                || p.p_type == PowerUpType::Teleport
                || p.p_type == PowerUpType::Decoy
                || p.p_type == PowerUpType::Emp
                || p.p_type == PowerUpType::Nuke)
            && p.activation_time.is_none()
            && final_head == p.location
        {
            self.power_up = None;
            if self.stats.equipped_class == Some(crate::game::HeroClass::Mage) {
                self.power_up = Some(PowerUp {
                    p_type: PowerUpType::TimeFreeze,
                    location: Point {
                        x: 0,
                        y: 0,
                    },
                    activation_time: Some(
                        web_time::SystemTime::now()
                            .duration_since(web_time::SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    ),
                });
            }
        }
    }
    fn process_equipment_box_collision(&mut self, final_head: Point) {
        if let Some(pos) = self.equipment_boxes.iter().position(|&p| p == final_head) {
            self.equipment_boxes.remove(pos);
            self.spawn_particles(
                f32::from(final_head.x),
                f32::from(final_head.y),
                30,
                crate::color::Color::Cyan,
                'E',
            );

            let possible_equipment = [
                crate::game::Equipment::SpikedHelmet,
                crate::game::Equipment::HeavyArmor,
                crate::game::Equipment::SpeedTail,
                crate::game::Equipment::MagnetRing,
            ];
            let item = possible_equipment[self.rng.gen_range(0..possible_equipment.len())];

            if !self.stats.unlocked_equipment.contains(&item) {
                self.stats.unlocked_equipment.push(item);
                self.save_stats();
            }
            crate::game::beep();
        }
    }

    fn check_poison_food_collision(&mut self, final_head: Point, player: u8) {
        if self.poison_food.is_some_and(|(poison_p, _)| final_head == poison_p) {
            if !self.stats.unlocked_achievements.contains(&Achievement::PoisonEater) {
                self.stats.unlocked_achievements.push(Achievement::PoisonEater);
                self.save_stats();
            }
            self.spawn_particles(
                f32::from(final_head.x),
                f32::from(final_head.y),
                15,
                crate::color::Color::Magenta,
                'X',
            );
            self.score = self.score.saturating_sub(10);
            if player == 1 {
                self.snake.shrink_tail();
            } else if player == 2
                && let Some(p2) = &mut self.player2
            {
                p2.shrink_tail();
            }
            self.poison_food = None;
            beep();
        }
    }
    fn process_egg_collision(&mut self, final_head: Point) {
        if let Some(egg) = self.eggs_on_board.remove(&final_head) {
            self.spawn_particles(
                f32::from(final_head.x),
                f32::from(final_head.y),
                15,
                crate::color::Color::White,
                'O',
            );
            *self.stats.inventory_eggs.entry(egg).or_insert(0) += 1;
            self.score += 50;
            self.stats.total_score += 50;
            self.spawn_floating_text(
                f32::from(final_head.x),
                f32::from(final_head.y),
                "+50".to_string(),
                crate::color::Color::White,
            );
            beep();
        }
    }

    fn process_resource_collision(&mut self, final_head: Point) {
        if let Some(res) = self.resources.remove(&final_head) {
            let color = match res {
                Resource::Wood | Resource::Gold => crate::color::Color::Yellow,
                Resource::Iron => crate::color::Color::White,
                Resource::Diamond => crate::color::Color::Cyan,
            };
            self.spawn_particles(f32::from(final_head.x), f32::from(final_head.y), 15, color, '*');
            *self.stats.inventory.entry(res).or_insert(0) += 1;
            let points = match res {
                Resource::Wood => 10,
                Resource::Iron => 20,
                Resource::Gold => 50,
                Resource::Diamond => 100,
            };
            self.score += points;
            self.stats.total_score += points;
            self.spawn_floating_text(
                f32::from(final_head.x),
                f32::from(final_head.y),
                format!("+{points}"),
                color,
            );
            beep();
        }
    }

    fn check_crop_collision(&mut self, final_head: Point, is_multiplier: bool) -> bool {
        let mut crop_eaten = false;
        let mut new_crops = Vec::new();
        for crop in &self.crops {
            if crop.position == final_head {
                if crop.growth_stage == 2 {
                    crop_eaten = true;
                } else {
                    new_crops.push(crop.clone());
                }
            } else {
                new_crops.push(crop.clone());
            }
        }
        self.crops = new_crops;

        if crop_eaten {
            self.spawn_particles(
                f32::from(final_head.x),
                f32::from(final_head.y),
                25,
                crate::color::Color::Yellow,
                '¥',
            );

            let added_score = if is_multiplier {
                50
            } else {
                25
            };
            let mut coins_earned = 50;
            if let Some(&double_coins_level) =
                self.in_game_upgrades.get(&InGameUpgrade::DoubleCoins)
            {
                coins_earned *= 1 + double_coins_level;
            }
            if self.skin == '₿' {
                coins_earned *= 2;
            }
            if self.stats.unlocked_artifacts.contains(&crate::game::Artifact::CoinAmulet) {
                coins_earned *= 2;
            }
            if self.stats.faction == Some(crate::game::Faction::EmeraldPythons) {
                let multiplier = (f64::from(self.stats.faction_rep) / 1000.0).mul_add(0.01, 1.1);
                coins_earned = (f64::from(coins_earned) * multiplier).round() as u32;
            }

            if self.stats.faction.is_some() {
                self.stats.faction_rep += 5;
            }
            self.score += added_score;
            self.stats.total_score += added_score;
            self.stats.coins += coins_earned;
            self.food_eaten_session += 1;
            self.spawn_floating_text(
                f32::from(final_head.x),
                f32::from(final_head.y),
                format!("+{added_score}"),
                crate::color::Color::Yellow,
            );
            beep();
            self.gain_xp(5);
            true
        } else {
            false
        }
    }

    fn check_bonus_food_collision(&mut self, final_head: Point, is_multiplier: bool) -> bool {
        if self.bonus_food.is_some_and(|(bonus_p, _)| final_head == bonus_p) {
            self.spawn_particles(
                f32::from(final_head.x),
                f32::from(final_head.y),
                15,
                crate::color::Color::Magenta,
                '★',
            );
            if let Some(last_time) = self.last_food_time {
                if last_time.elapsed() < Duration::from_secs(5) {
                    self.combo += 1;
                } else {
                    self.combo = 1;
                }
            } else {
                self.combo = 1;
            }
            self.last_food_time = Some(web_time::Instant::now());
            let diff_multiplier = match self.difficulty {
                Difficulty::Easy => 1,
                Difficulty::Normal => 2,
                Difficulty::Hard => 3,
                Difficulty::Insane => 5,
                Difficulty::GodMode => 10,
            };
            let mut added_score = if is_multiplier {
                10 * diff_multiplier
            } else {
                5 * diff_multiplier
            };
            added_score *= std::cmp::max(1, self.combo);
            let mut coin_multiplier =
                f64::from(self.stats.upgrade_coin_multiplier).mul_add(0.20, 1.0);
            if self.skin == '₿' {
                coin_multiplier *= 2.0;
            }
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "Score is positive and bounded"
            )]
            let mut coins_earned = (f64::from(added_score) * coin_multiplier).round() as u32;
            if let Some(&double_coins_level) =
                self.in_game_upgrades.get(&InGameUpgrade::DoubleCoins)
            {
                coins_earned *= 1 + double_coins_level;
                added_score *= 1 + double_coins_level;
            }
            if self.stats.faction == Some(crate::game::Faction::EmeraldPythons) {
                let multiplier = (f64::from(self.stats.faction_rep) / 1000.0).mul_add(0.01, 1.1);
                coins_earned = (f64::from(coins_earned) * multiplier).round() as u32;
            }

            if self.stats.faction.is_some() {
                self.stats.faction_rep += 5;
            }
            self.score += added_score;
            self.food_eaten_session += 1;
            self.spawn_floating_text(
                f32::from(final_head.x),
                f32::from(final_head.y),
                format!("+{added_score}"),
                crate::color::Color::Green,
            );
            self.stats.total_score += added_score;
            self.stats.total_food_eaten += 1;
            self.stats.coins += coins_earned;
            self.bonus_food = None;
            self.update_bounty_progress(crate::game::BountyType::EatFood(0), 1);
            self.update_quest_progress(crate::game::QuestType::CollectCoins, coins_earned);
            beep();
            self.gain_xp(1);
            true
        } else {
            false
        }
    }
    pub(crate) fn process_food_collision(
        &mut self,
        final_head: Point,
        is_multiplier: bool,
    ) -> bool {
        self.spawn_particles(
            f32::from(final_head.x),
            f32::from(final_head.y),
            8,
            crate::color::Color::Green,
            '+',
        );
        if let Some(last_time) = self.last_food_time {
            if last_time.elapsed() < Duration::from_secs(5) {
                self.combo += 1;
            } else {
                self.combo = 1;
            }
        } else {
            self.combo = 1;
        }
        self.last_food_time = Some(web_time::Instant::now());
        let diff_multiplier = match self.difficulty {
            Difficulty::Easy => 1,
            Difficulty::Normal => 2,
            Difficulty::Hard => 3,
            Difficulty::Insane => 5,
            Difficulty::GodMode => 10,
        };
        let mut added_score = if is_multiplier {
            2 * diff_multiplier
        } else {
            diff_multiplier
        };
        added_score *= std::cmp::max(1, self.combo);
        let mut coin_multiplier = f64::from(self.stats.upgrade_coin_multiplier).mul_add(0.20, 1.0);
        if self.skin == '₿' {
            coin_multiplier *= 2.0;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "Score is positive and bounded"
        )]
        let mut coins_earned = (f64::from(added_score) * coin_multiplier).round() as u32;
        if let Some(&double_coins_level) = self.in_game_upgrades.get(&InGameUpgrade::DoubleCoins) {
            coins_earned *= 1 + double_coins_level;
            added_score *= 1 + double_coins_level;
        }

        if self.stats.faction == Some(crate::game::Faction::EmeraldPythons) {
            let multiplier = (f64::from(self.stats.faction_rep) / 1000.0).mul_add(0.01, 1.1);
            coins_earned = (f64::from(coins_earned) * multiplier).round() as u32;
        }

        if self.stats.faction.is_some() {
            self.stats.faction_rep += 5;
        }
        self.score += added_score;
        self.food_eaten_session += 1;
        self.spawn_floating_text(
            f32::from(final_head.x),
            f32::from(final_head.y),
            format!("+{added_score}"),
            crate::color::Color::Green,
        );
        self.stats.total_score += added_score;
        self.stats.total_food_eaten += 1;
        self.stats.coins += coins_earned;
        self.update_bounty_progress(crate::game::BountyType::EatFood(0), 1);
        self.update_quest_progress(crate::game::QuestType::CollectCoins, coins_earned);
        beep();
        if self.mode == GameMode::Campaign && self.food_eaten_session >= self.campaign_level * 5 {
            self.campaign_level += 1;
            self.food_eaten_session = 0;
            self.obstacles = self.generate_campaign_obstacles();
            let body_map = self.snake.body_map.clone();
            self.obstacles.retain(|p| !body_map.contains_key(p));
        }
        let avoid = |p: &Point| {
            self.obstacles.contains(p)
                || *p == final_head
                || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                || self.power_up.as_ref().is_some_and(|pu| *p == pu.location)
        };
        if let Some(new_food) = Self::get_random_empty_point(
            self.width,
            self.height,
            &self.snake,
            avoid,
            &mut self.rng,
            self.safe_zone_margin,
        ) {
            self.food = new_food;
            self.gain_xp(1);

            if self.mode == GameMode::Zombie {
                let margin = self.safe_zone_margin;
                let bot_avoid = |p: &Point| {
                    self.obstacles.contains(p)
                        || self.snake.body_map.contains_key(p)
                        || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p))
                        || self.bots.iter().any(|b| b.body_map.contains_key(p))
                };
                if let Some(pos) = Self::get_random_empty_point(
                    self.width,
                    self.height,
                    &self.snake,
                    bot_avoid,
                    &mut self.rng,
                    margin,
                ) {
                    self.bots.push(Snake::new(pos));
                    self.bots_autopilot_paths.push(Vec::new());
                }
            }

            true
        } else {
            self.gain_xp(1);
            false
        }
    }
    fn add_obstacles_if_needed(&mut self, old_food_eaten_session: u32, final_head: Point) {
        if self.mode == GameMode::Campaign
            || self.mode == GameMode::Maze
            || self.mode == GameMode::Cave
            || self.mode == GameMode::CustomLevel
            || self.mode == GameMode::DailyChallenge
            || self.mode == GameMode::WeeklyChallenge
        {
            return;
        }
        let new_obs_count =
            (self.food_eaten_session / 5).saturating_sub(old_food_eaten_session / 5);
        if new_obs_count > 0 {
            let avoid = |p: &Point| {
                let dx = i32::from(p.x).abs_diff(i32::from(final_head.x));
                let dy = i32::from(p.y).abs_diff(i32::from(final_head.y));
                self.obstacles.contains(p)
                    || (dx <= 2 && dy <= 2)
                    || *p == self.food
                    || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                    || self.power_up.as_ref().is_some_and(|pu| *p == pu.location)
            };
            let new_obstacles = Self::generate_obstacles(
                self.width,
                self.height,
                &self.snake,
                avoid,
                &mut self.rng,
                new_obs_count as usize,
                self.safe_zone_margin,
            );
            self.obstacles.extend(new_obstacles);
        }
    }
    pub fn update_quest_progress(&mut self, q_type: crate::game::QuestType, amount: u32) {
        let mut completed_any = false;
        for quest in &mut self.stats.active_quests {
            if quest.q_type == q_type && quest.status == crate::game::QuestStatus::Active {
                if q_type == crate::game::QuestType::ReachScore {
                    // Score is absolute, not additive over time, so just update it if the amount is larger
                    if amount > quest.progress {
                        quest.progress = amount;
                    }
                } else {
                    quest.progress += amount;
                }
                if quest.progress >= quest.target {
                    quest.progress = quest.target;
                    quest.status = crate::game::QuestStatus::Completed;
                    self.stats.coins += quest.reward;
                    self.stats.completed_quests.push(q_type);
                    completed_any = true;
                    self.chat_log.push_back((
                        format!(
                            "SYSTEM: Quest Completed! '{}' - Reward: {} Coins",
                            quest.name, quest.reward
                        ),
                        crate::color::Color::Yellow,
                    ));
                }
            }
        }
        if completed_any {
            self.stats.active_quests.retain(|q| q.status != crate::game::QuestStatus::Completed);
            crate::game::beep();
            self.save_stats();
        }
    }

    pub fn update_bounty_progress(&mut self, b_type: crate::game::BountyType, amount: u32) {
        let mut bounty_completed = false;
        let mut reward = 0;
        if let Some(ref mut active) = self.stats.active_bounty {
            let is_match = matches!(
                (active.b_type.clone(), b_type),
                (crate::game::BountyType::EatFood(_), crate::game::BountyType::EatFood(_))
                    | (
                        crate::game::BountyType::KillBosses(_),
                        crate::game::BountyType::KillBosses(_)
                    )
                    | (
                        crate::game::BountyType::SurviveTime(_),
                        crate::game::BountyType::SurviveTime(_)
                    )
            );
            if is_match {
                active.progress += amount;
                if active.is_completed() {
                    bounty_completed = true;
                    reward = active.reward_coins;
                }
            }
        }
        if bounty_completed {
            self.stats.active_bounty = None;
            self.stats.completed_bounties += 1;
            self.stats.coins += reward;
            self.save_stats();
        }
    }

    pub fn check_achievements(&mut self) {
        let mut new_achievements = Vec::new();
        if !self.stats.unlocked_achievements.contains(&Achievement::FirstBlood)
            && self.stats.games_played > 0
        {
            new_achievements.push(Achievement::FirstBlood);
        }
        if !self.stats.unlocked_achievements.contains(&Achievement::HighScorer) && self.score >= 100
        {
            new_achievements.push(Achievement::HighScorer);
        }
        if !self.stats.unlocked_achievements.contains(&Achievement::Rich)
            && self.stats.coins >= 1000
        {
            new_achievements.push(Achievement::Rich);
        }
        if !self.stats.unlocked_achievements.contains(&Achievement::BotUser)
            && self.used_bot_this_session
        {
            new_achievements.push(Achievement::BotUser);
        }
        if !self.stats.unlocked_achievements.contains(&Achievement::BossSlayer)
            && self.mode == GameMode::BossRush
            && self.campaign_level > 5
        {
            new_achievements.push(Achievement::BossSlayer);
        }
        if !self.stats.unlocked_achievements.contains(&Achievement::MassiveMultiplayerEnthusiast)
            && self.mode == GameMode::MassiveMultiplayer
        {
            new_achievements.push(Achievement::MassiveMultiplayerEnthusiast);
        }
        if !new_achievements.is_empty() {
            self.stats.unlocked_achievements.extend(new_achievements);
            self.save_stats();
        }
    }
    fn handle_win(&mut self) {
        self.stats.games_played += 1;
        self.stats.total_time_s += self.start_time.elapsed().as_secs();
        self.save_stats();
        self.check_achievements();
        if let (GameMode::Speedrun, Ok(json)) =
            (self.mode, serde_json::to_string(&self.current_replay))
        {
            let _ = Self::atomic_write("ghost.json", json);
        }
        let is_high_score = self.high_scores.len() < 5
            || self.score > self.high_scores.last().map_or(0, |(_, s)| *s);
        if is_high_score && self.score > 0 {
            if self.used_bot_this_session {
                self.save_high_score("[BOT]".to_string(), self.score);
                self.state = GameState::GameWon;
            } else {
                self.previous_state = Some(GameState::GameWon);
                self.state = GameState::EnterName;
                self.player_name.clear();
            }
        } else {
            self.state = GameState::GameWon;
        }
        if self.score > self.high_score {
            self.high_score = self.score;
        }
    }
    fn manage_portals(&mut self) {
        let spawn_chance = if self.skin == 'Ξ' {
            0.02
        } else {
            0.005
        };
        if self.portals.is_none() && self.rng.gen_bool(spawn_chance) {
            let avoid = |p: &Point| {
                self.obstacles.contains(p)
                    || *p == self.food
                    || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                    || self.power_up.as_ref().is_some_and(|pu| *p == pu.location)
            };
            if let Some(portal1) = Self::get_random_empty_point(
                self.width,
                self.height,
                &self.snake,
                avoid,
                &mut self.rng,
                self.safe_zone_margin,
            ) {
                let avoid2 = |p: &Point| avoid(p) || *p == portal1;
                if let Some(portal2) = Self::get_random_empty_point(
                    self.width,
                    self.height,
                    &self.snake,
                    avoid2,
                    &mut self.rng,
                    self.safe_zone_margin,
                ) {
                    self.portals = Some((portal1, portal2));
                }
            }
        }
    }
    fn manage_power_ups(&mut self) {
        if self.power_up.is_none() && self.rng.gen_bool(0.02) {
            let avoid = |p: &Point| {
                self.obstacles.contains(p)
                    || *p == self.food
                    || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
            };
            if let Some(location) = Self::get_random_empty_point(
                self.width,
                self.height,
                &self.snake,
                avoid,
                &mut self.rng,
                self.safe_zone_margin,
            ) {
                let p_type = match self.rng.gen_range(0..15) {
                    0 => PowerUpType::SlowDown,
                    1 => PowerUpType::SpeedBoost,
                    2 => PowerUpType::Invincibility,
                    3 => PowerUpType::PassThroughWalls,
                    4 => PowerUpType::Shrink,
                    5 => PowerUpType::ClearObstacles,
                    6 => PowerUpType::ScoreMultiplier,
                    7 => PowerUpType::Teleport,
                    8 => PowerUpType::Magnet,
                    9 => PowerUpType::TimeFreeze,
                    10 => PowerUpType::Reverse,
                    11 => PowerUpType::Decoy,
                    12 => PowerUpType::Emp,
                    13 => PowerUpType::Nuke,
                    _ => PowerUpType::ExtraLife,
                };
                self.power_up = Some(PowerUp {
                    p_type,
                    location,
                    activation_time: None,
                });
            }
        }
    }
    fn manage_merchant(&mut self) {
        if self.merchant.is_none() {
            let spawn_chance = 0.005; // 0.5% chance to spawn per tick
            if self.rng.gen_bool(spawn_chance) {
                let avoid = |p: &Point| {
                    self.obstacles.contains(p)
                        || *p == self.food
                        || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                        || self.poison_food.is_some_and(|(pp, _)| *p == pp)
                        || self.power_up.as_ref().is_some_and(|pu| *p == pu.location)
                        || self.resources.contains_key(p)
                };
                if let Some(pos) = Self::get_random_empty_point(
                    self.width,
                    self.height,
                    &self.snake,
                    avoid,
                    &mut self.rng,
                    self.safe_zone_margin,
                ) {
                    self.merchant = Some(pos);
                }
            }
        } else {
            // Despawn merchant occasionally if we want, or just leave it
            let despawn_chance = 0.001; // 0.1% chance to leave
            if self.rng.gen_bool(despawn_chance) {
                self.merchant = None;
            }
        }
    }

    fn manage_eggs(&mut self) {
        let spawn_chance = 0.005;
        if self.rng.gen_bool(spawn_chance) {
            let avoid = |p: &Point| {
                self.obstacles.contains(p)
                    || *p == self.food
                    || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                    || self.power_up.as_ref().is_some_and(|pu| *p == pu.location)
                    || self.resources.contains_key(p)
                    || self.eggs_on_board.contains_key(p)
            };
            if let Some(pos) = Self::get_random_empty_point(
                self.width,
                self.height,
                &self.snake,
                avoid,
                &mut self.rng,
                self.safe_zone_margin,
            ) {
                let egg = match self.rng.gen_range(0..100) {
                    0..=60 => crate::game::EggType::Common,
                    61..=90 => crate::game::EggType::Rare,
                    _ => crate::game::EggType::Legendary,
                };
                self.eggs_on_board.insert(pos, egg);
            }
        }
    }

    fn manage_resources(&mut self) {
        let spawn_chance = 0.01;
        if self.rng.gen_bool(spawn_chance) {
            let avoid = |p: &Point| {
                self.obstacles.contains(p)
                    || *p == self.food
                    || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                    || self.power_up.as_ref().is_some_and(|pu| *p == pu.location)
                    || self.resources.contains_key(p)
            };
            if let Some(pos) = Self::get_random_empty_point(
                self.width,
                self.height,
                &self.snake,
                avoid,
                &mut self.rng,
                self.safe_zone_margin,
            ) {
                let res = match self.rng.gen_range(0..100) {
                    0..=40 => Resource::Wood,
                    41..=70 => Resource::Iron,
                    71..=90 => Resource::Gold,
                    _ => Resource::Diamond,
                };
                self.resources.insert(pos, res);
            }
        }
    }

    fn manage_bonus_food(&mut self) {
        let spawn_chance = if self.skin == 'Ð' {
            if self.weather == Weather::Rain {
                0.12
            } else {
                0.04
            }
        } else {
            if self.weather == Weather::Rain {
                0.03
            } else {
                0.01
            }
        };
        if let Some((_, spawn_time)) = self.bonus_food {
            if spawn_time.elapsed() > Duration::from_secs(5) {
                self.bonus_food = None;
            }
        } else if self.rng.gen_bool(spawn_chance) {
            let avoid = |p: &Point| {
                self.obstacles.contains(p)
                    || *p == self.food
                    || self.power_up.as_ref().is_some_and(|pu| *p == pu.location)
            };
            if let Some(bonus) = Self::get_random_empty_point(
                self.width,
                self.height,
                &self.snake,
                avoid,
                &mut self.rng,
                self.safe_zone_margin,
            ) {
                self.bonus_food = Some((bonus, web_time::Instant::now()));
            }
        }
    }
    #[must_use]
    pub const fn calculate_next_head_dir(head: Point, dir: Direction) -> Point {
        match dir {
            Direction::Up => Point {
                x: head.x,
                y: head.y.wrapping_sub(1),
            },
            Direction::Down => Point {
                x: head.x,
                y: head.y + 1,
            },
            Direction::Left => Point {
                x: head.x.wrapping_sub(1),
                y: head.y,
            },
            Direction::Right => Point {
                x: head.x + 1,
                y: head.y,
            },
        }
    }
    #[must_use]
    pub fn get_final_p(&self, p: Point) -> Option<Point> {
        if let Some((portal1, portal2)) = self.portals {
            if p == portal1 {
                return Some(portal2);
            } else if p == portal2 {
                return Some(portal1);
            }
        }
        let can_pass_through_walls = self.power_up.as_ref().is_some_and(|pu| {
            pu.p_type == PowerUpType::PassThroughWalls
                && pu.activation_time.is_some_and(|t| {
                    web_time::SystemTime::now()
                        .duration_since(web_time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .saturating_sub(t)
                        < self.powerup_duration()
                })
        }) || self.stats.equipped_vehicle
            == Some(crate::game::Vehicle::Spaceship);
        if (self.wrap_mode || can_pass_through_walls || self.mode == GameMode::Zen)
            && self.mode != GameMode::BattleRoyale
        {
            Some(self.calculate_wrapped_head(p))
        } else if self.mode == GameMode::DungeonCrawler && self.is_door(p) {
            Some(p) // allow door traversal
        } else {
            let margin = if self.mode == GameMode::BattleRoyale {
                self.safe_zone_margin
            } else {
                0
            };
            if p.x <= margin
                || p.x >= self.width - 1 - margin
                || p.y <= margin
                || p.y >= self.height - 1 - margin
            {
                None
            } else {
                Some(p)
            }
        }
    }

    #[must_use]
    #[expect(clippy::collapsible_if)]
    pub fn bot_smart_pathfind(
        &self,
        start: Point,
        target: Point,
        checking_player: u8,
    ) -> Option<Direction> {
        if self.flow_field_targets.contains(&target)
            && self.mode != crate::game::GameMode::CaptureTheFlag
        {
            if let Some(flow_field) = &self.flow_field {
                if let Some(&dir) = flow_field.get(&start) {
                    return Some(dir);
                }
            }
        }
        self.astar_pathfind(start, target, checking_player)
    }

    #[must_use]
    pub fn astar_pathfind(
        &self,
        start: Point,
        target: Point,
        checking_player: u8,
    ) -> Option<Direction> {
        let mut open_set = std::collections::BinaryHeap::new();
        let mut g_score = std::collections::HashMap::new();
        let mut first_step = std::collections::HashMap::new();
        let mut tie_breaker_counter = 0u64;

        g_score.insert(start, 0u16);

        let calc_dist = |p1: Point, p2: Point| -> u16 {
            let mut dx = p1.x.abs_diff(p2.x);
            let mut dy = p1.y.abs_diff(p2.y);
            if (self.wrap_mode || self.mode == GameMode::Zen) && self.mode != GameMode::BattleRoyale
            {
                dx = std::cmp::min(dx, self.width.saturating_sub(2).saturating_sub(dx));
                dy = std::cmp::min(dy, self.height.saturating_sub(2).saturating_sub(dy));
            }
            dx.saturating_add(dy)
        };

        let heuristic = |p: Point| -> u16 {
            let mut penalty = 0u16;
            if checking_player == 1 {
                if let Some(p2) = &self.player2 {
                    for part in &p2.body {
                        let d = calc_dist(p, *part);
                        if d < 4 {
                            penalty = penalty.saturating_add((4 - d) * 10);
                        }
                    }
                }
                for bot in &self.bots {
                    if bot.head() == start {
                        continue;
                    }
                    for part in &bot.body {
                        let d = calc_dist(p, *part);
                        if d < 4 {
                            penalty = penalty.saturating_add((4 - d) * 10);
                        }
                    }
                }
            } else if checking_player == 2 {
                for part in &self.snake.body {
                    let d = calc_dist(p, *part);
                    if d < 4 {
                        penalty = penalty.saturating_add((4 - d) * 10);
                    }
                }
                for bot in &self.bots {
                    if bot.head() == start {
                        continue;
                    }
                    for part in &bot.body {
                        let d = calc_dist(p, *part);
                        if d < 4 {
                            penalty = penalty.saturating_add((4 - d) * 10);
                        }
                    }
                }
            } else if checking_player == 3 {
                for part in &self.snake.body {
                    let d = calc_dist(p, *part);
                    if d < 4 {
                        penalty = penalty.saturating_add((4 - d) * 10);
                    }
                }
                if let Some(p2) = &self.player2 {
                    for part in &p2.body {
                        let d = calc_dist(p, *part);
                        if d < 4 {
                            penalty = penalty.saturating_add((4 - d) * 10);
                        }
                    }
                }
                for bot in &self.bots {
                    if bot.head() == start {
                        continue;
                    }
                    for part in &bot.body {
                        let d = calc_dist(p, *part);
                        if d < 4 {
                            penalty = penalty.saturating_add((4 - d) * 10);
                        }
                    }
                }
            }
            for boss in &self.bosses {
                if target == boss.position {
                    continue;
                }
                let d = calc_dist(p, boss.position);
                if d < 5 {
                    penalty = penalty.saturating_add((5 - d) * 10);
                }
            }
            if let Some((pf, _)) = self.poison_food {
                let d = calc_dist(p, pf);
                if d < 4 {
                    penalty = penalty.saturating_add((4 - d) * 10);
                }
            }
            for l in &self.lasers {
                let d = calc_dist(p, l.position);
                if d < 4 {
                    penalty = penalty.saturating_add((4 - d) * 5);
                }
            }
            for m in &self.mines {
                let d = calc_dist(p, *m);
                if d < 4 {
                    penalty = penalty.saturating_add((4 - d) * 10);
                }
            }
            for t in &self.turrets {
                let d = calc_dist(p, t.position);
                if d < 4 {
                    penalty = penalty.saturating_add((4 - d) * 10);
                }
            }
            if let Some(bh) = self.black_hole {
                let d = calc_dist(p, bh);
                if d < 5 {
                    penalty = penalty.saturating_add((5 - d) * 10);
                }
            }
            if let Some(col) = self.lightning_column {
                let dx = p.x.abs_diff(col);
                if dx < 3 {
                    penalty = penalty.saturating_add((3 - dx) * 50);
                }
            }
            for m in &self.meteors {
                let dx = p.x.abs_diff(m.position.x);
                if dx < 2 && p.y >= m.position.y {
                    let dy = p.y.abs_diff(m.position.y);
                    if dy < 10 {
                        penalty = penalty.saturating_add((10 - dy) * 5);
                    }
                }
            }

            let dist_direct = calc_dist(p, target);
            if let Some((portal1, portal2)) = self.portals {
                let dist_via_portal1 = calc_dist(p, portal1)
                    .saturating_add(calc_dist(portal2, target))
                    .saturating_add(1);
                let dist_via_portal2 = calc_dist(p, portal2)
                    .saturating_add(calc_dist(portal1, target))
                    .saturating_add(1);
                std::cmp::min(dist_direct, std::cmp::min(dist_via_portal1, dist_via_portal2))
                    .saturating_add(penalty)
            } else {
                dist_direct.saturating_add(penalty)
            }
        };

        tie_breaker_counter += 1;
        open_set.push(AStarState {
            f_score: heuristic(start),
            tie_breaker: tie_breaker_counter,
            position: start,
        });

        let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];

        let mut iterations = 0;
        while let Some(AStarState {
            position: current,
            ..
        }) = open_set.pop()
        {
            iterations += 1;
            if iterations > 3000 {
                break; // Prevent infinite loops
            }
            if current == target {
                return first_step.get(&current).copied();
            }

            let current_g = *g_score.get(&current).unwrap_or(&u16::MAX);

            for &d in &dirs {
                let next_p = Self::calculate_next_head_dir(current, d);
                let tentative_g = current_g.saturating_add(1);

                if let Some(final_p) = self.get_final_p(next_p)
                    && !self.obstacles.contains(&final_p)
                    && self.is_safe_final_p(final_p, tentative_g, checking_player)
                    && tentative_g < *g_score.get(&final_p).unwrap_or(&u16::MAX)
                {
                    if final_p == target {
                        // Optimally return if target reached
                        return first_step.get(&current).copied().or(Some(d));
                    }
                    g_score.insert(final_p, tentative_g);
                    if current == start {
                        first_step.insert(final_p, d);
                    } else if let Some(&first) = first_step.get(&current) {
                        first_step.insert(final_p, first);
                    }
                    tie_breaker_counter += 1;
                    open_set.push(AStarState {
                        f_score: tentative_g.saturating_add(heuristic(final_p)),
                        tie_breaker: tie_breaker_counter,
                        position: final_p,
                    });
                }
            }
        }
        None
    }
    #[must_use]
    #[expect(clippy::too_many_lines)]
    pub fn is_safe_final_p(&self, final_p: Point, steps: u16, checking_player: u8) -> bool {
        let is_invincible = self.mode == GameMode::Zen
            || self.power_up.as_ref().is_some_and(|pu| {
                pu.p_type == PowerUpType::Invincibility
                    && pu.activation_time.is_some_and(|t| {
                        web_time::SystemTime::now()
                            .duration_since(web_time::SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                            .saturating_sub(t)
                            < self.powerup_duration()
                    })
            });
        let is_time_frozen = self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::TimeFreeze
                && p.activation_time.is_some_and(|t| {
                    web_time::SystemTime::now()
                        .duration_since(web_time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .saturating_sub(t)
                        < self.powerup_duration()
                })
        });
        if !is_invincible {
            if self.mode == GameMode::Miner && self.obstacles.contains(&final_p) {
                // In Miner mode, obstacles can be broken, so they are safe to walk into
            } else if self.obstacles.contains(&final_p) {
                // Determine if Juggernaut could destroy this obstacle
                let mut juggernaut_will_destroy = false;
                for boss in &self.bosses {
                    if boss.kind == BossType::Juggernaut {
                        let active_steps =
                            u32::from(steps).saturating_sub(u32::from(boss.state_timer));
                        let mut move_threshold = u32::from(if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                1,
                                3_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255) / 5,
                                ),
                            )
                        } else {
                            2
                        });
                        move_threshold = std::cmp::max(1, move_threshold / 2);
                        if boss.health <= boss.max_health / 2 {
                            move_threshold = std::cmp::max(1, move_threshold / 2);
                        }
                        let moves = (active_steps + u32::from(boss.move_timer)) / move_threshold;
                        let mut dist = u32::from(final_p.x.abs_diff(boss.position.x))
                            + u32::from(final_p.y.abs_diff(boss.position.y));

                        if let Some((portal1, portal2)) = self.portals {
                            let dist_via_p1 = u32::from(final_p.x.abs_diff(portal1.x))
                                + u32::from(final_p.y.abs_diff(portal1.y))
                                + u32::from(portal2.x.abs_diff(boss.position.x))
                                + u32::from(portal2.y.abs_diff(boss.position.y));
                            let dist_via_p2 = u32::from(final_p.x.abs_diff(portal2.x))
                                + u32::from(final_p.y.abs_diff(portal2.y))
                                + u32::from(portal1.x.abs_diff(boss.position.x))
                                + u32::from(portal1.y.abs_diff(boss.position.y));
                            dist = std::cmp::min(dist, std::cmp::min(dist_via_p1, dist_via_p2));
                        }
                        if dist <= moves {
                            juggernaut_will_destroy = true;
                            break;
                        }
                    }
                }
                if !juggernaut_will_destroy {
                    return false;
                }
            }
            if self.poison_food.is_some_and(|(pp, _)| pp == final_p) {
                return false;
            }
            // Merchant is considered safe.
            if self.mines.contains(&final_p) {
                return false;
            }
            if self.black_hole.is_some_and(|bh| bh == final_p) {
                return false;
            }
            for meteor in &self.meteors {
                if meteor.position == final_p
                    || (meteor.position.x == final_p.x
                        && meteor.position.y <= final_p.y
                        && meteor.position.y + steps >= final_p.y)
                {
                    return false;
                }
            }
            if let Some(col) = self.lightning_column
                && final_p.x == col
            {
                return false;
            }
            if steps == 1 {
                let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
                if checking_player == 1 {
                    if let Some(p2) = &self.player2 {
                        for &d in &dirs {
                            let p2_next_head = Self::calculate_next_head_dir(p2.head(), d);
                            if let Some(final_p2_next) = self.get_final_p(p2_next_head)
                                && final_p == final_p2_next
                            {
                                return false;
                            }
                        }
                    }
                } else if checking_player == 2 {
                    for &d in &dirs {
                        let p1_next_head = Self::calculate_next_head_dir(self.snake.head(), d);
                        if let Some(final_p1_next) = self.get_final_p(p1_next_head)
                            && final_p == final_p1_next
                        {
                            return false;
                        }
                    }
                } else if checking_player == 4 {
                    // Check against other bots' possible next moves to avoid head-on collisions in bot vs bot
                    let mut other_bots_count = 0;
                    for b in &self.bots {
                        for &d in &dirs {
                            let b_next_head = Self::calculate_next_head_dir(b.head(), d);
                            if let Some(final_b_next) = self.get_final_p(b_next_head)
                                && final_p == final_b_next
                            {
                                other_bots_count += 1;
                                break; // counted for this bot
                            }
                        }
                    }
                    if other_bots_count > 1 {
                        return false;
                    }
                }
            }
            for boss in &self.bosses {
                let is_dungeon_uncleared = self.mode == GameMode::DungeonCrawler
                    && !self.dungeon_grid.get(&self.current_room_coords).is_some_and(|r| r.cleared);
                if !(is_dungeon_uncleared && final_p == boss.position) {
                    if final_p == boss.position {
                        return false;
                    }
                    if is_time_frozen {
                        if final_p == boss.position {
                            return false;
                        }
                    } else if checking_player != 3
                        && boss.state_timer < u8::try_from(steps).unwrap_or(u8::MAX)
                    {
                        let mut move_threshold = u32::from(if self.mode == GameMode::BossRush {
                            std::cmp::max(
                                1,
                                3_u8.saturating_sub(
                                    u8::try_from(self.campaign_level).unwrap_or(255) / 5,
                                ),
                            )
                        } else {
                            2
                        });
                        if boss.kind == BossType::Charger
                            || boss.kind == BossType::Juggernaut
                            || boss.kind == BossType::Phantom
                        {
                            move_threshold = std::cmp::max(1, move_threshold / 2);
                        }
                        if boss.health <= boss.max_health / 2 {
                            move_threshold = std::cmp::max(1, move_threshold / 2);
                        }
                        let active_steps =
                            u32::from(steps).saturating_sub(u32::from(boss.state_timer));
                        let moves = (active_steps + u32::from(boss.move_timer)) / move_threshold;
                        let mut dist = u32::from(final_p.x.abs_diff(boss.position.x))
                            + u32::from(final_p.y.abs_diff(boss.position.y));

                        if let Some((portal1, portal2)) = self.portals {
                            let dist_via_p1 = u32::from(final_p.x.abs_diff(portal1.x))
                                + u32::from(final_p.y.abs_diff(portal1.y))
                                + u32::from(portal2.x.abs_diff(boss.position.x))
                                + u32::from(portal2.y.abs_diff(boss.position.y));
                            let dist_via_p2 = u32::from(final_p.x.abs_diff(portal2.x))
                                + u32::from(final_p.y.abs_diff(portal2.y))
                                + u32::from(portal1.x.abs_diff(boss.position.x))
                                + u32::from(portal1.y.abs_diff(boss.position.y));
                            dist = std::cmp::min(dist, std::cmp::min(dist_via_p1, dist_via_p2));
                        }

                        if boss.kind == BossType::Teleporter
                            || boss.kind == BossType::Spawner
                            || boss.kind == BossType::Trapper
                            || boss.kind == BossType::Necromancer
                            || boss.kind == BossType::ShadowClone
                            || boss.kind == BossType::Mimic
                            || boss.kind == BossType::Gorgon
                            || boss.kind == BossType::Shooter
                            || boss.kind == BossType::VampireLord
                            || boss.kind == BossType::Kraken
                            || boss.kind == BossType::Alchemist
                            || boss.kind == BossType::Puffer
                            || boss.kind == BossType::Dragon
                            || boss.kind == BossType::Mage
                        {
                            if final_p == boss.position
                                || (boss.kind == BossType::Shooter && dist <= moves)
                            {
                                return false;
                            }
                        } else if dist <= moves {
                            if boss.kind == BossType::Juggernaut {
                                // Juggernaut can move through obstacles, so we assume any tile within distance could be unsafe
                                return false;
                            }
                            return false;
                        }
                        if boss.kind == BossType::Shooter
                            || boss.kind == BossType::Puffer
                            || boss.kind == BossType::Dragon
                            || boss.kind == BossType::Mage
                        {
                            let mut shoot_threshold =
                                u32::from(if self.mode == GameMode::BossRush {
                                    std::cmp::max(
                                        if boss.kind == BossType::Puffer
                                            || boss.kind == BossType::Dragon
                                        {
                                            10
                                        } else if boss.kind == BossType::Mage {
                                            15
                                        } else {
                                            5
                                        },
                                        (if boss.kind == BossType::Puffer {
                                            30_u8
                                        } else if boss.kind == BossType::Dragon {
                                            20_u8
                                        } else if boss.kind == BossType::Mage {
                                            30_u8
                                        } else {
                                            15_u8
                                        })
                                        .saturating_sub(
                                            u8::try_from(self.campaign_level).unwrap_or(255),
                                        ),
                                    )
                                } else if boss.kind == BossType::Puffer
                                    || boss.kind == BossType::Mage
                                {
                                    30
                                } else if boss.kind == BossType::Dragon {
                                    20
                                } else {
                                    15
                                });
                            if boss.health <= boss.max_health / 2 {
                                shoot_threshold = std::cmp::max(
                                    if boss.kind == BossType::Puffer {
                                        5
                                    } else if boss.kind == BossType::Dragon {
                                        2
                                    } else if boss.kind == BossType::Mage {
                                        5
                                    } else {
                                        1
                                    },
                                    shoot_threshold / 2,
                                );
                            }
                            let shoots =
                                (active_steps + u32::from(boss.shoot_timer)) / shoot_threshold;
                            if shoots > 0 {
                                if boss.kind == BossType::Mage {
                                    // The mage will spawn a meteor exactly at the target's current head.
                                    // So if we are considering staying at or moving to a point that is the same as
                                    // the CURRENT head of the player/bot being targeted, it's dangerous.
                                    // However, the meteor takes 15 ticks to hit.
                                    // This is an immediate prediction: "Where will the mage shoot?"
                                    // We know it shoots at self.snake.head() or decoy.
                                    let target_pos = if let Some((decoy_pos, _)) = self.decoy {
                                        decoy_pos
                                    } else {
                                        // Depending on checking_player we might want to be more specific,
                                        // but self.snake.head() is standard targeting.
                                        self.snake.head()
                                    };

                                    // We don't know the exact time it will take for the bot to reach `final_p`.
                                    // If `final_p` IS the target_pos, a meteor is about to be spawned there.
                                    // Better to avoid `target_pos` right now to ensure the bot paths away from it.
                                    if final_p == target_pos {
                                        return false;
                                    }
                                } else if final_p.x == boss.position.x
                                    || final_p.y == boss.position.y
                                {
                                    // A boss shoots lasers in 4 directions, meaning any point on the same X or Y axis *might* be hit,
                                    // but blocking the ENTIRE axis makes the bot fail to pathfind around the boss entirely if it's far.
                                    // Only consider it unsafe if within the same column/row AND fairly close.
                                    let mut dist = u32::from(final_p.x.abs_diff(boss.position.x))
                                        + u32::from(final_p.y.abs_diff(boss.position.y));

                                    if let Some((portal1, portal2)) = self.portals {
                                        let dist_via_p1 = u32::from(final_p.x.abs_diff(portal1.x))
                                            + u32::from(final_p.y.abs_diff(portal1.y))
                                            + u32::from(portal2.x.abs_diff(boss.position.x))
                                            + u32::from(portal2.y.abs_diff(boss.position.y));
                                        let dist_via_p2 = u32::from(final_p.x.abs_diff(portal2.x))
                                            + u32::from(final_p.y.abs_diff(portal2.y))
                                            + u32::from(portal1.x.abs_diff(boss.position.x))
                                            + u32::from(portal1.y.abs_diff(boss.position.y));
                                        dist = std::cmp::min(
                                            dist,
                                            std::cmp::min(dist_via_p1, dist_via_p2),
                                        );
                                    }

                                    // A laser travels 2 tiles per tick.
                                    // It takes the laser roughly `dist / 2` ticks to reach `final_p`.
                                    // We are evaluating the safety of `final_p` at `t = active_steps`.
                                    // A laser hits this tile at `t` if `t % shoot_threshold == dist / 2`.
                                    // So we check if `active_steps` is close to the expected arrival time of any laser fired.
                                    // We add a small buffer for safety.
                                    let expected_arrival_mod = (dist / 2) % shoot_threshold;
                                    let step_mod = (active_steps + u32::from(boss.shoot_timer))
                                        % shoot_threshold;

                                    let diff = step_mod.abs_diff(expected_arrival_mod);
                                    let true_diff =
                                        std::cmp::min(diff, shoot_threshold.saturating_sub(diff));

                                    if true_diff <= 2 {
                                        // A laser will be here at around the time we arrive.
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            for l in &self.lasers {
                if final_p == l.position {
                    return false;
                }
                if is_time_frozen {
                    if final_p == l.position {
                        return false;
                    }
                } else {
                    let check_laser_threat = |laser_pos: Point, dist_offset: u32| -> bool {
                        let dx = i32::from(final_p.x) - i32::from(laser_pos.x);
                        let dy = i32::from(final_p.y) - i32::from(laser_pos.y);
                        let on_ray = match l.direction {
                            Direction::Up => dx == 0 && dy <= 0,
                            Direction::Down => dx == 0 && dy >= 0,
                            Direction::Left => dy == 0 && dx <= 0,
                            Direction::Right => dy == 0 && dx >= 0,
                        };
                        if on_ray {
                            let d =
                                u32::try_from(dx.abs().max(dy.abs())).unwrap_or(0) + dist_offset;
                            let step_dist = u32::from(steps) * 2;

                            // A laser travels 2 units per step.
                            // If `d` is smaller than `step_dist`, the laser has already passed this point
                            // or is passing it this step.
                            // However, we only care if the laser hits us AT `steps` or after (if it's a constant threat).
                            // A better approximation is: does the laser reach or pass `final_p` in `steps` ticks?
                            if step_dist.abs_diff(d) <= 2 || d <= step_dist + 1 {
                                return true;
                            }
                        }
                        false
                    };

                    if check_laser_threat(l.position, 0) {
                        return false;
                    }

                    if let Some((portal1, portal2)) = self.portals {
                        // Check if laser hits portal1 and exits portal2
                        let p1_dx = i32::from(portal1.x) - i32::from(l.position.x);
                        let p1_dy = i32::from(portal1.y) - i32::from(l.position.y);
                        let hits_p1 = match l.direction {
                            Direction::Up => p1_dx == 0 && p1_dy <= 0,
                            Direction::Down => p1_dx == 0 && p1_dy >= 0,
                            Direction::Left => p1_dy == 0 && p1_dx <= 0,
                            Direction::Right => p1_dy == 0 && p1_dx >= 0,
                        };
                        if hits_p1 {
                            let dist_to_p1 =
                                u32::try_from(p1_dx.abs().max(p1_dy.abs())).unwrap_or(0);
                            if check_laser_threat(portal2, dist_to_p1) {
                                return false;
                            }
                        }

                        // Check if laser hits portal2 and exits portal1
                        let p2_dx = i32::from(portal2.x) - i32::from(l.position.x);
                        let p2_dy = i32::from(portal2.y) - i32::from(l.position.y);
                        let hits_p2 = match l.direction {
                            Direction::Up => p2_dx == 0 && p2_dy <= 0,
                            Direction::Down => p2_dx == 0 && p2_dy >= 0,
                            Direction::Left => p2_dy == 0 && p2_dx <= 0,
                            Direction::Right => p2_dy == 0 && p2_dx >= 0,
                        };
                        if hits_p2 {
                            let dist_to_p2 =
                                u32::try_from(p2_dx.abs().max(p2_dy.abs())).unwrap_or(0);
                            if check_laser_threat(portal1, dist_to_p2) {
                                return false;
                            }
                        }
                    }
                }
            }
            if let Some(col) = self.lightning_column
                && final_p.x == col
            {
                return false;
            }
            if let Some(pos) = self.snake.body.iter().position(|&p| p == final_p) {
                let steps_to_clear =
                    u16::try_from(self.snake.body.len().saturating_sub(pos)).unwrap_or(u16::MAX);
                if steps < steps_to_clear {
                    return false;
                }
            }
            if let Some(p2) = &self.player2
                && let Some(pos) = p2.body.iter().position(|&p| p == final_p)
            {
                let steps_to_clear =
                    u16::try_from(p2.body.len().saturating_sub(pos)).unwrap_or(u16::MAX);
                if steps < steps_to_clear {
                    return false;
                }
            }
            for b in &self.bots {
                if let Some(pos) = b.body.iter().position(|&p| p == final_p) {
                    let steps_to_clear =
                        u16::try_from(b.body.len().saturating_sub(pos)).unwrap_or(u16::MAX);
                    if steps < steps_to_clear {
                        return false;
                    }
                }
            }
            for t in &self.turrets {
                if final_p == t.position {
                    return false;
                }
            }
            if self.black_hole.is_some_and(|bh| final_p == bh) {
                return false;
            }
            if steps == 1 {
                let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
                if checking_player == 1 {
                    if let Some(p2) = &self.player2 {
                        for &d in &dirs {
                            let p2_next_head = Self::calculate_next_head_dir(p2.head(), d);
                            if let Some(final_p2_next) = self.get_final_p(p2_next_head)
                                && final_p == final_p2_next
                            {
                                return false;
                            }
                        }
                    }
                } else if checking_player == 2 {
                    for &d in &dirs {
                        let p1_next_head = Self::calculate_next_head_dir(self.snake.head(), d);
                        if let Some(final_p1_next) = self.get_final_p(p1_next_head)
                            && final_p == final_p1_next
                        {
                            return false;
                        }
                    }
                } else if checking_player == 4 {
                    // Check against other bots' possible next moves to avoid head-on collisions in bot vs bot
                    let mut other_bots_count = 0;
                    for b in &self.bots {
                        // Assuming the bot checking has head distance > 1 from `b` head usually
                        // We check if final_p could be reached by this bot.
                        for &d in &dirs {
                            let b_next_head = Self::calculate_next_head_dir(b.head(), d);
                            if let Some(final_b_next) = self.get_final_p(b_next_head)
                                && final_p == final_b_next
                            {
                                other_bots_count += 1;
                                break; // counted for this bot
                            }
                        }
                    }
                    // Since the current bot is in self.bots, its own possible next moves
                    // will include `final_p` (which is what we are evaluating).
                    // So `other_bots_count` will be at least 1. If it's > 1, another bot can also move here.
                    if other_bots_count > 1 {
                        return false;
                    }
                }
            }
        }
        true
    }
    pub fn calculate_autopilot_move(&mut self) -> Option<Direction> {
        let start = self.snake.head();
        let current_dir = self.snake.direction;
        let mut targets = vec![self.food];
        if let Some((bf_p, _)) = self.bonus_food {
            targets.push(bf_p);
        }
        if let Some(pu) = &self.power_up
            && pu.activation_time.is_none()
        {
            targets.push(pu.location);
        }
        if let Some(goblin) = &self.goblin {
            targets.push(goblin.position);
        }
        if let Some(merchant) = self.merchant {
            targets.push(merchant);
        }
        if self.mode == GameMode::KingOfTheHill
            && let Some(koth_pos) = self.koth_zone
        {
            targets.insert(0, koth_pos);
        }
        if self.mode == GameMode::DungeonCrawler {
            targets.clear();
            if let Some(room) = self.dungeon_grid.get(&self.current_room_coords) {
                if room.cleared {
                    if room.north_door {
                        targets.insert(
                            0,
                            Point {
                                x: self.width / 2,
                                y: 0,
                            },
                        );
                    }
                    if room.south_door {
                        targets.insert(
                            0,
                            Point {
                                x: self.width / 2,
                                y: self.height - 1,
                            },
                        );
                    }
                    if room.west_door {
                        targets.insert(
                            0,
                            Point {
                                x: 0,
                                y: self.height / 2,
                            },
                        );
                    }
                    if room.east_door {
                        targets.insert(
                            0,
                            Point {
                                x: self.width - 1,
                                y: self.height / 2,
                            },
                        );
                    }
                } else {
                    for boss in &self.bosses {
                        targets.insert(0, boss.position);
                    }
                }
            } else {
                for boss in &self.bosses {
                    targets.insert(0, boss.position);
                }
            }
        }
        if self.mode == GameMode::CaptureTheFlag {
            if self.p1_has_flag {
                targets = vec![Point {
                    x: 2,
                    y: self.height / 2,
                }];
            } else if let Some(p2_flag) = self.p2_flag {
                targets = vec![p2_flag];
            } else if let Some(p2) = &self.player2 {
                targets = vec![p2.head()];
            } else {
                targets = vec![Point {
                    x: self.width.saturating_sub(3),
                    y: self.height / 2,
                }];
            }
        }
        if let Some((dir, path)) = self.astar_search(start, current_dir, &targets, 1) {
            self.autopilot_path = path;
            return Some(dir);
        }
        self.autopilot_path.clear();
        self.flood_fill_fallback(start, current_dir, 1)
    }
    pub fn calculate_p2_autopilot_move(&mut self) -> Option<Direction> {
        if let Some(p2) = &self.player2 {
            let start = p2.head();
            let current_dir = p2.direction;
            let mut targets = vec![self.food];
            if let Some((bf_p, _)) = self.bonus_food {
                targets.push(bf_p);
            }
            if let Some(pu) = &self.power_up
                && pu.activation_time.is_none()
            {
                targets.push(pu.location);
            }
            if let Some(goblin) = &self.goblin {
                targets.push(goblin.position);
            }
            if self.mode == GameMode::KingOfTheHill
                && let Some(koth_pos) = self.koth_zone
            {
                targets.insert(0, koth_pos);
            }
            if self.mode == GameMode::DungeonCrawler
                && let Some(room) = self.dungeon_grid.get(&self.current_room_coords)
            {
                if room.cleared {
                    if room.north_door {
                        targets.insert(
                            0,
                            Point {
                                x: self.width / 2,
                                y: 0,
                            },
                        );
                    }
                    if room.south_door {
                        targets.insert(
                            0,
                            Point {
                                x: self.width / 2,
                                y: self.height - 1,
                            },
                        );
                    }
                    if room.west_door {
                        targets.insert(
                            0,
                            Point {
                                x: 0,
                                y: self.height / 2,
                            },
                        );
                    }
                    if room.east_door {
                        targets.insert(
                            0,
                            Point {
                                x: self.width - 1,
                                y: self.height / 2,
                            },
                        );
                    }
                } else {
                    for boss in &self.bosses {
                        // If uncleared, try to move toward the boss but not exact position if possible, maybe add an offset
                        targets.insert(
                            0,
                            Point {
                                x: boss.position.x.saturating_add(1),
                                y: boss.position.y,
                            },
                        );
                    }
                }
            }
            if self.mode == GameMode::CaptureTheFlag {
                if self.p2_has_flag {
                    targets = vec![Point {
                        x: self.width.saturating_sub(3),
                        y: self.height / 2,
                    }];
                } else if let Some(p1_flag) = self.p1_flag {
                    targets = vec![
                        Point {
                            x: p1_flag.x.saturating_add(1),
                            y: p1_flag.y,
                        },
                        p1_flag,
                    ]; // To move toward it, maybe the path was slightly different due to avoidance
                } else {
                    targets = vec![self.snake.head()];
                }
            }
            if let Some((dir, path)) = self.astar_search(start, current_dir, &targets, 2) {
                self.p2_autopilot_path = path;
                return Some(dir);
            }
            self.p2_autopilot_path.clear();
            self.flood_fill_fallback(start, current_dir, 2)
        } else {
            None
        }
    }
    #[expect(clippy::too_many_lines, reason = "Search algorithm is inherently complex and long")]
    fn astar_search(
        &self,
        start: Point,
        current_dir: Direction,
        targets: &[Point],
        checking_player: u8,
    ) -> Option<(Direction, Vec<Point>)> {
        let mut open_set = std::collections::BinaryHeap::new();
        let mut g_score = std::collections::HashMap::new();
        let mut first_step = std::collections::HashMap::new();
        let mut came_from = std::collections::HashMap::new();
        let mut tie_breaker_counter = 0u64;
        g_score.insert(start, 0);
        let heuristic = |p: Point| -> u16 {
            let can_pass_through_walls = self.power_up.as_ref().is_some_and(|pu| {
                pu.p_type == PowerUpType::PassThroughWalls
                    && pu.activation_time.is_some_and(|time| {
                        web_time::SystemTime::now()
                            .duration_since(web_time::SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                            .saturating_sub(time)
                            < self.powerup_duration()
                    })
            }) || self.stats.equipped_vehicle
                == Some(crate::game::Vehicle::Spaceship);
            let calc_dist = |p1: Point, p2: Point| -> u16 {
                let mut dx = p1.x.abs_diff(p2.x);
                let mut dy = p1.y.abs_diff(p2.y);
                if (self.wrap_mode || can_pass_through_walls || self.mode == GameMode::Zen)
                    && self.mode != GameMode::BattleRoyale
                {
                    dx = std::cmp::min(dx, self.width.saturating_sub(2).saturating_sub(dx));
                    dy = std::cmp::min(dy, self.height.saturating_sub(2).saturating_sub(dy));
                }
                dx.saturating_add(dy)
            };
            let mut penalty = 0_u16;
            if checking_player == 1 {
                if let Some(p2) = &self.player2 {
                    for part in &p2.body {
                        let d = calc_dist(p, *part);
                        if d < 4 {
                            penalty = penalty.saturating_add((4 - d) * 10);
                        }
                    }
                }
                for bot in &self.bots {
                    if bot.head() == start {
                        continue;
                    }
                    for part in &bot.body {
                        let d = calc_dist(p, *part);
                        if d < 4 {
                            penalty = penalty.saturating_add((4 - d) * 10);
                        }
                    }
                }
            } else if checking_player == 2 {
                for part in &self.snake.body {
                    let d = calc_dist(p, *part);
                    if d < 4 {
                        penalty = penalty.saturating_add((4 - d) * 10);
                    }
                }
                for bot in &self.bots {
                    if bot.head() == start {
                        continue;
                    }
                    for part in &bot.body {
                        let d = calc_dist(p, *part);
                        if d < 4 {
                            penalty = penalty.saturating_add((4 - d) * 10);
                        }
                    }
                }
            } else if checking_player == 4 {
                for part in &self.snake.body {
                    let d = calc_dist(p, *part);
                    if d < 4 {
                        penalty = penalty.saturating_add((4 - d) * 10);
                    }
                }
                if let Some(p2) = &self.player2 {
                    for part in &p2.body {
                        let d = calc_dist(p, *part);
                        if d < 4 {
                            penalty = penalty.saturating_add((4 - d) * 10);
                        }
                    }
                }
                for bot in &self.bots {
                    if bot.head() == start {
                        continue;
                    }
                    for part in &bot.body {
                        let d = calc_dist(p, *part);
                        if d < 4 {
                            penalty = penalty.saturating_add((4 - d) * 10);
                        }
                    }
                }
            }
            for boss in &self.bosses {
                if targets.contains(&boss.position) {
                    continue;
                }
                let d = calc_dist(p, boss.position);
                if d < 5 {
                    penalty = penalty.saturating_add((5 - d) * 10);
                }
            }
            if let Some((pf, _)) = self.poison_food {
                let d = calc_dist(p, pf);
                if d < 4 {
                    penalty = penalty.saturating_add((4 - d) * 10);
                }
            }
            for l in &self.lasers {
                let d = calc_dist(p, l.position);
                if d < 4 {
                    penalty = penalty.saturating_add((4 - d) * 5);
                }
            }
            for m in &self.mines {
                let d = calc_dist(p, *m);
                if d < 4 {
                    penalty = penalty.saturating_add((4 - d) * 10);
                }
            }
            for t in &self.turrets {
                let d = calc_dist(p, t.position);
                if d < 4 {
                    penalty = penalty.saturating_add((4 - d) * 10);
                }
            }
            if let Some(bh) = self.black_hole {
                let d = calc_dist(p, bh);
                if d < 5 {
                    penalty = penalty.saturating_add((5 - d) * 10);
                }
            }
            if let Some(col) = self.lightning_column {
                let dx = p.x.abs_diff(col);
                if dx < 3 {
                    penalty = penalty.saturating_add((3 - dx) * 50);
                }
            }
            for m in &self.meteors {
                let dx = p.x.abs_diff(m.position.x);
                if dx < 2 && p.y >= m.position.y {
                    let dy = p.y.abs_diff(m.position.y);
                    if dy < 10 {
                        penalty = penalty.saturating_add((10 - dy) * 5);
                    }
                }
            }
            if let Some((pf_p, _)) = self.poison_food {
                let d = p.x.abs_diff(pf_p.x) + p.y.abs_diff(pf_p.y);
                if d < 4 {
                    penalty = penalty.saturating_add((4 - d) * 10);
                }
            }
            targets
                .iter()
                .map(|t| {
                    let dist_direct = calc_dist(p, *t);
                    if let Some((portal1, portal2)) = self.portals {
                        let dist_via_portal1 =
                            calc_dist(p, portal1).saturating_add(calc_dist(portal2, *t));
                        let dist_via_portal2 =
                            calc_dist(p, portal2).saturating_add(calc_dist(portal1, *t));
                        std::cmp::min(
                            dist_direct,
                            std::cmp::min(dist_via_portal1, dist_via_portal2),
                        )
                    } else {
                        dist_direct
                    }
                })
                .min()
                .unwrap_or(0)
                .saturating_add(penalty)
        };
        let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
        for &d in &dirs {
            if d.is_opposite(current_dir) {
                continue;
            }
            let next_p = Self::calculate_next_head_dir(start, d);
            if let Some(final_p) = self.get_final_p(next_p)
                && self.is_safe_final_p(final_p, 1, checking_player)
                && !self.obstacles.contains(&final_p)
            {
                let cost = 1;
                g_score.insert(final_p, cost);
                first_step.insert(final_p, d);
                came_from.insert(final_p, start);
                tie_breaker_counter += 1;
                open_set.push(AStarState {
                    f_score: cost + heuristic(final_p),
                    tie_breaker: tie_breaker_counter,
                    position: final_p,
                });
            }
        }
        let mut iterations = 0;
        while let Some(AStarState {
            position: current,
            ..
        }) = open_set.pop()
        {
            iterations += 1;
            if iterations > 3000 {
                break; // Prevent infinite loops
            }

            if targets.contains(&current) {
                let mut path = vec![current];
                let mut curr = current;
                while let Some(&prev) = came_from.get(&curr) {
                    if prev == start {
                        break;
                    }
                    path.push(prev);
                    curr = prev;
                }
                path.reverse();
                return first_step.get(&current).copied().map(|d| (d, path));
            }
            let current_g = *g_score.get(&current).unwrap_or(&u16::MAX);
            for &d in &dirs {
                let next_p = Self::calculate_next_head_dir(current, d);
                let tentative_g = current_g.saturating_add(1);
                if let Some(final_p) = self.get_final_p(next_p)
                    && (self.is_safe_final_p(final_p, tentative_g, checking_player)
                        && !self.obstacles.contains(&final_p)
                        || (targets.contains(&final_p) && tentative_g > 1))
                    && tentative_g < *g_score.get(&final_p).unwrap_or(&u16::MAX)
                {
                    came_from.insert(final_p, current);
                    g_score.insert(final_p, tentative_g);
                    first_step.insert(
                        final_p,
                        *first_step
                            .get(&current)
                            .expect("current should be present in first_step mapping"),
                    );
                    tie_breaker_counter += 1;
                    open_set.push(AStarState {
                        f_score: tentative_g.saturating_add(heuristic(final_p)),
                        tie_breaker: tie_breaker_counter,
                        position: final_p,
                    });
                }
            }
        }
        None
    }
    fn flood_fill_fallback(
        &self,
        start: Point,
        current_dir: Direction,
        checking_player: u8,
    ) -> Option<Direction> {
        let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
        let mut best_dir = None;
        let mut max_open_space = 0;
        for &d in &dirs {
            if d.is_opposite(current_dir) {
                continue;
            }
            let next_p = Self::calculate_next_head_dir(start, d);
            if let Some(final_p) = self.get_final_p(next_p)
                && self.is_safe_final_p(final_p, 1, checking_player)
                && !self.obstacles.contains(&final_p)
            {
                let mut visited = std::collections::HashSet::new();
                let mut queue: std::collections::VecDeque<(Point, u16)> =
                    std::collections::VecDeque::new();
                visited.insert(final_p);
                queue.push_back((final_p, 1));
                let mut open_space = 0;
                let max_search_depth = 100;
                while let Some((curr, steps)) = queue.pop_front() {
                    open_space += 1;
                    if open_space >= max_search_depth {
                        break;
                    }
                    for &next_d in &dirs {
                        let step_p = Self::calculate_next_head_dir(curr, next_d);
                        let next_steps = steps.saturating_add(1);
                        if let Some(valid_p) = self.get_final_p(step_p)
                            && self.is_safe_final_p(valid_p, next_steps, checking_player)
                            && !self.obstacles.contains(&valid_p)
                            && !visited.contains(&valid_p)
                        {
                            visited.insert(valid_p);
                            queue.push_back((valid_p, next_steps));
                        }
                    }
                }
                if open_space > max_open_space {
                    max_open_space = open_space;
                    best_dir = Some(d);
                }
            }
        }
        best_dir
    }
    const fn calculate_wrapped_head(&self, next_head: Point) -> Point {
        let mut x = next_head.x;
        let mut y = next_head.y;
        if x == 0 {
            x = self.width - 2;
        } else if x >= self.width - 1 {
            x = 1;
        }
        if y == 0 {
            y = self.height - 2;
        } else if y >= self.height - 1 {
            y = 1;
        }
        Point {
            x,
            y,
        }
    }
    pub(crate) fn handle_death(&mut self, cause: &str) {
        let head = self.snake.head();
        self.spawn_particles(
            f32::from(head.x),
            f32::from(head.y),
            30,
            crate::color::Color::Red,
            'X',
        );
        if self.stats.equipped_class == Some(crate::game::HeroClass::Rogue)
            && self.rng.gen_bool(0.2)
        {
            crate::game::beep();
        } else if self.stats.unlocked_artifacts.contains(&crate::game::Artifact::GhostCloak)
            && self.rng.gen_bool(0.10)
        {
            // Ghost Cloak saves you
        } else {
            self.lives = self.lives.saturating_sub(1);
        }
        self.just_died = true;
        beep();
        if self.lives == 0 {
            self.stats.games_played += 1;
            self.stats.total_time_s += self.start_time.elapsed().as_secs();
            self.save_stats();
            self.check_achievements();
            self.death_message = cause.to_string();
            let is_high_score = self.high_scores.len() < 5
                || self.score > self.high_scores.last().map_or(0, |(_, s)| *s);
            if is_high_score && self.score > 0 {
                if self.used_bot_this_session {
                    self.save_high_score("[BOT]".to_string(), self.score);
                    self.state = GameState::GameOver;
                } else {
                    self.state = GameState::EnterName;
                    self.player_name.clear();
                }
            } else {
                self.state = GameState::GameOver;
            }
            if self.score > self.high_score {
                self.high_score = self.score;
            }
        } else {
            self.respawn();
        }
    }
}
