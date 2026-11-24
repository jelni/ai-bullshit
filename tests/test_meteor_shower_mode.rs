use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn meteor_shower_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::MeteorShower;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game
}

#[test]
fn test_meteor_shower_reset_clears_state() {
    let mut game = Game::new(30, 30, false, 'x', Theme::Classic, Difficulty::Normal);
    game.meteor_strikes = 5;
    game.meteor_strike_column = Some(7);
    game.meteors.push(snake_game::game::Meteor {
        position: Point {
            x: 3,
            y: 3,
        },
        timer: 0,
    });
    game.mode = GameMode::MeteorShower;
    game.reset();

    assert_eq!(game.weather, Weather::Clear);
    assert_eq!(game.meteor_strikes, 0);
    assert_eq!(game.meteor_strike_column, None);
    assert!(game.meteors.is_empty());
}

#[test]
fn test_meteor_shower_weather_stays_clear() {
    let mut game = meteor_shower_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..60 {
        game.update();
        if game.state != snake_game::game::GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = snake_game::game::GameState::Playing;
        }
        assert_eq!(
            game.weather,
            Weather::Clear,
            "Meteor Shower mode must keep the skies clear on every tick so meteors stay visible"
        );
    }
}

#[test]
fn test_meteor_shower_scheduled_strike_every_6_ticks() {
    let mut game = meteor_shower_game(30, 30);
    game.obstacles.clear();
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must spawn a guaranteed meteor strike.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = snake_game::game::GameState::Playing;
        game.tick_counter = tick;
        let strikes_before = game.meteor_strikes;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.meteor_strikes > strikes_before,
            "a scheduled meteor strike must fire on tick {} (strikes before: {}, after: {})",
            tick + 1,
            strikes_before,
            game.meteor_strikes
        );
        let col = game.meteor_strike_column.expect("a scheduled strike must record its column");
        assert!(
            game.meteors.iter().any(|m| m.position.x == col && m.position.y == 1),
            "the scheduled strike must spawn a meteor at the top of column {col}"
        );
    }
    assert!(game.meteor_strikes >= 6);
}

#[test]
fn test_meteor_shower_strike_never_targets_snake_column() {
    let mut game = meteor_shower_game(20, 20);
    game.obstacles.clear();
    game.wrap_mode = true; // keep the snake alive while it circles the board
    game.lives = 1000; // ambient meteors may kill; the strike schedule must not
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.food = Point {
        x: 15,
        y: 15,
    };

    for _ in 0..36 {
        game.state = snake_game::game::GameState::Playing;
        // Drive the snake straight right so the pre-update state is exactly what
        // the strike picker sees when manage_meteors runs.
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        // Columns the strike picker must never choose this tick: every column the
        // snake occupies plus the column its head is about to enter.
        let mut forbidden: Vec<u16> = game.snake.body_map.keys().map(|p| p.x).collect();
        let head = game.snake.head();
        forbidden.push(head.x);
        forbidden.push(snake_game::game::Game::calculate_next_head_dir(head, Direction::Right).x);
        let strikes_before = game.meteor_strikes;
        game.update();
        if game.state != snake_game::game::GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = snake_game::game::GameState::Playing;
        }
        if game.meteor_strikes > strikes_before {
            let col = game.meteor_strike_column.unwrap();
            assert!(
                !forbidden.contains(&col),
                "a scheduled meteor strike must never target a snake column (col {col}, forbidden {forbidden:?})"
            );
        }
    }

    assert!(
        game.meteor_strikes >= 6,
        "expected at least 6 strikes over 36 ticks, got {}",
        game.meteor_strikes
    );
}

#[test]
fn test_meteor_shower_meteor_column_is_lethal() {
    let mut game = meteor_shower_game(20, 20);
    game.obstacles.clear();
    game.meteors.push(snake_game::game::Meteor {
        position: Point {
            x: 5,
            y: 3,
        },
        timer: 0,
    });

    assert!(
        !game.is_safe_final_p(
            Point {
                x: 5,
                y: 5
            },
            2,
            1
        ),
        "is_safe_final_p must reject a cell the falling meteor will reach"
    );
    assert!(
        game.is_safe_final_p(
            Point {
                x: 5,
                y: 5
            },
            1,
            1
        ),
        "the cell is still safe before the meteor reaches it"
    );
    assert!(
        game.is_safe_final_p(
            Point {
                x: 6,
                y: 5
            },
            2,
            1
        ),
        "a different column must remain safe"
    );
}

#[test]
fn test_meteor_shower_bot_survives_the_shower() {
    let mut game = meteor_shower_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.update();
        if game.state != snake_game::game::GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = snake_game::game::GameState::Playing;
        }
    }

    assert!(
        game.state == snake_game::game::GameState::Playing && game.lives > 0,
        "Autopilot should survive the meteor shower (lives: {}, strikes: {}, score: {})",
        game.lives,
        game.meteor_strikes,
        game.score
    );
    assert!(
        game.meteor_strikes >= 20,
        "expected at least 20 scheduled strikes over 120 ticks, got {}",
        game.meteor_strikes
    );
}

#[test]
fn test_meteor_shower_strikes_persist_through_history_rewind() {
    let mut game = meteor_shower_game(30, 30);
    game.meteor_strikes = 9;
    game.save_history_state();
    assert_eq!(
        game.history.back().map(|s| s.meteor_strikes),
        Some(9),
        "the strike counter must be stored in HistoryState"
    );

    game.meteor_strikes = 0;
    game.rewind_time();
    assert_eq!(game.meteor_strikes, 9, "time-rewind must restore the meteor strike counter");
}

#[test]
fn test_meteor_shower_strikes_persist_through_save_file() {
    let game = meteor_shower_game(30, 30);
    let mut game = game;
    game.meteor_strikes = 7;
    let path = std::env::temp_dir()
        .join(format!("snake_game_meteor_shower_save_{}.json", std::process::id()));
    game.save_game_to_file(path.to_str().unwrap());
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(
        json["meteor_strikes"],
        serde_json::json!(7),
        "the strike counter must be serialized into SaveState"
    );
}
