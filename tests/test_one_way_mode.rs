use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn oneway_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::OneWay;
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
fn test_oneway_reset_seeds_signs_fairly() {
    let game = oneway_game(30, 30);

    assert_eq!(game.oneway_shuffles, 0);
    assert_eq!(game.oneway_flips, 0);
    assert_eq!(game.weather, Weather::Clear, "the skies stay clear");
    // sign count = clamp(30*30/60, 4, 8) = 8
    assert_eq!(game.oneway_signs.len(), 8);
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    for (sign, arrow) in &game.oneway_signs {
        assert!(
            !game.obstacles.contains(sign),
            "a sign must never sit on an obstacle: {sign:?}"
        );
        assert!(
            !game.snake.body_map.contains_key(sign),
            "a sign must never sit on the snake: {sign:?}"
        );
        assert_ne!(*sign, game.food, "a sign must never sit on the food");
        assert!(
            matches!(
                arrow,
                Direction::Up | Direction::Down | Direction::Left | Direction::Right
            ),
            "a sign must carry a valid arrow: {arrow:?}"
        );
        assert!(
            sign.x.abs_diff(spawn.x).saturating_add(sign.y.abs_diff(spawn.y)) >= 5,
            "a sign must spawn at least 5 cells from the spawn point: {sign:?}"
        );
    }
}

#[test]
fn test_oneway_entering_with_the_arrow_is_allowed() {
    let mut game = oneway_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.oneway_signs.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    // The sign at (11,10) points right, and the snake moves right: entry allowed
    game.oneway_signs.insert(Point { x: 11, y: 10 }, Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0; // tick 1: no scheduled shuffle
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "entering a sign along its arrow must be allowed (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "crossing a sign with the arrow must never kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_oneway_entering_against_the_arrow_kills() {
    let mut game = oneway_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.oneway_signs.clear();
    pin_ambient_spawners(&mut game);
    game.lives = 1;
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    // The sign at (11,10) points left, and the snake moves right: wall hit
    game.oneway_signs.insert(Point { x: 11, y: 10 }, Direction::Left);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0; // tick 1: no scheduled shuffle
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.lives, 0,
        "entering a sign against its arrow must kill like a wall (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state != GameState::Playing,
        "the snake must be dead after driving into a sign the wrong way"
    );
}

#[test]
fn test_oneway_shuffle_every_24_ticks_stays_fair() {
    let mut game = oneway_game(40, 40);
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
        let shuffles_before = game.oneway_shuffles;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        assert!(
            game.oneway_shuffles > shuffles_before,
            "a sign shuffle must happen on tick {}",
            tick + 1
        );
        for sign in game.oneway_signs.keys() {
            assert!(
                !pre_obstacles.contains(sign),
                "a relocated sign must not sit on an obstacle: {sign:?}"
            );
            assert!(
                !pre_body.contains(sign),
                "a relocated sign must not sit on the snake: {sign:?}"
            );
            assert_ne!(
                *sign, pre_next,
                "a relocated sign must not sit on the cell the snake is about to enter"
            );
            assert_ne!(*sign, pre_food, "a relocated sign must not sit on the food");
            assert!(
                sign.x.abs_diff(pre_head.x).saturating_add(sign.y.abs_diff(pre_head.y)) >= 4,
                "a relocated sign must keep its distance from snake heads: {sign:?}"
            );
        }
    }
    assert_eq!(game.oneway_shuffles, 5);
    assert_eq!(game.oneway_signs.len(), 8, "clamp(40*40/60, 4, 8) = 8");
}

#[test]
fn test_oneway_eating_food_spins_one_sign() {
    let mut game = oneway_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.oneway_signs.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 11, y: 10 };
    game.oneway_signs.insert(Point { x: 20, y: 20 }, Direction::Up);
    game.auto_pilot = false;

    game.tick_counter = 0; // tick 1: no scheduled shuffle
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert_eq!(game.oneway_flips, 1, "eating food must spin one sign");
    assert_eq!(
        game.oneway_signs.get(&Point { x: 20, y: 20 }),
        Some(&Direction::Right),
        "the spun sign must rotate 90 degrees clockwise (Up -> Right)"
    );
}

#[test]
fn test_oneway_scraps_signs_under_fresh_obstacles() {
    let mut game = oneway_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.oneway_signs.clear();
    game.oneway_signs.insert(Point { x: 14, y: 14 }, Direction::Up);
    // Something else (e.g. a meteor) drops an obstacle on the sign
    game.obstacles.insert(Point { x: 14, y: 14 });

    game.tick_counter = 0; // tick 1: no shuffle
    game.state = GameState::Playing;
    game.update();

    assert!(
        !game.oneway_signs.contains_key(&Point { x: 14, y: 14 }),
        "a sign under a fresh obstacle must be scrapped on the next tick"
    );
}

#[test]
fn test_oneway_bot_survives_the_alley() {
    let mut game = oneway_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;

    for _ in 0..200 {
        game.update();
        if game.state == GameState::Playing
            && let Some(arrow) = game.oneway_signs.get(&game.snake.head())
        {
            assert_eq!(
                *arrow, game.snake.direction,
                "the autopilot must only ever enter a sign along its arrow (head: {:?})",
                game.snake.head()
            );
        }
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
    }

    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "Autopilot should survive One-Way Alley (lives: {}, shuffles: {}, score: {})",
        game.lives,
        game.oneway_shuffles,
        game.score
    );
}

#[test]
fn test_oneway_persists_through_history_rewind() {
    let mut game = oneway_game(30, 30);
    game.oneway_shuffles = 7;
    game.oneway_flips = 3;
    game.oneway_signs.insert(Point { x: 12, y: 12 }, Direction::Down);
    game.save_history_state();
    game.oneway_shuffles = 0;
    game.oneway_flips = 0;
    game.oneway_signs.clear();

    game.rewind_time();

    assert_eq!(
        game.oneway_shuffles, 7,
        "oneway_shuffles must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.oneway_flips, 3);
    assert_eq!(
        game.oneway_signs.get(&Point { x: 12, y: 12 }),
        Some(&Direction::Down),
        "oneway_signs must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_oneway_persists_in_save_file() {
    let mut game = oneway_game(30, 30);
    game.oneway_shuffles = 9;
    game.oneway_flips = 4;
    let path = std::env::temp_dir().join(format!(
        "oneway_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"oneway_shuffles\":9")
            && json.contains("\"oneway_flips\":4")
            && json.contains("\"oneway_signs\""),
        "SaveState JSON must contain the one-way fields, got: {}",
        &json[..json.len().min(200)]
    );
}
