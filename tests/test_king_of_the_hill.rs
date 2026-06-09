use snake_game::game::{Difficulty, Game, GameMode, Theme};
use snake_game::snake::{Direction, Point, Snake};

#[test]
fn test_bot_targets_koth_zone() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::KingOfTheHill;

    // Set up snake
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;

    // Place food far away
    game.food = Point { x: 15, y: 15 };

    // Set up koth_zone near snake
    game.koth_zone = Some(Point { x: 9, y: 5 });

    let move_dir = game.calculate_autopilot_move();

    // Snake should move towards the koth_zone, which is at (9, 5).
    // The snake is at (5, 5) and facing Right.
    // The closest path to (9, 5) is moving Right.
    assert_eq!(move_dir, Some(Direction::Right), "Bot should target the koth_zone");
}

#[test]
fn test_bot_p2_targets_koth_zone() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::KingOfTheHill;

    // Set up player 2
    let mut p2 = Snake::new(Point { x: 5, y: 5 });
    p2.direction = Direction::Right;
    game.player2 = Some(p2);

    // Place food far away
    game.food = Point { x: 15, y: 15 };

    // Set up koth_zone near snake
    game.koth_zone = Some(Point { x: 9, y: 5 });

    let move_dir = game.calculate_p2_autopilot_move();

    // Player 2 should move towards the koth_zone, which is at (9, 5).
    // The snake is at (5, 5) and facing Right.
    // The closest path to (9, 5) is moving Right.
    assert_eq!(move_dir, Some(Direction::Right), "P2 Bot should target the koth_zone");
}
