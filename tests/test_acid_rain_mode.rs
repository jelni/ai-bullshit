use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn acid_rain_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::AcidRain;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
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
fn test_acid_rain_reset_forces_rain_weather() {
    let game = acid_rain_game(30, 30);

    assert_eq!(game.weather, Weather::Rain);
    assert_eq!(game.acid_drops, 0);
}

#[test]
fn test_acid_rain_weather_never_breaks() {
    let mut game = acid_rain_game(30, 30);
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
            "Acid Rain mode must force rain weather on every tick"
        );
    }
}

#[test]
fn test_acid_rain_scheduled_downpour_every_6_ticks() {
    let mut game = acid_rain_game(30, 30);
    game.obstacles.clear();
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must trigger a scheduled downpour.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let drops_before = game.acid_drops;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.acid_drops > drops_before,
            "a scheduled downpour must fire on tick {} (drops before: {}, after: {})",
            tick + 1,
            drops_before,
            game.acid_drops
        );
    }
    assert!(game.acid_drops >= 6);
}

#[test]
fn test_acid_rain_puddles_accumulate_every_6_ticks() {
    let mut game = acid_rain_game(30, 30);
    game.obstacles.clear();
    game.acid_puddles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = true;

    // Acid falls on every 6th tick: drive ticks 6, 12 and 18 directly
    for (i, tick) in [5_u32, 11, 17].iter().enumerate() {
        game.tick_counter = *tick;
        let drops_before = game.acid_drops;
        game.update();
        assert_eq!(
            game.acid_puddles.len(),
            i + 1,
            "one acid puddle should fall on tick {}",
            tick + 1
        );
        assert!(
            game.acid_drops > drops_before,
            "the drop counter must increase on tick {}",
            tick + 1
        );
        for puddle in &game.acid_puddles {
            assert!(game.obstacles.contains(puddle));
            assert!(!game.snake.body_map.contains_key(puddle));
        }
    }
}

#[test]
fn test_acid_rain_never_falls_on_snake_or_next_cell() {
    let mut game = Game::new(20, 20, true, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::AcidRain;
    game.reset();
    game.xp_to_next_level = u32::MAX;
    game.obstacles.clear();
    game.acid_puddles.clear();
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
        for puddle in &game.acid_puddles {
            assert!(
                !game.snake.body_map.contains_key(puddle),
                "acid must never fall on a snake cell (puddle at {puddle:?})"
            );
        }
    }
}

#[test]
fn test_acid_rain_puddle_is_lethal() {
    let mut game = acid_rain_game(20, 20);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Up;
    game.snake.direction_queue.push_back(Direction::Up);

    let puddle_pos = Point { x: 5, y: 4 };
    game.acid_puddles.insert(puddle_pos);
    game.obstacles.insert(puddle_pos);

    assert!(
        !game.is_safe_final_p(puddle_pos, 1, 1),
        "is_safe_final_p must reject an acid puddle (it is an obstacle)"
    );

    game.update();

    assert!(
        game.just_died || game.lives < 3,
        "Snake should die when moving into an acid puddle"
    );
}

#[test]
fn test_acid_rain_periodic_break_evaporates_puddles() {
    let mut game = acid_rain_game(20, 20);
    game.obstacles.clear();
    game.acid_puddles.clear();

    let puddle_a = Point { x: 2, y: 2 };
    let puddle_b = Point { x: 3, y: 3 };
    for p in [puddle_a, puddle_b] {
        game.acid_puddles.insert(p);
        game.obstacles.insert(p);
    }

    // Next update brings tick_counter to 200, which triggers the break in the rain
    game.tick_counter = 199;
    game.update();

    assert!(
        game.acid_puddles.is_empty(),
        "The break in the rain at tick 200 should evaporate all acid"
    );
    assert!(!game.obstacles.contains(&puddle_a));
    assert!(!game.obstacles.contains(&puddle_b));
}

#[test]
fn test_acid_rain_eating_food_neutralizes_adjacent_acid() {
    let mut game = acid_rain_game(20, 20);
    game.obstacles.clear();
    game.acid_puddles.clear();

    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.tick_counter = 0; // next tick is 1: no scheduled downpour interferes

    let adjacent_acid = Point { x: 6, y: 6 };
    game.acid_puddles.insert(adjacent_acid);
    game.obstacles.insert(adjacent_acid);
    let far_acid = Point { x: 15, y: 15 };
    game.acid_puddles.insert(far_acid);
    game.obstacles.insert(far_acid);

    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.acid_puddles.contains(&adjacent_acid),
        "Eating food should neutralize adjacent acid"
    );
    assert!(!game.obstacles.contains(&adjacent_acid));
    assert!(
        game.acid_puddles.contains(&far_acid),
        "Distant acid should be unaffected"
    );
}

#[test]
fn test_acid_rain_reset_seeds_initial_puddles() {
    let game = acid_rain_game(30, 30);

    assert_eq!(game.weather, Weather::Rain);
    assert_eq!(game.acid_drops, 0);
    assert!(
        !game.acid_puddles.is_empty(),
        "Acid Rain mode should start with an initial acid rainfall"
    );
    for puddle in &game.acid_puddles {
        assert!(
            game.obstacles.contains(puddle),
            "Acid puddles must also be obstacles so they block movement"
        );
        assert!(!game.snake.body_map.contains_key(puddle));
    }
    assert!(!game.acid_puddles.contains(&game.food));
}

#[test]
fn test_acid_rain_drops_persist_through_save_load() {
    let file_path = format!("acid_rain_save_test_{}.json", std::process::id());
    let _ = std::fs::remove_file(&file_path);

    let mut game = acid_rain_game(20, 20);
    game.acid_drops = 11;
    game.save_game_to_file(&file_path);

    let json = std::fs::read_to_string(&file_path).expect("save file should exist");
    assert!(
        json.contains("\"acid_drops\":11"),
        "the save file must contain the acid drop counter"
    );
    let _ = std::fs::remove_file(&file_path);

    // Round-trip through the public load path (reads savegame.json)
    let _ = std::fs::remove_file("savegame.json");
    game.save_game_to_file("savegame.json");
    let mut loaded = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    assert!(loaded.load_game());
    assert_eq!(loaded.mode, GameMode::AcidRain);
    assert_eq!(loaded.acid_drops, 11);
    let _ = std::fs::remove_file("savegame.json");
}

#[test]
fn test_acid_rain_drops_persist_through_time_rewind() {
    let mut game = acid_rain_game(20, 20);
    game.obstacles.clear();
    game.auto_pilot = true;

    // Run one scheduled downpour (tick 6)
    game.tick_counter = 5;
    game.update();
    keep_playing(&mut game);
    assert_eq!(game.acid_drops, 1, "expected exactly one downpour on tick 6");

    game.rewind_time();
    assert_eq!(
        game.acid_drops, 0,
        "time-rewind must restore the acid drop counter"
    );
}

#[test]
fn test_acid_rain_persists_through_history_rewind() {
    let mut game = acid_rain_game(30, 30);
    game.auto_pilot = true;

    // save_history_state runs at the start of every tick, so rewinding after
    // tick N restores the state as it was at the end of tick N - 1.
    let mut recorded = None;
    for i in 0..6 {
        game.state = GameState::Playing;
        game.update();
        if game.state != GameState::Playing {
            break;
        }
        if i == 4 {
            recorded = Some((game.acid_puddles.clone(), game.acid_drops));
        }
    }
    let Some((puddles_before, drops_before)) = recorded else {
        panic!("acid should be present after a few ticks");
    };

    // Scramble the live state, then rewind: the acid must be restored
    game.acid_puddles.clear();
    game.acid_drops = 999;
    game.rewind_time();

    assert_eq!(game.acid_puddles, puddles_before);
    assert_eq!(game.acid_drops, drops_before);
}

#[test]
fn test_acid_rain_bot_survives_the_rain() {
    let mut game = acid_rain_game(40, 30);
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
        "Autopilot should survive the acid rain (lives: {}, drops: {}, score: {})",
        game.lives,
        game.acid_drops,
        game.score
    );
    assert!(
        game.acid_drops >= 20,
        "expected at least 20 downpours over 120 ticks, got {}",
        game.acid_drops
    );
}
