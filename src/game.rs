use rand::Rng;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

use crate::snake::{Direction, Point, Snake};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::time::SystemTime;

use crate::config::GameConfig;

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum PowerUpType {
    SlowDown,
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

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
    Help,
    EnterName,
    Stats,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HighScore {
    pub name: String,
    pub score: u32,
}

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub snake: Snake,
    pub food: Point,
    pub obstacles: HashSet<Point>,
    pub score: u32,
    pub power_up: Option<PowerUp>,
    pub bonus_food: Option<(Point, u64)>, // elapsed time in seconds
    pub lives: u32,
    pub stats: Statistics,
}

#[derive(Serialize, Deserialize, Default, Clone)]
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
    pub high_scores: Vec<HighScore>,
    pub state: GameState,
    pub player_name_input: String,
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
    pub fn new(config: GameConfig) -> Self {
        let mut rng = rand::thread_rng();
        let start_x = config.width / 2;
        let start_y = config.height / 2;
        let snake = Snake::new(Point {
            x: start_x,
            y: start_y,
        });
        let obstacles = Self::generate_obstacles(config.width, config.height, &snake, &mut rng, 3);
        let food = Self::generate_food(config.width, config.height, &snake, &obstacles, &mut rng);
        let high_scores = Self::load_high_scores_from_file("highscores.json");
        let high_score = high_scores.first().map_or(0, |hs| hs.score);
        let stats = Self::load_stats_from_file("stats.json");
        Self {
            width: config.width,
            height: config.height,
            wrap_mode: config.wrap_mode,
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
            skin: config.skin,
            theme: config.theme,
            lives: 3,
            menu_selection: 0,
            stats,
            start_time: Instant::now(),
            death_message: String::new(),
            player_name_input: String::new(),
        }
    }

    pub fn load_high_scores_from_file(path: &str) -> Vec<HighScore> {
        let mut content = String::new();
        File::open(path)
            .and_then(|f| f.take(1024 * 1024).read_to_string(&mut content))
            .ok()
            .and_then(|_| serde_json::from_str::<Vec<HighScore>>(&content).ok())
            .unwrap_or_default()
    }

    pub fn load_stats_from_file(path: &str) -> Statistics {
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

        #[cfg(unix)]
        let mut options = fs::File::options();
        #[cfg(not(unix))]
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

    pub fn save_stats_to_file(&self, path: &str) {
        if let Ok(json) = serde_json::to_string(&self.stats) {
            let _ = Self::atomic_write(path, json);
        }
    }

    pub fn save_stats(&self) {
        self.save_stats_to_file("stats.json");
    }

    pub fn save_high_score_to_file(&mut self, name: String, score: u32, path: &str) {
        self.high_scores.push(HighScore { name, score });
        self.high_scores.sort_unstable_by(|a, b| b.score.cmp(&a.score));
        self.high_scores.truncate(5);
        if let Ok(json) = serde_json::to_string(&self.high_scores) {
            let _ = Self::atomic_write(path, json);
        }
    }

    pub fn save_high_score(&mut self, name: String, score: u32) {
        self.save_high_score_to_file(name, score, "highscores.json");
    }

    pub fn save_game_to_file(&self, path: &str) {
        let bonus_food = self.bonus_food.map(|(p, i)| {
            (p, i.elapsed().as_secs())
        });

        let state = SaveState {
            snake: Snake {
                body: self.snake.body.clone(),
                direction: self.snake.direction,
                next_direction: self.snake.next_direction,
            },
            food: self.food,
            obstacles: self.obstacles.clone(),
            score: self.score,
            power_up: self.power_up.clone(),
            bonus_food,
            lives: self.lives,
            stats: Statistics {
                games_played: self.stats.games_played,
                total_score: self.stats.total_score,
                total_food_eaten: self.stats.total_food_eaten,
                total_time_s: self.stats.total_time_s,
            },
        };
        if let Ok(json) = serde_json::to_string(&state) {
            let _ = Self::atomic_write(path, json);
        }
    }

    pub fn save_game(&self) {
        self.save_game_to_file("savegame.json");
    }

    pub fn load_game_from_file(&mut self, path: &str) -> bool {
        let mut content = String::new();
        File::open(path)
            .and_then(|f| f.take(1024 * 1024).read_to_string(&mut content))
            .ok()
            .and_then(|_| serde_json::from_str::<SaveState>(&content).ok())
            .is_some_and(|state| {
                self.snake = state.snake;
                self.food = state.food;
                self.obstacles = state.obstacles;
                self.score = state.score;
                self.power_up = state.power_up;
                self.bonus_food = state.bonus_food.map(|(p, s)| (p, Instant::now().checked_sub(Duration::from_secs(s)).unwrap_or_else(Instant::now)));
                self.lives = state.lives;
                self.stats = state.stats;
                self.state = GameState::Paused;
                true
            })
    }

    pub fn load_game(&mut self) -> bool {
        self.load_game_from_file("savegame.json")
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
            loop {
                let x = rng.gen_range(1..width - 1);
                let y = rng.gen_range(1..height - 1);
                let p = Point { x, y };
                // Ensure obstacle is not on snake and not too close to head to avoid instant death on start
                // Simple check: not on body.
                if !snake.body.contains(&p) && !obstacles.contains(&p) {
                    obstacles.insert(p);
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

        let is_invincible = self.power_up.as_ref().is_some_and(|p| {
            p.activation_time.is_some() && p.p_type == PowerUpType::Invincibility
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
            self.stats.total_food_eaten += 1;
            self.stats.total_score += 1;
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
            obstructions.insert(self.food);
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
            self.death_message = cause.to_string();
            if self.score > 0 && (self.high_scores.len() < 5 || self.score > self.high_scores.last().map_or(0, |hs| hs.score)) {
                self.state = GameState::EnterName;
                self.player_name_input.clear();
            } else {
                self.state = GameState::GameOver;
            }
        } else {
            self.respawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;

    fn get_test_config() -> GameConfig {
        GameConfig {
            width: 20,
            height: 20,
            wrap_mode: false,
            skin: 'O',
            theme: "dark".to_string(),
        }
    }

    #[test]
    fn test_load_stats_from_file() {
        let mut game = Game::new(get_test_config());
        game.stats.games_played = 5;
        game.stats.total_score = 100;

        let path = std::env::temp_dir().join("test_stats.json");
        let path_str = path.to_str().unwrap();

        game.save_stats_to_file(path_str);

        let loaded_stats = Game::load_stats_from_file(path_str);
        assert_eq!(loaded_stats.games_played, 5);
        assert_eq!(loaded_stats.total_score, 100);
    }

    #[test]
    fn test_load_game_from_file() {
        let mut game = Game::new(get_test_config());
        game.score = 42;
        game.lives = 2;

        let path = std::env::temp_dir().join("test_savegame.json");
        let path_str = path.to_str().unwrap();

        game.save_game_to_file(path_str);

        let mut loaded_game = Game::new(get_test_config());
        let success = loaded_game.load_game_from_file(path_str);

        assert!(success);
        assert_eq!(loaded_game.score, 42);
        assert_eq!(loaded_game.lives, 2);
    }
}
