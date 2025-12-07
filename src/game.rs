use rand::Rng;
use std::fs;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::config::GameConfig;
use crate::snake::{Direction, Point, Snake};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone)]
pub enum Sound {
    Food,
    PowerUp,
    Death,
}

pub fn beep(sound: Sound) {
    match sound {
        Sound::Food => print!("\x07"),
        Sound::PowerUp => print!("\x08"),
        Sound::Death => print!("\x09"),
    }
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

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum FoodType {
    Golden,
    ScoreBoost,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpType {
    SpeedBoost,
    Invincibility,
}

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub snake: Snake,
    pub food: Point,
    pub obstacles: Vec<Point>,
    pub score: u32,
    pub special_food: Option<(Point, u64, FoodType)>,
    pub power_up: Option<(Point, u64, PowerUpType)>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Statistics {
    pub games_played: u32,
    pub total_score: u32,
    pub total_food_eaten: u32,
    pub total_time_s: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HighScoreEntry {
    pub name: String,
    pub score: u32,
}

pub struct Game {
    pub config: GameConfig,
    pub snake: Snake,
    pub food: Point,
    pub special_food: Option<(Point, Instant, FoodType)>,
    pub obstacles: Vec<Point>,
    pub score: u32,
    pub high_score: u32,
    pub state: GameState,
    pub rng: rand::rngs::ThreadRng,
    pub just_died: bool,
    pub lives: u32,
    pub menu_selection: usize,
    pub stats: Statistics,
    pub start_time: Instant,
    pub death_message: String,
    pub level: u32,
    pub player_name: String,
    pub high_scores: Vec<HighScoreEntry>,
    pub power_up: Option<(Point, Instant, PowerUpType)>,
}

impl Game {
    pub fn new(config: GameConfig) -> Self {
        let mut rng = rand::thread_rng();
        let start_x = config.width / 2;
        let start_y = config.height / 2;
        let snake = Snake::new(Point { x: start_x, y: start_y });
        let obstacles = Self::generate_obstacles(config.width, config.height, &snake, &mut rng, 0);
        let food = Self::generate_food(config.width, config.height, &snake, &obstacles, &mut rng);
        let high_scores = Self::load_high_scores();
        let high_score = high_scores.first().map_or(0, |entry| entry.score);
        let stats = Self::load_stats();
        Self {
            config,
            snake,
            food,
            special_food: None,
            obstacles,
            score: 0,
            high_score,
            state: GameState::Menu,
            rng,
            just_died: false,
            lives: 3,
            menu_selection: 0,
            stats,
            start_time: Instant::now(),
            death_message: String::new(),
            level: 1,
            player_name: String::from("Player"),
            high_scores,
            power_up: None,
        }
    }

    pub fn load_high_scores() -> Vec<HighScoreEntry> {
        fs::read_to_string("highscores.json")
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
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

    fn save_high_scores(&self) {
        if let Ok(json) = serde_json::to_string(&self.high_scores) {
            let _ = fs::write("highscores.json", json);
        }
    }

    fn update_high_scores(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
            self.high_scores.push(HighScoreEntry {
                name: self.player_name.clone(),
                score: self.score,
            });
            self.high_scores.sort_by(|a, b| b.score.cmp(&a.score));
            self.high_scores.truncate(5);
            self.save_high_scores();
        }
    }

    pub fn get_player_name(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        self.state = GameState::Paused;
        let x_pos = (self.config.width / 2) - 10;
        let y_pos = self.config.height / 2;
        crossterm::execute!(
            stdout,
            crossterm::cursor::MoveTo(x_pos, y_pos),
            crossterm::style::Print("Enter your name: "),
            crossterm::cursor::Show,
        )?;
        let mut name = String::new();
        loop {
            if crossterm::event::poll(Duration::from_millis(100))? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    match key.code {
                        crossterm::event::KeyCode::Enter => break,
                        crossterm::event::KeyCode::Char(c) => {
                            name.push(c);
                            crossterm::execute!(stdout, crossterm::style::Print(c))?;
                        }
                        _ => {}
                    }
                }
            }
        }
        self.player_name = name;
        crossterm::execute!(stdout, crossterm::cursor::Hide)?;
        self.state = GameState::Playing;
        Ok(())
    }

    pub fn save_game(&self) {
        let state = SaveState {
            snake: Snake {
                body: self.snake.body.clone(),
                direction: self.snake.direction,
                next_direction: self.snake.next_direction,
                speed_multiplier: self.snake.speed_multiplier,
            },
            food: self.food,
            obstacles: self.obstacles.clone(),
            score: self.score,
            special_food: self.special_food.map(|(p, t, ft)| (p, t.elapsed().as_secs(), ft)),
            power_up: self.power_up.map(|(p, t, pt)| (p, t.elapsed().as_secs(), pt)),
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
                self.special_food = state.special_food.map(|(p, t, ft)| (p, Instant::now().checked_sub(Duration::from_secs(t)).unwrap(), ft));
                self.power_up = state.power_up.map(|(p, t, pt)| (p, Instant::now().checked_sub(Duration::from_secs(t)).unwrap(), pt));
                self.state = GameState::Paused;
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
                if !snake.body.contains(&p) && !obstacles.contains(&p) {
                    obstacles.push(p);
                    break;
                }
            }
        }
        obstacles
    }

    fn generate_wall_obstacles(width: u16, height: u16, snake: &Snake, rng: &mut rand::rngs::ThreadRng, num_walls: u32) -> Vec<Point> {
        let mut obstacles = Vec::new();
        for _ in 0..num_walls {
            let is_vertical = rng.gen_bool(0.5);
            let wall_len = rng.gen_range(5..height / 2);
            let start_x = rng.gen_range(2..width - 2);
            let start_y = rng.gen_range(2..height - 2);

            for i in 0..wall_len {
                let p = if is_vertical {
                    Point { x: start_x, y: start_y + i }
                } else {
                    Point { x: start_x + i, y: start_y }
                };

                if p.x < width - 1 && p.y < height - 1 && !snake.body.contains(&p) && !obstacles.contains(&p) {
                    obstacles.push(p);
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
        let start_x = self.config.width / 2;
        let start_y = self.config.height / 2;
        self.snake = Snake::new(Point { x: start_x, y: start_y });
        self.obstacles = Self::generate_wall_obstacles(self.config.width, self.config.height, &self.snake, &mut self.rng, self.level);
        self.food = Self::generate_food(self.config.width, self.config.height, &self.snake, &self.obstacles, &mut self.rng);
        self.special_food = None;
        self.score = 0;
        self.level = 1;
        self.lives = 3;
        self.state = GameState::Playing;
        self.just_died = false;
        self.power_up = None;
    }

    fn respawn(&mut self) {
        let start_x = self.config.width / 2;
        let start_y = self.config.height / 2;
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

        self.manage_special_food();
        self.manage_power_ups();

        let head = self.snake.head();
        let next_head = self.calculate_next_head(head);

        // Check collision with walls and obstacles
        let mut hit_wall = false;
        let final_head = if self.config.wrap_mode {
            self.calculate_wrapped_head(next_head)
        } else {
            if next_head.x == 0 || next_head.x >= self.config.width - 1 || next_head.y == 0 || next_head.y >= self.config.height - 1 {
                hit_wall = true;
            }
            next_head
        };

        if self.obstacles.contains(&final_head) {
            hit_wall = true;
        }

        if hit_wall {
            if let Some((_, _, PowerUpType::Invincibility)) = self.power_up {
                // Invincible, do nothing
            } else {
                self.handle_death("Hit Wall/Obstacle");
                return;
            }
        }

        // Check power-up collision
        if let Some((pos, _, power_up_type)) = self.power_up {
            if final_head == pos {
                match power_up_type {
                    PowerUpType::SpeedBoost => self.snake.speed_multiplier = 2,
                    PowerUpType::Invincibility => {}
                }
                self.power_up = None;
                beep(Sound::PowerUp);
            }
        }

        // Check special food collision
        let mut grow = false;
        if let Some((food_pos, _, food_type)) = self.special_food {
            if final_head == food_pos {
                grow = true;
                match food_type {
                    FoodType::Golden => self.score += 5,
                    FoodType::ScoreBoost => self.score *= 2,
                }
                self.special_food = None;
                beep(Sound::Food);
            }
        }

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
            beep(Sound::Food);

            if self.score.is_multiple_of(10) {
                self.level += 1;
                self.obstacles.clear();
                self.obstacles.extend(Self::generate_wall_obstacles(self.config.width, self.config.height, &self.snake, &mut self.rng, self.level));
            } else if self.score.is_multiple_of(5) {
                let new_obstacles = Self::generate_obstacles(self.config.width, self.config.height, &self.snake, &mut self.rng, 1);
                self.obstacles.extend(new_obstacles);
            }

            self.food = Self::generate_food(self.config.width, self.config.height, &self.snake, &self.obstacles, &mut self.rng);
        }

        self.snake.move_to(final_head, grow);
    }

    fn manage_power_ups(&mut self) {
        if let Some((_, spawn_time, _)) = self.power_up {
            if spawn_time.elapsed() > Duration::from_secs(10) {
                self.power_up = None;
                self.snake.speed_multiplier = 1;
            }
        } else if self.rng.gen_bool(0.01) {
            let mut obstructions = self.obstacles.clone();
            obstructions.push(self.food);
            let pos = Self::generate_food(self.config.width, self.config.height, &self.snake, &obstructions, &mut self.rng);
            let power_up_type = if self.rng.gen_bool(0.5) {
                PowerUpType::SpeedBoost
            } else {
                PowerUpType::Invincibility
            };
            self.power_up = Some((pos, Instant::now(), power_up_type));
        }
    }

    fn manage_special_food(&mut self) {
        if let Some((_, spawn_time, _)) = self.special_food {
            if spawn_time.elapsed() > Duration::from_secs(5) {
                self.special_food = None;
            }
        } else if self.rng.gen_bool(0.02) {
            let mut obstructions = self.obstacles.clone();
            obstructions.push(self.food);
            let pos = Self::generate_food(self.config.width, self.config.height, &self.snake, &obstructions, &mut self.rng);
            let food_type = if self.rng.gen_bool(0.5) {
                FoodType::Golden
            } else {
                FoodType::ScoreBoost
            };
            self.special_food = Some((pos, Instant::now(), food_type));
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
        if x == 0 { x = self.config.width - 2; }
        else if x >= self.config.width - 1 { x = 1; }

        if y == 0 { y = self.config.height - 2; }
        else if y >= self.config.height - 1 { y = 1; }
        Point { x, y }
    }

    fn handle_death(&mut self, cause: &str) {
        self.lives -= 1;
        self.just_died = true;
        beep(Sound::Death);

        // Update stats
        self.stats.games_played += 1;
        self.stats.total_time_s += self.start_time.elapsed().as_secs();
        self.save_stats();

        if self.lives == 0 {
            self.state = GameState::GameOver;
            self.death_message = cause.to_string();
            self.update_high_scores();
        } else {
            self.respawn();
        }
    }
}
