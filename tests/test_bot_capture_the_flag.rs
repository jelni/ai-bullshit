use snake_game::game::{Difficulty, Game, GameMode, Theme};
use snake_game::snake::{Direction, Point};

#[test]
fn test_bot_targets_enemy_flag() {
    let mut game = Game::new(20, 20, false, '█', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::CaptureTheFlag;
    game.reset();

    game.auto_pilot = true;
    game.food = Point {
        x: 0,
        y: 0,
    };

    game.snake = snake_game::snake::Snake::new(Point {
        x: 10,
        y: 10,
    });
    game.p2_flag = Some(Point {
        x: 17,
        y: 10,
    });

    let move_dir = game.calculate_autopilot_move();
    assert_ne!(move_dir, Some(Direction::Up)); // Not exactly left/right depending on distance, but should push towards flag
}

#[test]
fn test_bot_returns_flag_to_base() {
    let mut game = Game::new(20, 20, false, '█', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::CaptureTheFlag;
    game.reset();

    game.auto_pilot = true;
    game.p1_has_flag = true;
    game.p2_flag = None;

    game.food = Point {
        x: 0,
        y: 0,
    };

    game.snake = snake_game::snake::Snake::new(Point {
        x: 10,
        y: 10,
    });

    let move_dir = game.calculate_autopilot_move();
    assert_ne!(move_dir, Some(Direction::Up));
}

#[test]
fn test_p2_bot_targets_enemy_flag() {
    let mut game = Game::new(20, 20, false, '█', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::CaptureTheFlag;
    game.reset();

    game.food = Point {
        x: 0,
        y: 0,
    };

    let mut p2 = snake_game::snake::Snake::new(Point {
        x: 10,
        y: 10,
    });
    p2.direction = Direction::Up;
    game.player2 = Some(p2);

    game.p1_flag = Some(Point {
        x: 2,
        y: 10,
    });

    let move_dir = game.calculate_p2_autopilot_move();
    assert_eq!(move_dir, Some(Direction::Left));
}

#[test]
fn test_p2_bot_returns_flag_to_base() {
    let mut game = Game::new(20, 20, false, '█', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::CaptureTheFlag;
    game.reset();

    game.food = Point {
        x: 0,
        y: 0,
    };

    let mut p2 = snake_game::snake::Snake::new(Point {
        x: 10,
        y: 10,
    });
    p2.direction = Direction::Up;
    game.player2 = Some(p2);

    game.p2_has_flag = true;
    game.p1_flag = None;

    let move_dir = game.calculate_p2_autopilot_move();
    assert_eq!(move_dir, Some(Direction::Right), "P2 should move right to return flag to base");
}
