use snake_game::game::{Difficulty, Game, GameMode, Theme};
use snake_game::snake::{Direction, Point};

#[test]
fn test_capture_the_flag_initialization() {
    let mut game = Game::new(20, 20, false, '█', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::CaptureTheFlag;
    game.reset();

    assert!(game.p1_flag.is_some());
    assert!(game.p2_flag.is_some());
    assert!(!game.p1_has_flag);
    assert!(!game.p2_has_flag);
    assert_eq!(game.p1_score, 0);
    assert_eq!(game.p2_score, 0);
}

#[test]
fn test_capture_the_flag_pickup() {
    let mut game = Game::new(20, 20, false, '█', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::CaptureTheFlag;
    game.reset();

    // Position P1 right next to P2's flag
    if let Some(p2_flag) = game.p2_flag {
        game.snake.move_to(
            Point {
                x: p2_flag.x - 1,
                y: p2_flag.y,
            },
            false,
        );
        game.snake.direction_queue.push_back(Direction::Right);
    }

    // Simulate tick to move P1 into P2's flag
    game.update();

    assert!(game.p1_has_flag, "Player 1 should pick up the flag");
    assert!(game.p2_flag.is_none(), "Player 2's flag should disappear from the map");
}

#[test]
fn test_capture_the_flag_scoring() {
    let mut game = Game::new(20, 20, false, '█', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::CaptureTheFlag;
    game.reset();

    game.p1_has_flag = true;
    game.p2_flag = None;

    // Position P1 right next to P1's base to score
    let p1_base = Point {
        x: 2,
        y: game.height / 2,
    };
    game.snake.move_to(
        Point {
            x: p1_base.x + 1,
            y: p1_base.y,
        },
        false,
    );
    game.snake.direction_queue.push_back(Direction::Left);

    // Update tick
    game.update();

    assert_eq!(game.p1_score, 1, "Player 1 should score a point");
    assert!(!game.p1_has_flag, "Player 1 should no longer hold the flag");
    assert!(game.p2_flag.is_some(), "Player 2's flag should return to its base");
}

#[test]
fn test_capture_the_flag_death_drops_flag() {
    let mut game = Game::new(20, 20, false, '█', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::CaptureTheFlag;
    game.reset();

    game.p1_has_flag = true;
    game.p2_flag = None;

    // Force P1 into a wall
    game.snake.move_to(
        Point {
            x: 1,
            y: 1,
        },
        false,
    );
    game.snake.direction_queue.push_back(Direction::Up); // Move into top wall (y=0)

    game.update();

    // Player 1 should die and drop the flag
    assert!(!game.p1_has_flag, "Player 1 should lose the flag on death");
    assert!(game.p2_flag.is_some(), "Player 2's flag should be returned when Player 1 dies");
}
