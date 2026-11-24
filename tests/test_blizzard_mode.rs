use snake_game::game::{Difficulty, Game, GameMode, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn blizzard_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Blizzard;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_blizzard_reset_sets_snow_weather_and_initial_snowfall() {
    let game = blizzard_game(30, 30);

    assert_eq!(game.weather, Weather::Snow);
    assert!(
        !game.snow_tiles.is_empty(),
        "Blizzard mode should start with an initial snowfall"
    );
    for snow in &game.snow_tiles {
        assert!(
            game.obstacles.contains(snow),
            "Snow tiles must also be obstacles so they block movement"
        );
        assert!(!game.snake.body_map.contains_key(snow));
    }
    assert!(!game.snow_tiles.contains(&game.food));
}

#[test]
fn test_blizzard_snow_accumulates_over_ticks() {
    let mut game = blizzard_game(30, 30);
    game.obstacles.clear();
    game.snow_tiles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = true;

    // Snow falls on every 6th tick: drive ticks 6, 12 and 18 directly
    for (i, tick) in [5_u32, 11, 17].iter().enumerate() {
        game.tick_counter = *tick;
        game.update();
        assert_eq!(
            game.snow_tiles.len(),
            i + 1,
            "one snow tile should fall on tick {}",
            tick + 1
        );
        for snow in &game.snow_tiles {
            assert!(game.obstacles.contains(snow));
            assert!(!game.snake.body_map.contains_key(snow));
        }
    }
}

#[test]
fn test_blizzard_snow_is_lethal() {
    let mut game = blizzard_game(20, 20);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Up;
    game.snake.direction_queue.push_back(Direction::Up);

    let snow_pos = Point { x: 5, y: 4 };
    game.snow_tiles.insert(snow_pos);
    game.obstacles.insert(snow_pos);

    game.update();

    assert!(
        game.just_died || game.lives < 3,
        "Snake should die when moving into a snow tile"
    );
}

#[test]
fn test_blizzard_periodic_thaw_clears_snow() {
    let mut game = blizzard_game(20, 20);
    game.obstacles.clear();
    game.snow_tiles.clear();

    let snow_a = Point { x: 2, y: 2 };
    let snow_b = Point { x: 3, y: 3 };
    for p in [snow_a, snow_b] {
        game.snow_tiles.insert(p);
        game.obstacles.insert(p);
    }

    // Next update brings tick_counter to 200, which triggers the thaw
    game.tick_counter = 199;
    game.update();

    assert!(
        game.snow_tiles.is_empty(),
        "Thaw at tick 200 should melt all snow"
    );
    assert!(!game.obstacles.contains(&snow_a));
    assert!(!game.obstacles.contains(&snow_b));
}

#[test]
fn test_blizzard_eating_food_melts_adjacent_snow() {
    let mut game = blizzard_game(20, 20);
    game.obstacles.clear();
    game.snow_tiles.clear();

    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };

    let adjacent_snow = Point { x: 6, y: 6 };
    game.snow_tiles.insert(adjacent_snow);
    game.obstacles.insert(adjacent_snow);
    let far_snow = Point { x: 15, y: 15 };
    game.snow_tiles.insert(far_snow);
    game.obstacles.insert(far_snow);

    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.snow_tiles.contains(&adjacent_snow),
        "Eating food should melt adjacent snow"
    );
    assert!(!game.obstacles.contains(&adjacent_snow));
    assert!(
        game.snow_tiles.contains(&far_snow),
        "Distant snow should be unaffected"
    );
}

#[test]
fn test_blizzard_bot_survives_snowfall() {
    let mut game = blizzard_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..80 {
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
        "Autopilot should survive the blizzard (lives: {}, snow: {}, score: {})",
        game.lives,
        game.snow_tiles.len(),
        game.score
    );
    assert!(!game.snow_tiles.is_empty());
}
