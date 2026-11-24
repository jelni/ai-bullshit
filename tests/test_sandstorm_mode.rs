use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn sandstorm_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Sandstorm;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    // The Druid ignores ambient weather effects, so only the scheduled sandblasts
    // move the food in these tests (keeps the assertions deterministic).
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
fn test_sandstorm_reset_forces_sandstorm_weather() {
    let game = sandstorm_game(30, 30);

    assert_eq!(game.weather, Weather::Sandstorm);
    assert_eq!(game.sand_blasts, 0);
}

#[test]
fn test_sandstorm_weather_never_breaks() {
    let mut game = sandstorm_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..60 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        assert_eq!(
            game.weather,
            Weather::Sandstorm,
            "Sandstorm mode must force sandstorm weather on every tick"
        );
    }
}

#[test]
fn test_sandstorm_scheduled_blast_every_6_ticks() {
    let mut game = sandstorm_game(30, 30);
    game.obstacles.clear();
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must trigger a scheduled sandblast.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let blasts_before = game.sand_blasts;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.sand_blasts > blasts_before,
            "a scheduled sandblast must fire on tick {} (blasts before: {}, after: {})",
            tick + 1,
            blasts_before,
            game.sand_blasts
        );
    }
    assert!(game.sand_blasts >= 6);
}

#[test]
fn test_sandstorm_blast_shifts_food_to_adjacent_fair_cell() {
    let mut game = sandstorm_game(30, 30);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    let old_food = game.food;

    // Tick 6: scheduled sandblast must drift the food to an adjacent cell
    game.tick_counter = 5;
    game.update();

    assert!(
        game.sand_blasts >= 1,
        "a scheduled sandblast should have fired on tick 6"
    );
    let dx = game.food.x.abs_diff(old_food.x);
    let dy = game.food.y.abs_diff(old_food.y);
    assert!(
        dx + dy == 1,
        "a sandblast must move the food exactly one cell (old: {old_food:?}, new: {:?})",
        game.food
    );
    assert!(
        !game.obstacles.contains(&game.food),
        "the food must never land on an obstacle"
    );
    assert!(
        !game.snake.body_map.contains_key(&game.food),
        "the food must never land on the snake"
    );
}

#[test]
fn test_sandstorm_food_never_unfair_over_many_ticks() {
    let mut game = sandstorm_game(30, 30);
    game.obstacles.clear();
    game.auto_pilot = true;

    for _ in 0..120 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        assert!(
            !game.obstacles.contains(&game.food),
            "the food must never be covered by an obstacle (food: {:?})",
            game.food
        );
        assert!(
            !game.snake.body_map.contains_key(&game.food),
            "the food must never land on the snake body (food: {:?})",
            game.food
        );
        assert!(game.food.x >= 1 && game.food.x < game.width - 1);
        assert!(game.food.y >= 1 && game.food.y < game.height - 1);
    }

    assert!(
        game.sand_blasts >= 20,
        "expected at least 20 sandblasts over 120 ticks, got {}",
        game.sand_blasts
    );
}

#[test]
fn test_sandstorm_bot_survives_the_storm() {
    let mut game = sandstorm_game(40, 30);
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
        "Autopilot should survive the sandstorm (lives: {}, blasts: {}, score: {})",
        game.lives,
        game.sand_blasts,
        game.score
    );
    assert!(
        game.sand_blasts >= 20,
        "expected at least 20 sandblasts over 120 ticks, got {}",
        game.sand_blasts
    );
}

#[test]
fn test_sandstorm_blasts_persist_through_save_load() {
    let file_path = format!("sandstorm_save_test_{}.json", std::process::id());
    let _ = std::fs::remove_file(&file_path);

    let mut game = sandstorm_game(20, 20);
    game.sand_blasts = 11;
    game.save_game_to_file(&file_path);

    let json = std::fs::read_to_string(&file_path).expect("save file should exist");
    assert!(
        json.contains("\"sand_blasts\":11"),
        "the save file must contain the sandblast counter"
    );
    let _ = std::fs::remove_file(&file_path);

    // Round-trip through the public load path (reads savegame.json)
    let _ = std::fs::remove_file("savegame.json");
    game.save_game_to_file("savegame.json");
    let mut loaded = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    assert!(loaded.load_game());
    assert_eq!(loaded.mode, GameMode::Sandstorm);
    assert_eq!(loaded.sand_blasts, 11);
    let _ = std::fs::remove_file("savegame.json");
}

#[test]
fn test_sandstorm_blasts_persist_through_time_rewind() {
    let mut game = sandstorm_game(20, 20);
    game.obstacles.clear();
    game.auto_pilot = true;

    // Run one scheduled sandblast (tick 6)
    game.tick_counter = 5;
    game.update();
    keep_playing(&mut game);
    assert_eq!(game.sand_blasts, 1, "expected exactly one sandblast on tick 6");

    game.rewind_time();
    assert_eq!(
        game.sand_blasts, 0,
        "time-rewind must restore the sandblast counter"
    );
}
