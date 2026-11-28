use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn fault_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Faultline;
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
fn test_fault_reset_seeds_initial_wave_fairly() {
    let game = fault_game(30, 30);

    assert_eq!(game.fault_waves, 0);
    assert_eq!(game.fault_collapses, 0);
    assert_eq!(game.fault_heals, 0);
    assert_eq!(game.fault_repairs, 0);
    assert!(game.fault_pits.is_empty());
    // wave size = clamp(30*30/60, 4, 10) = 10
    assert_eq!(game.fault_tiles.len(), 10);
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    for (crack, ticks) in &game.fault_tiles {
        assert_eq!(*ticks, Game::FAULT_TICKS);
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
            !game.fault_pits.contains(crack),
            "a crack must never sit on a pit: {crack:?}"
        );
        assert!(
            crack.x.abs_diff(spawn.x).saturating_add(crack.y.abs_diff(spawn.y)) >= 5,
            "a crack must spawn at least 5 cells from the spawn point: {crack:?}"
        );
    }
}

#[test]
fn test_fault_wave_fires_every_20_ticks_and_stays_fair() {
    let mut game = fault_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = false;
    pin_ambient_spawners(&mut game);

    // update() increments tick_counter first, so 19 makes the next update run
    // tick 20, where the scheduled crack wave fires. The fairness guarantees
    // must be checked against the pre-update board state.
    for tick in [19_u32, 39, 59] {
        game.state = GameState::Playing;
        game.obstacles.clear();
        game.fault_tiles.clear();
        game.fault_pits.clear();
        // Park the snake deterministically so the pre-update state is exactly
        // the state the wave sees (the direction queue is popped first).
        game.snake = Snake::new(Point { x: 20, y: 20 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.food = Point { x: 5, y: 5 };
        game.tick_counter = tick;
        let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
        let pre_head = game.snake.head();
        let pre_next =
            snake_game::game::Game::calculate_next_head_dir(pre_head, game.snake.direction);
        let pre_food = game.food;
        let pre_obstacles: std::collections::HashSet<Point> = game.obstacles.clone();
        let waves_before = game.fault_waves;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        assert!(
            game.fault_waves > waves_before,
            "a crack wave must happen on tick {}",
            tick + 1
        );
        assert_eq!(
            game.fault_tiles.len(),
            10, // clamp(30*30/60, 4, 10) = 10
            "tick {}: a full wave must spawn",
            tick + 1
        );
        for (crack, ticks) in &game.fault_tiles {
            assert_eq!(*ticks, Game::FAULT_TICKS);
            assert!(
                !pre_obstacles.contains(crack),
                "a crack must not sit on an obstacle: {crack:?}"
            );
            assert!(!pre_body.contains(crack), "a crack must not sit on the snake: {crack:?}");
            assert_ne!(
                *crack, pre_next,
                "a crack must not sit on the cell the snake is about to enter"
            );
            assert_ne!(*crack, pre_food, "a crack must not sit on the food");
            assert!(
                crack.x.abs_diff(pre_head.x).saturating_add(crack.y.abs_diff(pre_head.y)) >= 4,
                "a crack must keep its distance from snake heads: {crack:?}"
            );
        }
    }
    assert_eq!(game.fault_waves, 3);
}

#[test]
fn test_fault_crack_collapses_into_a_lethal_pit() {
    let mut game = fault_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.fault_tiles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    // One crack far from the snake, about to collapse
    game.fault_tiles.insert(Point { x: 20, y: 20 }, 1);

    game.tick_counter = 0; // tick 1: no wave, no heal
    game.state = GameState::Playing;
    game.update();

    let pit = Point { x: 20, y: 20 };
    assert!(
        !game.fault_tiles.contains_key(&pit),
        "the crack must be consumed by the collapse"
    );
    assert!(
        game.fault_pits.contains(&pit),
        "the crack must collapse into a pit"
    );
    assert!(
        game.obstacles.contains(&pit),
        "a pit must be mirrored into obstacles so it kills on entry and blocks the bot"
    );
    assert_eq!(game.fault_collapses, 1);
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a fair collapse away from the snake must not kill it (lives: {})",
        game.lives
    );
}

#[test]
fn test_fault_collapse_is_deferred_while_a_snake_stands_on_the_crack() {
    let mut game = fault_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.fault_tiles.clear();
    pin_ambient_spawners(&mut game);
    // The snake stands on the crack cell, heading away so its next cell differs
    game.snake = Snake::new(Point { x: 20, y: 20 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 5, y: 5 };
    game.auto_pilot = false;
    game.fault_tiles.insert(Point { x: 20, y: 20 }, 1);

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    let crack = Point { x: 20, y: 20 };
    assert!(
        !game.fault_pits.contains(&crack),
        "a collapse must never happen under a snake's body"
    );
    assert!(
        game.fault_tiles.contains_key(&crack),
        "the crack stays armed (deferred) instead of collapsing"
    );
    assert_eq!(game.fault_collapses, 0);
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the deferred collapse must not kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_fault_collapse_is_deferred_when_the_next_head_enters_the_crack() {
    let mut game = fault_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.obstacles.clear();
    game.fault_tiles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    // The snake is about to enter (11,10): the crack there must not collapse
    game.fault_tiles.insert(Point { x: 11, y: 10 }, 1);

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    let crack = Point { x: 11, y: 10 };
    assert!(
        !game.fault_pits.contains(&crack),
        "a collapse must never happen on the cell a snake is about to enter"
    );
    assert!(
        game.fault_tiles.contains_key(&crack),
        "the crack stays armed (deferred) instead of collapsing"
    );
    assert_eq!(
        game.snake.head(),
        crack,
        "the snake walks onto the still-walkable crack safely (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.fault_collapses, 0);
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the deferred collapse must not kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_fault_pit_kills_on_entry() {
    let mut game = fault_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.fault_tiles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    // A collapsed pit sits right in the snake's path
    let pit = Point { x: 11, y: 10 };
    game.fault_pits.insert(pit);
    game.obstacles.insert(pit);
    let lives_before = game.lives;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert!(
        game.lives < lives_before || game.state != GameState::Playing,
        "entering a pit must kill the snake (lives: {} -> {})",
        lives_before,
        game.lives
    );
}

#[test]
fn test_fault_floor_heals_every_200_ticks() {
    let mut game = fault_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.fault_tiles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    let pit = Point { x: 20, y: 20 };
    game.fault_pits.insert(pit);
    game.obstacles.insert(pit);
    game.fault_tiles.insert(Point { x: 21, y: 21 }, 5);

    // update() increments tick_counter first, so 199 makes the next update run
    // tick 200, where the scheduled heal fires.
    game.tick_counter = 199;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert_eq!(game.fault_heals, 1);
    assert!(game.fault_pits.is_empty(), "the heal must fill in all pits");
    // Tick 200 is also a multiple of FAULT_PERIOD (20), so right after the
    // heal a fresh crack wave spawns: every crack is a fresh telegraph.
    assert!(
        game.fault_tiles.values().all(|t| *t == Game::FAULT_TICKS),
        "after the heal only fresh FAULT_TICKS telegraphs may remain: {:?}",
        game.fault_tiles
    );
    assert!(
        game.fault_waves >= 1,
        "the tick-200 wave must fire after the heal"
    );
    assert!(
        !game.obstacles.contains(&pit),
        "the healed pit must leave the obstacle mirror"
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the heal must not kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_fault_eating_food_repairs_adjacent_cracks() {
    let mut game = fault_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.fault_tiles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 11, y: 10 };
    game.auto_pilot = false;
    // Cracked cells adjacent to the eaten cell (11,10)
    game.fault_tiles.insert(Point { x: 12, y: 10 }, 3);
    game.fault_tiles.insert(Point { x: 11, y: 11 }, 3);
    game.fault_tiles.insert(Point { x: 15, y: 15 }, 3); // not adjacent: must survive

    game.tick_counter = 0; // tick 1: no wave
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert!(
        !game.fault_tiles.contains_key(&Point { x: 12, y: 10 })
            && !game.fault_tiles.contains_key(&Point { x: 11, y: 11 }),
        "cracks adjacent to the eaten cell must be repaired"
    );
    assert!(
        game.fault_tiles.contains_key(&Point { x: 15, y: 15 }),
        "a non-adjacent crack must survive the meal"
    );
    assert_eq!(game.fault_repairs, 2);
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "eating must never kill in Faultline mode (lives: {})",
        game.lives
    );
}

#[test]
fn test_fault_scraps_cracks_under_fresh_obstacles() {
    let mut game = fault_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.fault_tiles.clear();
    game.fault_tiles.insert(Point { x: 14, y: 14 }, 8);
    // Something else (e.g. a meteor) drops an obstacle on the crack
    game.obstacles.insert(Point { x: 14, y: 14 });

    game.tick_counter = 0; // tick 1: no wave
    game.state = GameState::Playing;
    game.update();

    assert!(
        !game.fault_tiles.contains_key(&Point { x: 14, y: 14 }),
        "a crack under a fresh obstacle must be scrapped on the next tick"
    );
}

#[test]
fn test_fault_autopilot_treats_cracks_and_pits_as_walls() {
    let mut game = fault_game(30, 30);
    game.obstacles.clear();
    game.fault_tiles.clear();
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.fault_tiles.insert(Point { x: 5, y: 5 }, Game::FAULT_TICKS);
    let pit = Point { x: 6, y: 6 };
    game.fault_pits.insert(pit);
    game.obstacles.insert(pit);

    assert!(
        !game.is_safe_final_p(Point { x: 5, y: 5 }, 1, 0),
        "is_safe_final_p must reject a cracked cell"
    );
    assert!(
        !game.is_safe_final_p(pit, 1, 0),
        "is_safe_final_p must reject a pit cell"
    );
    assert!(
        game.is_safe_final_p(Point { x: 8, y: 8 }, 1, 0),
        "a plain empty cell must stay safe"
    );
}

#[test]
fn test_fault_bot_survives_the_crumbling_floor() {
    let mut game = fault_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;

    for _ in 0..200 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
    }

    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "Autopilot should survive Faultline mode (lives: {}, waves: {}, collapses: {}, score: {})",
        game.lives,
        game.fault_waves,
        game.fault_collapses,
        game.score
    );
}

#[test]
fn test_fault_persists_through_history_rewind() {
    let mut game = fault_game(30, 30);
    game.fault_tiles.insert(Point { x: 12, y: 12 }, 7);
    game.fault_pits.insert(Point { x: 13, y: 13 });
    game.fault_waves = 3;
    game.fault_collapses = 5;
    game.fault_heals = 1;
    game.fault_repairs = 2;
    game.save_history_state();
    game.fault_tiles.clear();
    game.fault_pits.clear();
    game.fault_waves = 0;
    game.fault_collapses = 0;
    game.fault_heals = 0;
    game.fault_repairs = 0;

    game.rewind_time();

    assert_eq!(game.fault_tiles.get(&Point { x: 12, y: 12 }), Some(&7));
    assert!(game.fault_pits.contains(&Point { x: 13, y: 13 }));
    assert_eq!(game.fault_waves, 3);
    assert_eq!(game.fault_collapses, 5);
    assert_eq!(game.fault_heals, 1);
    assert_eq!(game.fault_repairs, 2);
}

#[test]
fn test_fault_persists_in_save_file() {
    let mut game = fault_game(30, 30);
    game.fault_tiles.insert(Point { x: 12, y: 12 }, 7);
    game.fault_waves = 3;
    game.fault_collapses = 5;
    game.fault_heals = 1;
    game.fault_repairs = 2;
    let path = std::env::temp_dir().join(format!(
        "fault_save_test_{}_{}.json",
        std::process::id(),
        3_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"fault_waves\":3")
            && json.contains("\"fault_collapses\":5")
            && json.contains("\"fault_heals\":1")
            && json.contains("\"fault_repairs\":2")
            && json.contains("\"fault_tiles\"")
            && json.contains("\"fault_pits\""),
        "SaveState JSON must contain the fault fields, got: {}",
        &json[..json.len().min(200)]
    );
}
