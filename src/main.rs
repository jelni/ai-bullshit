mod game;
mod snake;
mod ui;

use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self},
};
use game::{Game, GameState};
use snake::Direction;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = 40)]
    width: u16,

    #[arg(long, default_value_t = 20)]
    height: u16,

    #[arg(long, default_value_t = false)]
    wrap: bool,

    #[arg(long, default_value_t = '█')]
    skin: char,

    #[arg(long, value_enum, default_value_t = game::Theme::Classic)]
    theme: game::Theme,

    #[arg(long, value_enum, default_value_t = game::Difficulty::Normal)]
    difficulty: game::Difficulty,

    #[arg(long, default_value_t = false)]
    bot: bool,
}

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
        if term_width < args.width || term_height <= args.height {
            eprintln!(
                "Error: Terminal size ({term_width}x{term_height}) is smaller than game board \
                 ({0}x{1}). Resize terminal or use smaller board.",
                args.width,
                args.height + 1
            );
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
    let res = run_game(&mut stdout, &args);

    // Cleanup
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    if let Err(e) = res {
        eprintln!("Error: {e:?}");
    }

    Ok(())
}

#[expect(clippy::too_many_lines, reason = "Game loop inherently requires handling multiple states and events")]
fn run_game(stdout: &mut Stdout, args: &Args) -> io::Result<()> {
    let diff = args.difficulty;
    let mut game = Game::new(args.width, args.height, args.wrap, args.skin, args.theme, diff);

    if args.bot {
        game.auto_pilot = true;
        game.used_bot_this_game = true;
        game.state = GameState::Playing;
    }

    let mut last_tick = Instant::now();

    // Initial draw
    ui::draw(&game, stdout)?;

    let mut last_frame = Instant::now();

    'mainloop: loop {
        let base_tick_rate = match game.difficulty {
            game::Difficulty::Easy => Duration::from_millis(200),
            game::Difficulty::Normal => Duration::from_millis(150),
            game::Difficulty::Hard => Duration::from_millis(100),
            game::Difficulty::Insane => Duration::from_millis(60),
            game::Difficulty::GodMode => Duration::from_millis(30),
        };

        let delta = last_frame.elapsed();
        last_frame = Instant::now();

        if game.state != GameState::Playing {
            game.shift_timers(delta);
        }

        if game.state == GameState::Playing && game.just_died {
            // We just died (lost a life), show countdown before resuming
            game.just_died = false; // Reset flag so we don't loop here
            let start = Instant::now();
            for i in (1..=3).rev() {
                ui::draw_countdown(&game, stdout, i)?;
                std::thread::sleep(Duration::from_secs(1));
            }
            game.shift_timers(start.elapsed());
            last_frame = Instant::now();
            last_tick = Instant::now();
        }
        // Calculate dynamic tick rate based on food eaten
        // Base rate 150ms. Subtract 5ms per 1 food, capped at minimum 50ms
        let mut current_tick_rate = if game.food_eaten_session > 0 {
            base_tick_rate
                .saturating_sub(Duration::from_millis(u64::from(game.food_eaten_session) * 5))
                .max(Duration::from_millis(50))
        } else {
            base_tick_rate
        };

        if let Some(power_up) = &mut game.power_up
            && let Some(activation_time) = power_up.activation_time
        {
            if activation_time.elapsed().unwrap_or_default() < Duration::from_secs(5) {
                match power_up.p_type {
                    game::PowerUpType::SlowDown => {
                        current_tick_rate += Duration::from_millis(100); // Slow down
                    },
                    game::PowerUpType::SpeedBoost => {
                        current_tick_rate = current_tick_rate
                            .saturating_sub(Duration::from_millis(50))
                            .max(Duration::from_millis(30)); // Speed boost
                    },
                    game::PowerUpType::Invincibility
                    | game::PowerUpType::ExtraLife
                    | game::PowerUpType::PassThroughWalls
                    | game::PowerUpType::Shrink
                    | game::PowerUpType::ClearObstacles
                    | game::PowerUpType::ScoreMultiplier => {}, // Tick rate unaffected
                }
            } else {
                game.power_up = None; // Power-up expired
            }
        }

        let mut timeout = current_tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        while event::poll(timeout)? {
            // Use match to avoid collapsible_if lint without unstable features
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match handle_key_event(key.code, &mut game, stdout) {
                        KeyAction::Quit => {
                            game.save_game();
                            game.save_stats();
                            break 'mainloop;
                        },
                        KeyAction::BossKey => {
                            let boss_key_start = Instant::now();
                            handle_boss_key(&game, stdout);
                            game.shift_timers(boss_key_start.elapsed());
                            last_frame = Instant::now();
                            last_tick = Instant::now();
                        },
                        KeyAction::Continue => {},
                    }
                },
                _ => {},
            }

            timeout = current_tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if timeout.is_zero() {
                break;
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

enum KeyAction {
    Continue,
    Quit,
    BossKey,
}

fn handle_key_event(code: KeyCode, game: &mut Game, _stdout: &mut Stdout) -> KeyAction {
    if code == KeyCode::Char('b') || code == KeyCode::Char('B') {
        return KeyAction::BossKey;
    }

    let should_continue = match game.state {
        GameState::Menu => handle_menu_input(code, game),
        GameState::Playing => handle_playing_input(code, game),
        GameState::Paused => handle_paused_input(code, game),
        GameState::GameOver | GameState::GameWon => handle_game_over_input(code, game),
        GameState::Help => handle_help_input(code, game),
        GameState::Stats => handle_stats_input(code, game),
        GameState::EnterName => handle_enter_name_input(code, game),
        GameState::Settings => handle_settings_input(code, game),
        GameState::ConfirmQuit => handle_confirm_quit_input(code, game),
        GameState::NftShop => handle_nft_shop_input(code, game),
        GameState::Achievements => handle_achievements_input(code, game),
    };

    if should_continue {
        KeyAction::Continue
    } else {
        KeyAction::Quit
    }
}

fn handle_boss_key(game: &Game, stdout: &mut Stdout) {
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

fn handle_menu_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') => {
            game.previous_state = Some(GameState::Menu);
            game.state = GameState::ConfirmQuit;
        },
        KeyCode::Char(' ') | KeyCode::Enter => match game.menu_selection {
            0 => game.reset(),
            1 => {
                let _ = game.load_game();
            },
            2 => game.state = GameState::Settings,
            3 => game.state = GameState::NftShop,
            4 => game.state = GameState::Stats,
            5 => game.state = GameState::Achievements,
            6 => game.state = GameState::Help,
            7 => {
                game.previous_state = Some(GameState::Menu);
                game.state = GameState::ConfirmQuit;
            },
            _ => {},
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.menu_selection > 0 {
                game.menu_selection -= 1;
            } else {
                game.menu_selection = 7;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.menu_selection < 7 {
                game.menu_selection += 1;
            } else {
                game.menu_selection = 0;
            }
        },
        _ => {},
    }
    true
}

fn handle_playing_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') => {
            game.previous_state = Some(GameState::Playing);
            game.state = GameState::ConfirmQuit;
        },
        KeyCode::Char('p' | 'P') => game.state = GameState::Paused,
        KeyCode::Char('t' | 'T') => {
            game.auto_pilot = !game.auto_pilot;
            if game.auto_pilot {
                game.used_bot_this_game = true;
            }
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => game.handle_input(Direction::Up),
        KeyCode::Down | KeyCode::Char('s' | 'S') => game.handle_input(Direction::Down),
        KeyCode::Left | KeyCode::Char('a' | 'A') => game.handle_input(Direction::Left),
        KeyCode::Right | KeyCode::Char('d' | 'D') => game.handle_input(Direction::Right),
        _ => {},
    }
    true
}

fn handle_paused_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') => {
            game.previous_state = Some(GameState::Paused);
            game.state = GameState::ConfirmQuit;
        },
        KeyCode::Char('p' | 'P') => game.state = GameState::Playing,
        KeyCode::Char('s' | 'S') => {
            // 's' / 'S' is overloaded: moving Down vs Saving Game.
            // But since 's' triggers saving here, we can only move Down with Arrow keys.
            // We give Save game priority for 's' / 'S' in pause menu.
            game.save_game();
            game.save_stats();
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => game.handle_input(Direction::Up),
        KeyCode::Down => game.handle_input(Direction::Down),
        KeyCode::Left | KeyCode::Char('a' | 'A') => game.handle_input(Direction::Left),
        KeyCode::Right | KeyCode::Char('d' | 'D') => game.handle_input(Direction::Right),
        _ => {},
    }
    true
}

fn handle_game_over_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') => {
            game.previous_state = Some(GameState::GameOver);
            game.state = GameState::ConfirmQuit;
        },
        KeyCode::Char('r' | 'R') => game.reset(),
        KeyCode::Up | KeyCode::Char('w' | 'W') => game.handle_input(Direction::Up),
        KeyCode::Down | KeyCode::Char('s' | 'S') => game.handle_input(Direction::Down),
        KeyCode::Left | KeyCode::Char('a' | 'A') => game.handle_input(Direction::Left),
        KeyCode::Right | KeyCode::Char('d' | 'D') => game.handle_input(Direction::Right),
        _ => {},
    }
    true
}

const fn handle_stats_input(_code: KeyCode, game: &mut Game) -> bool {
    game.state = GameState::Menu;
    true
}

const fn handle_achievements_input(_code: KeyCode, game: &mut Game) -> bool {
    game.state = GameState::Menu;
    true
}

const fn handle_help_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        _ => {},
    }
    true
}

fn handle_enter_name_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Enter => {
            if !game.player_name.is_empty() {
                let name = game.player_name.clone();
                let score = game.score;
                game.save_high_score(name, score);
                game.state = game.previous_state.unwrap_or(GameState::GameOver);
            }
        },
        KeyCode::Backspace => {
            game.player_name.pop();
        },
        KeyCode::Char(c) if game.player_name.len() < 10 && c.is_alphanumeric() => {
            game.player_name.push(c);
        },
        _ => {},
    }
    true
}

const fn handle_confirm_quit_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('y' | 'Y') => return false,
        KeyCode::Char('n' | 'N') => {
            if let Some(state) = game.previous_state {
                game.state = state;
            }
        },
        _ => {},
    }
    true
}

fn handle_nft_shop_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.nft_selection > 0 {
                game.nft_selection -= 1;
            } else {
                game.nft_selection = crate::game::AVAILABLE_ITEMS.len() - 1;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.nft_selection < crate::game::AVAILABLE_ITEMS.len() - 1 {
                game.nft_selection += 1;
            } else {
                game.nft_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            let (item, price) = crate::game::AVAILABLE_ITEMS[game.nft_selection];
            match item {
                crate::game::ShopItem::Skin(skin) => {
                    if !game.stats.unlocked_skins.contains(&skin) && game.stats.coins >= price {
                        game.stats.coins -= price;
                        game.stats.unlocked_skins.push(skin);
                        game.save_stats();
                        crate::game::beep();
                    }
                },
                crate::game::ShopItem::Theme(theme) => {
                    if !game.stats.unlocked_themes.contains(&theme) && game.stats.coins >= price {
                        game.stats.coins -= price;
                        game.stats.unlocked_themes.push(theme);
                        game.save_stats();
                        crate::game::beep();
                    }
                },
            }
        },
        _ => {},
    }
    true
}

fn handle_settings_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 3; // Difficulty, Theme, Wrap, Skin
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 3 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Left | KeyCode::Char('a' | 'A') => match game.settings_selection {
            0 => {
                game.difficulty = game.difficulty.prev();
                game.update_high_scores();
            },
            1 => {
                let themes = &game.stats.unlocked_themes;
                let current_idx = themes.iter().position(|&t| t == game.theme).unwrap_or(0);
                let prev_idx = if current_idx > 0 {
                    current_idx - 1
                } else {
                    themes.len() - 1
                };
                game.theme = themes[prev_idx];
            },
            2 => game.wrap_mode = !game.wrap_mode,
            3 => {
                let skins = &game.stats.unlocked_skins;
                let current_idx = skins.iter().position(|&c| c == game.skin).unwrap_or(0);
                let prev_idx = if current_idx > 0 {
                    current_idx - 1
                } else {
                    skins.len() - 1
                };
                game.skin = skins[prev_idx];
            },
            _ => {},
        },
        KeyCode::Right | KeyCode::Enter | KeyCode::Char(' ' | 'd' | 'D') => {
            match game.settings_selection {
                0 => {
                    game.difficulty = game.difficulty.next();
                    game.update_high_scores();
                },
                1 => {
                    let themes = &game.stats.unlocked_themes;
                    let current_idx = themes.iter().position(|&t| t == game.theme).unwrap_or(0);
                    let next_idx = (current_idx + 1) % themes.len();
                    game.theme = themes[next_idx];
                },
                2 => game.wrap_mode = !game.wrap_mode,
                3 => {
                    let skins = &game.stats.unlocked_skins;
                    let current_idx = skins.iter().position(|&c| c == game.skin).unwrap_or(0);
                    let next_idx = (current_idx + 1) % skins.len();
                    game.skin = skins[next_idx];
                },
                _ => {},
            }
        },
        _ => {},
    }
    true
}
