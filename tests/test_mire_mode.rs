use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn mire_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Mire;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

// Block the ambient portal/mine spawners so single-tick wave tests are
// deterministic: an existing portal pair stops manage_portals from spawning,
// and a full mine rack (len >= 5) stops manage_mines from dropping new mines.
fn pin_ambient_spawners(game: &mut Game) {
    game.portals = Some((Point { x: 27, y: 27 }, Point { x: 28, y: 28 }));
    game.mines.insert(Point { x: 23, y: 1 });
    game.mines.insert(Point { x: 24, y: 1 });
    game.mines.insert(Point { x: 25, y: 1 });
    game.mines.insert(Point { x: 26, y: 1 });
    game.mines.insert(Point { x: 27, y: 1 });
}

#[test]
fn test_mire_reset_seeds_warnings_and_clears_state() {
    let game = mire_game(30, 30);

    assert!(game.mire_cells.is_empty());
    assert_eq!(game.mire_waves, 0);
    assert_eq!(
        game.mire_warnings.len(),
        Game::MIRE_INITIAL_WARNINGS,
        "reset must seed the initial telegraphed cracks"
    );
    let spawn = Point {
        x: game.width / 2,
        y: game.height / 2,
    };
    for crack in &game.mire_warnings {
        assert!(
            !game.obstacles.contains(crack),
            "a crack must never sit on an obstacle: {crack:?}"
        );
        assert!(
            !game.snake.body_map.contains_key(crack),
            "a crack must never sit on the snake: {crack:?}"
        );
        assert_ne!(*crack, game.food, "a crack must never sit on the food");
        assert!(
            crack.x.abs_diff(spawn.x).saturating_add(crack.y.abs_diff(spawn.y)) >= 5,
            "a crack must spawn at least 5 cells from the spawn point: {crack:?}"
        );
    }
}

#[test]
fn test_mire_telegraphs_then_sinks_every_6_ticks() {
    let mut game = mire_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.auto_pilot = false;
    pin_ambient_spawners(&mut game);

    // update() increments tick_counter first, so tick_counter = 2 runs tick 3,
    // which telegraphs the cells that will sink on tick 6.
    game.state = GameState::Playing;
    game.tick_counter = 2;
    game.update();
    assert_eq!(game.tick_counter, 3);
    assert!(
        !game.mire_warnings.is_empty(),
        "tick 3 must telegraph cracked cells"
    );
    assert!(
        game.mire_warnings.len() <= Game::MIRE_PER_WAVE,
        "at most MIRE_PER_WAVE fresh cracks per telegraph"
    );
    assert!(game.mire_cells.is_empty(), "no pools before the wave");

    // Tick 6 sinks: the warnings are consumed and mire pools come up.
    game.state = GameState::Playing;
    game.tick_counter = 5;
    game.update();
    assert_eq!(game.tick_counter, 6);
    assert_eq!(game.mire_waves, 1);
    assert!(
        game.mire_warnings.is_empty(),
        "the telegraph is consumed by the wave"
    );
    assert!(
        !game.mire_cells.is_empty(),
        "tick 6 must sink the telegraphed cells"
    );
    for (pool, ticks) in &game.mire_cells {
        assert!(
            game.obstacles.contains(pool),
            "pools must be mirrored into obstacles"
        );
        assert_eq!(*ticks, Game::MIRE_POOL_TICKS);
    }

    // Ticks 12 and 18 sink again.
    for tick in [11_u32, 17] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
    }
    assert_eq!(game.mire_waves, 3);
}

#[test]
fn test_mire_never_sinks_snake_or_next_cell() {
    let mut game = mire_game(20, 20);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    // The snake heads Left into (4,5); both (4,5) and its own head (5,5) are
    // telegraphed, so the wave must skip both.
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Left;
    game.snake.direction_queue.push_back(Direction::Left);
    game.food = Point { x: 15, y: 15 };
    game.mire_warnings.clear();
    game.mire_warnings.insert(Point { x: 4, y: 5 });
    game.mire_warnings.insert(Point { x: 5, y: 5 });
    game.auto_pilot = false;

    // Tick 6 is a wave tick (6 % 6 == 0) and not a telegraph tick.
    game.state = GameState::Playing;
    game.tick_counter = 5;
    game.update();

    assert_eq!(game.mire_waves, 1);
    let next = Point { x: 4, y: 5 };
    assert!(
        !game.mire_cells.contains_key(&next),
        "a wave must never sink the cell the snake is about to enter"
    );
    for seg in &game.snake.body {
        assert!(
            !game.mire_cells.contains_key(seg),
            "a wave must never sink the snake's body"
        );
    }
    assert!(
        !game.mire_cells.contains_key(&game.food),
        "a wave must never sink the food"
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the wave must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_mire_fairness_over_many_ticks() {
    let mut game = mire_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    pin_ambient_spawners(&mut game);
    game.food = Point { x: 25, y: 25 };
    let lives_before = game.lives;

    // Deterministic: park the snake at (8, 15) heading right before every
    // scheduled wave, so the wave must never sink its next cell (9, 15) or body.
    let next = Point { x: 9, y: 15 };
    for k in 1..=10_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = 6 * k - 1;
        game.update();
        assert!(
            !game.mire_cells.contains_key(&next),
            "wave {} sank the cell the snake was about to enter ({next:?})",
            game.mire_waves
        );
        for seg in &game.snake.body {
            assert!(
                !game.mire_cells.contains_key(seg),
                "wave {} sank the snake's body at {seg:?}",
                game.mire_waves
            );
        }
    }

    assert_eq!(
        game.lives, lives_before,
        "the snake must survive the waves (waves: {})",
        game.mire_waves
    );
    assert!(
        game.mire_waves >= 7,
        "expected several waves over 60 ticks, got {}",
        game.mire_waves
    );
}

#[test]
fn test_mire_pools_dry_up_after_pool_ticks() {
    let mut game = mire_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.auto_pilot = false;
    game.obstacles.clear();
    let drying = Point { x: 5, y: 5 };
    let staying = Point { x: 6, y: 6 };
    game.mire_cells.insert(drying, 1);
    game.obstacles.insert(drying);
    game.mire_cells.insert(staying, 3);
    game.obstacles.insert(staying);

    // Tick 11 is not a multiple of 6 and not 200, so no wave interferes.
    game.state = GameState::Playing;
    game.tick_counter = 10;
    game.update();

    assert!(
        !game.mire_cells.contains_key(&drying),
        "a pool with one tick left must dry up this tick"
    );
    assert!(
        !game.obstacles.contains(&drying),
        "a dried pool must leave obstacles"
    );
    assert_eq!(
        game.mire_cells.get(&staying),
        Some(&2),
        "a pool with more ticks left must stay and count down"
    );
}

#[test]
fn test_mire_settles_all_at_tick_200() {
    let mut game = mire_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.auto_pilot = true;

    // Sink a few waves first.
    for tick in [5_u32, 11, 17] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
    }
    assert!(!game.mire_cells.is_empty());
    assert!(game.mire_waves >= 3);

    // Tick 200 of the 250-tick cycle dries up all mire.
    game.state = GameState::Playing;
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.mire_cells.is_empty(),
        "all pools must dry up at tick 200"
    );
    assert!(
        game.mire_warnings.is_empty(),
        "all telegraph cracks must clear at tick 200"
    );
    for obs in game.obstacles.iter() {
        assert!(
            !game.mire_cells.contains_key(obs),
            "no obstacle may remain a recorded pool"
        );
    }
    assert!(
        game.mire_waves >= 3,
        "the wave counter keeps counting through a settling"
    );
}

#[test]
fn test_mire_eating_food_stirs_adjacent_pools() {
    let mut game = mire_game(20, 20);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    let pool = Point { x: 7, y: 5 };
    game.mire_cells.insert(pool, 5);
    game.obstacles.insert(pool);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.auto_pilot = false;

    // Tick 5 is not a multiple of 6 and not 200, so no wave interferes here.
    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.mire_cells.contains_key(&pool),
        "Eating food in Mire mode should stir the adjacent pool dry"
    );
    assert!(
        !game.obstacles.contains(&pool),
        "The dried pool must also leave obstacles"
    );
}

#[test]
fn test_mire_bot_treats_cracks_and_pools_as_walls() {
    let mut game = mire_game(30, 30);
    game.obstacles.clear();
    let crack = Point { x: 3, y: 3 };
    let pool = Point { x: 4, y: 4 };
    game.mire_warnings.insert(crack);
    game.mire_cells.insert(pool, 5);
    game.obstacles.insert(pool);

    assert!(
        !game.is_safe_final_p(crack, 1, 1),
        "the autopilot must treat a telegraphed crack as a wall"
    );
    assert!(
        !game.is_safe_final_p(pool, 1, 1),
        "the autopilot must treat a mire pool as a wall"
    );
    assert!(
        !game.is_safe_final_p(crack, 5, 1),
        "the autopilot must treat a crack as a wall even several steps out"
    );

    // The same cells are ordinary ground in a mode without mire.
    game.mode = GameMode::SinglePlayer;
    game.mire_cells.clear();
    game.mire_warnings.clear();
    assert!(
        game.is_safe_final_p(crack, 1, 1),
        "a crack must be safe ground outside Mire mode"
    );
}

#[test]
fn test_mire_bot_survives_the_waves() {
    let mut game = mire_game(40, 40);
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
        "Autopilot should survive the mire (lives: {}, waves: {}, score: {})",
        game.lives,
        game.mire_waves,
        game.score
    );
    assert!(
        game.mire_waves >= 5,
        "expected several waves over 150 ticks, got {}",
        game.mire_waves
    );
}

#[test]
fn test_mire_persists_through_history_rewind() {
    let mut game = mire_game(30, 30);
    game.mire_cells.insert(Point { x: 10, y: 11 }, 4);
    game.mire_warnings.insert(Point { x: 11, y: 10 });
    game.mire_waves = 7;
    game.save_history_state();
    game.mire_cells.clear();
    game.mire_warnings.clear();
    game.mire_waves = 0;

    game.rewind_time();

    assert_eq!(game.mire_cells.get(&Point { x: 10, y: 11 }), Some(&4));
    assert!(game.mire_warnings.contains(&Point { x: 11, y: 10 }));
    assert_eq!(
        game.mire_waves, 7,
        "mire_waves must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_mire_persists_in_save_file() {
    let mut game = mire_game(30, 30);
    game.mire_waves = 9;
    game.mire_cells.insert(Point { x: 12, y: 10 }, 3);
    game.mire_warnings.insert(Point { x: 11, y: 9 });
    let path = std::env::temp_dir().join(format!(
        "mire_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"mire_waves\":9"),
        "SaveState JSON must contain the wave counter, got: {}",
        &json[..json.len().min(200)]
    );
    assert!(
        json.contains("mire_cells") && json.contains("mire_warnings"),
        "SaveState JSON must contain the mire state, got: {}",
        &json[..json.len().min(400)]
    );
}
