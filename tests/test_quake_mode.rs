use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn quake_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Quake;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_quake_reset_forces_earthquake_weather() {
    let game = quake_game(30, 30);

    assert_eq!(game.weather, Weather::Earthquake);
    assert_eq!(game.quake_count, 0);
}

#[test]
fn test_quake_weather_never_breaks() {
    let mut game = quake_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..60 {
        game.update();
        if game.state != GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = GameState::Playing;
        }
        assert_eq!(
            game.weather,
            Weather::Earthquake,
            "Quake mode must force earthquake weather on every tick"
        );
    }
}

#[test]
fn test_quake_scheduled_quake_every_6_ticks() {
    let mut game = quake_game(30, 30);
    game.obstacles.clear();
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must trigger a scheduled quake.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let quakes_before = game.quake_count;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.quake_count > quakes_before,
            "a scheduled quake must fire on tick {} (quakes before: {}, after: {})",
            tick + 1,
            quakes_before,
            game.quake_count
        );
    }
    assert!(game.quake_count >= 6);
}

#[test]
fn test_quake_never_drops_rubble_on_snake_or_next_cell() {
    let mut game = quake_game(20, 20);
    // Druid ignores the ambient weather effects, so only the scheduled quake can move obstacles
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = false;

    // Next update runs tick 6: a scheduled quake fires with no obstacles around,
    // so it must drop exactly one rubble pile on a fair cell.
    game.tick_counter = 5;
    game.update();

    assert_eq!(game.quake_count, 1, "a scheduled quake should have fired on tick 6");
    let next = Point { x: 6, y: 5 };
    assert!(
        !game.obstacles.contains(&next),
        "a quake must never drop rubble on the cell the snake is about to enter"
    );
    for seg in &game.snake.body {
        assert!(
            !game.obstacles.contains(seg),
            "a quake must never drop rubble on the snake's body"
        );
    }
}

#[test]
fn test_quake_fairness_over_many_ticks() {
    let mut game = quake_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = true;
    let lives_before = game.lives;

    for _ in 0..60 {
        let head = game.snake.head();
        let next = match game.snake.direction {
            Direction::Up => Point {
                x: head.x,
                y: head.y.saturating_sub(1),
            },
            Direction::Down => Point {
                x: head.x,
                y: head.y.saturating_add(1).min(game.height - 1),
            },
            Direction::Left => Point {
                x: head.x.saturating_sub(1),
                y: head.y,
            },
            Direction::Right => Point {
                x: head.x.saturating_add(1).min(game.width - 1),
                y: head.y,
            },
        };
        let quakes_before = game.quake_count;
        game.update();
        if game.state != GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = GameState::Playing;
        }
        if game.quake_count > quakes_before {
            assert!(
                !game.obstacles.contains(&next),
                "scheduled quake dropped rubble on the cell the snake was about to enter ({next:?})"
            );
        }
    }

    assert_eq!(
        game.lives, lives_before,
        "the snake must survive the quakes (quakes: {})",
        game.quake_count
    );
    assert!(
        game.quake_count >= 10,
        "expected at least 10 scheduled quakes over 60 ticks, got {}",
        game.quake_count
    );
}

#[test]
fn test_quake_eating_food_stabilizes_the_ground() {
    let mut game = quake_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    let rubble = Point { x: 15, y: 15 };
    game.obstacles.insert(rubble);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.auto_pilot = false;

    // Tick 5 is not a multiple of 6, so no scheduled quake interferes here.
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.obstacles.contains(&rubble),
        "Eating food in Quake mode should stabilize the ground and clear a rubble pile"
    );
}

#[test]
fn test_quake_bot_survives_the_earthquake() {
    let mut game = quake_game(40, 30);
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
        "Autopilot should survive the earthquake (lives: {}, quakes: {}, score: {})",
        game.lives,
        game.quake_count,
        game.score
    );
    assert!(
        game.quake_count >= 20,
        "expected at least 20 quakes over 120 ticks, got {}",
        game.quake_count
    );
}

#[test]
fn test_quake_count_persists_through_history_rewind() {
    let mut game = quake_game(30, 30);
    game.quake_count = 7;
    game.save_history_state();
    game.quake_count = 0;

    game.rewind_time();

    assert_eq!(
        game.quake_count, 7,
        "quake_count must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_quake_count_persists_in_save_file() {
    let game = quake_game(30, 30);
    let mut game = game;
    game.quake_count = 9;
    let path = std::env::temp_dir().join(format!(
        "quake_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"quake_count\":9"),
        "SaveState JSON must contain the quake_count counter, got: {}",
        &json[..json.len().min(200)]
    );
}
