use rand::Rng;
use std::fs;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::snake::{Direction, Point, Snake};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum PowerUpType {
    SlowDown,
    SpeedBoost,
    Invincibility,
}

pub struct PowerUp {
    pub p_type: PowerUpType,
    pub location: Point,
    pub activation_time: Option<Instant>,
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
    pub power_up: Option<(PowerUpType, Point, Option<u64>)>,
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
    pub player_name: String,
}

impl Game {
    pub fn new(width: u16, height: u16, wrap_mode: bool, skin: char, theme: String) -> Self {
        let mut rng = rand::thread_rng();
        let start_x = width / 2;
        let start_y = height / 2;
        let snake = Snake::new(Point { x: start_x, y: start_y });
        let obstacles = Self::generate_obstacles(width, height, &snake, &mut rng, 3);
        let food = Self::generate_food(width, height, &snake, &obstacles, &mut rng);
        let high_score = Self::load_high_scores_static()
            .first()
            .map_or(0, |(_, score)| *score);
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
            player_name: String::new(),
        }
    }

    pub fn load_high_scores_static() -> Vec<(String, u32)> {
        fs::read_to_string("highscores.json")
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    fn load_stats() -> Statistics {
        fs::read_to_string("stats.json")
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    pub fn save_stats(&self) {
        if let Ok(json) = serde_json::to_string(&self.stats) {
            let _ = fs::write("stats.json", json);
        }
    }

    pub fn save_high_score(name: &str, score: u32) {
        let mut scores = Self::load_high_scores_static();
        scores.push((name.to_string(), score));
        scores.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        scores.truncate(5);
        if let Ok(json) = serde_json::to_string(&scores) {
            let _ = fs::write("highscores.json", json);
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
            power_up: self.power_up.as_ref().map(|p| {
                let elapsed = p.activation_time.map(|t| t.elapsed().as_secs());
                (p.p_type, p.location, elapsed)
            }),
        };
        if let Ok(json) = serde_json::to_string(&state) {
            let _ = fs::write("savegame.json", json);
        }
    }

    pub fn load_game(&mut self) -> bool {
        fs::read_to_string("savegame.json")
            .ok()
            .and_then(|content| serde_json::from_str::<SaveState>(&content).ok())
            .is_some_and(|state| {
                self.snake = state.snake;
                self.food = state.food;
                self.obstacles = state.obstacles;
                self.score = state.score;
                self.state = GameState::Paused;
                self.power_up = state.power_up.map(|(p_type, location, elapsed)| {
                    let activation_time = elapsed.and_then(|secs| {
                        Instant::now().checked_sub(Duration::from_secs(secs))
                    });
                    PowerUp {
                        p_type,
                        location,
                        activation_time,
                    }
                });
                true
            })
    }

    fn generate_obstacles(width: u16, height: u16, snake: &Snake, rng: &mut rand::rngs::ThreadRng, count: usize) -> Vec<Point> {
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

    fn generate_food(width: u16, height: u16, snake: &Snake, obstacles: &[Point], rng: &mut rand::rngs::ThreadRng) -> Point {
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
        self.snake = Snake::new(Point { x: start_x, y: start_y });
        self.obstacles = Self::generate_obstacles(self.width, self.height, &self.snake, &mut self.rng, 3);
        self.food = Self::generate_food(self.width, self.height, &self.snake, &self.obstacles, &mut self.rng);
        self.bonus_food = None;
        self.score = 0;
        self.lives = 3;
        self.state = GameState::Playing;
        self.just_died = false;
    }

    fn respawn(&mut self) {
        let start_x = self.width / 2;
        let start_y = self.height / 2;
        self.snake = Snake::new(Point { x: start_x, y: start_y });
        // Ensure snake doesn't spawn on obstacle
        // For simplicity in this game, we assume center is safe or we clear obstacles there.
        self.obstacles.retain(|p| !(p.x == start_x && (p.y >= start_y && p.y <= start_y + 2)));
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
        let is_invincible = self.power_up.as_ref().is_some_and(|p| {
            p.p_type == PowerUpType::Invincibility && p.activation_time.is_some()
        });

        let final_head = if self.wrap_mode || is_invincible {
            self.calculate_wrapped_head(next_head)
        } else {
            if next_head.x == 0 || next_head.x >= self.width - 1 || next_head.y == 0 || next_head.y >= self.height - 1 {
                hit_wall = true;
            }
            next_head
        };

        if self.obstacles.contains(&final_head) {
            hit_wall = true;
        }

        if hit_wall && !is_invincible {
            self.handle_death("Hit Wall/Obstacle");
            return;
        }

        // Check bonus food collision
        if let Some(p) = self.power_up.as_mut()
            && final_head == p.location && p.activation_time.is_none() {
                p.activation_time = Some(Instant::now());
                beep();
            }

        let mut grow = if self.bonus_food.is_some_and(|(bonus_p, _)| final_head == bonus_p) {
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
        if self.snake.body.contains(&final_head) && !is_invincible {
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
            self.stats.total_score += 1;
            self.stats.total_food_eaten += 1;
            beep();
            // Add a new obstacle every 5 points
            if self.score.is_multiple_of(5) {
                let new_obstacles = Self::generate_obstacles(self.width, self.height, &self.snake, &mut self.rng, 1);
                self.obstacles.extend(new_obstacles);
            }
            self.food = Self::generate_food(self.width, self.height, &self.snake, &self.obstacles, &mut self.rng);
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

            let location =
                Self::generate_food(self.width, self.height, &self.snake, &obstructions, &mut self.rng);

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
             obstructions.push(self.food);
             let bonus = Self::generate_food(self.width, self.height, &self.snake, &obstructions, &mut self.rng);
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
        if x == 0 { x = self.width - 2; }
        else if x >= self.width - 1 { x = 1; }

        if y == 0 { y = self.height - 2; }
        else if y >= self.height - 1 { y = 1; }
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
            let is_high_score = {
                 let scores = Self::load_high_scores_static();
                 scores.len() < 5 || scores.last().is_some_and(|&(_, s)| self.score > s)
            };
            if self.score > 0 && is_high_score {
                 self.state = GameState::EnterName;
            } else {
                 self.state = GameState::GameOver;
            }
        } else {
            self.respawn();
        }
    }
}
