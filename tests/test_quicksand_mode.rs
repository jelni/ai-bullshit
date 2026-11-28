use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn quicksand_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Quicksand;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

fn keep_playing(game: &mut Game) {
    if game.state != GameState::Playing {
        if game.lives == 0 {
            return;
        }
        game.state = GameState::Playing;
    }
}

// Block the ambient portal/mine spawners so single-tick tests are
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
fn test_quicksand_reset_seeds_patches_fairly() {
    let game = quicksand_game(30, 30);

    assert_eq!(game.quicksand_spreads, 0);
    assert_eq!(game.quicksand_sink, 0);
    assert_eq!(game.quicksand_sink2, 0);
    assert!(game.quicksand_bot_sinks.is_empty());
    assert!(
        game.quicksand.len() >= Game::QUICKSAND_INITIAL_PATCHES,
        "reset must seed at least {} patch centers, got {}",
        Game::QUICKSAND_INITIAL_PATCHES,
        game.quicksand.len()
    );
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    for cell in &game.quicksand {
        assert!(
            !game.obstacles.contains(cell),
            "quicksand must never sit on an obstacle: {cell:?}"
        );
        assert!(
            !game.snake.body_map.contains_key(cell),
            "quicksand must never sit on the snake: {cell:?}"
        );
        assert_ne!(*cell, game.food, "quicksand must never sit on the food");
        // Patch arms are one cell closer to the spawn than their center
        assert!(
            cell.x.abs_diff(spawn.x).saturating_add(cell.y.abs_diff(spawn.y))
                >= Game::QUICKSAND_MIN_SPAWN_DIST - 1,
            "quicksand must spawn away from the spawn point: {cell:?}"
        );
    }
}

#[test]
fn test_quicksand_spreads_on_schedule() {
    let mut game = quicksand_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.auto_pilot = false;
    let initial = game.quicksand.len();

    // update() increments tick_counter first, so tick_counter = 7 runs tick 8,
    // the first multiple of QUICKSAND_PERIOD (8)
    game.state = GameState::Playing;
    game.tick_counter = 7;
    game.update();
    assert_eq!(game.tick_counter, 8);
    assert_eq!(
        game.quicksand_spreads, 1,
        "tick 8 must trigger exactly one scheduled spread"
    );
    assert!(
        game.quicksand.len() >= initial,
        "a spread must never shrink the bog ({} -> {})",
        initial,
        game.quicksand.len()
    );

    // Ticks 16 and 24 spread again and the bog grows past its initial size
    for tick in [15_u32, 23] {
        keep_playing(&mut game);
        game.tick_counter = tick;
        game.update();
    }
    assert_eq!(game.quicksand_spreads, 3);
    assert!(
        game.quicksand.len() > initial,
        "the bog must grow over scheduled spreads ({} -> {})",
        initial,
        game.quicksand.len()
    );
}

#[test]
fn test_quicksand_never_claims_a_snake_over_many_ticks() {
    let mut game = quicksand_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;

    for _ in 0..150 {
        game.update();
        if game.state != GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = GameState::Playing;
        }
        for cell in &game.quicksand {
            assert!(
                !game.snake.body_map.contains_key(cell),
                "quicksand must never claim a cell of the snake's body: {cell:?}"
            );
        }
    }

    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "Autopilot should survive the bog (lives: {}, spreads: {}, score: {})",
        game.lives,
        game.quicksand_spreads,
        game.score
    );
    assert!(
        game.quicksand_spreads >= 10,
        "expected many scheduled spreads over 150 ticks, got {}",
        game.quicksand_spreads
    );
}

#[test]
fn test_quicksand_sink_counts_in_bog_and_resets_when_leaving() {
    let mut game = quicksand_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.quicksand.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 9, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.quicksand.insert(Point { x: 10, y: 10 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    // Tick 1: the head moves onto the bog cell (10,10) -> sink 1
    game.state = GameState::Playing;
    game.tick_counter = 0;
    game.update();
    assert_eq!(game.snake.head(), Point { x: 10, y: 10 });
    assert_eq!(game.quicksand_sink, 1, "standing in the bog must sink one tick");

    // Tick 2: the head leaves the bog -> sink resets
    keep_playing(&mut game);
    game.tick_counter = 1;
    game.update();
    assert_eq!(game.snake.head(), Point { x: 11, y: 10 });
    assert_eq!(
        game.quicksand_sink, 0,
        "leaving the bog must reset the sink counter"
    );
}

#[test]
fn test_quicksand_swallows_snake_after_sink_ticks() {
    let mut game = quicksand_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.obstacles.clear();
    game.quicksand.clear();
    pin_ambient_spawners(&mut game);
    // A long bog row the snake travels along, staying in the bog every tick
    for x in 5..=25 {
        game.quicksand.insert(Point { x, y: 10 });
    }
    game.snake = Snake::new(Point { x: 5, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    let lives_before = game.lives;

    game.state = GameState::Playing;
    game.tick_counter = 0;
    for _ in 0..Game::QUICKSAND_SINK_TICKS {
        game.update();
        if game.lives == 0 {
            break;
        }
        if game.state != GameState::Playing {
            // respawned after being swallowed; keep ticking is pointless
            break;
        }
    }

    assert!(
        game.lives < lives_before,
        "standing in quicksand for {} ticks must swallow the snake (lives: {} -> {})",
        Game::QUICKSAND_SINK_TICKS,
        lives_before,
        game.lives
    );
}

#[test]
fn test_quicksand_drains_at_tick_200() {
    let mut game = quicksand_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.auto_pilot = true;

    // Spread a few times first
    for tick in [7_u32, 15, 23] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
    }
    assert!(!game.quicksand.is_empty());
    assert!(game.quicksand_spreads >= 3);
    game.quicksand_sink = 4;

    // Tick 200 of the 250-tick cycle drains the whole bog
    keep_playing(&mut game);
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.quicksand.is_empty(),
        "the whole bog must drain at tick 200"
    );
    assert_eq!(game.quicksand_sink, 0, "the drain must reset sink counters");
    assert!(
        game.quicksand_spreads >= 3,
        "the spread counter is historical and must survive the drain"
    );
}

#[test]
fn test_quicksand_eating_food_stomps_adjacent_bog_and_resets_sink() {
    let mut game = quicksand_game(20, 20);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.quicksand.clear();
    pin_ambient_spawners(&mut game);
    let bog = Point { x: 7, y: 5 };
    let eaten = Point { x: 6, y: 5 };
    game.quicksand.insert(bog);
    game.quicksand.insert(eaten);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = eaten;
    game.quicksand_sink = 5;
    game.auto_pilot = false;

    // Tick 5 is not a multiple of QUICKSAND_PERIOD (8) and not 200, so no spread interferes
    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.quicksand.contains(&bog),
        "Eating food in Quicksand mode should stomp the adjacent bog cell"
    );
    assert_eq!(
        game.quicksand_sink, 0,
        "Eating food must shake the eater loose (reset the sink counter)"
    );
}

#[test]
fn test_quicksand_is_hard_blocked_for_the_autopilot() {
    let mut game = quicksand_game(30, 30);
    game.obstacles.clear();
    let bog = *game.quicksand.iter().next().expect("reset must seed bog");

    assert!(
        !game.is_safe_final_p(bog, 1, 1),
        "a quicksand cell must be unsafe in is_safe_final_p"
    );
    assert!(
        game.is_safe_final_p(Point { x: 1, y: 1 }, 1, 1),
        "a free cell must stay safe in is_safe_final_p"
    );
}

#[test]
fn test_quicksand_persists_through_history_rewind() {
    let mut game = quicksand_game(30, 30);
    game.quicksand.clear();
    game.quicksand.insert(Point { x: 10, y: 10 });
    game.quicksand.insert(Point { x: 11, y: 10 });
    game.quicksand_spreads = 7;
    game.quicksand_sink = 3;
    game.quicksand_sink2 = 2;
    game.quicksand_bot_sinks.insert(0, 4);
    game.save_history_state();
    game.quicksand.clear();
    game.quicksand_spreads = 0;
    game.quicksand_sink = 0;
    game.quicksand_sink2 = 0;
    game.quicksand_bot_sinks.clear();

    game.rewind_time();

    assert!(game.quicksand.contains(&Point { x: 10, y: 10 }));
    assert!(game.quicksand.contains(&Point { x: 11, y: 10 }));
    assert_eq!(
        game.quicksand_spreads, 7,
        "quicksand_spreads must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.quicksand_sink, 3);
    assert_eq!(game.quicksand_sink2, 2);
    assert_eq!(game.quicksand_bot_sinks.get(&0), Some(&4));
}

#[test]
fn test_quicksand_persists_in_save_file() {
    let mut game = quicksand_game(30, 30);
    game.quicksand_spreads = 9;
    game.quicksand.clear();
    game.quicksand.insert(Point { x: 12, y: 9 });
    game.quicksand_sink = 6;
    let path = std::env::temp_dir().join(format!(
        "quicksand_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"quicksand_spreads\":9"),
        "SaveState JSON must contain the spread counter, got: {}",
        &json[..json.len().min(200)]
    );
    assert!(
        json.contains("quicksand") && json.contains("quicksand_sink"),
        "SaveState JSON must contain the quicksand state, got: {}",
        &json[..json.len().min(400)]
    );
}
