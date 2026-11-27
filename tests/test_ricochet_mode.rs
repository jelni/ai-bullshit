use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn ricochet_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Ricochet;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

/// Build a snake whose body extends horizontally, so vertical bounces do not
/// run into its own body (mirrors real play where the body trails behind).
fn snake_at(x: u16, y: u16, dir: Direction) -> Snake {
    let mut s = Snake::new(Point { x, y });
    if matches!(dir, Direction::Up | Direction::Down) {
        s.body.clear();
        s.body.push_back(Point { x, y });
        s.body.push_back(Point { x: x - 1, y });
        s.body.push_back(Point { x: x - 2, y });
        s.rebuild_map();
    }
    s.direction = dir;
    s.direction_queue.push_back(dir);
    s
}

#[test]
fn test_ricochet_reset_clears_bounce_counter() {
    let game = ricochet_game(20, 20);
    assert_eq!(game.ricochet_bounces, 0);
    assert_eq!(game.mode, GameMode::Ricochet);
}

#[test]
fn test_ricochet_bounce_off_left_wall() {
    let mut game = ricochet_game(20, 20);
    game.obstacles.clear();
    game.auto_pilot = false;
    game.snake = snake_at(1, 5, Direction::Left);

    game.state = GameState::Playing;
    game.update();

    assert!(
        game.lives == 3 && !game.just_died,
        "the snake must survive the wall bounce (lives: {}, just_died: {})",
        game.lives,
        game.just_died
    );
    assert_eq!(game.ricochet_bounces, 1, "the bounce must be counted");
    assert_eq!(
        game.snake.head(),
        Point { x: 2, y: 5 },
        "the snake must bounce back inside the board"
    );
    assert_eq!(
        game.snake.direction,
        Direction::Right,
        "the bounce must reverse the direction"
    );
}

#[test]
fn test_ricochet_bounce_off_right_wall() {
    let mut game = ricochet_game(20, 20);
    game.obstacles.clear();
    game.auto_pilot = false;
    game.snake = snake_at(18, 5, Direction::Right);

    game.state = GameState::Playing;
    game.update();

    assert!(game.lives == 3 && !game.just_died);
    assert_eq!(game.ricochet_bounces, 1);
    assert_eq!(game.snake.head(), Point { x: 17, y: 5 });
    assert_eq!(game.snake.direction, Direction::Left);
}

#[test]
fn test_ricochet_bounce_off_top_wall() {
    let mut game = ricochet_game(20, 20);
    game.obstacles.clear();
    game.auto_pilot = false;
    game.snake = snake_at(5, 1, Direction::Up);

    game.state = GameState::Playing;
    game.update();

    assert!(game.lives == 3 && !game.just_died);
    assert_eq!(game.ricochet_bounces, 1);
    assert_eq!(game.snake.head(), Point { x: 5, y: 2 });
    assert_eq!(game.snake.direction, Direction::Down);
}

#[test]
fn test_ricochet_bounce_off_bottom_wall() {
    let mut game = ricochet_game(20, 20);
    game.obstacles.clear();
    game.auto_pilot = false;
    game.snake = snake_at(5, 18, Direction::Down);

    game.state = GameState::Playing;
    game.update();

    assert!(game.lives == 3 && !game.just_died);
    assert_eq!(game.ricochet_bounces, 1);
    assert_eq!(game.snake.head(), Point { x: 5, y: 17 });
    assert_eq!(game.snake.direction, Direction::Up);
}

#[test]
fn test_ricochet_repeated_bounces_never_kill() {
    let mut game = ricochet_game(20, 20);
    game.obstacles.clear();
    game.auto_pilot = false;

    // Place the snake one cell from the left wall heading into it, three times:
    // every hit must bounce, never kill
    for i in 0..3_u32 {
        game.state = GameState::Playing;
        game.snake = snake_at(1, 5, Direction::Left);
        let lives_before = game.lives;
        game.update();
        assert_eq!(
            game.lives, lives_before,
            "a wall bounce must never cost a life (head: {:?})",
            game.snake.head()
        );
        assert_eq!(game.ricochet_bounces, i + 1);
        assert_eq!(game.snake.head(), Point { x: 2, y: 5 });
        assert_eq!(game.snake.direction, Direction::Right);
    }
}

#[test]
fn test_ricochet_walls_still_lethal_in_other_modes() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::SinglePlayer;
    game.reset();
    game.xp_to_next_level = u32::MAX;
    game.obstacles.clear();
    game.auto_pilot = false;
    game.snake = snake_at(1, 5, Direction::Left);

    game.state = GameState::Playing;
    game.update();

    assert!(
        game.just_died || game.lives < 3,
        "in SinglePlayer the wall must still be lethal"
    );
}

#[test]
fn test_ricochet_no_bounce_when_not_at_wall() {
    let mut game = ricochet_game(20, 20);
    game.obstacles.clear();
    game.auto_pilot = false;
    game.snake = snake_at(10, 10, Direction::Right);

    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.ricochet_bounces, 0);
    assert_eq!(game.snake.head(), Point { x: 11, y: 10 });
    assert!(game.lives == 3);
}

#[test]
fn test_ricochet_bounces_persist_through_save_load() {
    let file_path = format!("ricochet_save_test_{}.json", std::process::id());
    let _ = std::fs::remove_file(&file_path);

    let mut game = ricochet_game(20, 20);
    game.ricochet_bounces = 9;
    game.save_game_to_file(&file_path);

    let json = std::fs::read_to_string(&file_path).expect("save file should exist");
    assert!(
        json.contains("\"ricochet_bounces\":9"),
        "the save file must contain the bounce counter"
    );
    let _ = std::fs::remove_file(&file_path);

    // Round-trip through the public load path (reads savegame.json)
    let _ = std::fs::remove_file("savegame.json");
    game.save_game_to_file("savegame.json");
    let mut loaded = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    assert!(loaded.load_game());
    assert_eq!(loaded.mode, GameMode::Ricochet);
    assert_eq!(loaded.ricochet_bounces, 9);
    let _ = std::fs::remove_file("savegame.json");
}

#[test]
fn test_ricochet_bounces_persist_through_time_rewind() {
    let mut game = ricochet_game(20, 20);
    game.obstacles.clear();
    game.auto_pilot = false;
    game.snake = snake_at(1, 5, Direction::Left);

    game.state = GameState::Playing;
    game.update();
    assert_eq!(game.ricochet_bounces, 1, "expected exactly one bounce");

    game.rewind_time();
    assert_eq!(
        game.ricochet_bounces, 0,
        "time-rewind must restore the bounce counter"
    );
}

#[test]
fn test_ricochet_bot_survives() {
    let mut game = ricochet_game(40, 30);
    game.auto_pilot = true;
    game.lives = 5;

    for _ in 0..120 {
        game.update();
        if game.state != GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = GameState::Playing;
        }
    }

    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "Autopilot should survive Ricochet mode (lives: {}, bounces: {}, score: {})",
        game.lives,
        game.ricochet_bounces,
        game.score
    );
}
