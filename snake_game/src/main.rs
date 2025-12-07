use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Color, SetForegroundColor},
    terminal::{self, Clear, ClearType},
    QueueableCommand,
};
use rand::Rng;
use std::{
    collections::VecDeque,
    io::{self, Stdout, Write},
    time::{Duration, Instant},
};

const WIDTH: u16 = 40;
const HEIGHT: u16 = 20;

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq)]
struct Point {
    x: u16,
    y: u16,
}

struct Snake {
    body: VecDeque<Point>,
    direction: Direction,
}

impl Snake {
    fn new(start: Point) -> Self {
        let mut body = VecDeque::new();
        // Head
        body.push_back(start);
        // Body segments below head (since we face UP)
        body.push_back(Point { x: start.x, y: start.y + 1 });
        body.push_back(Point { x: start.x, y: start.y + 2 });
        Self {
            body,
            direction: Direction::Up,
        }
    }

    fn head(&self) -> Point {
        *self.body.front().unwrap()
    }

    fn move_forward(&mut self, grow: bool) {
        let head = self.head();
        let new_head = match self.direction {
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

        self.body.push_front(new_head);
        if !grow {
            self.body.pop_back();
        }
    }
}

struct Game {
    snake: Snake,
    food: Point,
    score: u32,
    is_over: bool,
    rng: rand::rngs::ThreadRng,
}

impl Game {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let start_x = WIDTH / 2;
        let start_y = HEIGHT / 2;
        let snake = Snake::new(Point { x: start_x, y: start_y });
        let food = Self::generate_food(&snake.body, &mut rng);
        Self {
            snake,
            food,
            score: 0,
            is_over: false,
            rng,
        }
    }

    fn generate_food(snake_body: &VecDeque<Point>, rng: &mut rand::rngs::ThreadRng) -> Point {
        loop {
            // Food must be within walls (1..WIDTH-1, 1..HEIGHT-1)
            let x = rng.gen_range(1..WIDTH - 1);
            let y = rng.gen_range(1..HEIGHT - 1);
            let p = Point { x, y };
            if !snake_body.contains(&p) {
                return p;
            }
        }
    }

    fn update(&mut self) {
        if self.is_over {
            return;
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
        if next_head.x == 0 || next_head.x >= WIDTH - 1 || next_head.y == 0 || next_head.y >= HEIGHT - 1
        {
            self.is_over = true;
            return;
        }

        // Check collision with self
        // Note: The tail moves forward, so technically we shouldn't collide with the *current* tail position
        // if we are not growing. However, simplifying to "collide with any current body part" is safer
        // and standard for simple implementations to avoid "chasing tail" edge cases.
        // Also we haven't moved yet, so we check if next_head is in current body.
        // Exception: If next_head is the tail, and we are not growing, the tail will move away, so it's safe.
        // But implementing that logic correctly requires knowing if we grow.

        let grow = next_head == self.food;

        // Refined self collision check
        if self.snake.body.contains(&next_head) {
             if !grow && next_head == *self.snake.body.back().unwrap() {
                 // We are moving into the tail, but the tail will move. Safe.
             } else {
                 self.is_over = true;
                 return;
             }
        }

        if grow {
            self.score += 1;
            self.food = Self::generate_food(&self.snake.body, &mut self.rng);
        }

        self.snake.move_forward(grow);
    }

    fn draw(&self, stdout: &mut Stdout) -> io::Result<()> {
        // Clear screen
        stdout.queue(Clear(ClearType::All))?;

        // Draw borders
        stdout.queue(SetForegroundColor(Color::White))?;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if x == 0 || x == WIDTH - 1 || y == 0 || y == HEIGHT - 1 {
                    stdout.queue(cursor::MoveTo(x, y))?;
                    write!(stdout, "#")?;
                }
            }
        }

        // Draw food
        stdout.queue(cursor::MoveTo(self.food.x, self.food.y))?;
        stdout.queue(SetForegroundColor(Color::Red))?;
        write!(stdout, "O")?;

        // Draw snake
        stdout.queue(SetForegroundColor(Color::Green))?;
        for part in &self.snake.body {
            stdout.queue(cursor::MoveTo(part.x, part.y))?;
            write!(stdout, "█")?;
        }

        // Draw score
        stdout.queue(SetForegroundColor(Color::Reset))?;
        stdout.queue(cursor::MoveTo(0, HEIGHT))?;
        write!(stdout, "Score: {}", self.score)?;

        // Draw Game Over
        if self.is_over {
             let msg = "GAME OVER";
             // Safe unwrap because message is short constant
             let msg_len = u16::try_from(msg.len()).unwrap();
             let x_pos = (WIDTH / 2).saturating_sub(msg_len / 2);
             let y_pos = HEIGHT / 2;

             stdout.queue(SetForegroundColor(Color::Red))?;
             stdout.queue(cursor::MoveTo(x_pos, y_pos))?;
             write!(stdout, "{msg}")?;

             let sub_msg = "Press 'q' to quit, 'r' to restart";
             // Safe unwrap because message is short constant
             let sub_msg_len = u16::try_from(sub_msg.len()).unwrap();
             let x_sub = (WIDTH / 2).saturating_sub(sub_msg_len / 2);
             stdout.queue(cursor::MoveTo(x_sub, y_pos + 1))?;
             write!(stdout, "{sub_msg}")?;
             stdout.queue(SetForegroundColor(Color::Reset))?;
        }

        stdout.flush()?;
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    // We wrap the game loop in a result to ensure we can cleanup on error
    let res = run_game(&mut stdout);

    // Cleanup
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    if let Err(e) = res {
        eprintln!("Error: {e:?}");
    }

    Ok(())
}

fn run_game(stdout: &mut Stdout) -> io::Result<()> {
    let mut game = Game::new();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(150);

    // Initial draw
    game.draw(stdout)?;

    loop {
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('r') => {
                        if game.is_over {
                            game = Game::new();
                            last_tick = Instant::now(); // Reset tick so we don't jump
                            game.draw(stdout)?;
                        }
                    }
                    KeyCode::Up => if game.snake.direction != Direction::Down { game.snake.direction = Direction::Up; },
                    KeyCode::Down => if game.snake.direction != Direction::Up { game.snake.direction = Direction::Down; },
                    KeyCode::Left => if game.snake.direction != Direction::Right { game.snake.direction = Direction::Left; },
                    KeyCode::Right => if game.snake.direction != Direction::Left { game.snake.direction = Direction::Right; },
                    _ => {}
                }
        }

        if last_tick.elapsed() >= tick_rate {
            if !game.is_over {
                game.update();
                game.draw(stdout)?;
            }
            last_tick = Instant::now();
        }
    }

    Ok(())
}
