use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn monsoon_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Monsoon;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    // The Druid ignores ambient weather effects, so only the scheduled downpours
    // matter in these tests (keeps the assertions deterministic).
    game.stats.equipped_class = Some(HeroClass::Druid);
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

#[test]
fn test_monsoon_reset_forces_rain_weather_and_initial_puddles() {
    let game = monsoon_game(30, 30);

    assert_eq!(game.weather, Weather::Rain);
    assert_eq!(game.monsoon_downpours, 0);
    assert!(
        !game.puddle_tiles.is_empty(),
        "Monsoon mode should start with an initial rainfall"
    );
    for puddle in &game.puddle_tiles {
        assert!(
            game.obstacles.contains(puddle),
            "Puddle tiles must also be obstacles so they block movement"
        );
        assert!(!game.snake.body_map.contains_key(puddle));
        assert!(puddle.x < game.width && puddle.y < game.height);
    }
    assert!(!game.puddle_tiles.contains(&game.food));
}

#[test]
fn test_monsoon_weather_never_breaks() {
    let mut game = monsoon_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..60 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        assert_eq!(
            game.weather,
            Weather::Rain,
            "Monsoon mode must force rain weather on every tick"
        );
    }
}

#[test]
fn test_monsoon_scheduled_downpour_every_6_ticks() {
    let mut game = monsoon_game(30, 30);
    game.obstacles.clear();
    game.puddle_tiles.clear();
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must trigger a scheduled downpour.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let downpours_before = game.monsoon_downpours;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.monsoon_downpours > downpours_before,
            "a scheduled downpour must fire on tick {} (downpours before: {}, after: {})",
            tick + 1,
            downpours_before,
            game.monsoon_downpours
        );
    }
    assert!(game.monsoon_downpours >= 6);
}

#[test]
fn test_monsoon_puddles_stay_fair_and_mirrored_into_obstacles() {
    let mut game = monsoon_game(30, 30);
    game.obstacles.clear();
    game.puddle_tiles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = true;

    for tick in [5_u32, 11, 17, 23, 29, 35, 41, 47] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
        keep_playing(&mut game);
        for puddle in &game.puddle_tiles {
            assert!(
                game.obstacles.contains(puddle),
                "every puddle must be mirrored into obstacles (puddle at {puddle:?})"
            );
            assert!(
                !game.snake.body_map.contains_key(puddle),
                "a puddle must never fall on the snake (puddle at {puddle:?})"
            );
            assert!(
                *puddle != game.food,
                "a puddle must never cover the food (puddle at {puddle:?})"
            );
            assert!(puddle.x < game.width && puddle.y < game.height);
        }
    }
    assert!(
        game.monsoon_downpours >= 8,
        "expected at least 8 downpours, got {}",
        game.monsoon_downpours
    );
    assert!(
        !game.puddle_tiles.is_empty(),
        "puddles should have accumulated over 8 downpours"
    );
}

#[test]
fn test_monsoon_puddle_never_falls_on_snake_or_next_cell() {
    let mut game = Game::new(20, 20, true, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Monsoon;
    game.reset();
    game.xp_to_next_level = u32::MAX;
    game.obstacles.clear();
    game.puddle_tiles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.auto_pilot = false;

    for _ in 0..80 {
        game.state = GameState::Playing;
        game.snake.direction = Direction::Right;
        game.update();
        if game.state != GameState::Playing {
            break;
        }
        for puddle in &game.puddle_tiles {
            assert!(
                !game.snake.body_map.contains_key(puddle),
                "a puddle must never fall on a snake cell (puddle at {puddle:?})"
            );
        }
    }
}

#[test]
fn test_monsoon_drain_at_tick_200_clears_all_puddles() {
    let mut game = monsoon_game(30, 30);
    game.obstacles.clear();
    game.puddle_tiles.clear();
    game.auto_pilot = true;
    game.puddle_tiles.insert(Point { x: 2, y: 2 });
    game.obstacles.insert(Point { x: 2, y: 2 });
    game.puddle_tiles.insert(Point { x: 3, y: 3 });
    game.obstacles.insert(Point { x: 3, y: 3 });
    let downpours_before = game.monsoon_downpours;

    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200, "update must run tick 200");
    assert!(
        game.puddle_tiles.is_empty(),
        "the periodic drain at tick 200 must clear all puddles"
    );
    assert!(
        !game.obstacles.contains(&Point { x: 2, y: 2 })
            && !game.obstacles.contains(&Point { x: 3, y: 3 }),
        "drained puddles must be removed from obstacles too"
    );
    assert_eq!(
        game.monsoon_downpours, downpours_before,
        "the drain tick must not count as a downpour"
    );
}

#[test]
fn test_monsoon_eating_food_drains_adjacent_puddles() {
    let mut game = monsoon_game(30, 30);
    game.obstacles.clear();
    game.puddle_tiles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.puddle_tiles.insert(Point { x: 7, y: 5 });
    game.obstacles.insert(Point { x: 7, y: 5 });
    game.puddle_tiles.insert(Point { x: 6, y: 6 });
    game.obstacles.insert(Point { x: 6, y: 6 });
    game.puddle_tiles.insert(Point { x: 20, y: 20 });
    game.obstacles.insert(Point { x: 20, y: 20 });

    game.tick_counter = 0;
    game.update();

    assert!(
        game.snake.head() == Point { x: 6, y: 5 },
        "the snake should have eaten the food at (6, 5), head at {:?}",
        game.snake.head()
    );
    assert!(
        !game.puddle_tiles.contains(&Point { x: 7, y: 5 })
            && !game.puddle_tiles.contains(&Point { x: 6, y: 6 }),
        "eating food must drain the puddles adjacent to the eaten cell"
    );
    assert!(
        !game.obstacles.contains(&Point { x: 7, y: 5 })
            && !game.obstacles.contains(&Point { x: 6, y: 6 }),
        "drained puddles must be removed from obstacles"
    );
    assert!(
        game.puddle_tiles.contains(&Point { x: 20, y: 20 }),
        "distant puddles must be untouched"
    );
}

#[test]
fn test_monsoon_puddle_cap_is_respected() {
    let mut game = monsoon_game(20, 20);
    game.auto_pilot = true;
    let cap = (u32::from(game.width) * u32::from(game.height)) / 10;

    for _ in 0..250 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        assert!(
            u32::try_from(game.puddle_tiles.len()).unwrap_or(u32::MAX) <= cap,
            "puddles must never exceed the {}% board cap ({} > {cap})",
            10,
            game.puddle_tiles.len()
        );
    }
}

#[test]
fn test_monsoon_bot_survives_the_monsoon() {
    let mut game = monsoon_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
    }

    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "Autopilot should survive the monsoon (lives: {}, downpours: {}, score: {})",
        game.lives,
        game.monsoon_downpours,
        game.score
    );
    assert!(
        game.monsoon_downpours >= 20,
        "expected at least 20 downpours over 120 ticks, got {}",
        game.monsoon_downpours
    );
}

#[test]
fn test_monsoon_state_persists_through_save_load() {
    let file_path = format!("monsoon_save_test_{}.json", std::process::id());
    let _ = std::fs::remove_file(&file_path);

    let mut game = monsoon_game(20, 20);
    game.puddle_tiles.clear();
    game.puddle_tiles.insert(Point { x: 12, y: 9 });
    game.obstacles.insert(Point { x: 12, y: 9 });
    game.monsoon_downpours = 11;
    game.save_game_to_file(&file_path);

    let json = std::fs::read_to_string(&file_path).expect("save file should exist");
    assert!(
        json.contains("\"monsoon_downpours\":11"),
        "the save file must contain the downpour counter"
    );
    let _ = std::fs::remove_file(&file_path);

    // Round-trip through the public load path (reads savegame.json)
    let _ = std::fs::remove_file("savegame.json");
    game.save_game_to_file("savegame.json");
    let mut loaded = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    assert!(loaded.load_game());
    assert_eq!(loaded.mode, GameMode::Monsoon);
    assert_eq!(loaded.monsoon_downpours, 11);
    assert!(loaded.puddle_tiles.contains(&Point { x: 12, y: 9 }));
    let _ = std::fs::remove_file("savegame.json");
}

#[test]
fn test_monsoon_state_persists_through_time_rewind() {
    let mut game = monsoon_game(20, 20);
    game.obstacles.clear();
    game.puddle_tiles.clear();
    game.auto_pilot = true;

    // Run one scheduled downpour (tick 6)
    game.tick_counter = 5;
    game.update();
    keep_playing(&mut game);
    assert_eq!(game.monsoon_downpours, 1, "expected exactly one downpour on tick 6");

    game.rewind_time();
    assert_eq!(
        game.monsoon_downpours, 0,
        "time-rewind must restore the downpour counter"
    );
    assert!(
        game.puddle_tiles.is_empty(),
        "time-rewind must restore the puddle set"
    );
}
