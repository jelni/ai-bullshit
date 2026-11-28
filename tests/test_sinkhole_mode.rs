use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn sinkhole_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Sinkhole;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_sinkhole_reset_seeds_patches_and_clears_state() {
    let game = sinkhole_game(30, 30);

    assert!(game.sinkhole_warnings.is_empty());
    assert_eq!(game.sinkhole_collapses, 0);
    assert_eq!(game.sinkholes.len(), Game::SINKHOLE_SEED);
    let spawn = Point {
        x: game.width / 2,
        y: game.height / 2,
    };
    for hole in &game.sinkholes {
        assert!(game.obstacles.contains(hole), "sinkhole must be mirrored into obstacles");
        assert!(!game.snake.body_map.contains_key(hole), "sinkhole must not spawn on the snake");
        assert!(hole != &game.food, "sinkhole must not spawn on the food");
        assert!(
            hole.x.abs_diff(spawn.x).saturating_add(hole.y.abs_diff(spawn.y)) >= 5,
            "sinkhole must spawn at least 5 cells from the spawn point"
        );
    }
}

#[test]
fn test_sinkhole_telegraphs_then_collapses_every_6_ticks() {
    let mut game = sinkhole_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.auto_pilot = false;

    // update() increments tick_counter first, so tick_counter = 2 runs tick 3,
    // which telegraphs the cells that will collapse on tick 6.
    game.state = GameState::Playing;
    game.tick_counter = 2;
    game.update();
    assert_eq!(game.tick_counter, 3);
    assert!(!game.sinkhole_warnings.is_empty(), "tick 3 must telegraph bubbling cells");
    for bubble in &game.sinkhole_warnings {
        assert!(!game.sinkholes.contains(bubble), "a warning must not already be sinkhole");
        assert!(!game.obstacles.contains(bubble), "a warning must not be an obstacle yet");
    }
    let seeded = game.sinkholes.len();

    // Tick 6 collapses: the warnings are consumed and sinkhole comes up.
    game.state = GameState::Playing;
    game.tick_counter = 5;
    game.update();
    assert_eq!(game.tick_counter, 6);
    assert_eq!(game.sinkhole_collapses, 1);
    assert!(game.sinkhole_warnings.is_empty(), "the telegraph is consumed by the collapse");
    assert!(game.sinkholes.len() > seeded, "tick 6 must collapse new sinkhole cells");
    for hole in &game.sinkholes {
        assert!(game.obstacles.contains(hole), "sinkhole must be mirrored into obstacles");
    }

    // Ticks 12 and 18 collapse again.
    for tick in [11_u32, 17] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
    }
    assert_eq!(game.sinkhole_collapses, 3);
}

#[test]
fn test_sinkhole_never_pits_snake_or_next_cell() {
    let mut game = sinkhole_game(20, 20);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.sinkholes.clear();
    // Park the snake heading Left so its next cell is (4, 5).
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Left;
    game.snake.direction_queue.push_back(Direction::Left);
    game.food = Point {
        x: 15,
        y: 15,
    };
    game.auto_pilot = false;

    // Tick 3 telegraphs, tick 6 collapses.
    game.state = GameState::Playing;
    game.tick_counter = 2;
    game.update();
    game.state = GameState::Playing;
    game.tick_counter = 5;
    game.update();

    assert_eq!(game.sinkhole_collapses, 1);
    let next = Point {
        x: 4,
        y: 5,
    };
    assert!(
        !game.sinkholes.contains(&next),
        "a collapse must never pit the cell the snake is about to enter"
    );
    for seg in &game.snake.body {
        assert!(!game.sinkholes.contains(seg), "a collapse must never pit the snake's body");
    }
    assert!(!game.sinkholes.contains(&game.food), "a collapse must never pit the food");
    for hole in &game.sinkholes {
        assert!(game.obstacles.contains(hole));
    }
}

#[test]
fn test_sinkhole_fairness_over_many_ticks() {
    let mut game = sinkhole_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point {
        x: 25,
        y: 25,
    };
    let lives_before = game.lives;

    // Deterministic: park the snake at (8, 15) heading right before every
    // scheduled collapse, so the collapse must never sinkhole its next cell (9, 15) or body.
    let next = Point {
        x: 9,
        y: 15,
    };
    for k in 1..=10_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point {
            x: 8,
            y: 15,
        });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = 6 * k - 1;
        game.update();
        assert!(
            !game.sinkholes.contains(&next),
            "collapse {} pit the cell the snake was about to enter ({next:?})",
            game.sinkhole_collapses
        );
        for seg in &game.snake.body {
            assert!(
                !game.sinkholes.contains(seg),
                "collapse {} pit the snake's body at {seg:?}",
                game.sinkhole_collapses
            );
        }
    }

    assert_eq!(
        game.lives, lives_before,
        "the snake must survive the collapses (collapses: {})",
        game.sinkhole_collapses
    );
    assert!(
        game.sinkhole_collapses >= 7,
        "expected several collapses over 60 ticks, got {}",
        game.sinkhole_collapses
    );
}

#[test]
fn test_sinkhole_heals_away_at_tick_200() {
    let mut game = sinkhole_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.auto_pilot = true;

    // Seep a few times first.
    for tick in [5_u32, 11, 17] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
    }
    assert!(!game.sinkholes.is_empty());
    assert!(game.sinkhole_collapses >= 3);

    // Tick 200 of the 250-tick cycle heals all pits away.
    game.state = GameState::Playing;
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(game.sinkholes.is_empty(), "all pits must heal away at tick 200");
    assert!(game.sinkhole_warnings.is_empty(), "all telegraph bubbles must clear at tick 200");
    for obstacle in &game.obstacles {
        assert!(!game.sinkholes.contains(obstacle), "no obstacle may remain recorded sinkhole");
    }
    assert!(
        game.sinkhole_collapses >= 3,
        "the collapse counter keeps counting through a heal"
    );
}

#[test]
fn test_sinkhole_eating_food_shores_up_adjacent_pit() {
    let mut game = sinkhole_game(20, 20);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.sinkholes.clear();
    let hole = Point {
        x: 7,
        y: 5,
    };
    game.sinkholes.insert(hole);
    game.obstacles.insert(hole);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point {
        x: 6,
        y: 5,
    };
    game.auto_pilot = false;

    // Tick 5 is not a multiple of 6 and not 200, so no collapse interferes here.
    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.sinkholes.contains(&hole),
        "Eating food in Sinkhole mode should shore up the adjacent pit"
    );
    assert!(!game.obstacles.contains(&hole), "The shored-up pit must also leave obstacles");
}

#[test]
fn test_sinkhole_weather_stays_clear() {
    let mut game = sinkhole_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(13);
    game.auto_pilot = false;
    game.state = GameState::Playing;
    game.tick_counter = 0;
    game.update();
    assert_eq!(game.weather, Weather::Clear, "the skies must stay clear so the pits stay visible");
}

#[test]
fn test_sinkhole_bot_survives_the_collapses() {
    let mut game = sinkhole_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;

    for _ in 0..150 {
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
        "Autopilot should survive the sinkhole (lives: {}, collapses: {}, score: {})",
        game.lives,
        game.sinkhole_collapses,
        game.score
    );
    assert!(
        game.sinkhole_collapses >= 5,
        "expected several collapses over 150 ticks, got {}",
        game.sinkhole_collapses
    );
}

#[test]
fn test_sinkhole_persists_through_history_rewind() {
    let mut game = sinkhole_game(30, 30);
    game.sinkholes.insert(Point {
        x: 10,
        y: 11,
    });
    game.sinkhole_warnings.insert(Point {
        x: 11,
        y: 10,
    });
    game.sinkhole_collapses = 7;
    game.save_history_state();
    game.sinkholes.clear();
    game.sinkhole_warnings.clear();
    game.sinkhole_collapses = 0;

    game.rewind_time();

    assert!(game.sinkholes.contains(&Point {
        x: 10,
        y: 11
    }));
    assert!(game.sinkhole_warnings.contains(&Point {
        x: 11,
        y: 10
    }));
    assert_eq!(
        game.sinkhole_collapses, 7,
        "sinkhole_collapses must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_sinkhole_persists_in_save_file() {
    let mut game = sinkhole_game(30, 30);
    game.sinkhole_collapses = 9;
    game.sinkholes.clear();
    game.sinkholes.insert(Point {
        x: 12,
        y: 10,
    });
    game.sinkhole_warnings.insert(Point {
        x: 11,
        y: 9,
    });
    let path = std::env::temp_dir().join(format!(
        "sinkhole_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"sinkhole_collapses\":9"),
        "SaveState JSON must contain the collapse counter, got: {}",
        &json[..json.len().min(200)]
    );
    assert!(
        json.contains("sinkholes") && json.contains("sinkhole_warnings"),
        "SaveState JSON must contain the sinkhole state, got: {}",
        &json[..json.len().min(400)]
    );
}
