use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn fortress_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Fortress;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_fortress_reset_clears_state() {
    let game = fortress_game(30, 30);

    assert!(game.fortress_walls.is_empty());
    assert_eq!(game.fortress_depth, 0);
    assert_eq!(game.fortress_rings, 0);
}

#[test]
fn test_fortress_closes_in_every_6_ticks() {
    let mut game = fortress_game(30, 30);
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must erect the next fortress ring.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let rings_before = game.fortress_rings;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.fortress_rings > rings_before,
            "a fortress ring must be erected on tick {} (rings before: {}, after: {})",
            tick + 1,
            rings_before,
            game.fortress_rings
        );
        assert!(
            !game.fortress_walls.is_empty(),
            "erecting a ring must add wall cells on tick {}",
            tick + 1
        );
        for wall in &game.fortress_walls {
            assert!(
                game.obstacles.contains(wall),
                "fortress walls must be mirrored into obstacles"
            );
        }
    }
    assert_eq!(game.fortress_rings, 6);
    assert_eq!(game.fortress_depth, 6);
}

#[test]
fn test_fortress_never_walls_snake_or_next_cell() {
    let mut game = fortress_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.fortress_walls.clear();
    game.fortress_depth = 0;
    game.fortress_rings = 0;
    game.snake = Snake::new(Point { x: 2, y: 2 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = false;

    // Tick 6 erects ring depth 1, which includes cells next to the snake.
    game.tick_counter = 5;
    game.update();

    assert_eq!(game.fortress_rings, 1);
    let next = Point { x: 3, y: 2 };
    assert!(
        !game.fortress_walls.contains(&next),
        "a ring must never wall the cell the snake is about to enter"
    );
    for seg in &game.snake.body {
        assert!(
            !game.fortress_walls.contains(seg),
            "a ring must never wall the snake's body"
        );
    }
    assert!(
        !game.fortress_walls.contains(&game.food),
        "a ring must never wall the food"
    );
}

#[test]
fn test_fortress_fairness_over_many_ticks() {
    let mut game = fortress_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 25, y: 25 };
    let lives_before = game.lives;

    // Deterministic: park the snake at (8, 15) heading right before every
    // scheduled ring, so the ring must never wall its next cell (9, 15) or body.
    let next = Point { x: 9, y: 15 };
    for k in 1..=10_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = 6 * k - 1;
        let rings_before = game.fortress_rings;
        game.update();
        if game.fortress_rings > rings_before {
            assert!(
                !game.fortress_walls.contains(&next),
                "ring {} walled the cell the snake was about to enter ({next:?})",
                game.fortress_depth
            );
            for seg in &game.snake.body {
                assert!(
                    !game.fortress_walls.contains(seg),
                    "ring {} walled the snake's body at {seg:?}",
                    game.fortress_depth
                );
            }
        }
    }

    assert_eq!(
        game.lives, lives_before,
        "the snake must survive the closing walls (rings: {})",
        game.fortress_rings
    );
    assert!(
        game.fortress_rings >= 7,
        "expected the rings to close in up to the depth cap (min(30, 30) / 4 = 7), got {}",
        game.fortress_rings
    );
}

#[test]
fn test_fortress_walls_retract_at_tick_200() {
    let mut game = fortress_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.auto_pilot = true;

    // Erect a few rings first.
    for tick in [5_u32, 11, 17] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
    }
    assert!(!game.fortress_walls.is_empty());
    assert!(game.fortress_depth > 0);

    // Tick 200 of the 250-tick cycle retracts all walls.
    game.state = GameState::Playing;
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.fortress_walls.is_empty(),
        "all fortress walls must retract at tick 200"
    );
    assert_eq!(game.fortress_depth, 0);
    for wall in &game.obstacles {
        let _ = wall;
    }
    assert!(
        game.fortress_rings >= 3,
        "the ring counter keeps counting through a retraction"
    );
}

#[test]
fn test_fortress_depth_is_capped() {
    let mut game = fortress_game(12, 12);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(9);
    game.auto_pilot = true;
    // max_depth = min(12, 12) / 4 = 3
    for tick in (5..180).step_by(6) {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
        if game.lives == 0 {
            break;
        }
        assert!(
            game.fortress_depth <= 3,
            "fortress depth must be capped at 3 on a 12x12 board, got {}",
            game.fortress_depth
        );
    }
}

#[test]
fn test_fortress_eating_food_breaches_adjacent_wall() {
    let mut game = fortress_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.fortress_walls.clear();
    game.fortress_depth = 0;
    game.fortress_rings = 0;
    let wall = Point { x: 7, y: 5 };
    game.fortress_walls.insert(wall);
    game.obstacles.insert(wall);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.auto_pilot = false;

    // Tick 5 is not a multiple of 6 and not 200, so no ring interferes here.
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.fortress_walls.contains(&wall),
        "Eating food in Fortress mode should breach the adjacent wall"
    );
    assert!(
        !game.obstacles.contains(&wall),
        "The breached wall must also leave obstacles"
    );
}

#[test]
fn test_fortress_bot_survives_the_closing_walls() {
    let mut game = fortress_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;

    for _ in 0..120 {
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
        "Autopilot should survive the fortress (lives: {}, rings: {}, score: {})",
        game.lives,
        game.fortress_rings,
        game.score
    );
    assert!(
        game.fortress_rings >= 5,
        "expected several rings over 120 ticks, got {}",
        game.fortress_rings
    );
    assert!(
        game.fortress_depth <= 10,
        "fortress depth must stay within the cap (min(40, 40) / 4 = 10), got {}",
        game.fortress_depth
    );
}

#[test]
fn test_fortress_persists_through_history_rewind() {
    let mut game = fortress_game(30, 30);
    game.fortress_rings = 7;
    game.fortress_depth = 3;
    game.save_history_state();
    game.fortress_rings = 0;
    game.fortress_depth = 0;

    game.rewind_time();

    assert_eq!(
        game.fortress_rings, 7,
        "fortress_rings must survive a time-rewind through HistoryState"
    );
    assert_eq!(
        game.fortress_depth, 3,
        "fortress_depth must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_fortress_persists_in_save_file() {
    let mut game = fortress_game(30, 30);
    game.fortress_rings = 9;
    game.fortress_depth = 2;
    let path = std::env::temp_dir().join(format!(
        "fortress_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"fortress_rings\":9") && json.contains("\"fortress_depth\":2"),
        "SaveState JSON must contain the fortress counters, got: {}",
        &json[..json.len().min(200)]
    );
}
