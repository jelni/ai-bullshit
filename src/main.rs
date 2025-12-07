mod game;
mod snake;

use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self},
};
use game::{Game, GameState};
use snake::Direction;
use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

#[derive(Parser, Debug, Clone, Copy)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = 40)]
    width: u16,

    #[arg(long, default_value_t = 20)]
    height: u16,

    #[arg(long, default_value_t = false)]
    wrap: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Validate args
    if args.width < 10 || args.height < 10 {
        eprintln!("Error: Width and height must be at least 10.");
        std::process::exit(1);
    }

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    // We wrap the game loop in a result to ensure we can cleanup on error
    let res = run_game(&mut stdout, args);

    // Cleanup
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    if let Err(e) = res {
        eprintln!("Error: {e:?}");
    }

    Ok(())
}

fn run_game(stdout: &mut Stdout, args: Args) -> io::Result<()> {
    let mut game = Game::new(args.width, args.height, args.wrap);
    let mut last_tick = Instant::now();
    let base_tick_rate = Duration::from_millis(150);

    // Initial draw
    game.draw(stdout)?;

    loop {
        // Calculate dynamic tick rate based on score
        // Base rate 150ms. Subtract 5ms per 1 score, capped at minimum 50ms
        let current_tick_rate = if game.score > 0 {
             base_tick_rate.saturating_sub(Duration::from_millis(u64::from(game.score) * 5)).max(Duration::from_millis(50))
        } else {
             base_tick_rate
        };

        let timeout = current_tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
             // Use match to avoid collapsible_if lint without unstable features
             match event::read()? {
                 Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char(' ') => {
                            if game.state == GameState::Menu {
                                game.reset();
                            }
                        }
                        KeyCode::Char('r') => {
                            if game.state == GameState::GameOver {
                                game.reset();
                            }
                        }
                        KeyCode::Char('p') => {
                            if game.state == GameState::Playing {
                                game.state = GameState::Paused;
                            } else if game.state == GameState::Paused {
                                game.state = GameState::Playing;
                            }
                        }
                        KeyCode::Up => if game.snake.direction != Direction::Down { game.snake.direction = Direction::Up; },
                        KeyCode::Down => if game.snake.direction != Direction::Up { game.snake.direction = Direction::Down; },
                        KeyCode::Left => if game.snake.direction != Direction::Right { game.snake.direction = Direction::Left; },
                        KeyCode::Right => if game.snake.direction != Direction::Left { game.snake.direction = Direction::Right; },
                        _ => {}
                    }
                 }
                 _ => {}
             }
        }

        if last_tick.elapsed() >= current_tick_rate {
            if game.state == GameState::Playing {
                game.update();
            }
            game.draw(stdout)?;
            last_tick = Instant::now();
        }
    }

    Ok(())
}
