use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn rink_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::IceRink;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot plays
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

fn open_rink(width: u16, height: u16) -> Game {
    let mut game = rink_game(width, height);
    game.obstacles.clear();
    game.ice_tiles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game
}

#[test]
fn test_ice_rink_reset_generates_ice_patches_and_clear_weather() {
    let game = rink_game(30, 30);

    assert_eq!(game.weather, Weather::Clear);
    assert_eq!(game.ice_slips, 0);
    assert!(
        !game.ice_tiles.is_empty(),
        "Ice Rink mode should scatter ice patches on the board"
    );
    for ice in &game.ice_tiles {
        assert!(!game.obstacles.contains(ice), "ice patches must be walkable");
        assert!(!game.snake.body_map.contains_key(ice));
    }
    assert!(!game.ice_tiles.contains(&game.food));
}

#[test]
fn test_ice_rink_weather_never_breaks() {
    let mut game = rink_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..60 {
        game.state = GameState::Playing;
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        assert_eq!(
            game.weather,
            Weather::Clear,
            "Ice Rink mode must keep clear weather on every tick"
        );
    }
}

#[test]
fn test_ice_rink_turn_blocked_on_ice() {
    let mut game = open_rink(20, 20);
    game.ice_tiles.insert(Point { x: 5, y: 5 }); // the snake's head is on ice
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Up);

    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.direction,
        Direction::Right,
        "a turn on an ice patch must be ignored (the snake keeps its direction)"
    );
    assert_eq!(game.ice_slips, 1, "the blocked turn must be counted as a slip");
}

#[test]
fn test_ice_rink_turn_allowed_off_ice() {
    let mut game = open_rink(20, 20); // no ice patches: the head is on normal floor
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Up);

    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.direction,
        Direction::Up,
        "a turn off the ice must be applied"
    );
    assert_eq!(game.ice_slips, 0);
    assert_eq!(game.snake.head(), Point { x: 5, y: 4 });
}

#[test]
fn test_ice_rink_straight_ahead_on_ice_is_allowed() {
    let mut game = open_rink(20, 20);
    game.ice_tiles.insert(Point { x: 5, y: 5 });
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);

    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.snake.direction, Direction::Right);
    assert_eq!(game.ice_slips, 0, "continuing straight is never a slip");
    assert_eq!(game.snake.head(), Point { x: 6, y: 5 });
}

#[test]
fn test_ice_rink_path_is_safe_rejects_turns_on_ice() {
    let mut game = open_rink(30, 30);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;

    // Straight then a turn at (12, 10)
    let path = vec![
        Point { x: 11, y: 10 },
        Point { x: 12, y: 10 },
        Point { x: 12, y: 11 },
    ];
    assert!(
        game.path_is_safe(Point { x: 10, y: 10 }, &path, &[], 1),
        "a turn off the ice must be safe"
    );

    game.ice_tiles.insert(Point { x: 12, y: 10 });
    assert!(
        !game.path_is_safe(Point { x: 10, y: 10 }, &path, &[], 1),
        "path_is_safe must reject a path that turns on an ice patch"
    );

    let straight = vec![
        Point { x: 11, y: 10 },
        Point { x: 12, y: 10 },
        Point { x: 13, y: 10 },
    ];
    assert!(
        game.path_is_safe(Point { x: 10, y: 10 }, &straight, &[], 1),
        "a straight path across ice must stay safe"
    );
}

#[test]
fn test_ice_rink_astar_never_turns_on_ice() {
    let mut game = open_rink(30, 30);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.food = Point { x: 10, y: 16 };
    game.ice_tiles.insert(Point { x: 10, y: 10 }); // start cell is ice

    let Some((first_dir, path)) =
        game.astar_search_verified(Point { x: 10, y: 10 }, Some(Direction::Right), &[game.food], 1)
    else {
        panic!("astar should still find a route to the food");
    };

    assert_eq!(
        first_dir,
        Direction::Right,
        "the first step from an ice cell must keep the current direction"
    );
    assert!(
        game.path_is_safe(Point { x: 10, y: 10 }, &path, &[game.food], 1),
        "the returned path must be ice-legal (verified-safe)"
    );
    let start = Point { x: 10, y: 10 };
    for i in 1..path.len() {
        let prev = if i == 1 { start } else { path[i - 2] };
        let cell = path[i - 1];
        let nxt = path[i];
        let dir_in = (
            i32::from(cell.x) - i32::from(prev.x),
            i32::from(cell.y) - i32::from(prev.y),
        );
        let dir_out = (
            i32::from(nxt.x) - i32::from(cell.x),
            i32::from(nxt.y) - i32::from(cell.y),
        );
        let turned = if i == 1 {
            dir_out != (1, 0) // start direction was Right
        } else {
            dir_in != dir_out
        };
        if turned {
            assert!(
                !game.ice_tiles.contains(&cell),
                "astar turned at {cell:?} which is an ice patch (path: {path:?})"
            );
        }
    }
}

#[test]
fn test_ice_rink_persists_through_save_load() {
    let file_path = format!("ice_rink_save_test_{}.json", std::process::id());
    let _ = std::fs::remove_file(&file_path);

    let mut game = rink_game(20, 20);
    game.ice_tiles.clear();
    game.ice_tiles.insert(Point { x: 12, y: 9 });
    game.ice_slips = 7;
    game.save_game_to_file(&file_path);

    let json = std::fs::read_to_string(&file_path).expect("save file should exist");
    assert!(
        json.contains("\"ice_slips\":7"),
        "the save file must contain the slip counter"
    );
    let _ = std::fs::remove_file(&file_path);

    let _ = std::fs::remove_file("savegame.json");
    game.save_game_to_file("savegame.json");
    let mut loaded = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    assert!(loaded.load_game());
    assert_eq!(loaded.mode, GameMode::IceRink);
    assert!(loaded.ice_tiles.contains(&Point { x: 12, y: 9 }));
    assert_eq!(loaded.ice_slips, 7);
    let _ = std::fs::remove_file("savegame.json");
}

#[test]
fn test_ice_rink_persists_through_time_rewind() {
    let mut game = open_rink(20, 20);
    game.ice_tiles.insert(Point { x: 8, y: 5 });
    game.auto_pilot = true;

    game.state = GameState::Playing;
    game.update();
    keep_playing(&mut game);
    let (ice_before, slips_before) = (game.ice_tiles.clone(), game.ice_slips);

    game.ice_tiles.clear();
    game.ice_slips = 999;
    game.rewind_time();

    assert_eq!(game.ice_tiles, ice_before);
    assert_eq!(game.ice_slips, slips_before);
}

#[test]
fn test_ice_rink_bot_survives_on_ice() {
    let mut game = rink_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.state = GameState::Playing;
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
    }

    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "Autopilot should survive on the ice rink (lives: {}, slips: {}, score: {})",
        game.lives,
        game.ice_slips,
        game.score
    );
}
