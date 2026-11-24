use snake_game::game::{Difficulty, Game, GameMode, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn storm_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Storm;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_storm_reset_forces_storm_weather() {
    let game = storm_game(30, 30);

    assert_eq!(game.weather, Weather::Storm);
    assert_eq!(game.storm_strikes, 0);
}

#[test]
fn test_storm_weather_never_breaks() {
    let mut game = storm_game(30, 30);
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
            Weather::Storm,
            "Storm mode must force storm weather on every tick"
        );
    }
}

#[test]
fn test_storm_scheduled_strike_every_6_ticks() {
    let mut game = storm_game(30, 30);
    game.obstacles.clear();
    game.auto_pilot = true;

    // Drive the scheduled ticks directly: update() increments tick_counter first,
    // so setting it to 6k - 1 makes the next update run tick 6k, which must strike.
    // (The active column may already be grounded again by food eaten the same tick,
    // so the reliable signal is the storm_strikes counter.)
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = snake_game::game::GameState::Playing;
        game.tick_counter = tick;
        let strikes_before = game.storm_strikes;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.storm_strikes > strikes_before,
            "a scheduled lightning strike must fire on tick {} (strikes before: {}, after: {})",
            tick + 1,
            strikes_before,
            game.storm_strikes
        );
    }
    assert!(game.storm_strikes >= 6);
}

#[test]
fn test_storm_strike_never_targets_snake_column() {
    let mut game = storm_game(20, 20);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = true;

    for _ in 0..60 {
        let next_x = game.snake.head().x.saturating_add(1).min(game.width - 1);
        game.update();
        if game.state != snake_game::game::GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = snake_game::game::GameState::Playing;
        }
        if let Some(col) = game.lightning_column {
            assert_ne!(
                col,
                game.snake.head().x,
                "a strike must never land on the column the snake occupies"
            );
            let _ = next_x;
        }
    }

    assert!(
        game.storm_strikes >= 10,
        "expected at least 10 strikes over 60 ticks, got {}",
        game.storm_strikes
    );
}

#[test]
fn test_storm_lightning_column_is_lethal() {
    let mut game = storm_game(20, 20);
    game.obstacles.clear();
    game.lightning_column = Some(5);

    // Moving up from (5, 5) enters column 5, which is being struck
    assert!(
        !game.is_safe_final_p(Point { x: 5, y: 4 }, 1, 1),
        "is_safe_final_p must reject the active lightning column"
    );
    assert!(
        game.is_safe_final_p(Point { x: 6, y: 5 }, 1, 1),
        "a different column must remain safe"
    );
}

#[test]
fn test_storm_eating_food_grounds_the_strike() {
    let mut game = storm_game(20, 20);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };

    // Next update brings tick_counter to 6, which triggers a scheduled strike
    game.tick_counter = 5;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        game.storm_strikes >= 1,
        "a scheduled strike should have fired on tick 6"
    );
    assert_eq!(
        game.lightning_column, None,
        "Eating food in Storm mode should ground (clear) the active lightning strike"
    );
}

#[test]
fn test_storm_bot_survives_the_thunderstorm() {
    let mut game = storm_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.update();
        if game.state != snake_game::game::GameState::Playing {
            if game.lives == 0 {
                break;
            }
            // Non-death interruption (e.g. merchant visit): resume playing
            game.state = snake_game::game::GameState::Playing;
        }
    }

    assert!(
        game.state == snake_game::game::GameState::Playing && game.lives > 0,
        "Autopilot should survive the storm (lives: {}, strikes: {}, score: {})",
        game.lives,
        game.storm_strikes,
        game.score
    );
    assert!(
        game.storm_strikes >= 20,
        "expected at least 20 strikes over 120 ticks, got {}",
        game.storm_strikes
    );
}
