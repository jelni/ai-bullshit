use crossterm::{
    cursor,
    style::{Color, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use rand::Rng;
use std::fs;
use std::io::{self, Stdout, Write};
use std::time::{Duration, Instant};

use crate::snake::{Direction, Point, Snake};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
}

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub snake: Snake,
    pub food: Point,
    pub obstacles: Vec<Point>,
    pub score: u32,
}

pub struct Game {
    pub width: u16,
    pub height: u16,
    pub wrap_mode: bool,
    pub snake: Snake,
    pub food: Point,
    pub bonus_food: Option<(Point, Instant)>,
    pub obstacles: Vec<Point>,
    pub score: u32,
    pub high_score: u32,
    pub state: GameState,
    pub rng: rand::rngs::ThreadRng,
    pub just_died: bool,
    pub skin: char,
}

impl Game {
    pub fn new(width: u16, height: u16, wrap_mode: bool, skin: char) -> Self {
        let mut rng = rand::thread_rng();
        let start_x = width / 2;
        let start_y = height / 2;
        let snake = Snake::new(Point { x: start_x, y: start_y });
        let obstacles = Self::generate_obstacles(width, height, &snake, &mut rng, 3);
        let food = Self::generate_food(width, height, &snake, &obstacles, &mut rng);
        let high_score = *Self::load_high_scores().first().unwrap_or(&0);
        Self {
            width,
            height,
            wrap_mode,
            snake,
            food,
            bonus_food: None,
            obstacles,
            score: 0,
            high_score,
            state: GameState::Menu,
            rng,
            just_died: false,
            skin,
        }
    }

    fn load_high_scores() -> Vec<u32> {
        fs::read_to_string("highscore.txt").map_or_else(
            |_| Vec::new(),
            |content| {
                content
                    .lines()
                    .filter_map(|line| line.trim().parse::<u32>().ok())
                    .collect()
            },
        )
    }

    fn save_high_score(score: u32) {
        let mut scores = Self::load_high_scores();
        scores.push(score);
        scores.sort_unstable_by(|a, b| b.cmp(a));
        scores.truncate(5);
        let content = scores
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        let _ = fs::write("highscore.txt", content);
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
        self.state = GameState::Playing;
        self.just_died = false;
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

        // Manage bonus food
        if let Some((_, spawn_time)) = self.bonus_food {
             if spawn_time.elapsed() > Duration::from_secs(5) {
                 self.bonus_food = None;
             }
        } else if self.rng.gen_bool(0.01) {
             // 1% chance per tick to spawn bonus food
             // Use temporary obstacles vector including food to prevent overlap
             let mut obstructions = self.obstacles.clone();
             obstructions.push(self.food);
             let bonus = Self::generate_food(self.width, self.height, &self.snake, &obstructions, &mut self.rng);
             self.bonus_food = Some((bonus, Instant::now()));
        }

        let head = self.snake.head();
        let next_head = match self.snake.direction {
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
        };

        // Check collision with walls
        let mut hit_wall = false;
        let final_head = if self.wrap_mode {
            let mut x = next_head.x;
            let mut y = next_head.y;
            if x == 0 { x = self.width - 2; }
            else if x >= self.width - 1 { x = 1; }

            if y == 0 { y = self.height - 2; }
            else if y >= self.height - 1 { y = 1; }
            Point { x, y }
        } else {
            if next_head.x == 0 || next_head.x >= self.width - 1 || next_head.y == 0 || next_head.y >= self.height - 1 {
                hit_wall = true;
            }
            next_head
        };

        // Check collision with obstacles
        if self.obstacles.contains(&final_head) {
            hit_wall = true;
        }

        if hit_wall {
            self.state = GameState::GameOver;
            self.just_died = true;
            if self.score > self.high_score {
                self.high_score = self.score;
                Self::save_high_score(self.high_score);
            }
            return;
        }

        // Check bonus food collision
        let grow = if self.bonus_food.is_some_and(|(bonus_p, _)| final_head == bonus_p) {
             self.score += 5;
             self.bonus_food = None;
             true
        } else {
             final_head == self.food
        };

        // Refined self collision check
        if self.snake.body.contains(&final_head) {
             if !grow && final_head == *self.snake.body.back().unwrap() {
                 // We are moving into the tail, but the tail will move. Safe.
             } else {
                 self.state = GameState::GameOver;
                 self.just_died = true;
                 if self.score > self.high_score {
                     self.high_score = self.score;
                     Self::save_high_score(self.high_score);
                 }
                 return;
             }
        }

        if grow && final_head == self.food {
            self.score += 1;
            // Add a new obstacle every 5 points
            if self.score.is_multiple_of(5) {
                let new_obstacles = Self::generate_obstacles(self.width, self.height, &self.snake, &mut self.rng, 1);
                self.obstacles.extend(new_obstacles);
            }
            self.food = Self::generate_food(self.width, self.height, &self.snake, &self.obstacles, &mut self.rng);
        }

        // We need to override the snake move because we might have wrapped
        // But snake.move_forward calculates next head internally based on direction.
        // We need to allow snake to accept a specific next position or update it logic.
        // Let's modify snake.move_forward to take the next_head.
        self.snake.move_to(final_head, grow);
    }

    pub fn draw(&self, stdout: &mut Stdout) -> io::Result<()> {
        // Clear screen
        stdout.queue(Clear(ClearType::All))?;

        match self.state {
            GameState::Menu => self.draw_menu(stdout)?,
            GameState::Playing | GameState::GameOver | GameState::Paused => self.draw_game(stdout)?,
        }

        stdout.flush()?;
        Ok(())
    }

    fn draw_menu(&self, stdout: &mut Stdout) -> io::Result<()> {
        let title = "SNAKE GAME";
        let msg = "Press SPACE to Start";
        let load = "Press 'l' to Load Game";
        let quit = "Press 'q' to Quit";

        stdout.queue(SetForegroundColor(Color::Green))?;
        stdout.queue(cursor::MoveTo((self.width / 2).saturating_sub(u16::try_from(title.len()).unwrap() / 2), self.height / 2 - 3))?;
        write!(stdout, "{title}")?;

        stdout.queue(SetForegroundColor(Color::White))?;
        stdout.queue(cursor::MoveTo((self.width / 2).saturating_sub(u16::try_from(msg.len()).unwrap() / 2), self.height / 2 - 1))?;
        write!(stdout, "{msg}")?;

        stdout.queue(SetForegroundColor(Color::Cyan))?;
        stdout.queue(cursor::MoveTo((self.width / 2).saturating_sub(u16::try_from(load.len()).unwrap() / 2), self.height / 2 + 1))?;
        write!(stdout, "{load}")?;

        stdout.queue(SetForegroundColor(Color::Red))?;
        stdout.queue(cursor::MoveTo((self.width / 2).saturating_sub(u16::try_from(quit.len()).unwrap() / 2), self.height / 2 + 3))?;
        write!(stdout, "{quit}")?;

        // Draw Leaderboard
        let scores = Self::load_high_scores();
        if !scores.is_empty() {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(cursor::MoveTo((self.width / 2).saturating_sub(10), self.height / 2 + 6))?;
            write!(stdout, "Top Scores:")?;
            for (i, s) in scores.iter().enumerate().take(5) {
                stdout.queue(cursor::MoveTo(
                    (self.width / 2).saturating_sub(10),
                    self.height / 2 + 7 + u16::try_from(i).unwrap_or(0),
                ))?;
                write!(stdout, "{}. {}", i + 1, s)?;
            }
        }
        Ok(())
    }

    fn draw_game(&self, stdout: &mut Stdout) -> io::Result<()> {
         // Draw borders
         if self.just_died {
             stdout.queue(SetForegroundColor(Color::Red))?;
         } else {
             stdout.queue(SetForegroundColor(Color::Blue))?;
         }

        for y in 0..self.height {
            for x in 0..self.width {
                if x == 0 || x == self.width - 1 || y == 0 || y == self.height - 1 {
                    stdout.queue(cursor::MoveTo(x, y))?;
                    write!(stdout, "#")?;
                }
            }
        }

        // Draw food
        stdout.queue(cursor::MoveTo(self.food.x, self.food.y))?;
        stdout.queue(SetForegroundColor(Color::Red))?;
        write!(stdout, "●")?;

        // Draw obstacles
        stdout.queue(SetForegroundColor(Color::Magenta))?;
        for obs in &self.obstacles {
            stdout.queue(cursor::MoveTo(obs.x, obs.y))?;
            write!(stdout, "X")?;
        }

        // Draw bonus food
        if let Some((bonus_p, _)) = self.bonus_food {
             stdout.queue(cursor::MoveTo(bonus_p.x, bonus_p.y))?;
             stdout.queue(SetForegroundColor(Color::Yellow))?;
             write!(stdout, "★")?;
        }

        // Draw snake
        stdout.queue(SetForegroundColor(Color::DarkGreen))?;
        for (i, part) in self.snake.body.iter().enumerate() {
            stdout.queue(cursor::MoveTo(part.x, part.y))?;
            if i == 0 {
                 // Head
                 let head_char = match self.snake.direction {
                     Direction::Up => '^',
                     Direction::Down => 'v',
                     Direction::Left => '<',
                     Direction::Right => '>',
                 };
                 write!(stdout, "{head_char}")?;
            } else {
                 // Body
                 write!(stdout, "{}", self.skin)?;
            }
        }

        // Draw score
        stdout.queue(SetForegroundColor(Color::Reset))?;
        stdout.queue(cursor::MoveTo(0, self.height))?;
        write!(stdout, "Score: {} | High Score: {}", self.score, self.high_score)?;

        // Draw Game Over
        if self.state == GameState::GameOver {
             let msg = "GAME OVER";
             let msg_len = u16::try_from(msg.len()).unwrap();
             let x_pos = (self.width / 2).saturating_sub(msg_len / 2);
             let y_pos = self.height / 2;

             stdout.queue(SetForegroundColor(Color::Red))?;
             stdout.queue(cursor::MoveTo(x_pos, y_pos))?;
             write!(stdout, "{msg}")?;

             let sub_msg = "Press 'q' to quit, 'r' to restart";
             let sub_msg_len = u16::try_from(sub_msg.len()).unwrap();
             let x_sub = (self.width / 2).saturating_sub(sub_msg_len / 2);
             stdout.queue(cursor::MoveTo(x_sub, y_pos + 1))?;
             write!(stdout, "{sub_msg}")?;
             stdout.queue(SetForegroundColor(Color::Reset))?;
        }

        if self.state == GameState::Paused {
             let msg = "PAUSED";
             let msg_len = u16::try_from(msg.len()).unwrap();
             let x_pos = (self.width / 2).saturating_sub(msg_len / 2);
             let y_pos = self.height / 2;

             stdout.queue(SetForegroundColor(Color::Yellow))?;
             stdout.queue(cursor::MoveTo(x_pos, y_pos))?;
             write!(stdout, "{msg}")?;

             let sub_msg = "Press 's' to Save & Quit, 'p' to Resume";
             let sub_msg_len = u16::try_from(sub_msg.len()).unwrap();
             let x_sub = (self.width / 2).saturating_sub(sub_msg_len / 2);
             stdout.queue(cursor::MoveTo(x_sub, y_pos + 1))?;
             write!(stdout, "{sub_msg}")?;

             stdout.queue(SetForegroundColor(Color::Reset))?;
        }
        Ok(())
    }
}
