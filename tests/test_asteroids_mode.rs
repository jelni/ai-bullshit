use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Meteor, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn asteroids_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Asteroids;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game
}

#[test]
fn test_asteroids_reset_clears_state() {
    let mut game = asteroids_game(30, 30);
    game.asteroid_strikes = 7;
    game.meteors.push(Meteor {
        position: Point { x: 5, y: 5 },
        timer: 0,
    });

    game.reset();

    assert_eq!(game.asteroid_strikes, 0);
    assert!(game.meteors.is_empty());
}

#[test]
fn test_asteroids_scheduled_shower_every_6_ticks() {
    let mut game = asteroids_game(30, 30);
    game.obstacles.clear();

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must trigger a scheduled meteor shower.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let strikes_before = game.asteroid_strikes;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.asteroid_strikes > strikes_before,
            "a scheduled meteor shower must fire on tick {} (strikes before: {}, after: {})",
            tick + 1,
            strikes_before,
            game.asteroid_strikes
        );
        assert!(
            game.meteors.iter().any(|m| m.position.y == 1),
            "the shower on tick {} must spawn fresh meteors at the top of the board",
            tick + 1
        );
    }
    assert!(game.asteroid_strikes >= 6);
}

#[test]
fn test_asteroids_shower_never_targets_snake_column() {
    let mut game = asteroids_game(40, 30);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.food = Point { x: 30, y: 20 };

    for tick in [5_u32, 11, 17, 23, 29, 35, 41, 47] {
        game.state = GameState::Playing;
        let head = game.snake.head();
        let next = Game::calculate_next_head_dir(head, game.snake.direction);
        let before: std::collections::HashSet<Point> =
            game.meteors.iter().map(|m| m.position).collect();
        game.tick_counter = tick;
        game.update();
        // Meteors spawned this tick are at the top row (y == 1) and were not on the
        // board before this tick (spawns never reuse a column with a meteor in flight)
        for m in &game.meteors {
            if m.position.y == 1 && !before.contains(&m.position) {
                assert_ne!(
                    m.position.x, head.x,
                    "a meteor must never spawn on the column the snake occupies"
                );
                assert_ne!(
                    m.position.x, next.x,
                    "a meteor must never spawn on the column the snake is about to enter"
                );
            }
        }
    }

    assert!(
        game.asteroid_strikes >= 8,
        "expected at least 8 showers over 8 scheduled ticks, got {}",
        game.asteroid_strikes
    );
}

#[test]
fn test_asteroids_falling_meteor_is_unsafe() {
    let mut game = asteroids_game(20, 20);
    game.obstacles.clear();
    game.meteors.clear();
    game.meteors.push(Meteor {
        position: Point { x: 5, y: 2 },
        timer: 0,
    });

    // The meteor at (5, 2) falls down column 5 and will be at y=6 within 4 steps
    assert!(
        !game.is_safe_final_p(Point { x: 5, y: 6 }, 4, 1),
        "is_safe_final_p must reject the lane a falling meteor will sweep through"
    );
    assert!(
        game.is_safe_final_p(Point { x: 6, y: 6 }, 4, 1),
        "a different column must remain safe"
    );
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
    });
    game.meteors.push(Meteor {
        position: Point { x: 9, y: 8 },
        timer: 0,
    });
    // Not a multiple of 6, so no scheduled shower interferes
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
fn test_asteroids_strikes_persist_through_save_and_history() {
    let mut game = asteroids_game(30, 30);
    game.obstacles.clear();

    game.state = GameState::Playing;
    game.tick_counter = 5;
    game.update();
    assert!(
        game.asteroid_strikes >= 1,
        "a scheduled shower should have fired on tick 6"
    );
    let strikes = game.asteroid_strikes;

    // The history (time-rewind) snapshots must carry the counter
    assert!(
        game.history.iter().any(|h| h.asteroid_strikes >= 1)
            || {
                game.update();
                game.history.iter().any(|h| h.asteroid_strikes == strikes)
            },
        "asteroid_strikes must be recorded in the history state"
    );

    // The save file must carry the counter too
    let path = std::env::temp_dir().join(format!(
        "snake_asteroids_save_{}.json",
        std::process::id()
    ));
    game.save_game_to_file(path.to_str().unwrap());
    let raw = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(
        json["asteroid_strikes"].as_u64(),
        Some(u64::from(strikes)),
        "asteroid_strikes must be persisted in SaveState"
    );
}

#[test]
fn test_asteroids_bot_survives_the_meteor_shower() {
    let mut game = asteroids_game(40, 30);
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
        "Autopilot should survive the meteor shower (lives: {}, strikes: {}, score: {})",
        game.lives,
        game.asteroid_strikes,
        game.score
    );
    assert!(
        game.asteroid_strikes >= 20,
        "expected at least 20 showers over 120 ticks, got {}",
        game.asteroid_strikes
    );
}
