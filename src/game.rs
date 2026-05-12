use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, Read, Write},
    time::{Duration, Instant, SystemTime},
};

use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::snake::{Direction, Point, Snake};

#[derive(Copy, Clone, Eq, PartialEq)]
struct AStarState {
    f_score: u16,
    position: crate::snake::Point,
}

impl Ord for AStarState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .f_score
            .cmp(&self.f_score)
            .then_with(|| other.position.x.cmp(&self.position.x))
            .then_with(|| other.position.y.cmp(&self.position.y))
    }
}

impl PartialOrd for AStarState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(
    clap::ValueEnum,
    Clone,
    Copy,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Default,
)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
    Insane,
    GodMode,
}

impl Difficulty {
    pub const fn next(self) -> Self {
        match self {
            Self::Easy => Self::Normal,
            Self::Normal => Self::Hard,
            Self::Hard => Self::Insane,
            Self::Insane => Self::GodMode,
            Self::GodMode => Self::Easy,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::Easy => Self::GodMode,
            Self::Normal => Self::Easy,
            Self::Hard => Self::Normal,
            Self::Insane => Self::Hard,
            Self::GodMode => Self::Insane,
        }
    }
}

#[derive(
    clap::ValueEnum,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Default,
    Copy,
)]
pub enum Theme {
    #[default]
    Classic,
    Dark,
    Retro,
    Neon,
    Ocean,
    Matrix,
    Premium,
    Cyberpunk,
    Rainbow,
    Hacker,
    Blockchain,
    Esports,
}

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum PowerUpType {
    SlowDown,
    SpeedBoost,
    Invincibility,
    ExtraLife,
    PassThroughWalls,
    Shrink,
    ClearObstacles,
    ScoreMultiplier,
    Teleport,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct PowerUp {
    pub p_type: PowerUpType,
    pub location: Point,
    #[serde_as(as = "Option<serde_with::TimestampSeconds<i64>>")]
    pub activation_time: Option<SystemTime>,
}

pub fn beep() {
    print!("\x07");
    let _ = io::stdout().flush();
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Debug, clap::ValueEnum)]
pub enum GameMode {
    #[default]
    SinglePlayer,
    Campaign,
    LocalMultiplayer,
    OnlineMultiplayer,
    PlayerVsBot,
    BotVsBot,
    BattleRoyale,
    TimeAttack,
    Survival,
    Zen,
    Maze,
    CustomLevel,
    Speedrun,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
    GameWon,
    Help,
    Settings,
    NftShop,
    Stats,
    Achievements,
    EnterName,
    ConfirmQuit,
    LevelEditor,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Achievement {
    FirstBlood,
    HighScorer,
    Rich,
    BotUser,
}

pub const fn default_lives() -> u32 {
    3
}

pub const fn default_wrap_mode() -> bool {
    false
}
pub const fn default_skin() -> char {
    '█'
}
pub const fn default_campaign_level() -> u32 {
    1
}

#[derive(Clone)]
pub struct HistoryState {
    pub snake: Snake,
    pub player2: Option<Snake>,
    pub food: Point,
    pub obstacles: HashSet<Point>,
    pub score: u32,
    pub bonus_food: Option<(Point, Instant)>,
    pub power_up: Option<PowerUp>,
    pub lives: u32,
    pub food_eaten_session: u32,
    pub campaign_level: u32,
    pub safe_zone_margin: u16,
    pub last_shrink_time: Instant,
    pub last_obstacle_spawn_time: Instant,
}

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    #[serde(default)]
    pub mode: GameMode,
    pub snake: Snake,
    #[serde(default)]
    pub player2: Option<Snake>,
    pub food: Point,
    pub obstacles: HashSet<Point>,
    pub score: u32,
    #[serde(default)]
    pub bonus_food: Option<(Point, u64)>, // elapsed seconds
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
    pub used_bot_this_game: bool,
    #[serde(default)]
    pub food_eaten_session: u32,
    #[serde(default = "default_campaign_level")]
    pub campaign_level: u32,
    #[serde(default)]
    pub safe_zone_margin: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShopItem {
    Skin(char),
    Theme(Theme),
}

pub const AVAILABLE_ITEMS: [(ShopItem, u32); 14] = [
    (ShopItem::Skin('💎'), 100),
    (ShopItem::Skin('👾'), 250),
    (ShopItem::Skin('🐍'), 500),
    (ShopItem::Skin('🚀'), 1000),
    (ShopItem::Skin('🦍'), 2000),
    (ShopItem::Skin('₿'), 5000),
    (ShopItem::Skin('Ξ'), 10_000),
    (ShopItem::Skin('Ð'), 25_000),
    (ShopItem::Theme(Theme::Premium), 5000),
    (ShopItem::Theme(Theme::Cyberpunk), 10_000),
    (ShopItem::Theme(Theme::Rainbow), 25_000),
    (ShopItem::Theme(Theme::Hacker), 50_000),
    (ShopItem::Theme(Theme::Blockchain), 100_000),
    (ShopItem::Theme(Theme::Esports), 250_000),
];

pub fn default_unlocked_themes() -> Vec<Theme> {
    vec![Theme::Classic, Theme::Dark, Theme::Retro, Theme::Neon, Theme::Ocean, Theme::Matrix]
}

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
}

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub symbol: char,
    pub color: crossterm::style::Color,
}

#[expect(clippy::struct_excessive_bools, reason = "Game struct naturally has many bools")]
pub struct Game {
    pub width: u16,
    pub height: u16,
    pub wrap_mode: bool,
    pub snake: Snake,
    pub food: Point,
    pub bonus_food: Option<(Point, Instant)>,
    pub power_up: Option<PowerUp>,
    pub obstacles: HashSet<Point>,
    pub score: u32,
    pub high_score: u32,
    pub high_scores: Vec<(String, u32)>,
    pub state: GameState,
    pub rng: rand::rngs::ThreadRng,
    pub just_died: bool,
    pub skin: char,
    pub theme: Theme,
    pub lives: u32,
    pub menu_selection: usize,
    pub settings_selection: usize,
    pub nft_selection: usize,
    pub stats: Statistics,
    pub start_time: Instant,
    pub death_message: String,
    pub difficulty: Difficulty,
    pub player_name: String,
    pub previous_state: Option<GameState>,
    pub auto_pilot: bool,
    #[expect(
        clippy::struct_field_names,
        reason = "Used specifically for game logic, name is fine"
    )]
    pub used_bot_this_game: bool,
    pub autopilot_path: Vec<Point>,
    pub food_eaten_session: u32,
    pub mode: GameMode,
    pub player2: Option<Snake>,
    pub campaign_level: u32,
    pub safe_zone_margin: u16,
    pub last_shrink_time: Instant,
    pub last_obstacle_spawn_time: Instant,
    pub history: std::collections::VecDeque<HistoryState>,
    pub editor_cursor: Option<Point>,
    pub particles: Vec<Particle>,
}

impl Game {
    pub fn new(
        width: u16,
        height: u16,
        wrap_mode: bool,
        skin: char,
        theme: Theme,
        difficulty: Difficulty,
        mode: GameMode,
    ) -> Self {
        let mut rng = rand::thread_rng();
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
        let obstacles = Self::generate_obstacles(width, height, &snake, avoid, &mut rng, obs_count, 0);
        let avoid_food = |p: &Point| obstacles.contains(p);
        let food = Self::get_random_empty_point(width, height, &snake, avoid_food, &mut rng, 0)
            .expect("Board cannot be full on start");

        // Migration step
        if std::path::Path::new("highscore.txt").exists()
            && !std::path::Path::new("highscore_normal.txt").exists()
        {
            let _ = std::fs::rename("highscore.txt", "highscore_normal.txt");
        }

        let high_scores = Self::load_high_scores_from_file(&Self::get_high_score_filename(difficulty));
        let high_score = high_scores.first().map_or(0, |(_, s)| *s);
        let stats = Self::load_stats();
        Self {
            width,
            height,
            wrap_mode,
            snake,
            food,
            bonus_food: None,
            power_up: None,
            obstacles,
            score: 0,
            high_score,
            high_scores,
            state: GameState::Menu,
            rng,
            just_died: false,
            skin,
            theme,
            lives: 3,
            menu_selection: 0,
            settings_selection: 0,
            nft_selection: 0,
            stats,
            start_time: Instant::now(),
            death_message: String::new(),
            difficulty,
            player_name: String::new(),
            previous_state: None,
            auto_pilot: false,
            used_bot_this_game: false,
            autopilot_path: Vec::new(),
            food_eaten_session: 0,
            mode,
            player2: None,
            campaign_level: 1,
            safe_zone_margin: 0,
            last_shrink_time: Instant::now(),
            last_obstacle_spawn_time: Instant::now(),
            history: std::collections::VecDeque::new(),
            editor_cursor: None,
            particles: Vec::new(),
        }
    }

    pub fn get_high_score_filename(difficulty: Difficulty) -> String {
        format!("highscore_{difficulty:?}.txt").to_lowercase()
    }

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

    fn atomic_write(path: &str, content: impl AsRef<[u8]>) -> io::Result<()> {
        let mut rng = rand::thread_rng();
        let suffix: u32 = rng.r#gen();
        let tmp_path = format!("{path}.{suffix}.tmp");

        let mut options = fs::File::options();
        options.write(true).create_new(true);

        #[cfg(unix)]
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

    pub fn save_stats_to_file(&self, path: &str) {
        if let Ok(json) = serde_json::to_string(&self.stats) {
            let _ = Self::atomic_write(path, json);
        }
    }

    pub fn save_high_score(&mut self, name: String, score: u32) {
        let filename = Self::get_high_score_filename(self.difficulty);
        self.save_high_score_to_file(&filename, name, score);
    }

    pub fn update_high_scores(&mut self) {
        self.high_scores = Self::load_high_scores_from_file(&Self::get_high_score_filename(self.difficulty));
        self.high_score = self.high_scores.first().map_or(0, |(_, s)| *s);
    }

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

    pub fn save_custom_level(&self) {
        if let Ok(json) = serde_json::to_string(&self.obstacles) {
            let _ = Self::atomic_write("custom_level.json", json);
        }
    }

    pub fn load_custom_level() -> HashSet<Point> {
        File::open("custom_level.json")
            .ok()
            .and_then(|f| serde_json::from_reader(f.take(1024 * 1024)).ok())
            .unwrap_or_default()
    }

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
            food: self.food,
            obstacles: self.obstacles.clone(),
            score: self.score,
            bonus_food: self.bonus_food.map(|(p, t)| (p, t.elapsed().as_secs())),
            power_up: self.power_up.clone(),
            lives: self.lives,
            difficulty: self.difficulty,
            theme: self.theme,
            wrap_mode: self.wrap_mode,
            skin: self.skin,
            auto_pilot: self.auto_pilot,
            used_bot_this_game: self.used_bot_this_game,
            food_eaten_session: self.food_eaten_session,
            campaign_level: self.campaign_level,
            safe_zone_margin: self.safe_zone_margin,
        };
        if let Ok(json) = serde_json::to_string(&state) {
            let _ = Self::atomic_write(path, json);
        }
    }

    pub fn load_game(&mut self) -> bool {
        self.load_game_from_file("savegame.json")
    }

    fn load_game_from_file(&mut self, path: &str) -> bool {
        File::open(path)
            .ok()
            .and_then(|f| serde_json::from_reader::<_, SaveState>(f.take(1024 * 1024)).ok())
            .is_some_and(|mut state| {
                // Validate bounds
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
                if let Some(pu) = &state.power_up
                    && !valid_point(&pu.location)
                {
                    return false;
                }

                if let Some(p2) = &state.player2
                    && !p2.body.iter().all(valid_point) {
                        return false;
                    }

                state.snake.rebuild_map();
                if let Some(p2) = &mut state.player2 {
                    p2.rebuild_map();
                }

                self.mode = state.mode;
                self.snake = state.snake;
                self.player2 = state.player2;
                self.food = state.food;
                self.obstacles = state.obstacles;
                self.score = state.score;
                self.bonus_food = state.bonus_food.and_then(|(p, elapsed)| {
                    Instant::now().checked_sub(Duration::from_secs(elapsed)).map(|t| (p, t))
                });
                self.lives = state.lives;
                self.power_up = state.power_up;
                self.difficulty = state.difficulty;
                self.theme = state.theme;
                self.wrap_mode = state.wrap_mode;
                self.skin = state.skin;
                self.auto_pilot = state.auto_pilot;
                self.used_bot_this_game = state.used_bot_this_game;
                self.food_eaten_session = state.food_eaten_session;
                self.campaign_level = state.campaign_level;
                self.safe_zone_margin = state.safe_zone_margin;
                self.last_shrink_time = Instant::now();
                self.last_obstacle_spawn_time = Instant::now();
                self.state = GameState::Paused;
                self.start_time = Instant::now();
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
        rng: &mut rand::rngs::ThreadRng,
        margin: u16,
    ) -> Option<Point> {
        let mut i = 0;
        loop {
            // Point must be within walls (1..WIDTH-1, 1..HEIGHT-1) and margin
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
                // Fallback if the board is completely full
                return None;
            }
        }
    }

    fn generate_obstacles(
        width: u16,
        height: u16,
        snake: &Snake,
        avoid: impl Fn(&Point) -> bool,
        rng: &mut rand::rngs::ThreadRng,
        count: usize,
        margin: u16,
    ) -> HashSet<Point> {
        let mut obstacles = HashSet::new();

        for _ in 0..count {
            let current_avoid = |p: &Point| avoid(p) || obstacles.contains(p);
            if let Some(p) = Self::get_random_empty_point(width, height, snake, current_avoid, rng, margin)
            {
                obstacles.insert(p);
            }
        }
        obstacles
    }

    pub fn shift_timers(&mut self, delta: Duration) {
        // Shift start time so time logic doesn't race when paused
        if let Some(new_time) = self.start_time.checked_add(delta) {
            self.start_time = new_time;
        }

        // Shift bonus food spawn time
        if let Some((pos, spawn_time)) = self.bonus_food
            && let Some(new_time) = spawn_time.checked_add(delta)
        {
            self.bonus_food = Some((pos, new_time));
        }

        // Shift power up activation time
        if let Some(power_up) = &mut self.power_up
            && let Some(activation_time) = power_up.activation_time
            && let Some(new_time) = activation_time.checked_add(delta)
        {
            power_up.activation_time = Some(new_time);
        }

        // Shift last shrink time
        if let Some(new_time) = self.last_shrink_time.checked_add(delta) {
            self.last_shrink_time = new_time;
        }

        // Shift last obstacle spawn time
        if let Some(new_time) = self.last_obstacle_spawn_time.checked_add(delta) {
            self.last_obstacle_spawn_time = new_time;
        }
    }

    pub fn generate_campaign_obstacles(&self) -> HashSet<Point> {
        let mut obstacles = HashSet::new();
        if self.campaign_level == 1 {
            // Level 1: empty board
            return obstacles;
        } else if self.campaign_level == 2 {
            // Level 2: horizontal line
            let y = self.height / 2;
            let start_x = (self.width / 2).saturating_sub(2).max(1);
            let end_x = (self.width / 2 + 2).min(self.width - 2);
            for x in start_x..=end_x {
                obstacles.insert(Point { x, y });
            }
        } else {
            // Level 3+: cross
            let center_x = self.width / 2;
            let center_y = self.height / 2;
            obstacles.insert(Point { x: center_x, y: center_y });
            obstacles.insert(Point { x: center_x.saturating_sub(1).max(1), y: center_y });
            obstacles.insert(Point { x: center_x + 1, y: center_y });
            obstacles.insert(Point { x: center_x, y: center_y.saturating_sub(1).max(1) });
            obstacles.insert(Point { x: center_x, y: center_y + 1 });
        }
        obstacles
    }

    pub fn reset(&mut self) {
        let start_x = self.width / 2;
        let start_y = self.height / 2;

        if self.mode == GameMode::Campaign {
            self.campaign_level = 1;
        }

        match self.mode {
            GameMode::SinglePlayer | GameMode::Campaign | GameMode::TimeAttack | GameMode::Speedrun | GameMode::Survival | GameMode::Zen | GameMode::Maze | GameMode::CustomLevel => {
                self.snake = Snake::new(Point {
                    x: start_x,
                    y: start_y,
                });
                self.player2 = None;
            },
            GameMode::LocalMultiplayer | GameMode::OnlineMultiplayer | GameMode::PlayerVsBot | GameMode::BotVsBot | GameMode::BattleRoyale => {
                self.snake = Snake::new(Point {
                    x: start_x - 5,
                    y: start_y,
                });
                self.player2 = Some(Snake::new(Point {
                    x: start_x + 5,
                    y: start_y,
                }));
            }
        }

        let obs_count = if self.mode == GameMode::Zen || self.mode == GameMode::Maze {
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
            if self.mode == GameMode::SinglePlayer || self.mode == GameMode::Campaign || self.mode == GameMode::TimeAttack || self.mode == GameMode::Speedrun || self.mode == GameMode::Survival || self.mode == GameMode::Zen || self.mode == GameMode::Maze || self.mode == GameMode::CustomLevel {
                p.x == start_x && p.y == start_y - 1
            } else {
                (p.x == start_x + 5 || p.x == start_x - 5) && p.y == start_y - 1
            }
        };

        let empty_snake = Snake::new(Point { x: 1, y: 1 });
        let ref_snake = if self.mode == GameMode::SinglePlayer || self.mode == GameMode::Campaign || self.mode == GameMode::TimeAttack || self.mode == GameMode::Speedrun || self.mode == GameMode::Survival || self.mode == GameMode::Zen || self.mode == GameMode::Maze || self.mode == GameMode::CustomLevel { &self.snake } else { &empty_snake }; // For collision we'll just check avoid and body maps later

        if self.mode == GameMode::CustomLevel {
            self.obstacles = Self::load_custom_level();
            let body_map = self.snake.body_map.clone();
            self.obstacles.retain(|p| !body_map.contains_key(p));
        } else if self.mode == GameMode::Campaign {
            self.obstacles = self.generate_campaign_obstacles();
            // remove obstacles that collide with snake body
            let body_map = self.snake.body_map.clone();
            self.obstacles.retain(|p| !body_map.contains_key(p));
        } else if self.mode == GameMode::Maze {
            self.obstacles.clear();
            let y1 = self.height / 3;
            let y2 = 2 * self.height / 3;
            for x in 5..(self.width - 5) {
                self.obstacles.insert(Point { x, y: y1 });
                self.obstacles.insert(Point { x, y: y2 });
            }
            let body_map = self.snake.body_map.clone();
            self.obstacles.retain(|p| !body_map.contains_key(p));
        } else {
            let mut obstacles = HashSet::new();
            for _ in 0..obs_count {
                let current_avoid = |p: &Point| {
                    avoid(p) || obstacles.contains(p) || self.snake.body_map.contains_key(p) || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p))
                };
                if let Some(p) = Self::get_random_empty_point(self.width, self.height, ref_snake, current_avoid, &mut self.rng, 0) {
                    obstacles.insert(p);
                }
            }
            self.obstacles = obstacles;
        }

        let avoid_food = |p: &Point| self.obstacles.contains(p) || self.snake.body_map.contains_key(p) || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p));
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
        self.power_up = None;
        self.score = 0;
        self.lives = 3;
        self.state = GameState::Playing;
        self.just_died = false;
        self.start_time = Instant::now();
        self.food_eaten_session = 0;
        self.auto_pilot = false;
        self.used_bot_this_game = false;
        self.safe_zone_margin = 0;
        self.last_shrink_time = Instant::now();
        self.last_obstacle_spawn_time = Instant::now();
        self.history.clear();
        self.particles.clear();
    }

    fn respawn(&mut self) {
        let start_x = self.width / 2;
        let start_y = self.height / 2;

        match self.mode {
            GameMode::SinglePlayer | GameMode::Campaign | GameMode::TimeAttack | GameMode::Speedrun | GameMode::Survival | GameMode::Zen | GameMode::Maze | GameMode::CustomLevel => {
                self.snake = Snake::new(Point {
                    x: start_x,
                    y: start_y,
                });
                self.player2 = None;
                self.obstacles.retain(|p| {
                    !(p.x == start_x && (p.y >= start_y.saturating_sub(1) && p.y <= start_y + 2))
                });
            },
            GameMode::LocalMultiplayer | GameMode::OnlineMultiplayer | GameMode::PlayerVsBot | GameMode::BotVsBot | GameMode::BattleRoyale => {
                self.snake = Snake::new(Point {
                    x: start_x - 5,
                    y: start_y,
                });
                self.player2 = Some(Snake::new(Point {
                    x: start_x + 5,
                    y: start_y,
                }));
                self.obstacles.retain(|p| {
                    !((p.x == start_x - 5 && (p.y >= start_y.saturating_sub(1) && p.y <= start_y + 2)) ||
                      (p.x == start_x + 5 && (p.y >= start_y.saturating_sub(1) && p.y <= start_y + 2)))
                });
            }
        }

        self.safe_zone_margin = 0;
        self.last_shrink_time = Instant::now();
        self.last_obstacle_spawn_time = Instant::now();
    }

    pub fn handle_input(&mut self, dir: Direction, player: u8) {
        // Prevent 180 degree turns and queue input if we already have one
        // We buffer up to 2 moves ahead to prevent "laggy" feel if user mashes keys.

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
            && let Some(p2) = &mut self.player2 {
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

    fn handle_autopilot_moves(&mut self) {
        // --- Handle Player 1 Autopilot ---
        if (self.auto_pilot || self.mode == GameMode::BotVsBot)
            && self.snake.direction_queue.is_empty()
            && let Some(dir) = self.calculate_autopilot_move()
        {
            self.snake.direction_queue.push_back(dir);
        }

        // --- Handle Player 2 Autopilot ---
        if self.mode == GameMode::PlayerVsBot || self.mode == GameMode::BotVsBot {
            let is_empty = self.player2.as_ref().is_some_and(|p2| p2.direction_queue.is_empty());
            if is_empty
                && let Some(dir) = self.calculate_p2_autopilot_move()
                    && let Some(p2) = &mut self.player2 {
                        p2.direction_queue.push_back(dir);
                    }
        }
    }

    fn calculate_final_heads(&self) -> (Point, Option<Point>, bool, bool) {
        let head1 = self.snake.head();
        let next_head1 = Self::calculate_next_head_dir(head1, self.snake.direction);

        let next_head2_opt = self.player2.as_ref().map(|p2| {
            let head2 = p2.head();
            Self::calculate_next_head_dir(head2, p2.direction)
        });

        let can_pass_through_walls = self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::PassThroughWalls
                && p.activation_time
                    .is_some_and(|t| t.elapsed().unwrap_or_default() < Duration::from_secs(5))
        });

        let mut hit_wall1 = false;
        let final_head1 = if (self.wrap_mode || can_pass_through_walls || self.mode == GameMode::Zen) && self.mode != GameMode::BattleRoyale {
            self.calculate_wrapped_head(next_head1)
        } else {
            let margin = if self.mode == GameMode::BattleRoyale { self.safe_zone_margin } else { 0 };
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
            if (self.wrap_mode || can_pass_through_walls || self.mode == GameMode::Zen) && self.mode != GameMode::BattleRoyale {
                self.calculate_wrapped_head(next_head2)
            } else {
                let margin = if self.mode == GameMode::BattleRoyale { self.safe_zone_margin } else { 0 };
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
            self.food = state.food;
            self.obstacles = state.obstacles;
            self.score = state.score;
            self.bonus_food = state.bonus_food;
            self.power_up = state.power_up;
            self.lives = state.lives;
            self.food_eaten_session = state.food_eaten_session;
            self.campaign_level = state.campaign_level;
            self.safe_zone_margin = state.safe_zone_margin;
            self.last_shrink_time = state.last_shrink_time;
            self.last_obstacle_spawn_time = state.last_obstacle_spawn_time;
        }
    }

    pub fn save_history_state(&mut self) {
        let state = HistoryState {
            snake: self.snake.clone(),
            player2: self.player2.clone(),
            food: self.food,
            obstacles: self.obstacles.clone(),
            score: self.score,
            bonus_food: self.bonus_food,
            power_up: self.power_up.clone(),
            lives: self.lives,
            food_eaten_session: self.food_eaten_session,
            campaign_level: self.campaign_level,
            safe_zone_margin: self.safe_zone_margin,
            last_shrink_time: self.last_shrink_time,
            last_obstacle_spawn_time: self.last_obstacle_spawn_time,
        };

        self.history.push_back(state);
        if self.history.len() > 50 {
            self.history.pop_front();
        }
    }

    pub fn spawn_particles(&mut self, x: f32, y: f32, count: usize, color: crossterm::style::Color, symbol: char) {
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

    #[expect(clippy::too_many_lines, reason = "Game loop inherently requires handling multiple states and events")]
    pub fn update(&mut self) {
        if self.state != GameState::Playing {
            return;
        }

        self.save_history_state();

        for p in &mut self.particles {
            p.x += p.vx;
            p.y += p.vy;
            p.lifetime -= 1.0;
        }
        self.particles.retain(|p| p.lifetime > 0.0);

        if self.mode == GameMode::TimeAttack && self.start_time.elapsed() >= Duration::from_secs(60) {
            self.handle_death("Time's up!");
            return;
        }

        if self.mode == GameMode::Speedrun && self.food_eaten_session >= 50 {
            self.handle_win();
            return;
        }

        if self.mode == GameMode::BattleRoyale && self.last_shrink_time.elapsed() >= Duration::from_secs(10) {
            let max_margin = (self.width.min(self.height) / 2).saturating_sub(2);
            if self.safe_zone_margin < max_margin {
                self.safe_zone_margin += 1;
                self.last_shrink_time = Instant::now();

                // Relocate out-of-bounds food
                if self.food.x <= self.safe_zone_margin || self.food.x >= self.width - 1 - self.safe_zone_margin ||
                   self.food.y <= self.safe_zone_margin || self.food.y >= self.height - 1 - self.safe_zone_margin {
                    let avoid_food = |p: &Point| self.obstacles.contains(p) || self.snake.body_map.contains_key(p) || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p));
                    if let Some(new_food) = Self::get_random_empty_point(self.width, self.height, &self.snake, avoid_food, &mut self.rng, self.safe_zone_margin) {
                        self.food = new_food;
                    }
                }

                // Relocate out-of-bounds bonus food
                if let Some((bp, _)) = self.bonus_food
                    && (bp.x <= self.safe_zone_margin || bp.x >= self.width - 1 - self.safe_zone_margin ||
                       bp.y <= self.safe_zone_margin || bp.y >= self.height - 1 - self.safe_zone_margin) {
                    self.bonus_food = None; // just remove it
                }

                // Relocate out-of-bounds power-up
                if let Some(pu) = &self.power_up
                    && (pu.location.x <= self.safe_zone_margin || pu.location.x >= self.width - 1 - self.safe_zone_margin ||
                       pu.location.y <= self.safe_zone_margin || pu.location.y >= self.height - 1 - self.safe_zone_margin) {
                    self.power_up = None; // just remove it
                }
                crate::game::beep(); // Beep on map shrink
            }
        }

        if self.mode == GameMode::Survival && self.last_obstacle_spawn_time.elapsed() >= Duration::from_secs(3) {
            self.last_obstacle_spawn_time = Instant::now();
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

        self.handle_autopilot_moves();

        // --- Apply Input ---
        if let Some(dir) = self.snake.direction_queue.pop_front() {
            self.snake.direction = dir;
        }

        if let Some(p2) = &mut self.player2
            && let Some(dir) = p2.direction_queue.pop_front() {
                p2.direction = dir;
            }

        self.manage_bonus_food();
        self.manage_power_ups();

        // --- Calculate Next Heads ---
        let (final_head1, final_head2_opt, hit_wall1, hit_wall2) = self.calculate_final_heads();

        let hit_obstacle1 = self.obstacles.contains(&final_head1);
        let hit_obstacle2 = final_head2_opt.is_some_and(|fh2| self.obstacles.contains(&fh2));

        let out_of_bounds1 = if self.mode == GameMode::BattleRoyale {
            final_head1.x <= self.safe_zone_margin || final_head1.x >= self.width - 1 - self.safe_zone_margin ||
            final_head1.y <= self.safe_zone_margin || final_head1.y >= self.height - 1 - self.safe_zone_margin
        } else { false };

        let out_of_bounds2 = if self.mode == GameMode::BattleRoyale {
            final_head2_opt.is_some_and(|fh2| {
                fh2.x <= self.safe_zone_margin || fh2.x >= self.width - 1 - self.safe_zone_margin ||
                fh2.y <= self.safe_zone_margin || fh2.y >= self.height - 1 - self.safe_zone_margin
            })
        } else { false };

        let is_invincible = self.mode == GameMode::Zen || self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::Invincibility
                && p.activation_time
                    .is_some_and(|t| t.elapsed().unwrap_or_default() < Duration::from_secs(5))
        });

        // --- Resolution ---

        let mut p1_dead = false;
        let mut p2_dead = false;

        if hit_wall1 || out_of_bounds1 { p1_dead = true; }
        if hit_obstacle1 && !is_invincible { p1_dead = true; }

        if hit_wall2 || out_of_bounds2 { p2_dead = true; }
        if hit_obstacle2 && !is_invincible { p2_dead = true; }

        // Head-to-Head
        if let Some(final_head2) = final_head2_opt
            && final_head1 == final_head2 {
                p1_dead = true;
                p2_dead = true;
            }

        let old_food_eaten_session = self.food_eaten_session;
        let is_multiplier = self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::ScoreMultiplier
                && p.activation_time
                    .is_some_and(|t| t.elapsed().unwrap_or_default() < Duration::from_secs(5))
        });

        let mut p1_grow = self.check_bonus_food_collision(final_head1, is_multiplier);
        let mut p2_grow = final_head2_opt.is_some_and(|fh2| self.check_bonus_food_collision(fh2, is_multiplier));

        // Process food collisions (first come first serve, resolving P1 first for simplicity unless we want true tie)
        if final_head1 == self.food {
            p1_grow = true;
            if !self.process_food_collision(final_head1, is_multiplier) {
                // Board is full
                self.snake.move_to(final_head1, p1_grow);
                self.handle_win();
                return;
            }
        } else if let Some(final_head2) = final_head2_opt
            && final_head2 == self.food {
                p2_grow = true;
                if !self.process_food_collision(final_head2, is_multiplier) {
                    if let Some(p2) = &mut self.player2 {
                        p2.move_to(final_head2, p2_grow);
                    }
                    self.handle_win();
                    return;
                }
            }

        let (body_p1_dead, body_p2_dead) = self.check_body_collisions(final_head1, final_head2_opt, is_invincible, p1_grow, p2_grow);
        if body_p1_dead { p1_dead = true; }
        if body_p2_dead { p2_dead = true; }

        // Process deaths
        if p1_dead && p2_dead {
            self.handle_death("Draw! Both snakes died!");
            return;
        } else if p1_dead {
            if self.mode == GameMode::SinglePlayer || self.mode == GameMode::TimeAttack || self.mode == GameMode::Speedrun || self.mode == GameMode::Survival {
                self.handle_death("You Died!");
            } else {
                self.handle_death("Player 2 Wins!");
            }
            return;
        } else if p2_dead {
            self.handle_death("Player 1 Wins!");
            return;
        }

        // All good, process power ups
        self.process_power_up_collision(final_head1);
        if let Some(final_head2) = final_head2_opt {
            self.process_power_up_collision(final_head2);
        }

        // Moving the snakes
        self.add_obstacles_if_needed(old_food_eaten_session, final_head1);

        self.snake.move_to(final_head1, p1_grow);
        if let Some(final_head2) = final_head2_opt
            && let Some(p2) = &mut self.player2 {
                p2.move_to(final_head2, p2_grow);
            }
    }

    fn check_body_collisions(&self, final_head1: Point, final_head2_opt: Option<Point>, is_invincible: bool, p1_grow: bool, p2_grow: bool) -> (bool, bool) {
        let mut p1_dead = false;
        let mut p2_dead = false;

        // Body collisions
        // P1 hits itself
        if self.snake.body_map.contains_key(&final_head1) && !is_invincible {
            let is_tail = self.snake.body.back().is_some_and(|tail| final_head1 == *tail);
            if !p1_grow && is_tail {
                // Safe
            } else {
                p1_dead = true;
            }
        }

        // P2 hits itself
        if let Some(final_head2) = final_head2_opt
            && let Some(p2) = &self.player2
                && p2.body_map.contains_key(&final_head2) && !is_invincible {
                    let is_tail = p2.body.back().is_some_and(|tail| final_head2 == *tail);
                    if !p2_grow && is_tail {
                        // Safe
                    } else {
                        p2_dead = true;
                    }
                }

        // Cross-collisions
        if let Some(final_head2) = final_head2_opt {
            if self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(&final_head1)) && !is_invincible {
                let is_tail = self.player2.as_ref().unwrap().body.back().is_some_and(|tail| final_head1 == *tail);
                if !p2_grow && is_tail {
                    // Safe, moving into tail of p2 that will move
                } else {
                    p1_dead = true;
                }
            }
            if self.snake.body_map.contains_key(&final_head2) && !is_invincible {
                let is_tail = self.snake.body.back().is_some_and(|tail| final_head2 == *tail);
                if !p1_grow && is_tail {
                    // Safe, moving into tail of p1 that will move
                } else {
                    p2_dead = true;
                }
            }
        }
        (p1_dead, p2_dead)
    }

    fn process_power_up_collision(&mut self, final_head: Point) {
        let hit_power_up = if let Some(p) = self.power_up.as_ref()
            && final_head == p.location
        {
            true
        } else {
            false
        };

        if hit_power_up {
            self.spawn_particles(f32::from(final_head.x), f32::from(final_head.y), 20, crossterm::style::Color::Yellow, '*');
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

                        // Wrap within 1..width-1 and 1..height-1
                        let inner_width = i32::from(self.width) - 2;
                        let inner_height = i32::from(self.height) - 2;

                        // normalized coordinate (0-based)
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
                p.activation_time = Some(SystemTime::now());
            }
            beep();
        }

        // Remove power up instantly if it was an instant effect that was just activated
        if let Some(p) = self.power_up.as_ref()
            && (p.p_type == PowerUpType::ExtraLife
                || p.p_type == PowerUpType::Shrink
                || p.p_type == PowerUpType::ClearObstacles
                || p.p_type == PowerUpType::Teleport)
            && p.activation_time.is_none()
            && final_head == p.location
        {
            self.power_up = None;
        }
    }

    fn check_bonus_food_collision(&mut self, final_head: Point, is_multiplier: bool) -> bool {
        if self.bonus_food.is_some_and(|(bonus_p, _)| final_head == bonus_p) {
            self.spawn_particles(f32::from(final_head.x), f32::from(final_head.y), 15, crossterm::style::Color::Magenta, '★');
            let diff_multiplier = match self.difficulty {
                Difficulty::Easy => 1,
                Difficulty::Normal => 2,
                Difficulty::Hard => 3,
                Difficulty::Insane => 5,
                Difficulty::GodMode => 10,
            };
            let added_score = if is_multiplier {
                10 * diff_multiplier
            } else {
                5 * diff_multiplier
            };
            self.score += added_score;
            self.food_eaten_session += 1;
            self.stats.total_score += added_score;
            self.stats.total_food_eaten += 1;
            self.stats.coins += added_score;
            self.bonus_food = None;
            beep();
            true
        } else {
            false
        }
    }

    fn process_food_collision(&mut self, final_head: Point, is_multiplier: bool) -> bool {
        self.spawn_particles(f32::from(final_head.x), f32::from(final_head.y), 8, crossterm::style::Color::Green, '+');
        let diff_multiplier = match self.difficulty {
            Difficulty::Easy => 1,
            Difficulty::Normal => 2,
            Difficulty::Hard => 3,
            Difficulty::Insane => 5,
            Difficulty::GodMode => 10,
        };
        let added_score = if is_multiplier {
            2 * diff_multiplier
        } else {
            diff_multiplier
        };
        self.score += added_score;
        self.food_eaten_session += 1;
        self.stats.total_score += added_score;
        self.stats.total_food_eaten += 1;
        self.stats.coins += added_score;
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
        if let Some(new_food) =
            Self::get_random_empty_point(self.width, self.height, &self.snake, avoid, &mut self.rng, self.safe_zone_margin)
        {
            self.food = new_food;
            true
        } else {
            false
        }
    }

    fn add_obstacles_if_needed(&mut self, old_food_eaten_session: u32, final_head: Point) {
        if self.mode == GameMode::Campaign || self.mode == GameMode::Maze || self.mode == GameMode::CustomLevel {
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

    pub fn check_achievements(&mut self) {
        let mut new_achievements = Vec::new();
        if !self.stats.unlocked_achievements.contains(&Achievement::FirstBlood) && self.stats.games_played > 0 {
            new_achievements.push(Achievement::FirstBlood);
        }
        if !self.stats.unlocked_achievements.contains(&Achievement::HighScorer) && self.score >= 100 {
            new_achievements.push(Achievement::HighScorer);
        }
        if !self.stats.unlocked_achievements.contains(&Achievement::Rich) && self.stats.coins >= 1000 {
            new_achievements.push(Achievement::Rich);
        }
        if !self.stats.unlocked_achievements.contains(&Achievement::BotUser) && self.used_bot_this_game {
            new_achievements.push(Achievement::BotUser);
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

        let is_high_score = self.high_scores.len() < 5
            || self.score > self.high_scores.last().map_or(0, |(_, s)| *s);
        if is_high_score && self.score > 0 {
            if self.used_bot_this_game {
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
                let p_type = match self.rng.gen_range(0..9) {
                    0 => PowerUpType::SlowDown,
                    1 => PowerUpType::SpeedBoost,
                    2 => PowerUpType::Invincibility,
                    3 => PowerUpType::PassThroughWalls,
                    4 => PowerUpType::Shrink,
                    5 => PowerUpType::ClearObstacles,
                    6 => PowerUpType::ScoreMultiplier,
                    7 => PowerUpType::Teleport,
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

    fn manage_bonus_food(&mut self) {
        if let Some((_, spawn_time)) = self.bonus_food {
            if spawn_time.elapsed() > Duration::from_secs(5) {
                self.bonus_food = None;
            }
        } else if self.rng.gen_bool(0.01) {
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
                self.bonus_food = Some((bonus, Instant::now()));
            }
        }
    }

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

    pub fn get_final_p(&self, p: Point) -> Option<Point> {
        let can_pass_through_walls = self.power_up.as_ref().is_some_and(|pu| {
            pu.p_type == PowerUpType::PassThroughWalls
                && pu
                    .activation_time
                    .is_some_and(|t| t.elapsed().unwrap_or_default() < Duration::from_secs(5))
        });

        if (self.wrap_mode || can_pass_through_walls || self.mode == GameMode::Zen) && self.mode != GameMode::BattleRoyale {
            Some(self.calculate_wrapped_head(p))
        } else {
            let margin = if self.mode == GameMode::BattleRoyale { self.safe_zone_margin } else { 0 };
            if p.x <= margin || p.x >= self.width - 1 - margin || p.y <= margin || p.y >= self.height - 1 - margin {
                None // Hit wall or out of bounds
            } else {
                Some(p)
            }
        }
    }

    pub fn is_safe_final_p(&self, final_p: Point, steps: u16, _checking_player: u8) -> bool {
        let is_invincible = self.mode == GameMode::Zen || self.power_up.as_ref().is_some_and(|pu| {
            pu.p_type == PowerUpType::Invincibility
                && pu
                    .activation_time
                    .is_some_and(|t| t.elapsed().unwrap_or_default() < Duration::from_secs(5))
        });

        if !is_invincible {
            if self.obstacles.contains(&final_p) {
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
                && let Some(pos) = p2.body.iter().position(|&p| p == final_p) {
                    let steps_to_clear =
                        u16::try_from(p2.body.len().saturating_sub(pos)).unwrap_or(u16::MAX);
                    if steps < steps_to_clear {
                        return false;
                    }
                }
        }

        true
    }

    pub fn calculate_autopilot_move(&mut self) -> Option<Direction> {
        let start = self.snake.head();

        let mut targets = vec![self.food];
        if let Some((bf_p, _)) = self.bonus_food {
            targets.push(bf_p);
        }
        if let Some(pu) = &self.power_up
            && pu.activation_time.is_none()
        {
            targets.push(pu.location);
        }

        if let Some((dir, path)) = self.astar_search(start, &targets, 1) {
            self.autopilot_path = path;
            return Some(dir);
        }

        self.autopilot_path.clear();
        self.flood_fill_fallback(start, 1)
    }

    pub fn calculate_p2_autopilot_move(&self) -> Option<Direction> {
        if let Some(p2) = &self.player2 {
            let start = p2.head();

            let mut targets = vec![self.food];
            if let Some((bf_p, _)) = self.bonus_food {
                targets.push(bf_p);
            }
            if let Some(pu) = &self.power_up
                && pu.activation_time.is_none()
            {
                targets.push(pu.location);
            }

            if let Some((dir, _path)) = self.astar_search(start, &targets, 2) {
                // not showing bot path for player2
                return Some(dir);
            }

            self.flood_fill_fallback(start, 2)
        } else {
            None
        }
    }

    fn astar_search(&self, start: Point, targets: &[Point], checking_player: u8) -> Option<(Direction, Vec<Point>)> {
        let mut open_set = std::collections::BinaryHeap::new();
        let mut g_score = std::collections::HashMap::new();
        let mut first_step = std::collections::HashMap::new();
        let mut came_from = std::collections::HashMap::new();

        g_score.insert(start, 0);

        let heuristic = |p: Point| -> u16 {
            let can_pass_through_walls = self.power_up.as_ref().is_some_and(|pu| {
                pu.p_type == PowerUpType::PassThroughWalls
                    && pu.activation_time.is_some_and(|time| {
                        time.elapsed().unwrap_or_default() < Duration::from_secs(5)
                    })
            });
            targets
                .iter()
                .map(|t| {
                    let mut dx = p.x.abs_diff(t.x);
                    let mut dy = p.y.abs_diff(t.y);
                    if (self.wrap_mode || can_pass_through_walls || self.mode == GameMode::Zen) && self.mode != GameMode::BattleRoyale {
                        dx = std::cmp::min(dx, self.width.saturating_sub(2).saturating_sub(dx));
                        dy = std::cmp::min(dy, self.height.saturating_sub(2).saturating_sub(dy));
                    }
                    dx.saturating_add(dy)
                })
                .min()
                .unwrap_or(0)
        };

        let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
        for &d in &dirs {
            let next_p = Self::calculate_next_head_dir(start, d);
            if let Some(final_p) = self.get_final_p(next_p)
                && self.is_safe_final_p(final_p, 1, checking_player)
            {
                let cost = 1;
                g_score.insert(final_p, cost);
                first_step.insert(final_p, d);
                came_from.insert(final_p, start);
                open_set.push(AStarState {
                    f_score: cost + heuristic(final_p),
                    position: final_p,
                });
            }
        }

        while let Some(AStarState {
            position: current,
            ..
        }) = open_set.pop()
        {
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
                    && self.is_safe_final_p(final_p, tentative_g, checking_player)
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
                    open_set.push(AStarState {
                        f_score: tentative_g.saturating_add(heuristic(final_p)),
                        position: final_p,
                    });
                }
            }
        }

        None
    }

    fn flood_fill_fallback(&self, start: Point, checking_player: u8) -> Option<Direction> {
        let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
        let mut best_dir = None;
        let mut max_open_space = 0;

        for &d in &dirs {
            let next_p = Self::calculate_next_head_dir(start, d);
            if let Some(final_p) = self.get_final_p(next_p)
                && self.is_safe_final_p(final_p, 1, checking_player)
            {
                let mut visited = std::collections::HashSet::new();
                let mut queue: std::collections::VecDeque<(Point, u16)> =
                    std::collections::VecDeque::new();

                visited.insert(final_p);
                queue.push_back((final_p, 1));

                let mut open_space = 0;
                let max_search_depth = 100; // Limit search to avoid performance issues

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

    fn handle_death(&mut self, cause: &str) {
        let head = self.snake.head();
        self.spawn_particles(f32::from(head.x), f32::from(head.y), 30, crossterm::style::Color::Red, 'X');

        self.lives -= 1;
        self.just_died = true;
        beep();

        if self.lives == 0 {
            // Update stats on Game Over
            self.stats.games_played += 1;
            self.stats.total_time_s += self.start_time.elapsed().as_secs();
            self.save_stats();
            self.check_achievements();

            self.death_message = cause.to_string();
            let is_high_score = self.high_scores.len() < 5
                || self.score > self.high_scores.last().map_or(0, |(_, s)| *s);
            if is_high_score && self.score > 0 {
                if self.used_bot_this_game {
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

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use super::*;

    #[test]
    fn test_save_and_load_settings() {
        let file_path = "savegame_test_settings.json";
        let _ = std::fs::remove_file(file_path);

        let mut game1 = Game::new(
            20,
            20,
            true, // wrap mode true
            '@',  // custom skin
            crate::game::Theme::Neon,
            crate::game::Difficulty::Hard,
            GameMode::SinglePlayer,
        );

        // Put game in a valid state
        game1.snake.body.clear();
        game1.snake.body.push_back(Point {
            x: 10,
            y: 10,
        });
        game1.food = Point {
            x: 5,
            y: 5,
        };
        game1.obstacles.clear();

        game1.save_game_to_file(file_path);

        let mut game2 = Game::new(
            20,
            20,
            false,
            '█',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Easy,
            GameMode::SinglePlayer,
        );
        let success = game2.load_game_from_file(file_path);

        assert!(success);
        assert_eq!(game2.difficulty, crate::game::Difficulty::Hard);
        assert_eq!(game2.theme, crate::game::Theme::Neon);
        assert!(game2.wrap_mode);
        assert_eq!(game2.skin, '@');

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_save_and_load_high_scores() {
        let file_path = "highscore_test.txt";

        // Clean up from prior runs if necessary
        let _ = std::fs::remove_file(file_path);

        let mut game = Game::new(
            20,
            20,
            false,
            '#',
            crate::game::Theme::Dark,
            crate::game::Difficulty::Normal,
            GameMode::SinglePlayer,
        );
        game.high_scores.clear(); // Ensure clean state

        // Save initial score
        game.save_high_score_to_file(file_path, "Alice".to_string(), 100);

        // Save a higher score
        game.save_high_score_to_file(file_path, "Bob".to_string(), 200);

        // Save a lower score
        game.save_high_score_to_file(file_path, "Charlie".to_string(), 50);

        // Load scores back from the test file
        let loaded_scores = Game::load_high_scores_from_file(file_path);

        // Check if length is correct and scores are sorted
        assert_eq!(loaded_scores.len(), 3);
        assert_eq!(loaded_scores[0], ("Bob".to_string(), 200));
        assert_eq!(loaded_scores[1], ("Alice".to_string(), 100));
        assert_eq!(loaded_scores[2], ("Charlie".to_string(), 50));

        // Cleanup
        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_save_and_load_auto_pilot() {
        let mut game = Game::new(20, 20, false, '#', Theme::Dark, Difficulty::Normal,
            GameMode::SinglePlayer);
        game.auto_pilot = true;

        let file_path = "savegame_test_autopilot.json";
        game.save_game_to_file(file_path);

        let mut new_game = Game::new(20, 20, false, '#', Theme::Dark, Difficulty::Normal,
            GameMode::SinglePlayer);
        assert!(!new_game.auto_pilot);

        let loaded = new_game.load_game_from_file(file_path);
        assert!(loaded);
        assert!(new_game.auto_pilot);

        // Cleanup
        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_reset_clears_power_up() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
            GameMode::SinglePlayer,
        );
        game.power_up = Some(PowerUp {
            p_type: PowerUpType::SpeedBoost,
            location: crate::snake::Point {
                x: 5,
                y: 5,
            },
            activation_time: None,
        });
        game.reset();
        assert!(game.power_up.is_none(), "Power-up should be cleared on reset");
    }

    #[test]
    fn test_load_game_dos_protection() {
        let file_path = "savegame_test_dos.json";
        let mut file = File::create(file_path).expect("Failed to create dos test file");
        // Write 2 MB of garbage data
        let data = vec![b'a'; 2 * 1024 * 1024];
        file.write_all(&data).expect("Failed to write to dos test file");

        let mut game = Game::new(
            20,
            20,
            false,
            '#',
            crate::game::Theme::Dark,
            crate::game::Difficulty::Normal,
            GameMode::SinglePlayer,
        );
        // Should not panic or crash out of memory, just return false
        let loaded = game.load_game_from_file(file_path);
        assert!(!loaded);

        // Cleanup
        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_reset_clears_bot_flags() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
            GameMode::SinglePlayer,
        );
        game.auto_pilot = true;
        game.used_bot_this_game = true;
        game.reset();
        assert!(
            !game.used_bot_this_game && !game.auto_pilot,
            "Bot flags should be cleared on reset"
        );
    }

    #[test]
    fn test_calculate_autopilot_move_to_food() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
            GameMode::SinglePlayer,
        );

        // Setup snake at (10, 10) facing Up
        game.snake = crate::snake::Snake::new(crate::snake::Point {
            x: 10,
            y: 10,
        });

        // Place food directly above the snake
        game.food = crate::snake::Point {
            x: 10,
            y: 8,
        };

        // Calculate autopilot move
        let next_move = game.calculate_autopilot_move();
        assert_eq!(next_move, Some(crate::snake::Direction::Up));
    }
}
