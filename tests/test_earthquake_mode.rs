use snake_game::game::{Difficulty, Game, GameMode, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn earthquake_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Earthquake;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_earthquake_reset_forces_earthquake_weather_and_initial_rubble() {
    let game = earthquake_game(30, 30);

    assert_eq!(game.weather, Weather::Earthquake);
    assert_eq!(game.tremor_count, 0);
    assert!(
        !game.rubble_tiles.is_empty(),
        "Earthquake mode should start with an initial pile of rubble"
    );
    for rubble in &game.rubble_tiles {
        assert!(
            game.obstacles.contains(rubble),
            "Rubble tiles must also be obstacles so they block movement"
        );
        assert!(!game.snake.body_map.contains_key(rubble));
    }
    assert!(!game.rubble_tiles.contains(&game.food));
}

#[test]
fn test_earthquake_weather_never_breaks() {
    let mut game = earthquake_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..40 {
        game.update();
        if game.state != snake_game::game::GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = snake_game::game::GameState::Playing;
        }
        assert_eq!(
            game.weather,
            Weather::Earthquake,
            "Earthquake mode must force earthquake weather on every tick"
        );
    }
}

#[test]
fn test_earthquake_tremor_every_5_ticks() {
    let mut game = earthquake_game(30, 30);
    game.obstacles.clear();
    game.rubble_tiles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = true;

    // A tremor fires on every 5th tick: drive ticks 5, 10 and 15 directly
    for (i, tick) in [4_u32, 9, 14].iter().enumerate() {
        game.tick_counter = *tick;
        game.update();
        assert_eq!(
            game.tremor_count,
            u32::try_from(i).unwrap_or(u32::MAX) + 1,
            "one tremor should fire on tick {}",
            tick + 1
        );
        for rubble in &game.rubble_tiles {
            assert!(game.obstacles.contains(rubble));
            assert!(!game.snake.body_map.contains_key(rubble));
        }
        assert!(!game.rubble_tiles.contains(&game.food));
    }
}

#[test]
fn test_earthquake_tremor_never_spawns_on_next_head_cell() {
    let mut game = earthquake_game(20, 20);
    game.food = Point { x: 15, y: 15 };

    for tick in [4_u32, 9, 14, 19, 24] {
        game.obstacles.clear();
        game.rubble_tiles.clear();
        game.snake = Snake::new(Point { x: 5, y: 5 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = tick;
        game.update();
        let next_head = Point { x: 6, y: 5 };
        assert!(
            !game.rubble_tiles.contains(&next_head),
            "a tremor must never drop rubble on the cell the snake is about to enter"
        );
        assert!(
            !game.obstacles.contains(&next_head) || game.snake.body_map.contains_key(&next_head),
            "the next head cell must never become a hard obstacle this tick"
        );
    }
}

#[test]
fn test_earthquake_rubble_is_lethal() {
    let mut game = earthquake_game(20, 20);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Up;
    game.snake.direction_queue.push_back(Direction::Up);

    let rubble_pos = Point { x: 5, y: 4 };
    game.rubble_tiles.insert(rubble_pos);
    game.obstacles.insert(rubble_pos);

    game.update();

    assert!(
        game.just_died || game.lives < 3,
        "Snake should die when moving into a rubble tile"
    );
}

#[test]
fn test_earthquake_periodic_settling_clears_rubble() {
    let mut game = earthquake_game(20, 20);
    game.obstacles.clear();
    game.rubble_tiles.clear();

    let rubble_a = Point { x: 2, y: 2 };
    let rubble_b = Point { x: 3, y: 3 };
    for p in [rubble_a, rubble_b] {
        game.rubble_tiles.insert(p);
        game.obstacles.insert(p);
    }

    // Next update brings tick_counter to 200, which triggers the settling
    game.tick_counter = 199;
    game.update();

    assert!(
        game.rubble_tiles.is_empty(),
        "Settling at tick 200 should clear all rubble"
    );
    assert!(!game.obstacles.contains(&rubble_a));
    assert!(!game.obstacles.contains(&rubble_b));
}

#[test]
fn test_earthquake_eating_food_shakes_rubble_loose() {
    let mut game = earthquake_game(20, 20);
    game.obstacles.clear();
    game.rubble_tiles.clear();

    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };

    let adjacent_rubble = Point { x: 6, y: 6 };
    game.rubble_tiles.insert(adjacent_rubble);
    game.obstacles.insert(adjacent_rubble);
    let far_rubble = Point { x: 15, y: 15 };
    game.rubble_tiles.insert(far_rubble);
    game.obstacles.insert(far_rubble);

    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.rubble_tiles.contains(&adjacent_rubble),
        "Eating food should shake adjacent rubble loose"
    );
    assert!(!game.obstacles.contains(&adjacent_rubble));
    assert!(
        game.rubble_tiles.contains(&far_rubble),
        "Distant rubble should be unaffected"
    );
}

#[test]
fn test_earthquake_rubble_persisted_in_save_and_history() {
    let mut game = earthquake_game(30, 30);
    game.obstacles.clear();
    game.rubble_tiles.clear();
    let rubble_a = Point { x: 2, y: 2 };
    let rubble_b = Point { x: 3, y: 3 };
    for p in [rubble_a, rubble_b] {
        game.rubble_tiles.insert(p);
        game.obstacles.insert(p);
    }
    game.tremor_count = 7;

    game.save_history_state();
    let hist = game.history.back().expect("a history state should be recorded");
    assert_eq!(hist.rubble_tiles, game.rubble_tiles);
    assert_eq!(hist.tremor_count, 7);

    let path = std::env::temp_dir().join(format!(
        "snake_game_quake_save_test_{}.json",
        std::process::id()
    ));
    game.save_game_to_file(path.to_str().expect("valid temp path"));
    let json = std::fs::read_to_string(&path).expect("save file should be readable");
    let _ = std::fs::remove_file(&path);
    let v: serde_json::Value =
        serde_json::from_str(&json).expect("save file should be valid JSON");
    assert_eq!(v["tremor_count"], 7);
    assert_eq!(
        v["rubble_tiles"].as_array().expect("rubble_tiles array").len(),
        2
    );
}

#[test]
fn test_earthquake_bot_survives_the_quake() {
    let mut game = earthquake_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..100 {
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
        "Autopilot should survive the earthquake (lives: {}, tremors: {}, score: {})",
        game.lives,
        game.tremor_count,
        game.score
    );
    assert!(
        game.tremor_count >= 20,
        "expected at least 20 tremors over 100 ticks, got {}",
        game.tremor_count
    );
}
