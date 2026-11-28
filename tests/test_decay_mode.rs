use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn decay_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Decay;
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
fn test_decay_reset_seeds_cracks_fairly() {
    let game = decay_game(30, 30);

    assert_eq!(game.decay_collapses, 0);
    assert_eq!(game.decay_tremors, 0);
    assert_eq!(game.decay_repairs, 0);
    assert!(game.decay_pits.is_empty());
    // seed count = clamp(30*30/100, 2, 6) = 6
    assert_eq!(game.decay_cracks.len(), 6);
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    for (crack, timer) in &game.decay_cracks {
        assert_eq!(*timer, Game::DECAY_DELAY);
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
fn test_decay_head_cell_cracks_each_tick() {
    let mut game = decay_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0; // tick 1: no tremor
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 1);
    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "the snake must move right (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.decay_cracks.contains_key(&Point { x: 10, y: 10 }),
        "the head cell the snake stood on must be cracked: {:?}",
        game.decay_cracks
    );

    // Tick 2 cracks the new head cell
    game.snake.direction_queue.push_back(Direction::Right);
    game.tick_counter = 1;
    game.state = GameState::Playing;
    game.update();
    assert!(
        game.decay_cracks.contains_key(&Point { x: 11, y: 10 }),
        "the new head cell must be cracked on the next tick: {:?}",
        game.decay_cracks
    );
}

#[test]
fn test_decay_crack_collapses_into_a_pit() {
    let mut game = decay_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    // A crack one tick from collapsing on a far, unoccupied cell
    game.decay_cracks.clear();
    game.decay_cracks.insert(Point { x: 20, y: 20 }, 1);

    game.tick_counter = 0; // tick 1: no tremor
    game.state = GameState::Playing;
    game.update();

    assert!(
        game.decay_pits.contains(&Point { x: 20, y: 20 }),
        "the expired crack must collapse into a pit: {:?}",
        game.decay_pits
    );
    assert!(
        game.obstacles.contains(&Point { x: 20, y: 20 }),
        "a pit must be mirrored into obstacles so it blocks movement"
    );
    assert!(
        !game.decay_cracks.contains_key(&Point { x: 20, y: 20 }),
        "the collapsed cell must leave the crack map"
    );
    assert_eq!(game.decay_collapses, 1);
}

#[test]
fn test_decay_collapse_is_deferred_on_the_next_head_cell() {
    let mut game = decay_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    // A crack about to expire exactly on the cell the snake is about to enter
    game.decay_cracks.clear();
    game.decay_cracks.insert(Point { x: 11, y: 10 }, 1);

    game.tick_counter = 0; // tick 1: no tremor
    game.state = GameState::Playing;
    game.update();

    assert!(
        !game.decay_pits.contains(&Point { x: 11, y: 10 }),
        "a crack must never collapse on the cell a snake is about to enter"
    );
    assert_eq!(
        game.decay_collapses, 0,
        "the collapse must be deferred while the next-head cell is cracked"
    );
    assert!(
        game.decay_cracks.contains_key(&Point { x: 11, y: 10 }),
        "the deferred crack must stay in the crack map"
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the deferred collapse must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_decay_tremor_every_6th_tick_stays_fair() {
    let mut game = decay_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = false;

    // update() increments tick_counter first, so 5 makes the next update run
    // tick 6. The tremor fires at the start of that tick, so the fairness
    // guarantees must be checked against the pre-update board state.
    for tick in [5_u32, 11, 17, 23, 29] {
        game.state = GameState::Playing;
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
        let tremors_before = game.decay_tremors;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        assert!(
            game.decay_tremors > tremors_before,
            "a tremor must happen on tick {}",
            tick + 1
        );
        for crack in game.decay_cracks.keys() {
            assert!(
                !pre_obstacles.contains(crack),
                "a tremor crack must not sit on an obstacle: {crack:?}"
            );
            // The head cell itself is cracked by the head-marking rule (and
            // deferred while occupied), so only the rest of the body is fair game
            if *crack != pre_head {
                assert!(
                    !pre_body.contains(crack),
                    "a tremor crack must not sit on the snake: {crack:?}"
                );
            }
            assert_ne!(
                *crack, pre_next,
                "a tremor crack must not sit on the cell the snake is about to enter"
            );
            assert_ne!(*crack, pre_food, "a tremor crack must not sit on the food");
        }
    }
    assert_eq!(game.decay_tremors, 5);
}

#[test]
fn test_decay_renewal_at_tick_200_refills_the_floor() {
    let mut game = decay_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    game.decay_cracks.clear();
    game.decay_cracks.insert(Point { x: 18, y: 18 }, 7);
    game.decay_pits.insert(Point { x: 14, y: 14 });
    game.obstacles.insert(Point { x: 14, y: 14 });

    game.tick_counter = 199; // the next update runs tick 200
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.decay_pits.is_empty(),
        "tick 200 must fill every pit back in: {:?}",
        game.decay_pits
    );
    assert!(
        game.decay_cracks.is_empty(),
        "tick 200 must seal every remaining crack: {:?}",
        game.decay_cracks
    );
    assert!(
        !game.obstacles.contains(&Point { x: 14, y: 14 }),
        "a refilled pit must leave the obstacle set"
    );
}

#[test]
fn test_decay_eating_food_patches_the_floor() {
    let mut game = decay_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 11, y: 10 };
    game.auto_pilot = false;
    // A crack and a pit adjacent (Manhattan <= 1) to the eaten cell (11,10)
    game.decay_cracks.clear();
    game.decay_cracks.insert(Point { x: 11, y: 11 }, 3);
    game.decay_pits.insert(Point { x: 12, y: 10 });
    game.obstacles.insert(Point { x: 12, y: 10 });

    game.tick_counter = 0; // tick 1: the snake eats the food at (11,10)
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert!(
        !game.decay_cracks.contains_key(&Point { x: 11, y: 11 }),
        "eating food must seal the adjacent crack"
    );
    assert!(
        !game.decay_pits.contains(&Point { x: 12, y: 10 }),
        "eating food must fill the adjacent pit"
    );
    assert!(
        !game.obstacles.contains(&Point { x: 12, y: 10 }),
        "a filled pit must leave the obstacle set"
    );
    assert!(
        !game.decay_cracks.contains_key(&Point { x: 10, y: 10 }),
        "the crack on the cell the snake just left is patched too"
    );
    // (11,11) crack + (12,10) pit + (10,10) head-cell crack
    assert_eq!(game.decay_repairs, 3);
}

#[test]
fn test_decay_never_collapses_under_a_snake_over_many_ticks() {
    let mut game = decay_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 35, y: 35 };
    pin_ambient_spawners(&mut game);

    for tick in 0..120_u32 {
        game.state = GameState::Playing;
        // Park the snake deterministically so the pre-update state is exactly
        // the state the collapse countdown sees
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = tick;
        game.update();
        if game.lives == 0 {
            // An ambient hazard (e.g. a meteor) got the parked snake; top up
            // and carry on - the invariant below only concerns decay pits
            game.lives = 3;
            game.state = GameState::Playing;
            continue;
        }
        let head = game.snake.head();
        assert!(
            !game.decay_pits.contains(&head),
            "tick {tick}: a pit must never form on the snake's head at {head:?}"
        );
        for seg in &game.snake.body {
            assert!(
                !game.decay_pits.contains(seg),
                "tick {tick}: a pit must never form on the snake's body at {seg:?}"
            );
        }
    }
}

#[test]
fn test_decay_autopilot_rejects_soon_collapsing_cracks() {
    let mut game = decay_game(30, 30);
    game.obstacles.clear();
    game.decay_cracks.clear();
    game.decay_cracks.insert(Point { x: 12, y: 10 }, 5);

    assert!(
        !game.is_safe_final_p(Point { x: 12, y: 10 }, 5, 1),
        "the autopilot must reject a cracked cell due to collapse within the arrival horizon"
    );
    assert!(
        !game.is_safe_final_p(Point { x: 12, y: 10 }, 9, 1),
        "a longer horizon must still reject the soon-collapsing crack"
    );
    assert!(
        game.is_safe_final_p(Point { x: 12, y: 10 }, 1, 1),
        "a crack that survives past the arrival step is walkable (occupied cells never collapse)"
    );
    assert!(
        game.is_safe_final_p(Point { x: 12, y: 12 }, 1, 1),
        "an intact cell far from hazards must stay safe"
    );
}

#[test]
fn test_decay_bot_survives_the_collapsing_floor() {
    let mut game = decay_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;
    pin_ambient_spawners(&mut game);

    for _ in 0..200 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
    }

    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "Autopilot should survive Decay mode (lives: {}, collapses: {}, tremors: {}, score: {})",
        game.lives,
        game.decay_collapses,
        game.decay_tremors,
        game.score
    );
}

#[test]
fn test_decay_persists_through_history_rewind() {
    let mut game = decay_game(30, 30);
    game.decay_collapses = 7;
    game.decay_tremors = 3;
    game.decay_repairs = 2;
    game.decay_cracks.insert(Point { x: 12, y: 12 }, 9);
    game.decay_pits.insert(Point { x: 13, y: 13 });
    game.save_history_state();
    game.decay_collapses = 0;
    game.decay_tremors = 0;
    game.decay_repairs = 0;
    game.decay_cracks.clear();
    game.decay_pits.clear();

    game.rewind_time();

    assert_eq!(
        game.decay_collapses, 7,
        "decay_collapses must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.decay_tremors, 3);
    assert_eq!(game.decay_repairs, 2);
    assert!(
        game.decay_cracks.contains_key(&Point { x: 12, y: 12 }),
        "decay_cracks must survive a time-rewind through HistoryState"
    );
    assert!(
        game.decay_pits.contains(&Point { x: 13, y: 13 }),
        "decay_pits must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_decay_persists_in_save_file() {
    let mut game = decay_game(30, 30);
    game.decay_collapses = 9;
    game.decay_tremors = 4;
    game.decay_repairs = 1;
    let path = std::env::temp_dir().join(format!(
        "decay_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"decay_collapses\":9")
            && json.contains("\"decay_tremors\":4")
            && json.contains("\"decay_repairs\":1")
            && json.contains("\"decay_pits\"")
            && json.contains("\"decay_cracks\""),
        "SaveState JSON must contain the decay fields, got: {}",
        &json[..json.len().min(200)]
    );
}
