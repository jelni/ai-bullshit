mod config;
mod game;
mod snake;
mod ui;

use clap::Parser;
use config::Args;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self},
};
use game::{Game, GameState, PowerUpType};
use snake::Direction;
use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Validate args
    if args.width < 10 || args.height < 10 {
        eprintln!("Error: Width and height must be at least 10.");
        std::process::exit(1);
    }

    // Check terminal size
    if let Ok((term_width, term_height)) = terminal::size() {
         // Use match or combinators to avoid collapsible_if lint in strict mode
         if term_width < args.width || term_height < args.height {
             eprintln!("Error: Terminal size ({term_width}x{term_height}) is smaller than game board ({0}x{1}). Resize terminal or use smaller board.", args.width, args.height);
             std::process::exit(1);
         }
    }

    // Panic Hook
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
        default_panic(info);
    }));

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
    let mut game = Game::new(args.width, args.height, args.wrap, args.skin, args.theme);
    let mut last_tick = Instant::now();
    let base_tick_rate = Duration::from_millis(150);

    // Initial draw
    ui::draw(&game, stdout)?;

    loop {
        if game.state == GameState::Playing && game.just_died {
             // We just died (lost a life), show countdown before resuming
             game.just_died = false; // Reset flag so we don't loop here
             for i in (1..=3).rev() {
                 ui::draw_countdown(&game, stdout, i)?;
                 std::thread::sleep(Duration::from_secs(1));
             }
             last_tick = Instant::now();
        }
        // Calculate dynamic tick rate based on score
        // Base rate 150ms. Subtract 5ms per 1 score, capped at minimum 50ms
        let mut current_tick_rate = if game.score > 0 {
            base_tick_rate
                .saturating_sub(Duration::from_millis(u64::from(game.score) * 5))
                .max(Duration::from_millis(50))
        } else {
            base_tick_rate
        };

        if let Some(power_up) = &mut game.power_up
            && let Some(activation_time) = power_up.activation_time {
                if activation_time.elapsed() < Duration::from_secs(5) {
                    match power_up.p_type {
                        PowerUpType::SlowDown => {
                            current_tick_rate += Duration::from_millis(100); // Slow down
                        }
                        PowerUpType::SpeedBoost => {
                            current_tick_rate = current_tick_rate.saturating_sub(Duration::from_millis(50)).max(Duration::from_millis(20)); // Speed up
                        }
                        PowerUpType::Invincibility => {}
                    }
                } else {
                    game.power_up = None; // Power-up expired
                }
            }

        let timeout = current_tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
             // Use match to avoid collapsible_if lint without unstable features
             match event::read()? {
                 Event::Key(key) if key.kind == KeyEventKind::Press => {
                     if !handle_key_event(key.code, &mut game, stdout) {
                         break;
                     }
                 }
                 _ => {}
             }
        }

        if last_tick.elapsed() >= current_tick_rate {
            if game.state == GameState::Playing {
                game.update();
            }
            ui::draw(&game, stdout)?;
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn handle_key_event(code: KeyCode, game: &mut Game, stdout: &mut Stdout) -> bool {
    if game.state == GameState::EnterName {
        match code {
            KeyCode::Enter => {
                if !game.player_name.is_empty() {
                    Game::save_high_score(&game.player_name, game.score);
                    if game.score > game.high_score {
                        game.high_score = game.score;
                    }
                    game.state = GameState::GameOver;
                }
            }
            KeyCode::Backspace => {
                game.player_name.pop();
            }
            KeyCode::Char(c) => {
                if c.is_ascii_alphanumeric() && game.player_name.len() < 10 {
                    game.player_name.push(c);
                }
            }
            _ => {}
        }
        return true;
    }

    match code {
        KeyCode::Char('q') => {
            if game.state == GameState::Playing || game.state == GameState::Paused {
                game.state = GameState::Menu;
            } else {
                return false;
            }
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            if game.state == GameState::Menu {
                match game.menu_selection {
                    0 => game.reset(),
                    1 => { let _ = game.load_game(); },
                    2 => game.state = GameState::Help,
                    3 => return false,
                    _ => {}
                }
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
        KeyCode::Char('s') => {
            if game.state == GameState::Paused {
                game.save_game();
                game.save_stats();
                game.state = GameState::Menu;
            }
        }
        KeyCode::Char('b') => {
            // Boss Key: Fake terminal mode
            let _ = execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen);
            let _ = terminal::disable_raw_mode();

            println!("user@workstation:~/projects/reports$ ");
            println!("user@workstation:~/projects/reports$ ./compile_report.sh");
            println!("Compiling report... [=================>         ] 65%");

            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);

            let _ = terminal::enable_raw_mode();
            let _ = execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide);
            let _ = ui::draw(game, stdout);
        }
        KeyCode::Up => {
            if game.state == GameState::Menu {
                if game.menu_selection > 0 {
                    game.menu_selection -= 1;
                } else {
                    game.menu_selection = 3;
                }
            } else {
                game.handle_input(Direction::Up);
            }
        },
        KeyCode::Down => {
            if game.state == GameState::Menu {
                if game.menu_selection < 3 {
                    game.menu_selection += 1;
                } else {
                    game.menu_selection = 0;
                }
            } else {
                game.handle_input(Direction::Down);
            }
        },
        KeyCode::Left => game.handle_input(Direction::Left),
        KeyCode::Right => game.handle_input(Direction::Right),
        _ => {}
    }
    true
}
