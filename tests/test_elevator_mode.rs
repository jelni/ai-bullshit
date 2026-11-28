use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn elevator_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Elevator;
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

// Block the ambient portal/mine spawners so single-tick ride tests are
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
fn test_elevator_reset_seeds_shafts_fairly() {
    let game = elevator_game(30, 30);

    assert_eq!(game.elev_rides, 0);
    assert_eq!(game.elev_shuffles, 0);
    assert_eq!(game.elev_overcharge, 0);
    // shaft count = clamp(30*30/160, 2, 5) = 5
    assert_eq!(game.elev_shafts.len(), 5);
    assert_eq!(game.elev_cooldowns.len(), 5);
    assert!(game.elev_cooldowns.iter().all(|&cd| cd == 0));
    let spawn = Point { x: 15, y: 15 };
    let mut cells: Vec<Point> = Vec::new();
    for (a, b) in &game.elev_shafts {
        assert_eq!(a.x, b.x, "a shaft must share one column: ({a:?}, {b:?})");
        let gap = a.y.abs_diff(b.y);
        assert!(
            gap >= Game::ELEV_MIN_GAP && gap <= Game::ELEV_MAX_GAP,
            "shaft gap must be within [ELEV_MIN_GAP, ELEV_MAX_GAP]: {gap}"
        );
        for end in [*a, *b] {
            assert!(
                !game.obstacles.contains(&end),
                "a shaft end must never sit on an obstacle: {end:?}"
            );
            assert!(
                !game.snake.body_map.contains_key(&end),
                "a shaft end must never sit on the snake: {end:?}"
            );
            assert_ne!(end, game.food, "a shaft end must never sit on the food");
            assert!(
                end.x > 0 && end.x < game.width - 1 && end.y > 0 && end.y < game.height - 1,
                "a shaft end must be inside the board: {end:?}"
            );
            assert!(
                end.x.abs_diff(spawn.x).saturating_add(end.y.abs_diff(spawn.y)) >= 5,
                "a shaft end must spawn at least 5 cells from the spawn point: {end:?}"
            );
            assert!(!cells.contains(&end), "shaft ends must not overlap: {end:?}");
            cells.push(end);
        }
    }
}

#[test]
fn test_elevator_ride_lifts_head_to_other_end() {
    let mut game = elevator_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    // The snake heads right into shaft end a at (11,10)
    game.elev_shafts.clear();
    game.elev_cooldowns.clear();
    game.elev_shafts.push((Point { x: 11, y: 10 }, Point { x: 11, y: 17 }));
    game.elev_cooldowns.push(0);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 17 },
        "the lift must carry the head to the other shaft end (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.elev_rides, 1);
    assert_eq!(
        game.elev_cooldowns[0],
        Game::ELEV_COOLDOWN,
        "a ride must put the shaft on cooldown"
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a ride must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_elevator_ride_works_from_either_end() {
    let mut game = elevator_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(43);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    // Snake heads up into shaft end b at (11,17) from below
    game.snake = Snake::new(Point { x: 11, y: 18 });
    game.snake.direction = Direction::Up;
    game.snake.direction_queue.push_back(Direction::Up);
    game.elev_shafts.clear();
    game.elev_cooldowns.clear();
    game.elev_shafts.push((Point { x: 11, y: 10 }, Point { x: 11, y: 17 }));
    game.elev_cooldowns.push(0);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "entering the bottom end must lift the head to the top end (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.elev_rides, 1);
}

#[test]
fn test_elevator_ride_fizzles_when_destination_blocked() {
    let mut game = elevator_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(9);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.elev_shafts.clear();
    game.elev_cooldowns.clear();
    game.elev_shafts.push((Point { x: 11, y: 10 }, Point { x: 11, y: 17 }));
    game.elev_cooldowns.push(0);
    // A mine blocks the destination: the ride must not happen
    game.mines.insert(Point { x: 11, y: 17 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "with no fair destination the snake must simply rest on the walkable shaft cell (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.elev_rides, 0, "no ride may be counted");
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a lift must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_elevator_cooldown_blocks_then_allows_rides() {
    let mut game = elevator_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.elev_shafts.clear();
    game.elev_cooldowns.clear();
    game.elev_shafts.push((Point { x: 11, y: 10 }, Point { x: 11, y: 17 }));
    game.elev_cooldowns.push(3); // the shaft is recharging
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    // Tick 1: heading into the shaft while it recharges - no ride
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "a recharging shaft must not lift the head (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.elev_rides, 0);
    assert_eq!(game.elev_cooldowns[0], 2, "the cooldown must count down each tick");

    // Tick 2: cooldown expired - the ride fires
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
    game.elev_cooldowns[0] = 0;
    game.tick_counter = 1;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 17 },
        "an expired cooldown must allow the ride (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.elev_rides, 1);
    assert_eq!(game.elev_cooldowns[0], Game::ELEV_COOLDOWN);
}

#[test]
fn test_elevator_eating_food_overcharges_and_skips_cooldown() {
    let mut game = elevator_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.elev_shafts.clear();
    game.elev_cooldowns.clear();
    game.elev_shafts.push((Point { x: 20, y: 5 }, Point { x: 20, y: 12 }));
    game.elev_cooldowns.push(0);
    game.auto_pilot = false;

    // Tick 1: the snake eats the food at (11,10)
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 11, y: 10 };
    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert_eq!(
        game.elev_overcharge,
        Game::ELEV_OVERCHARGE_TICKS,
        "eating food must overcharge the lifts for ELEV_OVERCHARGE_TICKS"
    );

    // Tick 2: head into a shaft while overcharged - the ride fires even with
    // the shaft on cooldown, and the cooldown is not refreshed
    game.snake = Snake::new(Point { x: 10, y: 12 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
    game.elev_shafts.clear();
    game.elev_cooldowns.clear();
    game.elev_shafts.push((Point { x: 11, y: 12 }, Point { x: 11, y: 19 }));
    game.elev_cooldowns.push(4); // recharging, but overcharge ignores it
    game.food = Point { x: 25, y: 25 };
    game.tick_counter = 1;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 19 },
        "an overcharged lift must ride even while recharging (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.elev_rides, 1);
    assert_eq!(
        game.elev_cooldowns[0], 3,
        "an overcharged ride must not refresh the cooldown (only the per-tick countdown)"
    );
    assert!(
        game.elev_overcharge < Game::ELEV_OVERCHARGE_TICKS,
        "the overcharge timer must count down every tick"
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "an overcharged ride must never kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_elevator_shuffle_every_20_ticks_stays_fair() {
    let mut game = elevator_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = false;

    // update() increments tick_counter first, so 19 makes the next update run
    // tick 20. The relocation fires at the start of that tick, so the
    // fairness guarantees must be checked against the pre-update board state.
    for tick in [19_u32, 39, 59, 79, 99] {
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
        let old_cells: Vec<Point> = game
            .elev_shafts
            .iter()
            .flat_map(|(a, b)| [*a, *b])
            .collect();
        let shuffles_before = game.elev_shuffles;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        assert!(
            game.elev_shuffles > shuffles_before,
            "a lift relocation must happen on tick {}",
            tick + 1
        );
        for (a, b) in &game.elev_shafts {
            assert_eq!(a.x, b.x, "a relocated shaft must share one column");
            let gap = a.y.abs_diff(b.y);
            assert!(
                gap >= Game::ELEV_MIN_GAP && gap <= Game::ELEV_MAX_GAP,
                "a relocated shaft gap must be within [ELEV_MIN_GAP, ELEV_MAX_GAP]: {gap}"
            );
            for end in [*a, *b] {
                assert!(
                    !pre_obstacles.contains(&end),
                    "a relocated shaft end must not sit on an obstacle: {end:?}"
                );
                assert!(
                    !pre_body.contains(&end),
                    "a relocated shaft end must not sit on the snake: {end:?}"
                );
                assert_ne!(end, pre_next, "a relocated shaft end must not sit on the cell the snake is about to enter");
                assert_ne!(end, pre_food, "a relocated shaft end must not sit on the food");
                assert!(
                    !old_cells.contains(&end),
                    "a relocated shaft end must move off its old cell: {end:?}"
                );
                assert!(
                    end.x.abs_diff(pre_head.x).saturating_add(end.y.abs_diff(pre_head.y)) >= 3,
                    "a relocated shaft end must keep its distance from snake heads: {end:?}"
                );
            }
        }
    }
    assert_eq!(game.elev_shuffles, 5);
    assert_eq!(game.elev_shafts.len(), 5, "clamp(40*40/160, 2, 5) = 5");
}

#[test]
fn test_elevator_ride_never_unfair_over_many_ticks() {
    let mut game = elevator_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 35, y: 35 };
    pin_ambient_spawners(&mut game);
    let mut total_rides = 0u32;

    // Park the snake heading into a shaft end before every non-shuffle update
    // and make sure the machine can never lift it into a wall, a hazard, or
    // itself
    for tick in 0..60_u32 {
        if (tick + 1).is_multiple_of(20) {
            continue; // relocation ticks move the shafts
        }
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.elev_shafts.clear();
        game.elev_cooldowns.clear();
        game.elev_shafts.push((Point { x: 9, y: 15 }, Point { x: 9, y: 22 }));
        game.elev_cooldowns.push(0);
        game.tick_counter = tick;
        let rides_before = game.elev_rides;
        let lives_before = game.lives;
        game.update();
        let ridden = game.elev_rides > rides_before;
        total_rides += game.elev_rides.saturating_sub(rides_before);
        if game.lives == 0 {
            // An ambient hazard (e.g. a meteor) got the parked snake; top up
            // and carry on - the ride assertions below only apply to ticks
            // where a ride actually happened and the snake survived.
            game.lives = 3;
            game.state = GameState::Playing;
            continue;
        }
        let head = game.snake.head();
        if ridden && game.lives == lives_before {
            assert!(
                head == Point { x: 9, y: 22 },
                "tick {tick}: a ride must land exactly on the other shaft end, got {head:?}"
            );
            assert!(
                !game.obstacles.contains(&head),
                "tick {tick}: the ride landed on an obstacle at {head:?}"
            );
            assert!(
                head.x > 0 && head.x < game.width - 1 && head.y > 0 && head.y < game.height - 1,
                "tick {tick}: the ride left the board at {head:?}"
            );
        }
    }

    assert!(
        total_rides >= 10,
        "expected the lift to carry the parked snake on most ticks, got {total_rides} rides"
    );
}

#[test]
fn test_elevator_scraps_shafts_under_fresh_obstacles() {
    let mut game = elevator_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.elev_shafts.clear();
    game.elev_cooldowns.clear();
    game.elev_shafts.push((Point { x: 14, y: 14 }, Point { x: 14, y: 20 }));
    game.elev_cooldowns.push(0);
    // Something else (e.g. a meteor) drops an obstacle on one shaft end
    game.obstacles.insert(Point { x: 14, y: 14 });

    game.tick_counter = 0; // tick 1: no relocation
    game.state = GameState::Playing;
    game.update();

    assert!(
        game.elev_shafts.is_empty(),
        "a shaft with an end under a fresh obstacle must be scrapped on the next tick"
    );
    assert!(game.elev_cooldowns.is_empty());
}

#[test]
fn test_elevator_bot_survives_the_lifts() {
    let mut game = elevator_game(40, 40);
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
        "Autopilot should survive Elevator mode (lives: {}, rides: {}, shuffles: {}, score: {})",
        game.lives,
        game.elev_rides,
        game.elev_shuffles,
        game.score
    );
}

#[test]
fn test_elevator_persists_through_history_rewind() {
    let mut game = elevator_game(30, 30);
    game.elev_rides = 7;
    game.elev_shuffles = 3;
    game.elev_overcharge = 5;
    game.elev_shafts = vec![(Point { x: 5, y: 5 }, Point { x: 5, y: 12 })];
    game.elev_cooldowns = vec![2];
    game.save_history_state();
    game.elev_rides = 0;
    game.elev_shuffles = 0;
    game.elev_overcharge = 0;
    game.elev_shafts.clear();
    game.elev_cooldowns.clear();

    game.rewind_time();

    assert_eq!(
        game.elev_rides, 7,
        "elev_rides must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.elev_shuffles, 3);
    assert_eq!(game.elev_overcharge, 5);
    assert_eq!(game.elev_cooldowns, vec![2]);
    assert_eq!(
        game.elev_shafts,
        vec![(Point { x: 5, y: 5 }, Point { x: 5, y: 12 })],
        "elev_shafts must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_elevator_persists_in_save_file() {
    let mut game = elevator_game(30, 30);
    game.elev_rides = 9;
    game.elev_shuffles = 4;
    game.elev_overcharge = 6;
    let path = std::env::temp_dir().join(format!(
        "elevator_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"elev_rides\":9")
            && json.contains("\"elev_shuffles\":4")
            && json.contains("\"elev_overcharge\":6")
            && json.contains("\"elev_shafts\"")
            && json.contains("\"elev_cooldowns\""),
        "SaveState JSON must contain the elevator fields, got: {}",
        &json[..json.len().min(200)]
    );
}
