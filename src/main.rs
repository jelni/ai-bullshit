mod color;
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

#[expect(
    clippy::too_many_lines,
    reason = "Game loop inherently requires handling multiple states and events"
)]
fn run_game(stdout: &mut Stdout, args: &Args) -> io::Result<()> {
    let diff = args.difficulty;
    let mut game = Game::new(args.width, args.height, args.wrap, args.skin, args.theme, diff);

    if args.bot {
        game.auto_pilot = true;
        game.used_bot_this_session = true;
        game.state = GameState::Playing;
    }

    let mut last_tick = Instant::now();

    // Initial draw
    ui::draw(&game, stdout)?;

    let mut last_frame = Instant::now();

    'mainloop: loop {
        let mut base_tick_rate = match game.difficulty {
            game::Difficulty::Easy => Duration::from_millis(200),
            game::Difficulty::Normal => Duration::from_millis(150),
            game::Difficulty::Hard => Duration::from_millis(100),
            game::Difficulty::Insane => Duration::from_millis(60),
            game::Difficulty::GodMode => Duration::from_millis(30),
        };

        match game.current_planet {
            game::Planet::Moon => {
                base_tick_rate = base_tick_rate.mul_f32(1.5);
            },
            game::Planet::Jupiter => {
                base_tick_rate = base_tick_rate.mul_f32(0.7);
            },
            _ => {},
        }

        if game.stats.faction == Some(crate::game::Faction::AzureCobras) {
            let reduction = 10 + (game.stats.faction_rep / 1000);
            base_tick_rate = base_tick_rate
                .saturating_sub(Duration::from_millis(u64::from(reduction)))
                .max(Duration::from_millis(30));
        }

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

        if game.skin == '🐍' {
            current_tick_rate = current_tick_rate
                .saturating_sub(Duration::from_millis(15))
                .max(Duration::from_millis(30));
        }

        if game.stats.equipped_gear == Some(crate::game::Equipment::SpeedTail) {
            current_tick_rate = current_tick_rate
                .saturating_sub(Duration::from_millis(10))
                .max(Duration::from_millis(30));
        }

        if game.stats.equipped_vehicle == Some(crate::game::Vehicle::Bike) {
            current_tick_rate = current_tick_rate
                .saturating_sub(Duration::from_millis(15))
                .max(Duration::from_millis(30));
        }

        let powerup_duration = game.powerup_duration();
        if let Some(power_up) = &mut game.power_up
            && let Some(activation_time) = power_up.activation_time
        {
            if web_time::SystemTime::now()
                .duration_since(web_time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .saturating_sub(activation_time)
                < powerup_duration
            {
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
                    | game::PowerUpType::ScoreMultiplier
                    | game::PowerUpType::Teleport
                    | game::PowerUpType::Magnet
                    | game::PowerUpType::TimeFreeze
                    | game::PowerUpType::Reverse
                    | game::PowerUpType::Decoy
                    | game::PowerUpType::Emp
                    | game::PowerUpType::Nuke => {}, // Tick rate unaffected
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
    if game.state != GameState::EnterName
        && (code == KeyCode::Char('b') || code == KeyCode::Char('B'))
    {
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
        GameState::SkillTree => handle_skill_tree_input(code, game),
        GameState::LevelEditor => handle_level_editor_input(code, game),
        GameState::LevelUp => handle_level_up_input(code, game),
        GameState::Crafting => handle_crafting_input(code, game),
        GameState::BountyBoard => handle_bounty_board_input(code, game),
        GameState::MerchantShop => handle_merchant_shop_input(code, game),
        GameState::CompanionCamp => handle_companion_camp_input(code, game),
        GameState::ClassSelect => handle_class_select_input(code, game),
        GameState::Equipment => handle_equipment_input(code, game),
        GameState::Casino => handle_casino_input(code, game),
        GameState::StockMarket => handle_stock_market_input(code, game),
        GameState::RealEstate => handle_real_estate_input(code, game),
        GameState::VehicleGarage => handle_vehicle_garage_input(code, game),
        GameState::Fishing => handle_fishing_input(code, game),
        GameState::BattlePass => handle_battle_pass_input(code, game),
        GameState::ArtifactShrine => handle_artifact_shrine_input(code, game),
        GameState::Hatchery => handle_hatchery_input(code, game),
        GameState::SpacePort => handle_space_port_input(code, game),
        GameState::FactionBase => handle_faction_base_input(code, game),
        GameState::MagicAcademy => handle_magic_academy_input(code, game),
        GameState::QuestLog => handle_quest_log_input(code, game),
        GameState::Bestiary => handle_bestiary_input(code, game),
        GameState::Tavern => handle_tavern_input(code, game),
        GameState::BlackMarket => handle_black_market_input(code, game),
        GameState::Bank => handle_bank_input(code, game),
        GameState::AuctionHouse => handle_auction_house_input(code, game),
        GameState::Gacha => handle_gacha_input(code, game),
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

#[expect(clippy::too_many_lines, reason = "Game menu requires handling multiple options")]
fn handle_menu_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') => {
            game.previous_state = Some(GameState::Menu);
            game.state = GameState::ConfirmQuit;
        },
        KeyCode::Char(' ') | KeyCode::Enter => match game.menu_selection {
            0 => {
                game.mode = game::GameMode::SinglePlayer;
                game.reset();
            },
            1 => {
                game.mode = game::GameMode::DailyChallenge;
                game.reset();
            },
            2 => {
                game.mode = game::GameMode::WeeklyChallenge;
                game.reset();
            },
            3 => {
                game.mode = game::GameMode::MonthlyChallenge;
                game.reset();
            },
            4 => {
                game.mode = game::GameMode::YearlyChallenge;
                game.reset();
            },
            5 => {
                game.mode = game::GameMode::DecadeChallenge;
                game.reset();
            },
            6 => {
                game.mode = game::GameMode::CenturyChallenge;
                game.reset();
            },
            7 => {
                game.mode = game::GameMode::MillenniumChallenge;
                game.reset();
            },
            8 => {
                game.mode = game::GameMode::Campaign;
                game.reset();
            },
            9 => {
                game.mode = game::GameMode::LocalMultiplayer;
                game.reset();
            },
            10 => {
                game.mode = game::GameMode::OnlineMultiplayer;
                game.reset();
            },
            11 => {
                game.mode = game::GameMode::Tournament;
                game.reset();
            },
            12 => {
                game.mode = game::GameMode::PlayerVsBot;
                game.reset();
            },
            13 => {
                game.mode = game::GameMode::BotVsBot;
                game.reset();
            },
            14 => {
                game.mode = game::GameMode::BattleRoyale;
                game.reset();
            },
            15 => {
                game.mode = game::GameMode::TimeAttack;
                game.reset();
            },
            16 => {
                game.mode = game::GameMode::Survival;
                game.reset();
            },
            17 => {
                game.mode = game::GameMode::Zen;
                game.reset();
            },
            18 => {
                game.mode = game::GameMode::Maze;
                game.reset();
            },
            19 => {
                game.mode = game::GameMode::Cave;
                game.reset();
            },
            20 => {
                game.mode = game::GameMode::Dungeon;
                game.reset();
            },
            21 => {
                game.mode = game::GameMode::Speedrun;
                game.reset();
            },
            22 => {
                game.mode = game::GameMode::FogOfWar;
                game.reset();
            },
            23 => {
                game.mode = game::GameMode::Evolution;
                game.reset();
            },
            24 => {
                game.mode = game::GameMode::BossRush;
                game.reset();
            },
            25 => {
                game.mode = game::GameMode::MassiveMultiplayer;
                game.reset();
            },
            26 => {
                game.mode = game::GameMode::Mirror;
                game.reset();
            },
            27 => {
                game.mode = game::GameMode::Flood;
                game.reset();
            },
            28 => {
                game.mode = game::GameMode::Vampire;
                game.reset();
            },
            29 => {
                game.mode = game::GameMode::Gravity;
                game.reset();
            },
            30 => {
                game.mode = game::GameMode::Tron;
                game.reset();
            },
            31 => {
                game.mode = game::GameMode::Zombie;
                game.reset();
            },
            32 => {
                game.mode = game::GameMode::Farmstead;
                game.reset();
            },
            33 => {
                game.mode = game::GameMode::PacMan;
                game.reset();
            },
            34 => {
                game.mode = game::GameMode::CaptureTheFlag;
                game.reset();
            },
            35 => {
                game.mode = game::GameMode::BulletHell;
                game.reset();
            },
            36 => {
                game.mode = game::GameMode::SnakeSurvivor;
                game.reset();
            },
            37 => {
                game.mode = game::GameMode::KingOfTheHill;
                game.reset();
            },
            38 => {
                game.mode = game::GameMode::Dodgeball;
                game.reset();
            },
            39 => {
                game.mode = game::GameMode::DungeonCrawler;
                game.reset();
            },

            40 => {
                game.mode = game::GameMode::Chaos;
                game.reset();
            },

            41 => {
                let _ = game.load_game();
            },
            42 => game.state = GameState::Settings,
            43 => game.state = GameState::NftShop,
            44 => game.state = GameState::SkillTree,
            45 => game.state = GameState::Stats,
            46 => game.state = GameState::Achievements,
            47 => game.state = GameState::Help,
            48 => {
                game.mode = game::GameMode::CustomLevel;
                game.reset();
            },
            49 => {
                game.state = GameState::LevelEditor;
                game.editor_cursor = Some(snake::Point {
                    x: game.width / 2,
                    y: game.height / 2,
                });
                game.obstacles.clear();
            },
            50 => {
                game.state = GameState::Crafting;
                game.settings_selection = 0; // Reusing selection variable
            },
            51 => {
                game.state = GameState::BountyBoard;
                game.settings_selection = 0;
            },
            52 => {
                game.state = GameState::CompanionCamp;
                game.settings_selection = 0;
            },
            53 => {
                game.state = GameState::ClassSelect;
                game.settings_selection = 0;
            },
            54 => {
                game.state = GameState::Equipment;
                game.settings_selection = 0;
            },
            55 => {
                game.state = GameState::Casino;
                game.settings_selection = 0;
            },
            56 => {
                game.state = GameState::StockMarket;
                game.settings_selection = 0;
            },
            57 => {
                game.state = GameState::RealEstate;
                game.settings_selection = 0;
            },
            58 => {
                game.state = GameState::VehicleGarage;
                game.settings_selection = 0;
            },
            59 => {
                game.state = GameState::Fishing;
                game.settings_selection = 0;
                game.is_fishing = false;
                game.fishing_progress = 0;
            },
            60 => {
                game.state = GameState::BattlePass;
                game.settings_selection = 0;
            },
            61 => {
                game.state = GameState::ArtifactShrine;
            },
            62 => {
                game.state = GameState::Hatchery;
                game.settings_selection = 0;
            },
            63 => {
                game.state = GameState::SpacePort;
                game.settings_selection = 0;
            },
            64 => {
                game.state = GameState::FactionBase;
                game.settings_selection = 0;
            },
            65 => {
                game.state = GameState::MagicAcademy;
                game.settings_selection = 0;
            },
            66 => {
                game.state = GameState::QuestLog;
            },
            67 => {
                game.state = GameState::Bestiary;
                game.settings_selection = 0;
            },
            68 => {
                game.state = GameState::Tavern;
                game.settings_selection = 0;
            },
            69 => {
                game.state = GameState::BlackMarket;
                game.settings_selection = 0;
            },
            70 => {
                game.state = GameState::Bank;
                game.settings_selection = 0;
            },
            71 => {
                game.state = GameState::AuctionHouse;
                game.settings_selection = 0;
            },
            72 => {
                game.state = GameState::Gacha;
                game.settings_selection = 0;
            },
            73 => {
                game.previous_state = Some(GameState::Menu);
                game.state = GameState::ConfirmQuit;
            },
            _ => {},
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.menu_selection > 0 {
                game.menu_selection -= 1;
            } else {
                game.menu_selection = 73;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.menu_selection < 73 {
                game.menu_selection += 1;
            } else {
                game.menu_selection = 0;
            }
        },
        _ => {},
    }
    true
}

fn handle_magic_academy_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            let total_options = 5;
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = total_options - 1;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            let total_options = 5;
            if game.settings_selection < total_options - 1 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            let spells = [
                crate::game::SpellType::Heal,
                crate::game::SpellType::Blink,
                crate::game::SpellType::Fireball,
                crate::game::SpellType::Shield,
            ];

            if game.settings_selection < 4 {
                let spell = spells[game.settings_selection];
                if game.stats.unlocked_spells.contains(&spell) {
                    game.stats.equipped_spell = Some(spell);
                } else if game.stats.coins >= 1000 {
                    game.stats.coins -= 1000;
                    game.stats.unlocked_spells.push(spell);
                    game.stats.equipped_spell = Some(spell);
                }
            } else if game.settings_selection == 4 {
                game.stats.equipped_spell = None;
            }
            game.save_stats();
        },
        _ => {},
    }
    true
}

fn handle_space_port_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 3;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 3 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            let selected_planet = match game.settings_selection {
                0 => crate::game::Planet::Earth,
                1 => crate::game::Planet::Moon,
                2 => crate::game::Planet::Mars,
                _ => crate::game::Planet::Jupiter,
            };

            if game.stats.unlocked_planets.contains(&selected_planet) {
                game.current_planet = selected_planet;
                crate::game::beep();
            } else if game.stats.coins >= 50 {
                game.stats.coins -= 50;
                game.stats.unlocked_planets.push(selected_planet);
                game.current_planet = selected_planet;
                crate::game::beep();
            }
        },
        _ => {},
    }
    true
}

fn handle_hatchery_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 2;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 2 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            if game.stats.incubator.is_none() {
                let egg_type = match game.settings_selection {
                    0 => crate::game::EggType::Common,
                    1 => crate::game::EggType::Rare,
                    _ => crate::game::EggType::Legendary,
                };

                let count = game.stats.inventory_eggs.get(&egg_type).copied().unwrap_or(0);
                if count > 0 {
                    *game.stats.inventory_eggs.get_mut(&egg_type).unwrap() -= 1;
                    let timer = match egg_type {
                        crate::game::EggType::Common => 100,
                        crate::game::EggType::Rare => 250,
                        crate::game::EggType::Legendary => 500,
                    };
                    game.stats.incubator = Some((egg_type, timer));
                    game.save_stats();
                    crate::game::beep();
                }
            }
        },
        _ => {},
    }
    true
}

fn handle_bounty_board_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.stats.active_bounty.is_none() {
                if game.settings_selection > 0 {
                    game.settings_selection -= 1;
                } else {
                    game.settings_selection = 2;
                }
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.stats.active_bounty.is_none() {
                if game.settings_selection < 2 {
                    game.settings_selection += 1;
                } else {
                    game.settings_selection = 0;
                }
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            if game.stats.active_bounty.is_some() {
                game.stats.active_bounty = None;
            } else {
                let bounty = match game.settings_selection {
                    1 => crate::game::Bounty::new(crate::game::BountyType::KillBosses(3), 1000),
                    2 => crate::game::Bounty::new(crate::game::BountyType::SurviveTime(120), 750),
                    _ => crate::game::Bounty::new(crate::game::BountyType::EatFood(50), 500),
                };
                game.stats.active_bounty = Some(bounty);
            }
            game.save_stats();
            crate::game::beep();
        },
        _ => {},
    }
    true
}

fn handle_crafting_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 3;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 3 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => match game.settings_selection {
            0 => {
                // Speed Potion: 3 Wood
                let wood =
                    game.stats.inventory.get(&crate::game::Resource::Wood).copied().unwrap_or(0);
                if wood >= 3 {
                    *game.stats.inventory.entry(crate::game::Resource::Wood).or_insert(0) -= 3;
                    *game
                        .stats
                        .crafted_items
                        .entry(crate::game::CraftableItem::SpeedPotion)
                        .or_insert(0) += 1;
                    game.save_stats();
                    crate::game::beep();
                }
            },
            1 => {
                // Iron Wall: 3 Iron
                let iron =
                    game.stats.inventory.get(&crate::game::Resource::Iron).copied().unwrap_or(0);
                if iron >= 3 {
                    *game.stats.inventory.entry(crate::game::Resource::Iron).or_insert(0) -= 3;
                    *game
                        .stats
                        .crafted_items
                        .entry(crate::game::CraftableItem::IronWall)
                        .or_insert(0) += 1;
                    game.save_stats();
                    crate::game::beep();
                }
            },
            2 => {
                // Golden Apple: 5 Gold
                let gold =
                    game.stats.inventory.get(&crate::game::Resource::Gold).copied().unwrap_or(0);
                if gold >= 5 {
                    *game.stats.inventory.entry(crate::game::Resource::Gold).or_insert(0) -= 5;
                    *game
                        .stats
                        .crafted_items
                        .entry(crate::game::CraftableItem::GoldenApple)
                        .or_insert(0) += 1;
                    game.save_stats();
                    crate::game::beep();
                }
            },
            3 => {
                // Diamond Sword: 1 Diamond
                let diamond =
                    game.stats.inventory.get(&crate::game::Resource::Diamond).copied().unwrap_or(0);
                if diamond >= 1 {
                    *game.stats.inventory.entry(crate::game::Resource::Diamond).or_insert(0) -= 1;
                    *game
                        .stats
                        .crafted_items
                        .entry(crate::game::CraftableItem::DiamondSword)
                        .or_insert(0) += 1;
                    game.save_stats();
                    crate::game::beep();
                }
            },
            _ => {},
        },
        _ => {},
    }
    true
}

fn handle_level_editor_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc => {
            game.save_custom_level();
            game.state = GameState::Menu;
        },
        KeyCode::Char('w' | 'W') | KeyCode::Up => {
            if let Some(cursor) = &mut game.editor_cursor
                && cursor.y > 1
            {
                cursor.y -= 1;
            }
        },
        KeyCode::Char('s' | 'S') | KeyCode::Down => {
            if let Some(cursor) = &mut game.editor_cursor
                && cursor.y < game.height - 2
            {
                cursor.y += 1;
            }
        },
        KeyCode::Char('a' | 'A') | KeyCode::Left => {
            if game.editor_cursor.as_ref().is_some_and(|c| c.x > 1) {
                game.editor_cursor.as_mut().unwrap().x -= 1;
            }
        },
        KeyCode::Char('d' | 'D') | KeyCode::Right => {
            if game.editor_cursor.as_ref().is_some_and(|c| c.x < game.width - 2) {
                game.editor_cursor.as_mut().unwrap().x += 1;
            }
        },
        KeyCode::Char(' ') => {
            if let Some(cursor) = game.editor_cursor {
                if game.obstacles.contains(&cursor) {
                    game.obstacles.remove(&cursor);
                } else {
                    game.obstacles.insert(cursor);
                }
            }
        },
        _ => {},
    }
    true
}

#[expect(clippy::too_many_lines, reason = "Game menu requires handling multiple inputs")]
fn handle_playing_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') => {
            game.previous_state = Some(GameState::Playing);
            game.state = GameState::ConfirmQuit;
        },
        KeyCode::Char('p' | 'P') => game.state = GameState::Paused,
        KeyCode::Char('f' | 'F') => {
            game.is_sprinting = !game.is_sprinting;
        },
        KeyCode::Char('t' | 'T') => {
            game.auto_pilot = !game.auto_pilot;
            if game.auto_pilot {
                game.used_bot_this_session = true;
            }
        },
        KeyCode::Char('w' | 'W') => {
            if (game.mode == game::GameMode::Mirror) ^ game.is_reverse_active() {
                game.handle_input(Direction::Down, 1);
            } else {
                game.handle_input(Direction::Up, 1);
            }
        },
        KeyCode::Char('s' | 'S') => {
            if (game.mode == game::GameMode::Mirror) ^ game.is_reverse_active() {
                game.handle_input(Direction::Up, 1);
            } else {
                game.handle_input(Direction::Down, 1);
            }
        },
        KeyCode::Char('a' | 'A') => {
            if (game.mode == game::GameMode::Mirror) ^ game.is_reverse_active() {
                game.handle_input(Direction::Right, 1);
            } else {
                game.handle_input(Direction::Left, 1);
            }
        },
        KeyCode::Char('d' | 'D') => {
            if (game.mode == game::GameMode::Mirror) ^ game.is_reverse_active() {
                game.handle_input(Direction::Left, 1);
            } else {
                game.handle_input(Direction::Right, 1);
            }
        },
        KeyCode::Up => {
            if game.mode == game::GameMode::SinglePlayer
                || game.mode == game::GameMode::Mirror
                || game.mode == game::GameMode::TimeAttack
                || game.mode == game::GameMode::Speedrun
                || game.mode == game::GameMode::DailyChallenge
                || game.mode == game::GameMode::WeeklyChallenge
                || game.mode == game::GameMode::MonthlyChallenge
                || game.mode == game::GameMode::YearlyChallenge
                || game.mode == game::GameMode::DecadeChallenge
                || game.mode == game::GameMode::FogOfWar
                || game.mode == game::GameMode::Evolution
                || game.mode == game::GameMode::BossRush
                || game.mode == game::GameMode::MassiveMultiplayer
                || game.mode == game::GameMode::Flood
                || game.mode == game::GameMode::Gravity
            {
                if (game.mode == game::GameMode::Mirror) ^ game.is_reverse_active() {
                    game.handle_input(Direction::Down, 1);
                } else {
                    game.handle_input(Direction::Up, 1);
                }
            } else {
                if game.is_reverse_active() {
                    game.handle_input(Direction::Down, 2);
                } else {
                    game.handle_input(Direction::Up, 2);
                }
            }
        },
        KeyCode::Down => {
            if game.mode == game::GameMode::SinglePlayer
                || game.mode == game::GameMode::Mirror
                || game.mode == game::GameMode::TimeAttack
                || game.mode == game::GameMode::Speedrun
                || game.mode == game::GameMode::DailyChallenge
                || game.mode == game::GameMode::WeeklyChallenge
                || game.mode == game::GameMode::MonthlyChallenge
                || game.mode == game::GameMode::YearlyChallenge
                || game.mode == game::GameMode::DecadeChallenge
                || game.mode == game::GameMode::FogOfWar
                || game.mode == game::GameMode::Evolution
                || game.mode == game::GameMode::BossRush
                || game.mode == game::GameMode::MassiveMultiplayer
                || game.mode == game::GameMode::Flood
                || game.mode == game::GameMode::Gravity
            {
                if (game.mode == game::GameMode::Mirror) ^ game.is_reverse_active() {
                    game.handle_input(Direction::Up, 1);
                } else {
                    game.handle_input(Direction::Down, 1);
                }
            } else {
                if game.is_reverse_active() {
                    game.handle_input(Direction::Up, 2);
                } else {
                    game.handle_input(Direction::Down, 2);
                }
            }
        },
        KeyCode::Left => {
            if game.mode == game::GameMode::SinglePlayer
                || game.mode == game::GameMode::Mirror
                || game.mode == game::GameMode::TimeAttack
                || game.mode == game::GameMode::Speedrun
                || game.mode == game::GameMode::DailyChallenge
                || game.mode == game::GameMode::WeeklyChallenge
                || game.mode == game::GameMode::MonthlyChallenge
                || game.mode == game::GameMode::YearlyChallenge
                || game.mode == game::GameMode::DecadeChallenge
                || game.mode == game::GameMode::FogOfWar
                || game.mode == game::GameMode::Evolution
                || game.mode == game::GameMode::BossRush
                || game.mode == game::GameMode::MassiveMultiplayer
                || game.mode == game::GameMode::Flood
                || game.mode == game::GameMode::Gravity
            {
                if (game.mode == game::GameMode::Mirror) ^ game.is_reverse_active() {
                    game.handle_input(Direction::Right, 1);
                } else {
                    game.handle_input(Direction::Left, 1);
                }
            } else {
                if game.is_reverse_active() {
                    game.handle_input(Direction::Right, 2);
                } else {
                    game.handle_input(Direction::Left, 2);
                }
            }
        },
        KeyCode::Right => {
            if game.mode == game::GameMode::SinglePlayer
                || game.mode == game::GameMode::Mirror
                || game.mode == game::GameMode::TimeAttack
                || game.mode == game::GameMode::Speedrun
                || game.mode == game::GameMode::DailyChallenge
                || game.mode == game::GameMode::WeeklyChallenge
                || game.mode == game::GameMode::MonthlyChallenge
                || game.mode == game::GameMode::YearlyChallenge
                || game.mode == game::GameMode::DecadeChallenge
                || game.mode == game::GameMode::FogOfWar
                || game.mode == game::GameMode::Evolution
                || game.mode == game::GameMode::BossRush
                || game.mode == game::GameMode::MassiveMultiplayer
                || game.mode == game::GameMode::Flood
                || game.mode == game::GameMode::Gravity
            {
                if (game.mode == game::GameMode::Mirror) ^ game.is_reverse_active() {
                    game.handle_input(Direction::Left, 1);
                } else {
                    game.handle_input(Direction::Right, 1);
                }
            } else {
                if game.is_reverse_active() {
                    game.handle_input(Direction::Left, 2);
                } else {
                    game.handle_input(Direction::Right, 2);
                }
            }
        },
        KeyCode::Char('z' | 'Z') => {
            game.rewind_time();
        },
        KeyCode::Char(' ') => game.shoot_laser(1),
        KeyCode::Enter => {
            if game.mode == game::GameMode::LocalMultiplayer
                || game.mode == game::GameMode::PlayerVsBot
            {
                game.shoot_laser(2);
            }
        },
        KeyCode::Char('1') => {
            let count = game
                .stats
                .crafted_items
                .get(&crate::game::CraftableItem::SpeedPotion)
                .copied()
                .unwrap_or(0);
            if count > 0 {
                *game
                    .stats
                    .crafted_items
                    .get_mut(&crate::game::CraftableItem::SpeedPotion)
                    .unwrap() -= 1;
                game.power_up = Some(crate::game::PowerUp {
                    p_type: crate::game::PowerUpType::SpeedBoost,
                    location: game.snake.head(),
                    activation_time: Some(
                        web_time::SystemTime::now()
                            .duration_since(web_time::SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    ),
                });
                crate::game::beep();
            }
        },
        KeyCode::Char('2') => {
            let count = game
                .stats
                .crafted_items
                .get(&crate::game::CraftableItem::IronWall)
                .copied()
                .unwrap_or(0);
            if count > 0 {
                *game
                    .stats
                    .crafted_items
                    .get_mut(&crate::game::CraftableItem::IronWall)
                    .unwrap() -= 1;
                if let Some(tail) = game.snake.body.back() {
                    let mut p = *tail;
                    // Put it slightly behind
                    match game.snake.direction {
                        crate::snake::Direction::Up => p.y = p.y.saturating_add(1),
                        crate::snake::Direction::Down => p.y = p.y.saturating_sub(1),
                        crate::snake::Direction::Left => p.x = p.x.saturating_add(1),
                        crate::snake::Direction::Right => p.x = p.x.saturating_sub(1),
                    }
                    if p.x > 0 && p.x < game.width - 1 && p.y > 0 && p.y < game.height - 1 {
                        game.obstacles.insert(p);
                    }
                }
                crate::game::beep();
            }
        },
        KeyCode::Char('3') => {
            let count = game
                .stats
                .crafted_items
                .get(&crate::game::CraftableItem::GoldenApple)
                .copied()
                .unwrap_or(0);
            if count > 0 {
                *game
                    .stats
                    .crafted_items
                    .get_mut(&crate::game::CraftableItem::GoldenApple)
                    .unwrap() -= 1;
                game.lives += 1;
                crate::game::beep();
            }
        },
        KeyCode::Char('4') => {
            let count = game
                .stats
                .crafted_items
                .get(&crate::game::CraftableItem::DiamondSword)
                .copied()
                .unwrap_or(0);
            if count > 0 {
                *game
                    .stats
                    .crafted_items
                    .get_mut(&crate::game::CraftableItem::DiamondSword)
                    .unwrap() -= 1;
                if !game.bosses.is_empty() {
                    // Deal massive damage to the first boss
                    game.bosses[0].health = game.bosses[0].health.saturating_sub(10);
                    crate::game::beep();
                }
            }
        },
        KeyCode::Char('e' | 'E') => {
            if let Some(spell) = game.stats.equipped_spell {
                let cost = spell.cost();
                if game.mana >= cost {
                    game.mana -= cost;
                    game.cast_spell(spell);
                }
            }
        },
        KeyCode::Char('5') => {
            if game.mode == game::game_mode::GameMode::Farmstead && game.stats.coins >= 10 {
                let head = game.snake.head();
                // Plant behind the head
                let p = crate::game::Game::calculate_next_head_dir(
                    head,
                    match game.snake.direction {
                        crate::snake::Direction::Up => crate::snake::Direction::Down,
                        crate::snake::Direction::Down => crate::snake::Direction::Up,
                        crate::snake::Direction::Left => crate::snake::Direction::Right,
                        crate::snake::Direction::Right => crate::snake::Direction::Left,
                    },
                );
                if p.x > 0
                    && p.x < game.width - 1
                    && p.y > 0
                    && p.y < game.height - 1
                    && !game.obstacles.contains(&p)
                    && !game.crops.iter().any(|c| c.position == p)
                {
                    game.stats.coins -= 10;
                    game.crops.push(crate::game::Crop {
                        position: p,
                        growth_stage: 0,
                        timer: 0,
                    });
                    crate::game::beep();
                }
            }
        },
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

fn handle_level_up_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.level_up_selection > 0 {
                game.level_up_selection -= 1;
            } else if !game.level_up_options.is_empty() {
                game.level_up_selection = game.level_up_options.len() - 1;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if !game.level_up_options.is_empty() {
                if game.level_up_selection < game.level_up_options.len() - 1 {
                    game.level_up_selection += 1;
                } else {
                    game.level_up_selection = 0;
                }
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            if !game.level_up_options.is_empty()
                && game.level_up_selection < game.level_up_options.len()
            {
                let chosen_upgrade = game.level_up_options[game.level_up_selection];
                *game.in_game_upgrades.entry(chosen_upgrade).or_insert(0) += 1;
                if chosen_upgrade == crate::game::InGameUpgrade::Turret {
                    game.spawn_turret();
                }
            }
            game.state = GameState::Playing;
        },
        _ => {},
    }
    true
}

fn handle_skill_tree_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.skill_tree_selection > 0 {
                game.skill_tree_selection -= 1;
            } else {
                game.skill_tree_selection = 3;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.skill_tree_selection < 3 {
                game.skill_tree_selection += 1;
            } else {
                game.skill_tree_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => match game.skill_tree_selection {
            0 => {
                let cost = 500 * (1 + u32::from(game.stats.upgrade_powerup_duration));
                if game.stats.upgrade_powerup_duration < 10 && game.stats.coins >= cost {
                    game.stats.coins -= cost;
                    game.stats.upgrade_powerup_duration += 1;
                    game.save_stats();
                    crate::game::beep();
                }
            },
            1 => {
                let cost = 1000 * (1 + u32::from(game.stats.upgrade_extra_lives));
                if game.stats.upgrade_extra_lives < 10 && game.stats.coins >= cost {
                    game.stats.coins -= cost;
                    game.stats.upgrade_extra_lives += 1;
                    game.save_stats();
                    crate::game::beep();
                }
            },
            2 => {
                let cost = 1500 * (1 + u32::from(game.stats.upgrade_laser_capacity));
                if game.stats.upgrade_laser_capacity < 10 && game.stats.coins >= cost {
                    game.stats.coins -= cost;
                    game.stats.upgrade_laser_capacity += 1;
                    game.save_stats();
                    crate::game::beep();
                }
            },
            3 => {
                let cost = 2000 * (1 + u32::from(game.stats.upgrade_coin_multiplier));
                if game.stats.upgrade_coin_multiplier < 10 && game.stats.coins >= cost {
                    game.stats.coins -= cost;
                    game.stats.upgrade_coin_multiplier += 1;
                    game.save_stats();
                    crate::game::beep();
                }
            },
            _ => {},
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

fn handle_merchant_shop_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Playing;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 3;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 3 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => match game.settings_selection {
            0 => {
                // Extra Life [Cost: 500]
                if game.stats.coins >= 500 {
                    game.stats.coins -= 500;
                    game.lives += 1;
                    game.save_stats();
                    crate::game::beep();
                }
            },
            1 => {
                // Diamond Sword [Cost: 1000]
                if game.stats.coins >= 1000 {
                    game.stats.coins -= 1000;
                    *game
                        .stats
                        .crafted_items
                        .entry(crate::game::CraftableItem::DiamondSword)
                        .or_insert(0) += 1;
                    game.save_stats();
                    crate::game::beep();
                }
            },
            2 => {
                // Speed Potion [Cost: 300]
                if game.stats.coins >= 300 {
                    game.stats.coins -= 300;
                    *game
                        .stats
                        .crafted_items
                        .entry(crate::game::CraftableItem::SpeedPotion)
                        .or_insert(0) += 1;
                    game.save_stats();
                    crate::game::beep();
                }
            },
            3 => {
                // Iron Wall [Cost: 100]
                if game.stats.coins >= 100 {
                    game.stats.coins -= 100;
                    *game
                        .stats
                        .crafted_items
                        .entry(crate::game::CraftableItem::IronWall)
                        .or_insert(0) += 1;
                    game.save_stats();
                    crate::game::beep();
                }
            },
            _ => {},
        },
        _ => {},
    }
    true
}

fn handle_companion_camp_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 2;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 2 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            let companion_type = match game.settings_selection {
                0 => crate::game::CompanionType::Collector,
                1 => crate::game::CompanionType::Fighter,
                _ => crate::game::CompanionType::Healer,
            };

            let is_unlocked = game.stats.unlocked_companions.contains(&companion_type);

            if is_unlocked {
                // Equip or unequip
                if game.stats.equipped_companion == Some(companion_type) {
                    game.stats.equipped_companion = None;
                } else {
                    game.stats.equipped_companion = Some(companion_type);
                }
                game.save_stats();
                crate::game::beep();
            } else {
                // Try to buy
                let cost = 1000;
                if game.stats.coins >= cost {
                    game.stats.coins -= cost;
                    game.stats.unlocked_companions.push(companion_type);
                    game.stats.equipped_companion = Some(companion_type);
                    game.save_stats();
                    crate::game::beep();
                }
            }
        },
        _ => {},
    }
    true
}

fn handle_class_select_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 5;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 5 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => match game.settings_selection {
            0 => {
                if game.stats.unlocked_classes.contains(&crate::game::HeroClass::Warrior) {
                    game.stats.equipped_class = Some(crate::game::HeroClass::Warrior);
                } else if game.stats.coins >= 500 {
                    game.stats.coins -= 500;
                    game.stats.unlocked_classes.push(crate::game::HeroClass::Warrior);
                    game.stats.equipped_class = Some(crate::game::HeroClass::Warrior);
                }
            },
            1 => {
                if game.stats.unlocked_classes.contains(&crate::game::HeroClass::Mage) {
                    game.stats.equipped_class = Some(crate::game::HeroClass::Mage);
                } else if game.stats.coins >= 500 {
                    game.stats.coins -= 500;
                    game.stats.unlocked_classes.push(crate::game::HeroClass::Mage);
                    game.stats.equipped_class = Some(crate::game::HeroClass::Mage);
                }
            },
            2 => {
                if game.stats.unlocked_classes.contains(&crate::game::HeroClass::Rogue) {
                    game.stats.equipped_class = Some(crate::game::HeroClass::Rogue);
                } else if game.stats.coins >= 500 {
                    game.stats.coins -= 500;
                    game.stats.unlocked_classes.push(crate::game::HeroClass::Rogue);
                    game.stats.equipped_class = Some(crate::game::HeroClass::Rogue);
                }
            },
            3 => {
                if game.stats.unlocked_classes.contains(&crate::game::HeroClass::Paladin) {
                    game.stats.equipped_class = Some(crate::game::HeroClass::Paladin);
                } else if game.stats.coins >= 500 {
                    game.stats.coins -= 500;
                    game.stats.unlocked_classes.push(crate::game::HeroClass::Paladin);
                    game.stats.equipped_class = Some(crate::game::HeroClass::Paladin);
                }
            },
            4 => {
                if game.stats.unlocked_classes.contains(&crate::game::HeroClass::Necromancer) {
                    game.stats.equipped_class = Some(crate::game::HeroClass::Necromancer);
                } else if game.stats.coins >= 500 {
                    game.stats.coins -= 500;
                    game.stats.unlocked_classes.push(crate::game::HeroClass::Necromancer);
                    game.stats.equipped_class = Some(crate::game::HeroClass::Necromancer);
                }
            },
            5 => {
                game.stats.equipped_class = None;
            },
            _ => {},
        },
        _ => {},
    }
    game.save_stats();
    true
}

fn handle_stock_market_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 3;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 3 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ' | 'b' | 'B') => {
            let amount = if matches!(code, KeyCode::Char('b' | 'B')) {
                10
            } else {
                1
            };
            let stock = match game.settings_selection {
                0 => crate::game::Stock::SnakeCorp,
                1 => crate::game::Stock::GoblinInc,
                2 => crate::game::Stock::BossDynamics,
                3 => crate::game::Stock::LaserTech,
                _ => return true,
            };

            let price = game.stats.stock_prices.get(&stock).copied().unwrap_or(100);
            let cost = price * amount;

            if game.stats.coins >= cost {
                game.stats.coins -= cost;
                *game.stats.portfolio.entry(stock).or_insert(0) += amount;
                crate::game::beep();
                game.save_stats();
            }
        },
        KeyCode::Char('d' | 'D' | 'l' | 'L') => {
            let amount = if matches!(code, KeyCode::Char('d' | 'D')) {
                10
            } else {
                1
            };
            let stock = match game.settings_selection {
                0 => crate::game::Stock::SnakeCorp,
                1 => crate::game::Stock::GoblinInc,
                2 => crate::game::Stock::BossDynamics,
                3 => crate::game::Stock::LaserTech,
                _ => return true,
            };

            let owned = game.stats.portfolio.get(&stock).copied().unwrap_or(0);
            if owned >= amount {
                let price = game.stats.stock_prices.get(&stock).copied().unwrap_or(100);
                *game.stats.portfolio.entry(stock).or_insert(0) -= amount;
                game.stats.coins += price * amount;
                crate::game::beep();
                game.save_stats();
            }
        },
        _ => {},
    }
    true
}

fn handle_casino_input(code: KeyCode, game: &mut Game) -> bool {
    use rand::Rng;
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 1;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 1 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => match game.settings_selection {
            0 => {
                // Slot Machine
                if game.stats.coins >= 100 {
                    game.stats.coins -= 100;
                    let s1 = game.rng.gen_range(0..5);
                    let s2 = game.rng.gen_range(0..5);
                    let s3 = game.rng.gen_range(0..5);
                    if s1 == s2 && s2 == s3 {
                        game.stats.coins += game.rng.gen_range(500..=2000);
                        crate::game::beep();
                    }
                    game.save_stats();
                }
            },
            1 => {
                // Roulette
                if game.stats.coins >= 50 {
                    game.stats.coins -= 50;
                    if game.rng.gen_bool(0.5) {
                        game.stats.coins += 100;
                        crate::game::beep();
                    }
                    game.save_stats();
                }
            },
            _ => {},
        },
        _ => {},
    }
    true
}

fn handle_equipment_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            let total_items = game.stats.unlocked_equipment.len() + 1; // +1 for "Unequip"
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = total_items.saturating_sub(1);
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            let total_items = game.stats.unlocked_equipment.len() + 1;
            if game.settings_selection < total_items.saturating_sub(1) {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            if game.settings_selection < game.stats.unlocked_equipment.len() {
                game.stats.equipped_gear =
                    Some(game.stats.unlocked_equipment[game.settings_selection]);
            } else {
                game.stats.equipped_gear = None;
            }
            game.save_stats();
        },
        _ => {},
    }
    true
}

fn handle_vehicle_garage_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            let total_items = 5; // 4 vehicles + 1 unequip
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = total_items - 1;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            let total_items = 5;
            if game.settings_selection < total_items - 1 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            let vehicles = [
                crate::game::Vehicle::Bike,
                crate::game::Vehicle::Car,
                crate::game::Vehicle::Tank,
                crate::game::Vehicle::Spaceship,
            ];

            if game.settings_selection < 4 {
                let v = vehicles[game.settings_selection];
                let cost = match v {
                    crate::game::Vehicle::Bike => 1000,
                    crate::game::Vehicle::Car => 2500,
                    crate::game::Vehicle::Tank => 5000,
                    crate::game::Vehicle::Spaceship => 10000,
                };
                if game.stats.unlocked_vehicles.contains(&v) {
                    game.stats.equipped_vehicle = Some(v);
                } else if game.stats.coins >= cost {
                    game.stats.coins -= cost;
                    game.stats.unlocked_vehicles.push(v);
                    game.stats.equipped_vehicle = Some(v);
                }
            } else {
                game.stats.equipped_vehicle = None;
            }
            game.save_stats();
        },
        _ => {},
    }
    true
}

fn handle_real_estate_input(code: KeyCode, game: &mut Game) -> bool {
    let options_count = 5;
    match code {
        KeyCode::Up => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = options_count - 1;
            }
        },
        KeyCode::Down => {
            if game.settings_selection < options_count - 1 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter => {
            let props = [
                crate::game::Property::Shack,
                crate::game::Property::Apartment,
                crate::game::Property::Mansion,
                crate::game::Property::Skyscraper,
            ];
            if game.settings_selection < 4 {
                let prop = props[game.settings_selection];
                let cost = prop.cost();
                if game.stats.coins >= cost {
                    game.stats.coins -= cost;
                    *game.stats.properties.entry(prop).or_insert(0) += 1;
                }
            } else {
                game.state = GameState::Menu;
            }
        },
        KeyCode::Esc => game.state = GameState::Menu,
        _ => {},
    }
    true
}

fn handle_fishing_input(code: KeyCode, game: &mut Game) -> bool {
    use rand::Rng;
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
            game.is_fishing = false;
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            if game.is_fishing {
                game.fishing_progress += 2 + u32::from(game.stats.fishing_rod_level);
                if game.fishing_progress >= 50 {
                    game.is_fishing = false;
                    game.fishing_progress = 0;

                    let rand_val = game.rng.gen_range(0..100);
                    let fish = if rand_val < 50 {
                        crate::game::Fish::Minnow
                    } else if rand_val < 80 {
                        crate::game::Fish::Salmon
                    } else if rand_val < 95 {
                        crate::game::Fish::Tuna
                    } else {
                        crate::game::Fish::Kraken
                    };

                    *game.stats.fish_caught.entry(fish).or_insert(0) += 1;
                    crate::game::beep();
                    game.save_stats();
                }
            } else {
                game.is_fishing = true;
                game.fishing_progress = 0;
            }
        },
        _ => {},
    }
    true
}

fn handle_battle_pass_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 49; // 50 tiers
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 49 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            let tier = u32::try_from(game.settings_selection + 1).unwrap_or(1);
            let required_xp = tier * 1000;
            if game.stats.battle_pass_xp >= required_xp
                && !game.stats.claimed_battle_pass_tiers.contains(&tier)
            {
                // Claim reward
                game.stats.claimed_battle_pass_tiers.push(tier);

                // Determine reward based on tier
                if tier.is_multiple_of(10) {
                    // Big reward (Skin or Theme)
                    if tier == 50 {
                        if !game.stats.unlocked_skins.contains(&'🚀') {
                            game.stats.unlocked_skins.push('🚀');
                        }
                    } else {
                        game.stats.coins += 5000;
                    }
                } else if tier.is_multiple_of(5) {
                    game.stats.coins += 2000;
                } else {
                    game.stats.coins += 500;
                }

                game.save_stats();
                crate::game::beep();
            }
        },
        _ => {},
    }
    true
}

fn handle_artifact_shrine_input(code: KeyCode, game: &mut Game) -> bool {
    use rand::Rng;
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            if game.stats.coins >= 1000 {
                game.stats.coins -= 1000;
                let artifacts = [
                    crate::game::Artifact::CoinAmulet,
                    crate::game::Artifact::LifeChalice,
                    crate::game::Artifact::GhostCloak,
                    crate::game::Artifact::MagnetStone,
                    crate::game::Artifact::TimeCrystal,
                ];
                let idx = game.rng.gen_range(0..artifacts.len());
                let artifact = artifacts[idx];

                if game.stats.unlocked_artifacts.contains(&artifact) {
                    game.stats.coins += 500;
                } else {
                    game.stats.unlocked_artifacts.push(artifact);
                }
                game.save_stats();
                crate::game::beep();
            }
        },
        _ => {},
    }
    true
}

fn handle_faction_base_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.stats.faction.is_none() {
                if game.settings_selection > 0 {
                    game.settings_selection -= 1;
                } else {
                    game.settings_selection = 2;
                }
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.stats.faction.is_none() {
                if game.settings_selection < 2 {
                    game.settings_selection += 1;
                } else {
                    game.settings_selection = 0;
                }
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            if game.stats.faction.is_some() {
                game.stats.faction = None;
                game.stats.faction_rep = 0;
            } else {
                let faction = match game.settings_selection {
                    0 => crate::game::Faction::CrimsonVipers,
                    1 => crate::game::Faction::AzureCobras,
                    _ => crate::game::Faction::EmeraldPythons,
                };
                game.stats.faction = Some(faction);
            }
            game.stats.faction_rep = 0;
            game.save_stats();
            crate::game::beep();
        },
        _ => {},
    }
    true
}

const fn handle_quest_log_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        _ => {},
    }
    true
}

const fn handle_bestiary_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            let total_bosses = 12;
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = total_bosses - 1;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            let total_bosses = 12;
            if game.settings_selection < total_bosses - 1 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        _ => {},
    }
    true
}

fn handle_tavern_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 3;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 3 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            match game.settings_selection {
                0 => {
                    // Talk to Barkeep
                    game.chat_log.push_back((
                        "Barkeep: Welcome to the Tavern, traveler! Stay a while and listen."
                            .to_string(),
                        crate::color::Color::Yellow,
                    ));
                },
                1 => {
                    // Play Dice
                    use rand::Rng;
                    if game.stats.coins >= 100 {
                        game.stats.coins -= 100;
                        let roll1 = game.rng.gen_range(1..=6);
                        let roll2 = game.rng.gen_range(1..=6);
                        let total = roll1 + roll2;
                        if total == 7 || total == 11 {
                            game.stats.coins += 250;
                            game.chat_log.push_back((
                                format!("Barkeep: You rolled {roll1} and {roll2} ({total}). You win 250 coins!"),
                                crate::color::Color::Green,
                            ));
                        } else {
                            game.chat_log.push_back((
                                format!("Barkeep: You rolled {roll1} and {roll2} ({total}). You lose 100 coins."),
                                crate::color::Color::Red,
                            ));
                        }
                        game.save_stats();
                        crate::game::beep();
                    }
                },
                2 => {
                    // Rest
                    if game.stats.coins >= 50 {
                        game.stats.coins -= 50;
                        game.lives += 1;
                        game.save_stats();
                        crate::game::beep();
                    }
                },
                3 => {
                    // Leave
                    game.state = GameState::Menu;
                },
                _ => {},
            }
        },
        _ => {},
    }
    true
}

fn handle_black_market_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 5;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 5 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => {
            match game.settings_selection {
                0 => {
                    // Buy Shadow Cloak
                    if game.stats.coins >= 5000
                        && !game
                            .stats
                            .unlocked_artifacts
                            .contains(&crate::game::Artifact::GhostCloak)
                    {
                        game.stats.coins -= 5000;
                        game.stats.unlocked_artifacts.push(crate::game::Artifact::GhostCloak);
                        crate::game::beep();
                        game.save_stats();
                    }
                },
                1 => {
                    // Buy Hacker Theme
                    if game.stats.coins >= 2000
                        && !game.stats.unlocked_themes.contains(&crate::game::Theme::Hacker)
                    {
                        game.stats.coins -= 2000;
                        game.stats.unlocked_themes.push(crate::game::Theme::Hacker);
                        crate::game::beep();
                        game.save_stats();
                    }
                },
                2 => {
                    // Buy Corrupted Egg
                    if game.stats.coins >= 3000 {
                        game.stats.coins -= 3000;
                        *game
                            .stats
                            .inventory_eggs
                            .entry(crate::game::EggType::Legendary)
                            .or_insert(0) += 1;
                        crate::game::beep();
                        game.save_stats();
                    }
                },
                3 => {
                    // Buy Forbidden Spell
                    if game.stats.coins >= 4000
                        && !game.stats.unlocked_spells.contains(&crate::game::SpellType::Fireball)
                    {
                        game.stats.coins -= 4000;
                        game.stats.unlocked_spells.push(crate::game::SpellType::Fireball);
                        crate::game::beep();
                        game.save_stats();
                    }
                },
                4 => {
                    // Sell Max Mana
                    if game.max_mana > 10 {
                        game.max_mana = game.max_mana.saturating_sub(10);
                        game.mana = std::cmp::min(game.mana, game.max_mana);
                        game.stats.coins += 1000;
                        crate::game::beep();
                        game.save_stats();
                    }
                },
                5 => {
                    // Leave
                    game.state = GameState::Menu;
                },
                _ => {},
            }
        },
        _ => {},
    }
    true
}

fn handle_bank_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 2;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 2 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => match game.settings_selection {
            0 => {
                // Deposit
                if game.stats.coins >= 100 {
                    game.stats.coins -= 100;
                    game.stats.bank_balance += 100;
                    crate::game::beep();
                    game.save_stats();
                }
            },
            1 => {
                // Withdraw
                if game.stats.bank_balance >= 100 {
                    game.stats.bank_balance -= 100;
                    game.stats.coins += 100;
                    crate::game::beep();
                    game.save_stats();
                }
            },
            2 => {
                // Leave
                game.state = GameState::Menu;
            },
            _ => {},
        },
        _ => {},
    }
    true
}

fn handle_auction_house_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 3;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 3 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => match game.settings_selection {
            0 => {
                // Bid on Mystery Artifact
                if game.stats.coins >= 5000
                    && !game.stats.unlocked_artifacts.contains(&crate::game::Artifact::LifeChalice)
                {
                    game.stats.coins -= 5000;
                    game.stats.unlocked_artifacts.push(crate::game::Artifact::LifeChalice);
                    crate::game::beep();
                    game.save_stats();
                }
            },
            1 => {
                // Bid on Rare Theme
                if game.stats.coins >= 2000
                    && !game.stats.unlocked_themes.contains(&crate::game::Theme::Matrix)
                {
                    game.stats.coins -= 2000;
                    game.stats.unlocked_themes.push(crate::game::Theme::Matrix);
                    crate::game::beep();
                    game.save_stats();
                }
            },
            2 => {
                // Bid on Epic Boss Pet
                if game.stats.coins >= 10000
                    && !game
                        .stats
                        .unlocked_companions
                        .contains(&crate::game::CompanionType::Fighter)
                {
                    game.stats.coins -= 10000;
                    game.stats.unlocked_companions.push(crate::game::CompanionType::Fighter);
                    crate::game::beep();
                    game.save_stats();
                }
            },
            3 => {
                // Leave
                game.state = GameState::Menu;
            },
            _ => {},
        },
        _ => {},
    }
    true
}

fn handle_gacha_input(code: KeyCode, game: &mut Game) -> bool {
    match code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc | KeyCode::Backspace => {
            game.state = GameState::Menu;
        },
        KeyCode::Up | KeyCode::Char('w' | 'W') => {
            if game.settings_selection > 0 {
                game.settings_selection -= 1;
            } else {
                game.settings_selection = 2;
            }
        },
        KeyCode::Down | KeyCode::Char('s' | 'S') => {
            if game.settings_selection < 2 {
                game.settings_selection += 1;
            } else {
                game.settings_selection = 0;
            }
        },
        KeyCode::Enter | KeyCode::Char(' ') => match game.settings_selection {
            0 => {
                // 1 Pull for 100
                if game.stats.coins >= 100 {
                    game.stats.coins -= 100;
                    do_gacha_pulls(game, 1);
                    crate::game::beep();
                    game.save_stats();
                } else {
                    game.death_message = "Not enough coins!".to_string();
                }
            },
            1 => {
                // 10 Pulls for 1000
                if game.stats.coins >= 1000 {
                    game.stats.coins -= 1000;
                    do_gacha_pulls(game, 10);
                    crate::game::beep();
                    game.save_stats();
                } else {
                    game.death_message = "Not enough coins!".to_string();
                }
            },
            2 => {
                // Leave
                game.state = GameState::Menu;
                game.death_message = String::new();
            },
            _ => {},
        },
        _ => {},
    }
    true
}

fn do_gacha_pulls(game: &mut Game, pulls: u32) {
    use crate::game::Resource;
    use rand::Rng;

    let mut pulled_items = Vec::new();

    for _ in 0..pulls {
        let roll = game.rng.gen_range(0..100);
        let resource = if roll < 50 {
            Resource::Wood
        } else if roll < 80 {
            Resource::Iron
        } else if roll < 95 {
            Resource::Gold
        } else {
            Resource::Diamond
        };

        *game.stats.inventory.entry(resource).or_insert(0) += 1;
        pulled_items.push(format!("{resource:?}"));
    }

    if pulls == 1 {
        game.death_message = format!("You got: {}", pulled_items[0]);
    } else {
        game.death_message = format!("You got {pulls} items!");
    }
}
