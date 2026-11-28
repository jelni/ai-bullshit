use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn erosion_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Erosion;
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
fn test_erosion_reset_starts_clean() {
    let game = erosion_game(30, 30);

    assert!(game.erode_timers.is_empty());
    assert!(game.erode_holes.is_empty());
    assert_eq!(game.erode_crumbles, 0);
    assert_eq!(game.erode_tremors, 0);
}

#[test]
fn test_erosion_stepping_on_a_cell_starts_its_timer() {
    let mut game = erosion_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    // manage_erosion refreshes the pre-move head cell every tick
    assert_eq!(
        game.erode_timers.get(&Point { x: 10, y: 10 }),
        Some(&Game::ERODE_TICKS),
        "the cell the head stands on must carry a full ERODE_TICKS timer"
    );

    game.tick_counter = 1;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 12, y: 10 },
        "the snake must have moved right (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.erode_timers.contains_key(&Point { x: 11, y: 10 }),
        "the cell the head entered must be weakening: {:?}",
        game.erode_timers
    );
}

#[test]
fn test_erosion_cell_crumbles_into_a_hole_after_its_timer() {
    let mut game = erosion_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    // A weakened cell far away from the snake, about to crumble
    game.erode_timers.insert(Point { x: 18, y: 18 }, 1);

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    let p = Point { x: 18, y: 18 };
    assert_eq!(game.erode_crumbles, 1, "the expired cell must crumble");
    assert!(
        game.erode_holes.contains(&p),
        "the crumbled cell must become a hole"
    );
    assert!(
        game.obstacles.contains(&p),
        "a hole must be mirrored into obstacles (lethal + A* hard constraint)"
    );
    assert!(
        !game.erode_timers.contains_key(&p),
        "the timer must be consumed by the crumble"
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a distant crumble must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_erosion_never_crumbles_under_a_snake_or_its_next_cell() {
    let mut game = erosion_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    // Both the head cell and the cell the snake is about to enter are at 1
    // tick from crumbling: neither may crumble this tick.
    game.erode_timers.insert(Point { x: 10, y: 10 }, 1);
    game.erode_timers.insert(Point { x: 11, y: 10 }, 1);
    // A body cell (below the head) is also about to crumble
    game.erode_timers.insert(Point { x: 10, y: 11 }, 1);

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.erode_crumbles, 0,
        "no cell occupied by a snake or about to be entered may crumble"
    );
    for p in [
        Point { x: 10, y: 10 },
        Point { x: 11, y: 10 },
        Point { x: 10, y: 11 },
    ] {
        assert!(
            !game.erode_holes.contains(&p),
            "fairness violation: hole opened on {p:?}"
        );
    }
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the floor must never crumble under the snake (lives: {})",
        game.lives
    );
    // On the next tick the new head cell is refreshed to a full timer
    game.tick_counter = 1;
    game.state = GameState::Playing;
    game.update();
    assert_eq!(
        game.erode_timers.get(&Point { x: 11, y: 10 }),
        Some(&Game::ERODE_TICKS),
        "the new head cell must be refreshed to ERODE_TICKS (got {:?})",
        game.erode_timers.get(&Point { x: 11, y: 10 })
    );
    assert!(
        !game.erode_holes.contains(&Point { x: 11, y: 10 }),
        "the head cell must still not be a hole"
    );
}

#[test]
fn test_erosion_tremor_every_6th_tick_opens_a_fair_hole() {
    let mut game = erosion_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 20, y: 20 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 35, y: 35 };
    game.auto_pilot = false;

    for (tick, expected_tremors) in [(5_u32, 1_u32), (11, 2), (17, 3)] {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 20, y: 20 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.food = Point { x: 35, y: 35 };
        game.tick_counter = tick;
        let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
        let pre_next =
            snake_game::game::Game::calculate_next_head_dir(game.snake.head(), game.snake.direction);
        let pre_food = game.food;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        assert_eq!(
            game.erode_tremors, expected_tremors,
            "a tremor must fire on tick {}",
            tick + 1
        );
        for hole in &game.erode_holes {
            assert!(
                !pre_body.contains(hole),
                "a tremor hole must never open on a snake body: {hole:?}"
            );
            assert_ne!(
                *hole, pre_next,
                "a tremor hole must never open on the cell the snake is about to enter"
            );
            assert_ne!(*hole, pre_food, "a tremor hole must never open on the food");
            assert!(
                game.obstacles.contains(hole),
                "a tremor hole must be mirrored into obstacles: {hole:?}"
            );
            assert!(
                hole.x > 0 && hole.x < game.width - 1 && hole.y > 0 && hole.y < game.height - 1,
                "a tremor hole must stay in bounds: {hole:?}"
            );
        }
    }
}

#[test]
fn test_erosion_ground_resettles_at_tick_200() {
    let mut game = erosion_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    // Simulate accumulated damage
    for p in [Point { x: 15, y: 15 }, Point { x: 16, y: 16 }] {
        game.erode_holes.insert(p);
        game.obstacles.insert(p);
    }
    game.erode_timers.insert(Point { x: 14, y: 14 }, 3);

    game.tick_counter = 199; // update() increments first, so this runs tick 200
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.erode_holes.is_empty(),
        "tick 200 must refill every hole: {:?}",
        game.erode_holes
    );
    assert!(
        game.erode_timers.is_empty(),
        "tick 200 must clear every weakened cell"
    );
    assert!(
        !game.obstacles.contains(&Point { x: 15, y: 15 })
            && !game.obstacles.contains(&Point { x: 16, y: 16 }),
        "refilled holes must leave the obstacle set"
    );
}

#[test]
fn test_erosion_eating_food_reinforces_adjacent_floor() {
    let mut game = erosion_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 11, y: 10 }; // the snake eats on tick 1
    game.auto_pilot = false;
    // Adjacent to the eaten cell (11,10): a hole at (11,11) and a weakened
    // cell at (11,9)
    game.erode_holes.insert(Point { x: 11, y: 11 });
    game.obstacles.insert(Point { x: 11, y: 11 });
    game.erode_timers.insert(Point { x: 11, y: 9 }, 5);

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten");
    assert!(
        !game.erode_holes.contains(&Point { x: 11, y: 11 }),
        "eating food must heal the adjacent hole"
    );
    assert!(
        !game.obstacles.contains(&Point { x: 11, y: 11 }),
        "the healed hole must leave the obstacle set"
    );
    assert!(
        !game.erode_timers.contains_key(&Point { x: 11, y: 9 }),
        "eating food must reinforce the adjacent weakened cell"
    );
}

#[test]
fn test_erosion_is_safe_final_p_rejects_holes_and_crumbling_cells() {
    let mut game = erosion_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.obstacles.clear();
    game.mines.clear();
    game.auto_pilot = false;
    // Park the snake far away from the tested cell
    game.snake = Snake::new(Point { x: 25, y: 25 });
    let p = Point { x: 15, y: 15 };

    assert!(
        game.is_safe_final_p(p, 0, 1),
        "a plain empty cell must be safe"
    );

    game.erode_holes.insert(p);
    assert!(
        !game.is_safe_final_p(p, 0, 1),
        "a hole must be rejected by the verified-safe check"
    );
    game.erode_holes.clear();

    game.erode_timers.insert(p, 10);
    assert!(
        game.is_safe_final_p(p, 5, 1),
        "a cell that crumbles in 10 ticks is still safe at step 5"
    );
    assert!(
        !game.is_safe_final_p(p, 10, 1),
        "a cell that crumbles in 10 ticks is unsafe at step 10"
    );
    assert!(
        !game.is_safe_final_p(p, 20, 1),
        "a cell that crumbles in 10 ticks is unsafe at step 20"
    );
}

#[test]
fn test_erosion_no_hole_ever_opens_on_an_occupied_or_next_cell() {
    let mut game = erosion_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 35, y: 35 };
    pin_ambient_spawners(&mut game);

    for tick in 0..60_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = tick;
        let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
        let pre_next =
            snake_game::game::Game::calculate_next_head_dir(game.snake.head(), game.snake.direction);
        game.update();
        if game.lives == 0 {
            // An ambient hazard got the parked snake; top up and carry on -
            // only the hole-fairness invariant is asserted here.
            game.lives = 3;
            game.state = GameState::Playing;
            continue;
        }
        for hole in &game.erode_holes {
            assert!(
                !pre_body.contains(hole),
                "tick {tick}: a hole opened on a snake body cell {hole:?}"
            );
            assert_ne!(
                *hole, pre_next,
                "tick {tick}: a hole opened on the cell the snake was about to enter"
            );
        }
        for p in game.snake.body.iter() {
            assert!(
                !game.erode_holes.contains(p),
                "tick {tick}: a hole exists under the snake at {p:?}"
            );
        }
    }

    assert!(
        game.erode_crumbles + game.erode_tremors > 0,
        "the floor must actually erode over 60 ticks (crumbles: {}, tremors: {})",
        game.erode_crumbles,
        game.erode_tremors
    );
}

#[test]
fn test_erosion_bot_survives_the_crumbling_floor() {
    let mut game = erosion_game(40, 40);
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
        "Autopilot should survive Erosion mode (lives: {}, crumbles: {}, tremors: {}, score: {})",
        game.lives,
        game.erode_crumbles,
        game.erode_tremors,
        game.score
    );
}

#[test]
fn test_erosion_persists_through_history_rewind() {
    let mut game = erosion_game(30, 30);
    game.erode_crumbles = 7;
    game.erode_tremors = 3;
    game.erode_holes.insert(Point { x: 12, y: 12 });
    game.erode_timers.insert(Point { x: 13, y: 13 }, 9);
    game.save_history_state();
    game.erode_crumbles = 0;
    game.erode_tremors = 0;
    game.erode_holes.clear();
    game.erode_timers.clear();

    game.rewind_time();

    assert_eq!(
        game.erode_crumbles, 7,
        "erode_crumbles must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.erode_tremors, 3);
    assert!(
        game.erode_holes.contains(&Point { x: 12, y: 12 }),
        "erode_holes must survive a time-rewind through HistoryState"
    );
    assert_eq!(
        game.erode_timers.get(&Point { x: 13, y: 13 }),
        Some(&9),
        "erode_timers must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_erosion_persists_in_save_file() {
    let mut game = erosion_game(30, 30);
    game.erode_crumbles = 9;
    game.erode_tremors = 4;
    game.erode_holes.insert(Point { x: 12, y: 12 });
    let path = std::env::temp_dir().join(format!(
        "erosion_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"erode_crumbles\":9")
            && json.contains("\"erode_tremors\":4")
            && json.contains("\"erode_holes\"")
            && json.contains("\"erode_timers\""),
        "SaveState JSON must contain the erosion fields, got: {}",
        &json[..json.len().min(200)]
    );
}
