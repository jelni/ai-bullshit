use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn flux_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Flux;
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

// Block the ambient portal/mine spawners so single-tick pull tests are
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
fn test_flux_reset_seeds_coils_fairly() {
    let game = flux_game(30, 30);

    assert_eq!(game.flux_pulls, 0);
    assert_eq!(game.flux_shuffles, 0);
    assert_eq!(game.flux_flips, 0);
    assert!(game.flux_attract, "coils must start in attract polarity");
    // coil count = clamp(30*30/80, 3, 6) = 6
    assert_eq!(game.flux_coils.len(), 6);
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    for coil in &game.flux_coils {
        assert!(
            !game.obstacles.contains(coil),
            "a coil must never sit on an obstacle: {coil:?}"
        );
        assert!(
            !game.snake.body_map.contains_key(coil),
            "a coil must never sit on the snake: {coil:?}"
        );
        assert_ne!(*coil, game.food, "a coil must never sit on the food");
        assert!(
            coil.x.abs_diff(spawn.x).saturating_add(coil.y.abs_diff(spawn.y)) >= 5,
            "a coil must spawn at least 5 cells from the spawn point: {coil:?}"
        );
    }
}

#[test]
fn test_flux_attract_pulls_snake_towards_coil() {
    let mut game = flux_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.flux_coils.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    // The snake heads right to (11,10); the coil at (13,10) is within
    // MAGNET_RADIUS and attracts the head one more cell to (12,10)
    game.flux_coils.insert(Point { x: 13, y: 10 });
    game.flux_attract = true;
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0; // tick 1: no scheduled shuffle
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 1);
    assert_eq!(game.flux_pulls, 1, "the coil must pull the snake");
    assert_eq!(
        game.snake.head(),
        Point { x: 12, y: 10 },
        "an attracting coil must drag the head one cell towards itself (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a pull must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_flux_repel_pushes_snake_away_from_coil() {
    let mut game = flux_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.flux_coils.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    // The snake heads right to (11,10); the coil at (8,10) is within
    // MAGNET_RADIUS and repels the head one more cell to (12,10)
    game.flux_coils.insert(Point { x: 8, y: 10 });
    game.flux_attract = false;
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.flux_pulls, 1, "the coil must repel the snake");
    assert_eq!(
        game.snake.head(),
        Point { x: 12, y: 10 },
        "a repelling coil must drag the head one cell away from itself (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a repel must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_flux_ignores_magnets_out_of_range() {
    let mut game = flux_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.obstacles.clear();
    game.flux_coils.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    // 19 cells away from the next head (11,10): far outside MAGNET_RADIUS
    game.flux_coils.insert(Point { x: 20, y: 20 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "an out-of-range coil must not move the snake (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.flux_pulls, 0, "no pull may be counted");
}

#[test]
fn test_flux_pull_fizzles_when_destination_is_unfair() {
    let mut game = flux_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(9);
    game.obstacles.clear();
    game.flux_coils.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.flux_coils.insert(Point { x: 13, y: 10 });
    game.flux_attract = true;
    // The attract destination (12,10) is blocked by an obstacle, so the pull
    // must fizzle and the snake keeps its normal move onto (11,10)
    game.obstacles.insert(Point { x: 12, y: 10 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "with an unfair drag destination the snake must keep its normal move (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.flux_pulls, 0, "no pull may be counted");
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a coil must never pull the snake into a kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_flux_pull_never_lands_on_a_hazard() {
    let mut game = flux_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.obstacles.clear();
    game.flux_coils.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.flux_coils.insert(Point { x: 13, y: 10 });
    game.flux_attract = true;
    // A mine sits on the attract destination (12,10): the pull must fizzle
    game.mines.insert(Point { x: 12, y: 10 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "the pull must skip hazard cells (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.flux_pulls, 0);
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the pull must never land on a hazard (lives: {})",
        game.lives
    );
}

#[test]
fn test_flux_shuffle_every_24_ticks_stays_fair() {
    let mut game = flux_game(40, 40);
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
        let shuffles_before = game.flux_shuffles;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        assert!(
            game.flux_shuffles > shuffles_before,
            "a coil shuffle must happen on tick {}",
            tick + 1
        );
        for coil in &game.flux_coils {
            assert!(
                !pre_obstacles.contains(coil),
                "a relocated coil must not sit on an obstacle: {coil:?}"
            );
            assert!(
                !pre_body.contains(coil),
                "a relocated coil must not sit on the snake: {coil:?}"
            );
            assert_ne!(*coil, pre_next, "a relocated coil must not sit on the cell the snake is about to enter");
            assert_ne!(*coil, pre_food, "a relocated coil must not sit on the food");
            assert!(
                coil.x.abs_diff(pre_head.x).saturating_add(coil.y.abs_diff(pre_head.y)) >= 4,
                "a relocated coil must keep its distance from snake heads: {coil:?}"
            );
        }
    }
    assert_eq!(game.flux_shuffles, 5);
    assert_eq!(game.flux_coils.len(), 6, "clamp(40*40/80, 3, 6) = 6");
}

#[test]
fn test_flux_eating_food_flips_polarity() {
    let mut game = flux_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.flux_coils.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 11, y: 10 };
    game.flux_coils.insert(Point { x: 20, y: 20 });
    game.flux_attract = true;
    game.auto_pilot = false;

    // Tick 1: the snake eats the food at (11,10)
    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert!(
        !game.flux_attract,
        "eating food must flip the coils to repel polarity"
    );
    assert_eq!(game.flux_flips, 1, "the flip must be counted");

    // Tick 2: the snake eats again and the polarity flips back to attract
    game.snake = Snake::new(Point { x: 10, y: 12 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 11, y: 12 };
    game.tick_counter = 1;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 2);
    assert!(
        game.flux_attract,
        "eating food again must flip the coils back to attract"
    );
    assert_eq!(game.flux_flips, 2);
}

#[test]
fn test_flux_pull_never_unfair_over_many_ticks() {
    let mut game = flux_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 35, y: 35 };
    pin_ambient_spawners(&mut game);
    let mut total_pulls = 0u32;

    // Park the snake heading right next to a coil before every non-shuffle
    // update and make sure the machine can never pull it into a wall, a
    // hazard, or itself
    for tick in 0..60_u32 {
        if (tick + 1).is_multiple_of(Game::FLUX_PERIOD) {
            continue; // shuffle ticks relocate the coils
        }
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.flux_coils.clear();
        game.flux_coils.insert(Point { x: 11, y: 15 });
        game.flux_attract = true;
        game.tick_counter = tick;
        let pulls_before = game.flux_pulls;
        let lives_before = game.lives;
        game.update();
        let pulled = game.flux_pulls > pulls_before;
        total_pulls += game.flux_pulls.saturating_sub(pulls_before);
        if game.lives == 0 {
            // An ambient hazard (e.g. a meteor) got the parked snake; top up
            // and carry on - the pull assertions below only apply to ticks
            // where a pull actually happened and the snake survived.
            game.lives = 3;
            game.state = GameState::Playing;
            continue;
        }
        let head = game.snake.head();
        if pulled && game.lives == lives_before {
            assert!(
                head.y == 15 && head.x == 10,
                "tick {tick}: the attract pull from (11,15) must drag the head from (9,15) to (10,15), got {head:?}"
            );
            assert!(
                !game.obstacles.contains(&head),
                "tick {tick}: the pull landed on an obstacle at {head:?}"
            );
            assert!(
                head.x > 0 && head.x < game.width - 1 && head.y > 0 && head.y < game.height - 1,
                "tick {tick}: the pull left the board at {head:?}"
            );
        }
    }

    assert!(
        total_pulls >= 10,
        "expected the coil to pull the parked snake on most ticks, got {total_pulls} pulls"
    );
}

#[test]
fn test_flux_scraps_magnets_under_fresh_obstacles() {
    let mut game = flux_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.flux_coils.clear();
    game.flux_coils.insert(Point { x: 14, y: 14 });
    // Something else (e.g. a meteor) drops an obstacle on the coil
    game.obstacles.insert(Point { x: 14, y: 14 });

    game.tick_counter = 0; // tick 1: no shuffle
    game.state = GameState::Playing;
    game.update();

    assert!(
        !game.flux_coils.contains(&Point { x: 14, y: 14 }),
        "a coil under a fresh obstacle must be scrapped on the next tick"
    );
}

#[test]
fn test_flux_bot_survives_the_magnetic_fields() {
    let mut game = flux_game(40, 40);
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
        "Autopilot should survive Flux mode (lives: {}, pulls: {}, shuffles: {}, score: {})",
        game.lives,
        game.flux_pulls,
        game.flux_shuffles,
        game.score
    );
}

#[test]
fn test_flux_persists_through_history_rewind() {
    let mut game = flux_game(30, 30);
    game.flux_pulls = 7;
    game.flux_shuffles = 3;
    game.flux_flips = 2;
    game.flux_attract = false;
    game.flux_coils.insert(Point { x: 12, y: 12 });
    game.save_history_state();
    game.flux_pulls = 0;
    game.flux_shuffles = 0;
    game.flux_flips = 0;
    game.flux_attract = true;
    game.flux_coils.clear();

    game.rewind_time();

    assert_eq!(
        game.flux_pulls, 7,
        "flux_pulls must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.flux_shuffles, 3);
    assert_eq!(game.flux_flips, 2);
    assert!(!game.flux_attract);
    assert!(
        game.flux_coils.contains(&Point { x: 12, y: 12 }),
        "coils must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_flux_persists_in_save_file() {
    let mut game = flux_game(30, 30);
    game.flux_pulls = 9;
    game.flux_shuffles = 4;
    game.flux_flips = 2;
    game.flux_attract = false;
    let path = std::env::temp_dir().join(format!(
        "flux_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"flux_pulls\":9")
            && json.contains("\"flux_shuffles\":4")
            && json.contains("\"flux_flips\":2")
            && json.contains("\"flux_attract\":false")
            && json.contains("\"flux_coils\""),
        "SaveState JSON must contain the coil fields, got: {}",
        &json[..json.len().min(200)]
    );
}
