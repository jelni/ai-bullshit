use rand::Rng;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use crate::snake::{Direction, Point, Snake};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::time::SystemTime;

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum PowerUpType {
    SlowDown,
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

pub struct Game {
    pub width: u16,
    pub height: u16,
    pub wrap_mode: bool,
    pub snake: Snake,
    pub food: Point,
    pub bonus_food: Option<(Point, Instant)>,
    pub power_up: Option<PowerUp>,
    pub obstacles: Vec<Point>,
    pub score: u32,
    pub high_score: u32,
    pub high_scores: Vec<u32>,
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
}

impl Game {
    pub fn new(width: u16, height: u16, wrap_mode: bool, skin: char, theme: String) -> Self {
        let mut rng = rand::thread_rng();
        let start_x = width / 2;
        let start_y = height / 2;
        let snake = Snake::new(Point {
            x: start_x,
            y: start_y,
        });
        let obstacles = Self::generate_obstacles(width, height, &snake, &mut rng, 3);
        let food = Self::generate_food(width, height, &snake, &obstacles, &mut rng);
        let high_scores = Self::load_high_scores_static();
        let high_score = *high_scores.first().unwrap_or(&0);
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
        }
    }

    pub fn load_high_scores_static() -> Vec<u32> {
        let mut content = String::new();
        File::open("highscore.txt")
            .and_then(|f| f.take(1024 * 1024).read_to_string(&mut content))
            .map_or_else(
                |_| Vec::new(),
                |_| {
                    content
                        .lines()
                        .filter_map(|line| line.trim().parse::<u32>().ok())
                        .collect()
                },
            )
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

        let mut options = fs::File::options();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.custom_flags(libc::O_NOFOLLOW);

        let mut file = options.open(&tmp_path)?;

        file.write_all(content.as_ref())?;
        file.sync_all()?;
        fs::rename(tmp_path, path)
    }

    pub fn save_stats(&self) {
        if let Ok(json) = serde_json::to_string(&self.stats) {
            let _ = Self::atomic_write("stats.json", json);
        }
    }

    fn save_high_score(&mut self, score: u32) {
        self.high_scores.push(score);
        self.high_scores.sort_unstable_by(|a, b| b.cmp(a));
        self.high_scores.truncate(5);
        let content = self.high_scores
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        let _ = Self::atomic_write("highscore.txt", content);
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
                // Simple check: not on body.
                if !snake.body.contains(&p) && !obstacles.contains(&p) {
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
        let start_x = self.width / 2;
        let start_y = self.height / 2;
        self.snake = Snake::new(Point {
            x: start_x,
            y: start_y,
        });
        self.obstacles =
            Self::generate_obstacles(self.width, self.height, &self.snake, &mut self.rng, 3);
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

        if hit_wall {
            self.handle_death("Hit Wall/Obstacle");
            return;
        }

        // Check bonus food collision
        if let Some(p) = self.power_up.as_mut()
            && final_head == p.location {
                p.activation_time = Some(SystemTime::now());
                beep();
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
            obstructions.push(self.food);
            if let Some((bonus_food_pos, _)) = self.bonus_food {
                obstructions.push(bonus_food_pos);
            }

            let location = Self::generate_food(
                self.width,
                self.height,
                &self.snake,
                &obstructions,
                &mut self.rng,
            );

            self.power_up = Some(PowerUp {
                p_type: PowerUpType::SlowDown,
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

        // Update stats
        self.stats.games_played += 1;
        self.stats.total_time_s += self.start_time.elapsed().as_secs();
        self.save_stats();

        if self.lives == 0 {
            self.state = GameState::GameOver;
            self.death_message = cause.to_string();
            if self.score > self.high_score {
                self.high_score = self.score;
                self.save_high_score(self.high_score);
            }
        } else {
            self.respawn();
        }
    }
}
