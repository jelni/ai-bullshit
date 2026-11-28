use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn tar_pit_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::TarPit;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_tar_pit_reset_seeds_patches_and_clears_state() {
    let game = tar_pit_game(30, 30);

    assert!(game.tar_warnings.is_empty());
    assert_eq!(game.tar_spreads, 0);
    assert_eq!(game.tar_patches.len(), Game::TAR_PATCHES);
    assert!(
        !game.tar_pits.is_empty(),
        "reset must seed active tar pit patches"
    );
    let spawn = Point {
        x: game.width / 2,
        y: game.height / 2,
    };
    for center in &game.tar_patches {
        assert!(
            game.tar_pits.contains(center),
            "a patch center must be active tar pit after reset"
        );
        assert!(
            !game.snake.body_map.contains_key(center),
            "a patch must not spawn on the snake"
        );
        assert!(center != &game.food, "a patch must not spawn on the food");
        assert!(
            center.x.abs_diff(spawn.x).saturating_add(center.y.abs_diff(spawn.y)) >= 5,
            "a patch must spawn at least 5 cells from the spawn point"
        );
    }
    for cell in &game.tar_pits {
        assert!(
            game.obstacles.contains(cell),
            "every active tar pit cell must be mirrored into obstacles"
        );
    }
}

#[test]
fn test_tar_pit_weather_is_pinned_clear() {
    let mut game = tar_pit_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.auto_pilot = false;
    game.weather = Weather::Rain;

    game.state = GameState::Playing;
    game.tick_counter = 10;
    game.update();

    assert_eq!(
        game.weather,
        Weather::Clear,
        "Tar Pit mode must pin the weather to Clear every tick"
    );
}

#[test]
fn test_tar_pit_telegraphs_then_activates_every_6_ticks() {
    let mut game = tar_pit_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.auto_pilot = false;
    let tar_before = game.tar_pits.len();

    // update() increments tick_counter first, so tick_counter = 2 runs tick 3,
    // which telegraphs the cells that will turn to tar pit on tick 6.
    game.state = GameState::Playing;
    game.tick_counter = 2;
    game.update();
    assert_eq!(game.tick_counter, 3);
    assert!(
        !game.tar_warnings.is_empty(),
        "tick 3 must telegraph loose-tar cells (spread or a new patch)"
    );
    assert_eq!(
        game.tar_pits.len(),
        tar_before,
        "no new tar pit before the activation tick"
    );
    assert_eq!(game.tar_spreads, 0);

    // Tick 6 activates: the warnings are consumed and become tar pit.
    game.state = GameState::Playing;
    game.tick_counter = 5;
    game.update();
    assert_eq!(game.tick_counter, 6);
    assert_eq!(game.tar_spreads, 1);
    assert!(
        game.tar_warnings.is_empty(),
        "the telegraph is consumed by the activation"
    );
    assert!(
        game.tar_pits.len() > tar_before,
        "tick 6 must turn telegraphed cells into tar pit"
    );
    for cell in &game.tar_pits {
        assert!(game.obstacles.contains(cell));
    }
}

#[test]
fn test_tar_pit_fairness_over_many_ticks() {
    let mut game = tar_pit_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 25, y: 25 };
    let lives_before = game.lives;

    // Deterministic: park the snake at (8, 15) heading right before every tick,
    // so each activation must never claim its next cell (9, 15) or its body.
    let next = Point { x: 9, y: 15 };
    for t in 1..=60_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = t - 1;
        game.update();
        if t % 6 == 0 {
            assert!(
                !game.tar_pits.contains(&next),
                "tick {t} claimed the cell the snake was about to enter ({next:?})"
            );
            for seg in &game.snake.body {
                assert!(
                    !game.tar_pits.contains(seg),
                    "tick {t} claimed the snake's body at {seg:?}"
                );
            }
        }
        assert!(
            !game.tar_pits.contains(&game.food),
            "tick {t} must never claim the food"
        );
    }

    assert_eq!(
        game.lives, lives_before,
        "the snake must survive the activations (spreads: {})",
        game.tar_spreads
    );
    assert!(
        game.tar_spreads >= 5,
        "expected several activations over 60 ticks, got {}",
        game.tar_spreads
    );
}

#[test]
fn test_tar_pit_caps_patches_and_cells() {
    let mut game = tar_pit_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(13);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.state = GameState::Playing;
        game.update();
    }

    assert!(
        game.tar_patches.len() <= Game::TAR_MAX_PATCHES,
        "patch centers must stay under the cap, got {}",
        game.tar_patches.len()
    );
    assert!(
        game.tar_pits.len() <= Game::TAR_MAX_CELLS,
        "active tar pit cells must stay under the cap, got {}",
        game.tar_pits.len()
    );
    for cell in &game.tar_pits {
        assert!(game.obstacles.contains(cell));
    }
}

#[test]
fn test_tar_pit_dries_up_at_tick_200() {
    let mut game = tar_pit_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.auto_pilot = true;

    // Activate a few times first.
    for tick in [2_u32, 5, 8, 11] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
    }
    assert!(!game.tar_pits.is_empty());
    assert!(game.tar_spreads >= 1);

    // Tick 200 of the 250-tick cycle dries up all the tar.
    game.state = GameState::Playing;
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.tar_pits.is_empty(),
        "all tar pit must dry up at tick 200"
    );
    assert!(
        game.tar_warnings.is_empty(),
        "all loose-tar warnings must clear at tick 200"
    );
    assert!(
        game.tar_patches.is_empty(),
        "all patch centers must clear at tick 200"
    );
    for obstacle in game.obstacles.iter() {
        assert!(
            !game.tar_pits.contains(obstacle),
            "no obstacle may remain recorded as tar pit"
        );
    }
    assert!(
        game.tar_spreads >= 1,
        "the spread counter keeps counting through a drying"
    );
}

#[test]
fn test_tar_pit_eating_food_stomps_adjacent_tar() {
    let mut game = tar_pit_game(20, 20);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.tar_pits.clear();
    game.tar_patches.clear();
    game.tar_warnings.clear();
    let tar = Point { x: 7, y: 5 };
    game.tar_pits.insert(tar);
    game.obstacles.insert(tar);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.auto_pilot = false;

    // Tick 5 is not a multiple of 6 and not 200, so no activation interferes here.
    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.tar_pits.contains(&tar),
        "Eating food in Tar Pit mode should stomp the adjacent tar"
    );
    assert!(
        !game.obstacles.contains(&tar),
        "The stomped tar must also leave obstacles"
    );
}

#[test]
fn test_tar_pit_bot_survives_the_tar() {
    let mut game = tar_pit_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;

    for _ in 0..150 {
        game.update();
        if game.state != GameState::Playing {
            if game.lives == 0 {
                break;
            }
            // Non-death interruption (e.g. merchant visit): resume playing
            game.state = GameState::Playing;
        }
    }

    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "Autopilot should survive the tar pit (lives: {}, spreads: {}, score: {})",
        game.lives,
        game.tar_spreads,
        game.score
    );
    assert!(
        game.tar_spreads >= 5,
        "expected several activations over 150 ticks, got {}",
        game.tar_spreads
    );
}

#[test]
fn test_tar_pit_persists_through_history_rewind() {
    let mut game = tar_pit_game(30, 30);
    game.tar_patches = vec![Point { x: 10, y: 10 }];
    game.tar_pits.insert(Point { x: 10, y: 11 });
    game.tar_warnings.insert(Point { x: 11, y: 10 });
    game.tar_spreads = 7;
    game.save_history_state();
    game.tar_pits.clear();
    game.tar_patches.clear();
    game.tar_warnings.clear();
    game.tar_spreads = 0;

    game.rewind_time();

    assert_eq!(game.tar_patches, vec![Point { x: 10, y: 10 }]);
    assert!(game.tar_pits.contains(&Point { x: 10, y: 11 }));
    assert!(game.tar_warnings.contains(&Point { x: 11, y: 10 }));
    assert_eq!(
        game.tar_spreads, 7,
        "tar_spreads must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_tar_pit_persists_in_save_file() {
    let mut game = tar_pit_game(30, 30);
    game.tar_spreads = 9;
    game.tar_patches = vec![Point { x: 12, y: 9 }];
    game.tar_pits.insert(Point { x: 12, y: 10 });
    game.tar_warnings.insert(Point { x: 11, y: 9 });
    let path = std::env::temp_dir().join(format!(
        "tar_pit_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"tar_spreads\":9"),
        "SaveState JSON must contain the spread counter, got: {}",
        &json[..json.len().min(200)]
    );
    assert!(
        json.contains("tar_pits")
            && json.contains("tar_patches")
            && json.contains("tar_warnings"),
        "SaveState JSON must contain the tar pit state, got: {}",
        &json[..json.len().min(400)]
    );
}
