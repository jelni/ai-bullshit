use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn conveyor_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Conveyor;
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
fn test_conveyor_reset_seeds_belts_and_clear_weather() {
    let game = conveyor_game(40, 30);

    assert_eq!(game.weather, Weather::Clear);
    assert!(
        !game.belt_tiles.is_empty(),
        "Conveyor mode must seed belt tiles at reset"
    );
    assert_eq!(game.belt_shifts, 0);
    for p in game.belt_tiles.keys() {
        assert!(
            !game.obstacles.contains(p),
            "a belt must never sit on an obstacle: {p:?}"
        );
        assert!(
            !game.snake.body_map.contains_key(p),
            "a belt must never sit on the snake: {p:?}"
        );
        assert!(p.x >= 1 && p.x < game.width - 1);
        assert!(p.y >= 1 && p.y < game.height - 1);
    }
}

#[test]
fn test_conveyor_scheduled_shuffle_every_6_ticks() {
    let mut game = conveyor_game(30, 30);
    game.obstacles.clear();
    game.belt_tiles.clear();
    game.belt_tiles.insert(Point { x: 10, y: 10 }, Direction::Up);
    game.belt_tiles.insert(Point { x: 20, y: 20 }, Direction::Left);
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must trigger a scheduled belt shuffle.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let before = game.belt_shifts;
        let dirs_before: std::collections::HashMap<Point, Direction> = game.belt_tiles.clone();
        game.update();
        keep_playing(&mut game);
        assert!(
            game.belt_shifts > before,
            "a scheduled belt shuffle must fire on tick {} (lives: {})",
            tick + 1,
            game.lives
        );
        for (p, d) in &dirs_before {
            if let Some(after) = game.belt_tiles.get(p) {
                let rotated = match d {
                    Direction::Up => Direction::Right,
                    Direction::Right => Direction::Down,
                    Direction::Down => Direction::Left,
                    Direction::Left => Direction::Up,
                };
                assert_eq!(after, &rotated, "every surviving belt must rotate 90 degrees clockwise ({p:?})");
            }
        }
    }
    assert!(game.belt_shifts >= 6);
}

#[test]
fn test_conveyor_belt_shoves_snake_one_extra_cell() {
    let mut game = conveyor_game(30, 30);
    game.obstacles.clear();
    game.belt_tiles.clear();
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    // The snake enters (11,10); the belt there points Right, so it is shoved to (12,10)
    game.belt_tiles.insert(Point { x: 11, y: 10 }, Direction::Right);
    game.food = Point { x: 25, y: 25 };

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 12, y: 10 },
        "the belt must shove the snake one extra cell in its direction (head: {:?})",
        game.snake.head()
    );
}

#[test]
fn test_conveyor_belt_shove_fizzles_on_unsafe_destination() {
    let mut game = conveyor_game(30, 30);
    game.obstacles.clear();
    game.belt_tiles.clear();
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    // The belt at (11,10) points Right into an obstacle at (12,10): the shove fizzles
    game.belt_tiles.insert(Point { x: 11, y: 10 }, Direction::Right);
    game.obstacles.insert(Point { x: 12, y: 10 });
    game.food = Point { x: 25, y: 25 };

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "a shove into an obstacle must fizzle; the snake stays on the belt cell (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the fizzled shove must never kill the snake"
    );
}

#[test]
fn test_conveyor_shove_never_unfair_over_many_ticks() {
    let mut game = conveyor_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        for p in game.belt_tiles.keys() {
            assert!(
                !game.obstacles.contains(p),
                "a belt must never be covered by an obstacle: {p:?}"
            );
        }
    }

    assert!(
        game.belt_shifts >= 18,
        "expected at least 18 belt shuffles over 120 ticks (respawns reset the tick counter), got {}",
        game.belt_shifts
    );
}

#[test]
fn test_conveyor_eating_food_rotates_one_belt() {
    let mut game = conveyor_game(30, 30);
    game.obstacles.clear();
    game.belt_tiles.clear();
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.belt_tiles.insert(Point { x: 20, y: 20 }, Direction::Up);
    game.food = Point { x: 11, y: 10 };

    game.tick_counter = 0; // tick 1: no scheduled shuffle, only the eat effect
    game.state = GameState::Playing;
    game.update();

    assert!(game.score > 0, "the snake should have eaten the food");
    assert_eq!(
        game.belt_tiles.get(&Point { x: 20, y: 20 }),
        Some(&Direction::Right),
        "eating food must rotate one belt 90 degrees clockwise"
    );
}

#[test]
fn test_conveyor_bot_survives_the_factory_floor() {
    let mut game = conveyor_game(40, 30);
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
        "Autopilot should survive Conveyor mode (lives: {}, shifts: {}, score: {})",
        game.lives,
        game.belt_shifts,
        game.score
    );
    assert!(
        game.belt_shifts >= 18,
        "expected at least 18 belt shuffles over 120 ticks (respawns reset the tick counter), got {}",
        game.belt_shifts
    );
}

#[test]
fn test_conveyor_state_persists_through_save_load() {
    let file_path = format!("conveyor_save_test_{}.json", std::process::id());
    let _ = std::fs::remove_file(&file_path);

    let mut game = conveyor_game(20, 20);
    game.belt_tiles.clear();
    game.belt_tiles.insert(Point { x: 12, y: 9 }, Direction::Down);
    game.belt_shifts = 11;
    game.save_game_to_file(&file_path);

    let json = std::fs::read_to_string(&file_path).expect("save file should exist");
    assert!(
        json.contains("\"belt_shifts\":11"),
        "the save file must contain the belt shuffle counter"
    );
    assert!(
        json.contains("\"belt_tiles\""),
        "the save file must contain the belt tiles"
    );
    let _ = std::fs::remove_file(&file_path);

    // Round-trip through the public load path (reads savegame.json)
    let _ = std::fs::remove_file("savegame.json");
    game.save_game_to_file("savegame.json");
    let mut loaded = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    assert!(loaded.load_game());
    assert_eq!(loaded.mode, GameMode::Conveyor);
    assert_eq!(
        loaded.belt_tiles.get(&Point { x: 12, y: 9 }),
        Some(&Direction::Down)
    );
    assert_eq!(loaded.belt_shifts, 11);
    let _ = std::fs::remove_file("savegame.json");
}
