use rand::Rng;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

use crate::config::GameConfig;
use crate::snake::{Direction, Point, Snake};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::time::SystemTime;

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum PowerUpType {
    SlowDown,
    Invincibility,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
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

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
    Help,
    EnterName,
}

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub snake: Snake,
    pub food: Point,
    pub obstacles: Vec<Point>,
    pub score: u32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Statistics {
    pub games_played: u32,
    pub total_score: u32,
    pub total_food_eaten: u32,
    pub total_time_s: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ScoreEntry {
    pub name: String,
    pub score: u32,
}

pub struct Game {
    pub config: GameConfig,
    pub snake: Snake,
    pub food: Point,
    pub bonus_food: Option<(Point, Instant)>,
    pub power_up: Option<PowerUp>,
    pub obstacles: Vec<Point>,
    pub score: u32,
    pub high_score: u32,
    pub high_scores: Vec<ScoreEntry>,
    pub state: GameState,
    pub rng: rand::rngs::ThreadRng,
    pub just_died: bool,
    pub lives: u32,
    pub menu_selection: usize,
    pub stats: Statistics,
    pub start_time: Instant,
    pub death_message: String,
    pub input_buffer: String,
}

impl Game {
    pub fn new(config: GameConfig) -> Self {
        let mut rng = rand::thread_rng();
        let start_x = config.width / 2;
        let start_y = config.height / 2;
        let snake = Snake::new(Point {
            x: start_x,
            y: start_y,
        });
        let obstacle_count = match config.difficulty {
            crate::config::Difficulty::Easy => 0,
            crate::config::Difficulty::Normal => 3,
            crate::config::Difficulty::Hard => 10,
        };
        let obstacles = Self::generate_obstacles(config.width, config.height, &snake, &mut rng, obstacle_count);
        let food = Self::generate_food(config.width, config.height, &snake, &obstacles, &mut rng);
        let high_scores = Self::load_high_scores_static();
        let high_score = high_scores.first().map_or(0, |e| e.score);
        let stats = Self::load_stats();
        Self {
            config,
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
            lives: 3,
            menu_selection: 0,
            stats,
            start_time: Instant::now(),
            death_message: String::new(),
            input_buffer: String::new(),
        }
    }

    pub fn load_high_scores_static() -> Vec<ScoreEntry> {
        let mut content = String::new();
        File::open("highscores.json")
            .and_then(|f| f.take(1024 * 1024).read_to_string(&mut content))
            .ok()
            .and_then(|_| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    fn load_stats() -> Statistics {
        let mut content = String::new();
        File::open("stats.json")
            .and_then(|f| f.take(1024 * 1024).read_to_string(&mut content))
            .ok()
            .and_then(|_| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    fn atomic_write(path: &str, content: impl AsRef<[u8]>) -> io::Result<()> {
        let mut rng = rand::thread_rng();
        let suffix: u32 = rng.r#gen();
        let tmp_path = format!("{path}.{suffix}.tmp");

        let mut file = fs::File::options()
            .write(true)
            .create_new(true)
            .open(&tmp_path)?;

        file.write_all(content.as_ref())?;
        file.sync_all()?;
        fs::rename(tmp_path, path)
    }

    pub fn save_stats(&self) {
        if let Ok(json) = serde_json::to_string(&self.stats) {
            let _ = Self::atomic_write("stats.json", json);
        }
    }

    pub fn save_high_score(&mut self, name: String, score: u32) {
        let entry = ScoreEntry { name, score };
        self.high_scores.push(entry);
        self.high_scores.sort_unstable_by(|a, b| b.score.cmp(&a.score));
        self.high_scores.truncate(5);

        if let Ok(json) = serde_json::to_string(&self.high_scores) {
            let _ = Self::atomic_write("highscores.json", json);
        }
    }

    pub fn save_game(&self) {
        let state = SaveState {
            snake: Snake {
                body: self.snake.body.clone(),
                direction: self.snake.direction,
                next_direction: self.snake.next_direction,
            },
            food: self.food,
            obstacles: self.obstacles.clone(),
            score: self.score,
        };
        if let Ok(json) = serde_json::to_string(&state) {
            let _ = Self::atomic_write("savegame.json", json);
        }
    }

    pub fn load_game(&mut self) -> bool {
        let mut content = String::new();
        File::open("savegame.json")
            .and_then(|f| f.take(1024 * 1024).read_to_string(&mut content))
            .ok()
            .and_then(|_| serde_json::from_str::<SaveState>(&content).ok())
            .is_some_and(|state| {
                self.snake = state.snake;
                self.food = state.food;
                self.obstacles = state.obstacles;
                self.score = state.score;
                self.state = GameState::Paused;
                true
            })
    }

    fn generate_obstacles(
        width: u16,
        height: u16,
        snake: &Snake,
        rng: &mut rand::rngs::ThreadRng,
        count: usize,
    ) -> Vec<Point> {
        let mut obstacles = Vec::new();
        for _ in 0..count {
            loop {
                let x = rng.gen_range(1..width - 1);
                let y = rng.gen_range(1..height - 1);
                let p = Point { x, y };
                // Ensure obstacle is not on snake and not too close to head to avoid instant death on start
                let dx = x.abs_diff(snake.head().x);
                let dy = y.abs_diff(snake.head().y);
                let is_safe_distance = dx > 3 || dy > 3;

                if !snake.body.contains(&p) && !obstacles.contains(&p) && is_safe_distance {
                    obstacles.push(p);
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
        obstacles: &[Point],
        rng: &mut rand::rngs::ThreadRng,
    ) -> Point {
        loop {
            // Food must be within walls (1..WIDTH-1, 1..HEIGHT-1)
            let x = rng.gen_range(1..width - 1);
            let y = rng.gen_range(1..height - 1);
            let p = Point { x, y };
            if !snake.body.contains(&p) && !obstacles.contains(&p) {
                return p;
            }
        }
    }

    pub fn reset(&mut self) {
        let start_x = self.config.width / 2;
        let start_y = self.config.height / 2;
        self.snake = Snake::new(Point {
            x: start_x,
            y: start_y,
        });
        let obstacle_count = match self.config.difficulty {
            crate::config::Difficulty::Easy => 0,
            crate::config::Difficulty::Normal => 3,
            crate::config::Difficulty::Hard => 10,
        };
        self.obstacles =
            Self::generate_obstacles(self.config.width, self.config.height, &self.snake, &mut self.rng, obstacle_count);
        self.food = Self::generate_food(
            self.config.width,
            self.config.height,
            &self.snake,
            &self.obstacles,
            &mut self.rng,
        );
        self.bonus_food = None;
        self.score = 0;
        self.lives = 3;
        self.state = GameState::Playing;
        self.just_died = false;
    }

    fn respawn(&mut self) {
        let start_x = self.config.width / 2;
        let start_y = self.config.height / 2;
        self.snake = Snake::new(Point {
            x: start_x,
            y: start_y,
        });
        // Ensure snake doesn't spawn on obstacle or too close to head
        self.obstacles.retain(|p| {
            let dx = p.x.abs_diff(start_x);
            let dy = p.y.abs_diff(start_y);
            dx > 3 || dy > 3
        });
    }

    pub fn handle_input(&mut self, dir: Direction) {
        // Prevent 180 degree turns and queue input if we already have one
        // If we have a next_direction, it means we already queued a move for the *next* frame.
        // We only buffer 1 move ahead to prevent "laggy" feel if user mashes keys.
        if self.snake.next_direction.is_some() {
            return;
        }

        let current_dir = self.snake.direction;
        let is_opposite = matches!(
            (current_dir, dir),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        );

        if !is_opposite && dir != current_dir {
            self.snake.next_direction = Some(dir);
        }
    }

    pub fn update(&mut self) {
        if self.state != GameState::Playing {
            return;
        }

        if let Some(dir) = self.snake.next_direction.take() {
            self.snake.direction = dir;
        }

        self.manage_bonus_food();
        self.manage_power_ups();

        let head = self.snake.head();
        let next_head = self.calculate_next_head(head);

        // Check collision with walls and obstacles
        let mut hit_wall = false;
        let final_head = if self.config.wrap_mode {
            self.calculate_wrapped_head(next_head)
        } else {
            if next_head.x == 0
                || next_head.x >= self.config.width - 1
                || next_head.y == 0
                || next_head.y >= self.config.height - 1
            {
                hit_wall = true;
            }
            next_head
        };

        if self.obstacles.contains(&final_head) {
            hit_wall = true;
        }

        // Check bonus food collision
        if let Some(p) = self.power_up.as_mut()
            && final_head == p.location {
                p.activation_time = Some(SystemTime::now());
                beep();
            }

        let is_invincible = self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::Invincibility && p.activation_time.is_some()
        });

        if hit_wall {
            if is_invincible {
                // Ignore wall collision
            } else {
                self.handle_death("Hit Wall/Obstacle");
                return;
            }
        }

        let mut grow = if self
            .bonus_food
            .is_some_and(|(bonus_p, _)| final_head == bonus_p)
        {
            self.score += 5;
            self.bonus_food = None;
            beep();
            true
        } else {
            false
        };

        // Refined self collision check
        if self.snake.body.contains(&final_head) {
            if !grow && final_head == *self.snake.body.back().unwrap() {
                // We are moving into the tail, but the tail will move. Safe.
            } else if is_invincible {
                // Ignore self collision
            } else {
                self.handle_death("Hit Self");
                return;
            }
        }

        if final_head == self.food {
            grow = true;
        }

        if grow && final_head == self.food {
            self.score += 1;
            beep();
            // Add a new obstacle every 5 points
            if self.score.is_multiple_of(5) {
                let new_obstacles = Self::generate_obstacles(
                    self.config.width,
                    self.config.height,
                    &self.snake,
                    &mut self.rng,
                    1,
                );
                self.obstacles.extend(new_obstacles);
            }
            self.food = Self::generate_food(
                self.config.width,
                self.config.height,
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
            obstructions.push(self.food);
            if let Some((bonus_food_pos, _)) = self.bonus_food {
                obstructions.push(bonus_food_pos);
            }

            let location = Self::generate_food(
                self.config.width,
                self.config.height,
                &self.snake,
                &obstructions,
                &mut self.rng,
            );

            let p_type = if self.rng.gen_bool(0.5) {
                PowerUpType::SlowDown
            } else {
                PowerUpType::Invincibility
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
            obstructions.push(self.food);
            let bonus = Self::generate_food(
                self.config.width,
                self.config.height,
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
            x = self.config.width - 2;
        } else if x >= self.config.width - 1 {
            x = 1;
        }

        if y == 0 {
            y = self.config.height - 2;
        } else if y >= self.config.height - 1 {
            y = 1;
        }
        Point { x, y }
    }

    fn handle_death(&mut self, cause: &str) {
        self.lives -= 1;
        self.just_died = true;
        beep();

        // Update stats
        self.stats.games_played += 1;
        self.stats.total_time_s += self.start_time.elapsed().as_secs();
        self.save_stats();

        if self.lives == 0 {
            self.death_message = cause.to_string();
            // Since we don't have top 5 yet, just check if high score
            if self.score > self.high_score {
                self.state = GameState::EnterName;
                self.input_buffer.clear();
            } else {
                self.state = GameState::GameOver;
            }
        } else {
            self.respawn();
        }
    }
}
