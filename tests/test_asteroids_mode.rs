use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Meteor, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn asteroids_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Asteroids;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_asteroids_reset_clears_shower_counter() {
    let mut game = asteroids_game(30, 30);
    game.meteor_showers = 42;
    game.reset();

    assert_eq!(game.meteor_showers, 0);
    assert!(game.meteors.is_empty());
}

#[test]
fn test_asteroids_scheduled_shower_every_12_ticks() {
    let mut game = asteroids_game(40, 30);
    game.obstacles.clear();
    game.auto_pilot = true;
    game.lives = 100;

    // update() increments tick_counter first, so setting it to 12k - 1 makes the
    // next update run tick 12k, which must trigger a scheduled meteor shower.
    for tick in [11_u32, 23, 35, 47, 59, 71] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let showers_before = game.meteor_showers;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.meteor_showers > showers_before,
            "a scheduled meteor shower must fire on tick {} (showers before: {}, after: {})",
            tick + 1,
            showers_before,
            game.meteor_showers
        );
    }
    assert!(game.meteor_showers >= 6);
}

#[test]
fn test_asteroids_shower_spawns_burst_of_meteors() {
    let mut game = asteroids_game(40, 30);
    game.obstacles.clear();
    game.meteors.clear();
    game.state = GameState::Playing;
    game.tick_counter = 11;

    game.update();

    assert_eq!(game.meteor_showers, 1, "tick 12 must trigger exactly one shower");
    // Fresh meteors spawn on the top row (y == 1); the shower alone drops 3
    // (the ambient 15% spawn may add one more).
    let fresh: Vec<&Meteor> = game.meteors.iter().filter(|m| m.position.y == 1).collect();
    assert!(
        fresh.len() >= 3,
        "a shower should spawn at least 3 meteors on the top row, got {}",
        fresh.len()
    );
    for m in &fresh {
        assert!(m.position.x >= 1 && m.position.x < game.width - 1);
    }
}

#[test]
fn test_asteroids_never_spawns_on_snake_column() {
    let mut game = asteroids_game(20, 20);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.lives = 100;

    for _ in 0..60 {
        let head_before = game.snake.head();
        let dir_before = game.snake.direction;
        let next_before = Game::calculate_next_head_dir(head_before, dir_before);
        game.state = GameState::Playing;
        game.update();
        // Meteors that spawned this tick sit on the top row (y == 1)
        for m in &game.meteors {
            if m.position.y == 1 {
                assert_ne!(
                    m.position.x, head_before.x,
                    "a meteor must never spawn in the snake's current column"
                );
                assert_ne!(
                    m.position.x, next_before.x,
                    "a meteor must never spawn in the column the snake is about to enter"
                );
            }
        }
        if game.state != GameState::Playing && game.lives == 0 {
            break;
        }
    }

    assert!(
        game.meteor_showers >= 5,
        "expected at least 5 showers over 60 ticks, got {}",
        game.meteor_showers
    );
}

#[test]
fn test_asteroids_meteor_lane_blocks_verified_paths() {
    let mut game = asteroids_game(20, 20);
    game.obstacles.clear();
    game.goblin = None;
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.food = Point { x: 16, y: 10 };
    game.meteors.clear();
    game.meteors.push(Meteor {
        position: Point { x: 12, y: 9 },
        timer: 0,
        vx: 0,
    });

    // The cell in the meteor's fall lane is unsafe at its true arrival step
    assert!(
        !game.is_safe_final_p(Point { x: 12, y: 10 }, 2, 1),
        "is_safe_final_p must reject the meteor's fall lane"
    );
    // A different column stays safe
    assert!(game.is_safe_final_p(Point { x: 13, y: 10 }, 3, 1));

    // The straight path to the food crosses the lane and must be rejected
    let direct = vec![
        Point { x: 11, y: 10 },
        Point { x: 12, y: 10 },
        Point { x: 13, y: 10 },
        Point { x: 14, y: 10 },
        Point { x: 15, y: 10 },
        Point { x: 16, y: 10 },
    ];
    assert!(
        !game.path_is_safe(Point { x: 10, y: 10 }, &direct, &[game.food], 1),
        "the direct path through the meteor lane must fail verification"
    );

    // A detour above the meteor verifies
    let detour = vec![
        Point { x: 11, y: 10 },
        Point { x: 11, y: 9 },
        Point { x: 11, y: 8 },
        Point { x: 12, y: 8 },
        Point { x: 13, y: 8 },
        Point { x: 14, y: 8 },
        Point { x: 15, y: 8 },
        Point { x: 16, y: 8 },
        Point { x: 16, y: 9 },
        Point { x: 16, y: 10 },
    ];
    assert!(
        game.path_is_safe(Point { x: 10, y: 10 }, &detour, &[game.food], 1),
        "the detour around the meteor must verify as safe"
    );
}

#[test]
fn test_asteroids_autopilot_never_steps_onto_a_meteor() {
    let mut game = asteroids_game(20, 20);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.food = Point { x: 16, y: 10 };
    game.meteors.clear();
    game.meteors.push(Meteor {
        position: Point { x: 12, y: 9 },
        timer: 0,
        vx: 0,
    });
    game.auto_pilot = true;
    game.lives = 100;

    for _ in 0..12 {
        game.state = GameState::Playing;
        game.update();
        assert!(
            !game.meteors.iter().any(|m| m.position == game.snake.head()),
            "the autopilot must never end a tick with its head on a meteor cell"
        );
        if game.state != GameState::Playing && game.lives == 0 {
            break;
        }
    }
}

#[test]
fn test_asteroids_showers_persist_through_time_rewind() {
    let mut game = asteroids_game(30, 30);
    game.meteor_showers = 7;
    game.save_history_state();

    game.meteor_showers = 99;
    game.rewind_time();

    assert_eq!(
        game.meteor_showers, 7,
        "time-rewind must restore the shower counter"
    );
}

#[test]
fn test_asteroids_showers_saved_to_file() {
    let mut game = asteroids_game(30, 30);
    game.meteor_showers = 5;
    let path = std::env::temp_dir().join(format!(
        "snake_asteroids_save_{}_showers.json",
        std::process::id()
    ));
    let path_str = path.to_string_lossy().to_string();

    game.save_game_to_file(&path_str);

    let content = std::fs::read_to_string(&path).expect("save file should exist");
    let v: serde_json::Value =
        serde_json::from_str(&content).expect("save file should be valid JSON");
    assert_eq!(v["meteor_showers"], 5, "showers must persist in SaveState");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_other_modes_have_no_scheduled_showers() {
    let mut game = Game::new(30, 30, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::SinglePlayer;
    game.reset();
    game.obstacles.clear();
    game.auto_pilot = true;
    game.lives = 100;
    game.xp_to_next_level = u32::MAX;

    for tick in [11_u32, 23, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
        assert_eq!(
            game.meteor_showers, 0,
            "only Asteroids mode schedules meteor showers"
        );
    }
}

#[test]
fn test_asteroids_eating_food_deflects_lowest_meteor() {
    let mut game = asteroids_game(20, 20);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.food = Point { x: 6, y: 5 };
    game.meteors.clear();
    game.meteors.push(Meteor {
        position: Point { x: 8, y: 3 },
        timer: 0,
        vx: 0,
    });
    game.meteors.push(Meteor {
        position: Point { x: 9, y: 8 },
        timer: 0,
        vx: 0,
    });
    // Not a multiple of 12, so no scheduled shower interferes
    game.tick_counter = 1;

    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game
            .meteors
            .iter()
            .any(|m| m.position.x == 9 && m.position.y >= 8),
        "eating food in Asteroids mode should deflect the lowest (most imminent) meteor"
    );
    assert!(
        game.meteors.iter().any(|m| m.position.x == 8),
        "the higher meteor should still be in flight"
    );
}

#[test]
fn test_asteroids_never_spawns_in_a_column_with_a_meteor_in_flight() {
    let mut game = asteroids_game(20, 20);
    game.obstacles.clear();
    game.meteors.clear();
    game.meteors.push(Meteor {
        position: Point { x: 7, y: 10 },
        timer: 0,
        vx: 0,
    });
    game.state = GameState::Playing;
    game.tick_counter = 11; // next update runs tick 12 -> scheduled shower

    game.update();

    assert_eq!(game.meteor_showers, 1, "tick 12 must trigger exactly one shower");
    for m in &game.meteors {
        if m.position.y == 1 {
            assert_ne!(
                m.position.x, 7,
                "no new meteor may spawn in a column that already has a meteor in flight"
            );
        }
    }
}

fn drift_game(width: u16, height: u16) -> Game {
    let mut game = asteroids_game(width, height);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.meteors.clear();
    game.lives = 1000;
    game.state = GameState::Playing;
    game
}

#[test]
fn test_asteroids_shower_first_meteor_drifts() {
    let mut game = drift_game(30, 30);
    game.auto_pilot = true;

    // Showers fire on every 12th tick; the first meteor of a shower gets vx = ±1.
    let mut saw_drifting = false;
    for tick in [11_u32, 23, 35, 47] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
        if game.meteors.iter().any(|m| m.vx != 0) {
            saw_drifting = true;
        }
    }
    assert!(
        saw_drifting,
        "the first meteor of a 12th-tick shower must drift sideways (vx != 0)"
    );
}

#[test]
fn test_drifting_meteor_moves_sideways_and_bounces() {
    let mut game = drift_game(20, 20);
    game.meteors.push(Meteor {
        position: Point { x: 17, y: 5 },
        timer: 0,
        vx: 1,
    });

    game.update();
    let m = game.meteors.iter().find(|m| m.vx != 0).unwrap();
    assert_eq!((m.position.x, m.position.y), (17, 5), "no drift until the fall tick");
    assert_eq!(m.timer, 1);

    game.update();
    let m = game.meteors.iter().find(|m| m.vx != 0).unwrap();
    assert_eq!((m.position.x, m.position.y), (18, 6), "even row: falls one row and drifts one column");

    game.update();
    game.update();
    let m = game.meteors.iter().find(|m| m.vx != 0).unwrap();
    assert_eq!((m.position.x, m.position.y), (18, 7), "odd row: falls without drifting");

    game.update();
    game.update();
    let m = game.meteors.iter().find(|m| m.vx != 0).unwrap();
    assert_eq!((m.position.x, m.position.y), (17, 8), "even row: drifts and bounces off the right wall");
    assert_eq!(m.vx, -1, "vx must flip at the right edge");

    for _ in 0..10 {
        game.state = GameState::Playing;
        game.update();
        for m in &game.meteors {
            assert!((1..=18).contains(&m.position.x), "meteor x stays in bounds");
        }
    }
}

#[test]
fn test_is_safe_final_p_tracks_drifting_meteor() {
    let mut game = drift_game(20, 20);
    game.snake = Snake::new(Point { x: 2, y: 15 });
    // Drifting right one column per two fallen rows: s=1 -> (10,2), s=2 -> (10,3),
    // s=4 -> (11,4), s=6 -> (11,5), s=8 -> (12,6), ...
    game.meteors.push(Meteor {
        position: Point { x: 10, y: 2 },
        timer: 0,
        vx: 1,
    });

    assert!(
        !game.is_safe_final_p(Point { x: 10, y: 2 }, 1, 1),
        "predicted meteor cell at step 1 must be unsafe"
    );
    assert!(
        !game.is_safe_final_p(Point { x: 10, y: 3 }, 2, 1),
        "predicted meteor cell at step 2 must be unsafe"
    );
    assert!(
        !game.is_safe_final_p(Point { x: 11, y: 4 }, 4, 1),
        "predicted meteor cell at step 4 must be unsafe"
    );
    assert!(
        game.is_safe_final_p(Point { x: 11, y: 4 }, 2, 1),
        "the same cell is safe before the meteor arrives"
    );
    assert!(
        game.is_safe_final_p(Point { x: 11, y: 3 }, 4, 1),
        "the cell the meteor has already drifted away from stays safe"
    );
}

#[test]
fn test_drifting_meteor_kills_snake() {
    let mut game = drift_game(20, 20);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right; // moving to (11,10)
    // After the meteor's tick it lands exactly on (11,10): timer 1 -> falls to y=10
    // (even row, drifts) -> x=11.
    game.meteors.push(Meteor {
        position: Point { x: 10, y: 9 },
        timer: 1,
        vx: 1,
    });

    let lives_before = game.lives;
    game.update();
    assert_eq!(
        game.lives,
        lives_before - 1,
        "a drifting meteor landing on the head must kill the snake"
    );
}

#[test]
fn test_meteor_vx_survives_serialization_and_defaults() {
    // Old saves have no vx field: it must default to 0.
    let legacy: Meteor =
        serde_json::from_str(r#"{"position":{"x":3,"y":4},"timer":1}"#)
            .expect("legacy meteor JSON without vx must still deserialize");
    assert_eq!(legacy.vx, 0);

    let m = Meteor {
        position: Point { x: 7, y: 2 },
        timer: 0,
        vx: -1,
    };
    let json = serde_json::to_string(&vec![m]).unwrap();
    let back: Vec<Meteor> = serde_json::from_str(&json).unwrap();
    assert_eq!(back[0].vx, -1, "vx must round-trip through SaveState JSON");
}

#[test]
fn test_non_asteroids_meteors_do_not_drift() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.obstacles.clear();
    game.meteors.clear();
    game.lives = 1000;
    game.state = GameState::Playing;
    game.meteors.push(Meteor {
        position: Point { x: 10, y: 5 },
        timer: 0,
        vx: 0,
    });

    for n in 1..=6_u16 {
        game.state = GameState::Playing;
        game.update();
        assert!(
            game
                .meteors
                .iter()
                .any(|m| m.position == Point { x: 10, y: 5 + n / 2 }),
            "vx = 0 meteors fall straight down (expected column x=10 at y={})",
            5 + n / 2
        );
    }
}
