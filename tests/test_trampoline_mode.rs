use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn trampoline_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Trampoline;
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

// Block the ambient portal/mine spawners so single-tick launch tests are
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
fn test_trampoline_reset_seeds_pads_fairly() {
    let game = trampoline_game(30, 30);

    assert_eq!(game.tramp_flings, 0);
    assert_eq!(game.tramp_shuffles, 0);
    assert_eq!(game.tramp_overcharge, 0);
    // pad count = clamp(30*30/60, 3, 8) = 8
    assert_eq!(game.tramp_pads.len(), 8);
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    for pad in &game.tramp_pads {
        assert!(
            !game.obstacles.contains(pad),
            "a pad must never sit on an obstacle: {pad:?}"
        );
        assert!(
            !game.snake.body_map.contains_key(pad),
            "a pad must never sit on the snake: {pad:?}"
        );
        assert_ne!(*pad, game.food, "a pad must never sit on the food");
        assert!(
            pad.x.abs_diff(spawn.x).saturating_add(pad.y.abs_diff(spawn.y)) >= 5,
            "a pad must spawn at least 5 cells from the spawn point: {pad:?}"
        );
    }
}

#[test]
fn test_trampoline_launch_flings_snake_along_direction() {
    let mut game = trampoline_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.tramp_pads.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    // The snake heads right into the pad at (11,10)
    game.tramp_pads.insert(Point { x: 11, y: 10 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0; // tick 1: no scheduled shuffle
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 1);
    assert_eq!(game.tramp_flings, 1, "the pad must fling the snake");
    assert_eq!(
        game.snake.head(),
        Point { x: 14, y: 10 },
        "the fling must launch TRAMP_LAUNCH (3) cells along the direction (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a fling must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_trampoline_launch_shortens_around_a_blocked_path() {
    let mut game = trampoline_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.obstacles.clear();
    game.tramp_pads.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.tramp_pads.insert(Point { x: 11, y: 10 });
    // The 3-cell path is blocked at (13,10) and the 2-cell path shares that
    // cell, so only the 1-cell fling to (12,10) is fair
    game.obstacles.insert(Point { x: 13, y: 10 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 12, y: 10 },
        "the fling must shorten to the longest fully fair path (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.tramp_flings, 1);
}

#[test]
fn test_trampoline_launch_fizzles_when_no_path_is_fair() {
    let mut game = trampoline_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(9);
    game.obstacles.clear();
    game.tramp_pads.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.tramp_pads.insert(Point { x: 11, y: 10 });
    // Even the 1-cell path is blocked, so no fling may happen
    game.obstacles.insert(Point { x: 12, y: 10 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "with no fair launch path the snake must simply rest on the walkable pad (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.tramp_flings, 0, "no fling may be counted");
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a pad must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_trampoline_launch_never_lands_on_a_hazard() {
    let mut game = trampoline_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.obstacles.clear();
    game.tramp_pads.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.tramp_pads.insert(Point { x: 11, y: 10 });
    // A mine blocks the 3-cell landing (14,10): the fling must shorten to 2
    game.mines.insert(Point { x: 14, y: 10 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 13, y: 10 },
        "the fling must skip hazard cells on its path (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the fling must never land on a hazard (lives: {})",
        game.lives
    );
}

#[test]
fn test_trampoline_shuffle_every_24_ticks_stays_fair() {
    let mut game = trampoline_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = false;

    // update() increments tick_counter first, so 23 makes the next update run
    // tick 24. The shuffle fires at the start of that tick, so the fairness
    // guarantees must be checked against the pre-update board state.
    for tick in [23_u32, 47, 71, 95, 119] {
        game.state = GameState::Playing;
        // Park the snake deterministically so the pre-update state is exactly
        // the state the shuffle sees (the direction queue is popped first).
        game.snake = Snake::new(Point { x: 20, y: 20 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.food = Point { x: 35, y: 35 };
        game.tick_counter = tick;
        let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
        let pre_head = game.snake.head();
        let pre_next =
            snake_game::game::Game::calculate_next_head_dir(pre_head, game.snake.direction);
        let pre_food = game.food;
        let pre_obstacles: std::collections::HashSet<Point> = game.obstacles.clone();
        let shuffles_before = game.tramp_shuffles;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        assert!(
            game.tramp_shuffles > shuffles_before,
            "a pad shuffle must happen on tick {}",
            tick + 1
        );
        for pad in &game.tramp_pads {
            assert!(
                !pre_obstacles.contains(pad),
                "a relocated pad must not sit on an obstacle: {pad:?}"
            );
            assert!(
                !pre_body.contains(pad),
                "a relocated pad must not sit on the snake: {pad:?}"
            );
            assert_ne!(*pad, pre_next, "a relocated pad must not sit on the cell the snake is about to enter");
            assert_ne!(*pad, pre_food, "a relocated pad must not sit on the food");
            assert!(
                pad.x.abs_diff(pre_head.x).saturating_add(pad.y.abs_diff(pre_head.y)) >= 4,
                "a relocated pad must keep its distance from snake heads: {pad:?}"
            );
        }
    }
    assert_eq!(game.tramp_shuffles, 5);
    assert_eq!(game.tramp_pads.len(), 8, "clamp(40*40/60, 3, 8) = 8");
}

#[test]
fn test_trampoline_eating_food_overcharges_the_pads() {
    let mut game = trampoline_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.tramp_pads.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 11, y: 10 };
    game.tramp_pads.insert(Point { x: 20, y: 20 });
    game.auto_pilot = false;

    // Tick 1: the snake eats the food at (11,10)
    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert_eq!(
        game.tramp_overcharge,
        Game::TRAMP_OVERCHARGE_TICKS,
        "eating food must overcharge the pads for TRAMP_OVERCHARGE_TICKS"
    );

    // Tick 2: head into a pad while overcharged - the fling goes 5 cells
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 10, y: 12 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
    game.tramp_pads.clear();
    game.tramp_pads.insert(Point { x: 11, y: 12 });
    game.food = Point { x: 25, y: 25 };
    game.tick_counter = 1;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 16, y: 12 },
        "an overcharged pad must fling TRAMP_LAUNCH_CHARGED (5) cells (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.tramp_flings, 1);
    assert!(
        game.tramp_overcharge < Game::TRAMP_OVERCHARGE_TICKS,
        "the overcharge timer must count down every tick"
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "an overcharged fling must never kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_trampoline_fling_never_unfair_over_many_ticks() {
    let mut game = trampoline_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 35, y: 35 };
    pin_ambient_spawners(&mut game);
    let mut total_flings = 0u32;

    // Park the snake heading into a pad before every non-shuffle update and
    // make sure the machine can never fling it into a wall, a hazard, or itself
    for tick in 0..60_u32 {
        if (tick + 1).is_multiple_of(24) {
            continue; // shuffle ticks relocate the pads
        }
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tramp_pads.clear();
        game.tramp_pads.insert(Point { x: 9, y: 15 });
        game.tick_counter = tick;
        let flings_before = game.tramp_flings;
        let lives_before = game.lives;
        game.update();
        let flung = game.tramp_flings > flings_before;
        total_flings += game.tramp_flings.saturating_sub(flings_before);
        if game.lives == 0 {
            // An ambient hazard (e.g. a meteor) got the parked snake; top up
            // and carry on - the fling assertions below only apply to ticks
            // where a fling actually happened and the snake survived.
            game.lives = 3;
            game.state = GameState::Playing;
            continue;
        }
        let head = game.snake.head();
        if flung && game.lives == lives_before {
            assert!(
                head.y == 15 && head.x > 9 && head.x <= 9 + Game::TRAMP_LAUNCH,
                "tick {tick}: a fling must land on the launch line within TRAMP_LAUNCH cells, got {head:?}"
            );
            assert!(
                !game.obstacles.contains(&head),
                "tick {tick}: the fling landed on an obstacle at {head:?}"
            );
            assert!(
                head.x > 0 && head.x < game.width - 1 && head.y > 0 && head.y < game.height - 1,
                "tick {tick}: the fling left the board at {head:?}"
            );
        }
    }

    assert!(
        total_flings >= 10,
        "expected the pad to fling the parked snake on most ticks, got {total_flings} flings"
    );
}

#[test]
fn test_trampoline_scraps_pads_under_fresh_obstacles() {
    let mut game = trampoline_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.tramp_pads.clear();
    game.tramp_pads.insert(Point { x: 14, y: 14 });
    // Something else (e.g. a meteor) drops an obstacle on the pad
    game.obstacles.insert(Point { x: 14, y: 14 });

    game.tick_counter = 0; // tick 1: no shuffle
    game.state = GameState::Playing;
    game.update();

    assert!(
        !game.tramp_pads.contains(&Point { x: 14, y: 14 }),
        "a pad under a fresh obstacle must be scrapped on the next tick"
    );
}

#[test]
fn test_trampoline_bot_survives_the_launch_pads() {
    let mut game = trampoline_game(40, 40);
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
        "Autopilot should survive Trampoline mode (lives: {}, flings: {}, shuffles: {}, score: {})",
        game.lives,
        game.tramp_flings,
        game.tramp_shuffles,
        game.score
    );
}

#[test]
fn test_trampoline_persists_through_history_rewind() {
    let mut game = trampoline_game(30, 30);
    game.tramp_flings = 7;
    game.tramp_shuffles = 3;
    game.tramp_overcharge = 5;
    game.tramp_pads.insert(Point { x: 12, y: 12 });
    game.save_history_state();
    game.tramp_flings = 0;
    game.tramp_shuffles = 0;
    game.tramp_overcharge = 0;
    game.tramp_pads.clear();

    game.rewind_time();

    assert_eq!(
        game.tramp_flings, 7,
        "tramp_flings must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.tramp_shuffles, 3);
    assert_eq!(game.tramp_overcharge, 5);
    assert!(
        game.tramp_pads.contains(&Point { x: 12, y: 12 }),
        "tramp_pads must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_trampoline_persists_in_save_file() {
    let mut game = trampoline_game(30, 30);
    game.tramp_flings = 9;
    game.tramp_shuffles = 4;
    game.tramp_overcharge = 6;
    let path = std::env::temp_dir().join(format!(
        "trampoline_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"tramp_flings\":9")
            && json.contains("\"tramp_shuffles\":4")
            && json.contains("\"tramp_overcharge\":6")
            && json.contains("\"tramp_pads\""),
        "SaveState JSON must contain the trampoline fields, got: {}",
        &json[..json.len().min(200)]
    );
}
