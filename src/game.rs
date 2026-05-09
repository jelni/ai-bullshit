use rand::Rng;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

use crate::snake::{Direction, Point, Snake};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::time::SystemTime;

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
}

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum PowerUpType {
    SlowDown,
    SpeedBoost,
    Invincibility,
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
    Help,
    EnterName,
    ConfirmQuit,
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
}

#[derive(Serialize, Deserialize, Default)]
pub struct Statistics {
    pub games_played: u32,
    pub total_score: u32,
    pub total_food_eaten: u32,
    pub total_time_s: u64,
}

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
    pub theme: String,
    pub lives: u32,
    pub menu_selection: usize,
    pub stats: Statistics,
    pub start_time: Instant,
    pub death_message: String,
    pub difficulty: Difficulty,
    pub player_name: String,
    pub previous_state: Option<GameState>,
}

impl Game {
    pub fn new(
        width: u16,
        height: u16,
        wrap_mode: bool,
        skin: char,
        theme: String,
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
        };
        let obstacles = Self::generate_obstacles(width, height, &snake, &mut rng, obs_count);
        let food = Self::generate_food(width, height, &snake, &obstacles, &mut rng);
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
            stats,
            start_time: Instant::now(),
            death_message: String::new(),
            difficulty,
            player_name: String::new(),
            previous_state: None,
        }
    }

    pub fn load_high_scores_static() -> Vec<(String, u32)> {
        Self::load_high_scores_from_file("highscore.txt")
    }

    pub fn load_high_scores_from_file(path: &str) -> Vec<(String, u32)> {
        let mut content = String::new();
        File::open(path)
            .and_then(|f| f.take(1024 * 1024).read_to_string(&mut content))
            .map_or_else(
                |_| Vec::new(),
                |_| {
                    content
                        .lines()
                        .filter_map(|line| {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            #[expect(clippy::collapsible_if, reason = "stable rust")]
                            if parts.len() >= 2 {
                                if let Some(score_str) = parts.last() {
                                    let name = parts[..parts.len() - 1].join(" ");
                                    if let Ok(score) = score_str.parse::<u32>() {
                                        return Some((name, score));
                                    }
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
        let mut content = String::new();
        File::open(path)
            .and_then(|f| f.take(1024 * 1024).read_to_string(&mut content))
            .ok()
            .and_then(|_| serde_json::from_str(&content).ok())
            .unwrap_or_default()
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
        self.high_scores.push((name, score));
        self.high_scores.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        self.high_scores.truncate(5);
        let content = self
            .high_scores
            .iter()
            .map(|(n, s)| format!("{n} {s}"))
            .collect::<Vec<_>>()
            .join("\n");
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
                state.snake.rebuild_map();
                self.snake = state.snake;
                self.food = state.food;
                self.obstacles = state.obstacles;
                self.score = state.score;
                self.bonus_food = state.bonus_food.and_then(|(p, elapsed)| {
                    Instant::now().checked_sub(Duration::from_secs(elapsed)).map(|t| (p, t))
                });
                self.power_up = state.power_up;
                self.state = GameState::Paused;
                self.start_time = Instant::now();
                true
            })
    }

    fn generate_obstacles(
        width: u16,
        height: u16,
        snake: &Snake,
        rng: &mut rand::rngs::ThreadRng,
        count: usize,
    ) -> HashSet<Point> {
        let mut obstacles = HashSet::new();

        for _ in 0..count {
            let mut i = 0;
            loop {
                let x = rng.gen_range(1..width - 1);
                let y = rng.gen_range(1..height - 1);
                let p = Point { x, y };
                // Ensure obstacle is not on snake and not too close to head to avoid instant death on start
                // Simple check: not on body.
                if !snake.body_map.contains_key(&p) && !obstacles.contains(&p) {
                    obstacles.insert(p);
                    break;
                }
                i += 1;
                if i >= 100 {
                    let mut empty = Vec::new();
                    for y_ in 1..height - 1 {
                        for x_ in 1..width - 1 {
                            let p_ = Point { x: x_, y: y_ };
                            if !snake.body_map.contains_key(&p_) && !obstacles.contains(&p_) {
                                empty.push(p_);
                            }
                        }
                    }
                    if !empty.is_empty() {
                        let idx = rng.gen_range(0..empty.len());
                        obstacles.insert(empty[idx]);
                    }
                    break;
                }
            }
        }
        obstacles
    }

    fn generate_food(
        width: u16,
        height: u16,
        snake: &Snake,
        obstacles: &HashSet<Point>,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Point {
        let mut i = 0;
        loop {
            // Food must be within walls (1..WIDTH-1, 1..HEIGHT-1)
            let x = rng.gen_range(1..width - 1);
            let y = rng.gen_range(1..height - 1);
            let p = Point { x, y };
            if !snake.body_map.contains_key(&p) && !obstacles.contains(&p) {
                return p;
            }
            i += 1;
            if i >= 100 {
                let mut empty = Vec::new();
                for y_ in 1..height - 1 {
                    for x_ in 1..width - 1 {
                        let p_ = Point { x: x_, y: y_ };
                        if !snake.body_map.contains_key(&p_) && !obstacles.contains(&p_) {
                            empty.push(p_);
                        }
                    }
                }
                if !empty.is_empty() {
                    let idx = rng.gen_range(0..empty.len());
                    return empty[idx];
                }
                // Fallback if the board is completely full
                return Point { x: 1, y: 1 };
            }
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
        };
        self.obstacles = Self::generate_obstacles(
            self.width,
            self.height,
            &self.snake,
            &mut self.rng,
            obs_count,
        );
        self.food = Self::generate_food(
            self.width,
            self.height,
            &self.snake,
            &self.obstacles,
            &mut self.rng,
        );
        self.bonus_food = None;
        self.score = 0;
        self.lives = 3;
        self.state = GameState::Playing;
        self.just_died = false;
        self.start_time = Instant::now();
    }

    fn respawn(&mut self) {
        let start_x = self.width / 2;
        let start_y = self.height / 2;
        self.snake = Snake::new(Point {
            x: start_x,
            y: start_y,
        });
        // Ensure snake doesn't spawn on obstacle
        // For simplicity in this game, we assume center is safe or we clear obstacles there.
        self.obstacles
            .retain(|p| !(p.x == start_x && (p.y >= start_y && p.y <= start_y + 2)));
    }

    pub fn handle_input(&mut self, dir: Direction) {
        // Prevent 180 degree turns and queue input if we already have one
        // We buffer up to 2 moves ahead to prevent "laggy" feel if user mashes keys.

        if self.snake.direction_queue.len() >= 2 {
            return;
        }

        let current_dir = self.snake.direction_queue.back().copied().unwrap_or(self.snake.direction);
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

        if let Some(dir) = self.snake.direction_queue.pop_front() {
            self.snake.direction = dir;
        }

        self.manage_bonus_food();
        self.manage_power_ups();

        let head = self.snake.head();
        let next_head = self.calculate_next_head(head);

        // Check collision with walls and obstacles
        let mut hit_wall = false;
        let final_head = if self.wrap_mode {
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

        if self.obstacles.contains(&final_head) {
            hit_wall = true;
        }

        let is_invincible = self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::Invincibility
                && p.activation_time
                    .is_some_and(|t| t.elapsed().unwrap_or_default() < Duration::from_secs(5))
        });

        if hit_wall && !is_invincible {
            self.handle_death("Hit Wall/Obstacle");
            return;
        }

        // Check bonus food collision
        #[expect(clippy::collapsible_if, reason = "stable rust")]
        if let Some(p) = self.power_up.as_mut() {
            if final_head == p.location {
                p.activation_time = Some(SystemTime::now());
                beep();
            }
        }

        let mut grow = if self
            .bonus_food
            .is_some_and(|(bonus_p, _)| final_head == bonus_p)
        {
            self.score += 5;
            self.stats.total_score += 5;
            self.stats.total_food_eaten += 1;
            self.bonus_food = None;
            beep();
            true
        } else {
            false
        };

        // Refined self collision check
        if self.snake.body_map.contains_key(&final_head) && !is_invincible {
            let is_tail = self
                .snake
                .body
                .back()
                .is_some_and(|tail| final_head == *tail);
            if !grow && is_tail {
                // We are moving into the tail, but the tail will move. Safe.
            } else {
                self.handle_death("Hit Self");
                return;
            }
        }

        if final_head == self.food {
            grow = true;
            self.score += 1;
            self.stats.total_score += 1;
            self.stats.total_food_eaten += 1;
            beep();
            // Add a new obstacle every 5 points
            if self.score.is_multiple_of(5) {
                let new_obstacles = Self::generate_obstacles(
                    self.width,
                    self.height,
                    &self.snake,
                    &mut self.rng,
                    1,
                );
                self.obstacles.extend(new_obstacles);
            }
            self.food = Self::generate_food(
                self.width,
                self.height,
                &self.snake,
                &self.obstacles,
                &mut self.rng,
            );
        }

        self.snake.move_to(final_head, grow);
    }

    fn manage_power_ups(&mut self) {
        if self.power_up.is_none() && self.rng.gen_bool(0.02) {
            let mut obstructions = self.obstacles.clone();
            obstructions.insert(self.food);
            if let Some((bonus_food_pos, _)) = self.bonus_food {
                obstructions.insert(bonus_food_pos);
            }

            let location = Self::generate_food(
                self.width,
                self.height,
                &self.snake,
                &obstructions,
                &mut self.rng,
            );

            let p_type = match self.rng.gen_range(0..3) {
                0 => PowerUpType::SlowDown,
                1 => PowerUpType::SpeedBoost,
                _ => PowerUpType::Invincibility,
            };

            self.power_up = Some(PowerUp {
                p_type,
                location,
                activation_time: None,
            });
        }
    }

    fn manage_bonus_food(&mut self) {
        if let Some((_, spawn_time)) = self.bonus_food {
            if spawn_time.elapsed() > Duration::from_secs(5) {
                self.bonus_food = None;
            }
        } else if self.rng.gen_bool(0.01) {
            let mut obstructions = self.obstacles.clone();
            obstructions.insert(self.food);
            if let Some(ref pu) = self.power_up {
                obstructions.insert(pu.location);
            }
            let bonus = Self::generate_food(
                self.width,
                self.height,
                &self.snake,
                &obstructions,
                &mut self.rng,
            );
            self.bonus_food = Some((bonus, Instant::now()));
        }
    }

    const fn calculate_next_head(&self, head: Point) -> Point {
        match self.snake.direction {
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
        Point { x, y }
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
                self.state = GameState::EnterName;
                self.player_name.clear();
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
    use super::*;
    use std::fs::File;
    use std::io::Write;

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
            String::from("dark"),
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
    fn test_load_game_dos_protection() {
        let file_path = "savegame_test_dos.json";
        let mut file = File::create(file_path).expect("Failed to create dos test file");
        // Write 2 MB of garbage data
        let data = vec![b'a'; 2 * 1024 * 1024];
        file.write_all(&data)
            .expect("Failed to write to dos test file");

        let mut game = Game::new(
            20,
            20,
            false,
            '#',
            String::from("dark"),
            crate::game::Difficulty::Normal,
        );
        // Should not panic or crash out of memory, just return false
        let loaded = game.load_game_from_file(file_path);
        assert!(!loaded);

        // Cleanup
        let _ = std::fs::remove_file(file_path);
    }
}
