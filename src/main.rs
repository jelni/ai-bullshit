mod config;
mod game;
mod snake;
mod ui;

use crate::config::{Args, GameConfig};
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

fn main() -> io::Result<()> {
    let args = Args::parse();
    let config = GameConfig::from_args(args);

    // Validate args
    if config.width < 10 || config.height < 10 {
        eprintln!("Error: Width and height must be at least 10.");
        std::process::exit(1);
    }

    // Check terminal size
    if let Ok((term_width, term_height)) = terminal::size() {
        // Use match or combinators to avoid collapsible_if lint in strict mode
        if term_width < config.width || term_height < config.height {
            eprintln!("Error: Terminal size ({term_width}x{term_height}) is smaller than game board ({0}x{1}). Resize terminal or use smaller board.", config.width, config.height);
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
    let res = run_game(&mut stdout, config);

    // Cleanup
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    if let Err(e) = res {
        eprintln!("Error: {e:?}");
    }

    Ok(())
}

fn run_game(stdout: &mut Stdout, config: GameConfig) -> io::Result<()> {
    let mut game = Game::new(config);
    let mut last_tick = Instant::now();
    let base_tick_rate = Duration::from_millis(150);

    // Initial draw
    ui::draw(&game, stdout)?;

    let mut first_game = true;

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
            base_tick_rate.saturating_sub(Duration::from_millis(u64::from(game.score) * 5)).max(Duration::from_millis(50))
        } else {
            base_tick_rate
        };

        current_tick_rate /= game.snake.speed_multiplier;

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
                if first_game {
                    game.get_player_name(stdout)?;
                    first_game = false;
                }
                game.update();
            }
            ui::draw(&game, stdout)?;
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn handle_key_event(code: KeyCode, game: &mut Game, stdout: &mut Stdout) -> bool {
    let mut should_continue = true;
    match game.state {
        GameState::Menu => handle_menu_keys(code, game, &mut should_continue),
        GameState::Playing => handle_playing_keys(code, game),
        GameState::Paused => handle_paused_keys(code, game),
        GameState::GameOver => handle_game_over_keys(code, game),
        GameState::Help => {
            if code == KeyCode::Char('q') {
                game.state = GameState::Menu;
            }
        }
    }

    // Global key bindings
    if code == KeyCode::Char('b') {
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

    should_continue
}

fn handle_menu_keys(code: KeyCode, game: &mut Game, should_continue: &mut bool) {
    match code {
        KeyCode::Char(' ') | KeyCode::Enter => match game.menu_selection {
            0 => game.reset(),
            1 => {
                let _ = game.load_game();
            }
            2 => game.state = GameState::Help,
            3 => *should_continue = false,
            _ => {}
        },
        KeyCode::Up => {
            if game.menu_selection > 0 {
                game.menu_selection -= 1;
            } else {
                game.menu_selection = 3;
            }
        }
        KeyCode::Down => {
            if game.menu_selection < 3 {
                game.menu_selection += 1;
            } else {
                game.menu_selection = 0;
            }
        }
        KeyCode::Char('q') => *should_continue = false,
        _ => {}
    }
}

fn handle_playing_keys(code: KeyCode, game: &mut Game) {
    match code {
        KeyCode::Up => game.handle_input(Direction::Up),
        KeyCode::Down => game.handle_input(Direction::Down),
        KeyCode::Left => game.handle_input(Direction::Left),
        KeyCode::Right => game.handle_input(Direction::Right),
        KeyCode::Char('p') => game.state = GameState::Paused,
        KeyCode::Char('q') => game.state = GameState::Menu,
        _ => {}
    }
}

fn handle_paused_keys(code: KeyCode, game: &mut Game) {
    match code {
        KeyCode::Char('p') => game.state = GameState::Playing,
        KeyCode::Char('s') => {
            game.save_game();
            game.save_stats();
            game.state = GameState::Menu;
        }
        KeyCode::Char('q') => game.state = GameState::Menu,
        _ => {}
    }
}

fn handle_game_over_keys(code: KeyCode, game: &mut Game) {
    match code {
        KeyCode::Char('r') => game.reset(),
        KeyCode::Char('q') => game.state = GameState::Menu,
        _ => {}
    }
}
