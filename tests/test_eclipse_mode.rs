use snake_game::game::{Difficulty, Game, GameMode, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn eclipse_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Eclipse;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_eclipse_reset_forces_eclipse_weather() {
    let game = eclipse_game(30, 30);

    assert_eq!(game.weather, Weather::Eclipse);
    assert_eq!(game.eclipse_flares, 0);
    assert_eq!(game.eclipse_flare, 0);
}

#[test]
fn test_eclipse_weather_never_breaks() {
    let mut game = eclipse_game(30, 30);
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
            Weather::Eclipse,
            "Eclipse mode must force eclipse weather on every tick"
        );
    }
}

#[test]
fn test_eclipse_scheduled_flare_every_6_ticks() {
    let mut game = eclipse_game(30, 30);
    game.obstacles.clear();

    // Drive the scheduled ticks directly: update() increments tick_counter first,
    // so setting it to 6k - 1 makes the next update run tick 6k, which must flare.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = snake_game::game::GameState::Playing;
        game.tick_counter = tick;
        game.eclipse_flare = 0;
        // Park the snake in a corner so nothing is eaten and no flare is extended
        game.snake = Snake::new(Point { x: 5, y: 5 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.push_back(Direction::Right);
        game.food = Point { x: 20, y: 20 };
        let flares_before = game.eclipse_flares;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.eclipse_flares > flares_before,
            "a scheduled solar flare must fire on tick {} (flares before: {}, after: {})",
            tick + 1,
            flares_before,
            game.eclipse_flares
        );
        assert_eq!(
            game.eclipse_flare, 3,
            "a fresh flare must reveal the board for 3 ticks"
        );
    }
    assert!(game.eclipse_flares >= 6);
}

#[test]
fn test_eclipse_flare_decays_to_darkness() {
    let mut game = eclipse_game(30, 30);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 20, y: 20 };
    game.eclipse_flare = 3;

    // Three updates without a scheduled tick: the flare must burn down to 0
    for tick in [7_u32, 8, 9] {
        game.state = snake_game::game::GameState::Playing;
        game.tick_counter = tick;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
    }
    assert_eq!(
        game.eclipse_flare, 0,
        "the flare must decay to darkness after 3 ticks"
    );
    assert_eq!(
        game.eclipse_flares, 0,
        "non-scheduled ticks must not count as flares"
    );
}

#[test]
fn test_eclipse_eating_food_catches_sunlight() {
    let mut game = eclipse_game(30, 30);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };

    // Tick 5: the snake moves right and eats the food, catching sunlight
    game.tick_counter = 4;
    game.update();

    assert!(
        game.food_eaten_session >= 1,
        "Snake should have eaten the food"
    );
    assert_eq!(
        game.eclipse_flare, 6,
        "Eating food in Eclipse mode should reveal the board for 6 ticks (Sunlit!)"
    );
    assert_eq!(
        game.eclipse_flares, 0,
        "eating food must not count as a scheduled flare"
    );
}

#[test]
fn test_eclipse_bot_survives_the_darkness() {
    let mut game = eclipse_game(40, 30);
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
        "Autopilot should survive the eclipse (lives: {}, flares: {}, score: {})",
        game.lives,
        game.eclipse_flares,
        game.score
    );
    assert!(
        game.eclipse_flares >= 5,
        "expected several solar flares over 120 ticks, got {}",
        game.eclipse_flares
    );
}
