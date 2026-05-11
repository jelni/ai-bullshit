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
            .then_with(|| self.position.x.cmp(&other.position.x))
            .then_with(|| self.position.y.cmp(&other.position.y))
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
    EnterName,
    ConfirmQuit,
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

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub snake: Snake,
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
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShopItem {
    Skin(char),
    Theme(Theme),
}

pub const AVAILABLE_ITEMS: [(ShopItem, u32); 9] = [
    (ShopItem::Skin('💎'), 100),
    (ShopItem::Skin('👾'), 250),
    (ShopItem::Skin('🐍'), 500),
    (ShopItem::Skin('🚀'), 1000),
    (ShopItem::Skin('🦍'), 2000),
    (ShopItem::Theme(Theme::Premium), 5000),
    (ShopItem::Theme(Theme::Cyberpunk), 10000),
    (ShopItem::Theme(Theme::Rainbow), 25000),
    (ShopItem::Theme(Theme::Hacker), 50000),
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
}

impl Game {
    pub fn new(
        width: u16,
        height: u16,
        wrap_mode: bool,
        skin: char,
        theme: Theme,
        difficulty: Difficulty,
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
        let obstacles = Self::generate_obstacles(width, height, &snake, avoid, &mut rng, obs_count);
        let avoid_food = |p: &Point| obstacles.contains(p);
        let food = Self::get_random_empty_point(width, height, &snake, avoid_food, &mut rng)
            .expect("Board cannot be full on start");
        let high_scores = Self::load_high_scores_static();
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
        }
    }

    pub fn load_high_scores_static() -> Vec<(String, u32)> {
        Self::load_high_scores_from_file("highscore.txt")
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
        self.save_high_score_to_file("highscore.txt", name, score);
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

    pub fn save_game_to_file(&self, path: &str) {
        let state = SaveState {
            snake: Snake {
                body: self.snake.body.clone(),
                body_map: self.snake.body_map.clone(),
                direction: self.snake.direction,
                direction_queue: self.snake.direction_queue.clone(),
            },
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

                state.snake.rebuild_map();
                self.snake = state.snake;
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
                self.state = GameState::Paused;
                self.start_time = Instant::now();
                true
            })
    }

    fn get_random_empty_point(
        width: u16,
        height: u16,
        snake: &Snake,
        avoid: impl Fn(&Point) -> bool,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Option<Point> {
        let mut i = 0;
        loop {
            // Point must be within walls (1..WIDTH-1, 1..HEIGHT-1)
            let x = rng.gen_range(1..width - 1);
            let y = rng.gen_range(1..height - 1);
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
                for y_ in 1..height - 1 {
                    for x_ in 1..width - 1 {
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
    ) -> HashSet<Point> {
        let mut obstacles = HashSet::new();

        for _ in 0..count {
            let current_avoid = |p: &Point| avoid(p) || obstacles.contains(p);
            if let Some(p) = Self::get_random_empty_point(width, height, snake, current_avoid, rng)
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
    }

    pub fn reset(&mut self) {
        let start_x = self.width / 2;
        let start_y = self.height / 2;
        self.snake = Snake::new(Point {
            x: start_x,
            y: start_y,
        });
        let obs_count = match self.difficulty {
            Difficulty::Easy => 1,
            Difficulty::Normal => 3,
            Difficulty::Hard => 5,
            Difficulty::Insane => 10,
            Difficulty::GodMode => 20,
        };
        let avoid = |p: &Point| p.x == start_x && p.y == start_y - 1;
        self.obstacles = Self::generate_obstacles(
            self.width,
            self.height,
            &self.snake,
            avoid,
            &mut self.rng,
            obs_count,
        );
        let avoid_food = |p: &Point| self.obstacles.contains(p);
        self.food = Self::get_random_empty_point(
            self.width,
            self.height,
            &self.snake,
            avoid_food,
            &mut self.rng,
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
    }

    fn respawn(&mut self) {
        let start_x = self.width / 2;
        let start_y = self.height / 2;
        self.snake = Snake::new(Point {
            x: start_x,
            y: start_y,
        });
        // Ensure snake doesn't spawn on obstacle
        // We also clear start_y - 1 to prevent instant death upon spawn.
        self.obstacles.retain(|p| {
            !(p.x == start_x && (p.y >= start_y.saturating_sub(1) && p.y <= start_y + 2))
        });
    }

    pub fn handle_input(&mut self, dir: Direction) {
        // Prevent 180 degree turns and queue input if we already have one
        // We buffer up to 2 moves ahead to prevent "laggy" feel if user mashes keys.

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
    }

    pub fn update(&mut self) {
        if self.state != GameState::Playing {
            return;
        }

        if self.auto_pilot
            && self.snake.direction_queue.is_empty()
            && let Some(dir) = self.calculate_autopilot_move()
        {
            self.snake.direction_queue.push_back(dir);
        }

        if let Some(dir) = self.snake.direction_queue.pop_front() {
            self.snake.direction = dir;
        }

        self.manage_bonus_food();
        self.manage_power_ups();

        let head = self.snake.head();
        let next_head = self.calculate_next_head(head);

        let can_pass_through_walls = self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::PassThroughWalls
                && p.activation_time
                    .is_some_and(|t| t.elapsed().unwrap_or_default() < Duration::from_secs(5))
        });

        // Check collision with walls and obstacles
        let mut hit_wall = false;
        let final_head = if self.wrap_mode || can_pass_through_walls {
            self.calculate_wrapped_head(next_head)
        } else {
            if next_head.x == 0
                || next_head.x >= self.width - 1
                || next_head.y == 0
                || next_head.y >= self.height - 1
            {
                hit_wall = true;
            }
            next_head
        };

        let hit_obstacle = self.obstacles.contains(&final_head);

        let is_invincible = self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::Invincibility
                && p.activation_time
                    .is_some_and(|t| t.elapsed().unwrap_or_default() < Duration::from_secs(5))
        });

        if hit_wall {
            self.handle_death("Hit Wall");
            return;
        }

        if hit_obstacle && !is_invincible {
            self.handle_death("Hit Obstacle");
            return;
        }

        self.process_power_up_collision(final_head);

        let is_multiplier = self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::ScoreMultiplier
                && p.activation_time
                    .is_some_and(|t| t.elapsed().unwrap_or_default() < Duration::from_secs(5))
        });

        let old_food_eaten_session = self.food_eaten_session;

        let mut grow = self.check_bonus_food_collision(final_head, is_multiplier);

        // Refined self collision check
        if self.snake.body_map.contains_key(&final_head) && !is_invincible {
            let is_tail = self.snake.body.back().is_some_and(|tail| final_head == *tail);
            if !grow && is_tail {
                // We are moving into the tail, but the tail will move. Safe.
            } else {
                self.handle_death("Hit Self");
                return;
            }
        }

        if final_head == self.food {
            grow = true;
            if !self.process_food_collision(final_head, is_multiplier) {
                self.snake.move_to(final_head, grow);
                self.handle_win();
                return;
            }
        }

        self.add_obstacles_if_needed(old_food_eaten_session, final_head);

        self.snake.move_to(final_head, grow);
    }

    fn process_power_up_collision(&mut self, final_head: Point) {
        if let Some(p) = self.power_up.as_mut()
            && final_head == p.location
        {
            if p.p_type == PowerUpType::ExtraLife {
                self.lives += 1;
            } else if p.p_type == PowerUpType::Shrink {
                self.snake.shrink_tail();
            } else if p.p_type == PowerUpType::ClearObstacles {
                self.obstacles.clear();
            } else {
                p.activation_time = Some(SystemTime::now());
            }
            beep();
        }

        // Remove power up instantly if it was an instant effect that was just activated
        if let Some(p) = self.power_up.as_ref()
            && (p.p_type == PowerUpType::ExtraLife
                || p.p_type == PowerUpType::Shrink
                || p.p_type == PowerUpType::ClearObstacles)
            && p.activation_time.is_none()
            && final_head == p.location
        {
            self.power_up = None;
        }
    }

    fn check_bonus_food_collision(&mut self, final_head: Point, is_multiplier: bool) -> bool {
        if self.bonus_food.is_some_and(|(bonus_p, _)| final_head == bonus_p) {
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

        let avoid = |p: &Point| {
            self.obstacles.contains(p)
                || *p == final_head
                || self.bonus_food.is_some_and(|(bp, _)| *p == bp)
                || self.power_up.as_ref().is_some_and(|pu| *p == pu.location)
        };
        if let Some(new_food) =
            Self::get_random_empty_point(self.width, self.height, &self.snake, avoid, &mut self.rng)
        {
            self.food = new_food;
            true
        } else {
            false
        }
    }

    fn add_obstacles_if_needed(&mut self, old_food_eaten_session: u32, final_head: Point) {
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
            );
            self.obstacles.extend(new_obstacles);
        }
    }

    fn handle_win(&mut self) {
        self.stats.games_played += 1;
        self.stats.total_time_s += self.start_time.elapsed().as_secs();
        self.save_stats();

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
            ) {
                let p_type = match self.rng.gen_range(0..8) {
                    0 => PowerUpType::SlowDown,
                    1 => PowerUpType::SpeedBoost,
                    2 => PowerUpType::Invincibility,
                    3 => PowerUpType::PassThroughWalls,
                    4 => PowerUpType::Shrink,
                    5 => PowerUpType::ClearObstacles,
                    6 => PowerUpType::ScoreMultiplier,
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

    const fn calculate_next_head(&self, head: Point) -> Point {
        Self::calculate_next_head_dir(head, self.snake.direction)
    }

    pub fn get_final_p(&self, p: Point) -> Option<Point> {
        let can_pass_through_walls = self.power_up.as_ref().is_some_and(|pu| {
            pu.p_type == PowerUpType::PassThroughWalls
                && pu
                    .activation_time
                    .is_some_and(|t| t.elapsed().unwrap_or_default() < Duration::from_secs(5))
        });

        if self.wrap_mode || can_pass_through_walls {
            Some(self.calculate_wrapped_head(p))
        } else if p.x == 0 || p.x >= self.width - 1 || p.y == 0 || p.y >= self.height - 1 {
            None // Hit wall
        } else {
            Some(p)
        }
    }

    pub fn is_safe_final_p(&self, final_p: Point, steps: u16) -> bool {
        let is_invincible = self.power_up.as_ref().is_some_and(|pu| {
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

        if let Some((dir, path)) = self.astar_search(start, &targets) {
            self.autopilot_path = path;
            return Some(dir);
        }

        self.autopilot_path.clear();
        self.flood_fill_fallback(start)
    }

    fn astar_search(&self, start: Point, targets: &[Point]) -> Option<(Direction, Vec<Point>)> {
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
                    if self.wrap_mode || can_pass_through_walls {
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
                && self.is_safe_final_p(final_p, 1)
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
                    && self.is_safe_final_p(final_p, tentative_g)
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

    fn flood_fill_fallback(&self, start: Point) -> Option<Direction> {
        let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
        let mut best_dir = None;
        let mut max_open_space = 0;

        for &d in &dirs {
            let next_p = Self::calculate_next_head_dir(start, d);
            if let Some(final_p) = self.get_final_p(next_p)
                && self.is_safe_final_p(final_p, 1)
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
                            && self.is_safe_final_p(valid_p, next_steps)
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
        self.lives -= 1;
        self.just_died = true;
        beep();

        if self.lives == 0 {
            // Update stats on Game Over
            self.stats.games_played += 1;
            self.stats.total_time_s += self.start_time.elapsed().as_secs();
            self.save_stats();

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
        let mut game = Game::new(20, 20, false, '#', Theme::Dark, Difficulty::Normal);
        game.auto_pilot = true;

        let file_path = "savegame_test_autopilot.json";
        game.save_game_to_file(file_path);

        let mut new_game = Game::new(20, 20, false, '#', Theme::Dark, Difficulty::Normal);
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
