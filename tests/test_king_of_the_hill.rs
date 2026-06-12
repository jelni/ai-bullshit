use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

#[test]
fn test_bot_targets_koth_zone() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::KingOfTheHill;
    game.obstacles.clear();

    // Set koth_zone
    game.koth_zone = Some(Point {
        x: 2,
        y: 2,
    });

    // Set snake
    game.snake = Snake::new(Point {
        x: 5,
        y: 2,
    });
    game.snake.direction = Direction::Left;

    // Calculate path
    let next_move = game.calculate_autopilot_move();
    assert_eq!(next_move, Some(Direction::Left), "Bot should move towards koth_zone");
}

#[test]
fn test_bot_p2_targets_koth_zone() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::KingOfTheHill;
    game.obstacles.clear();

    // Set koth_zone
    game.koth_zone = Some(Point {
        x: 2,
        y: 2,
    });

    // Set p2
    let mut p2 = Snake::new(Point {
        x: 5,
        y: 2,
    });
    p2.direction = Direction::Left;
    game.player2 = Some(p2);

    // Calculate path
    let next_move = game.calculate_p2_autopilot_move();
    assert_eq!(next_move, Some(Direction::Left), "P2 Bot should move towards koth_zone");
}

#[test]
fn test_bot_targets_koth_zone_bots() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::KingOfTheHill;
    game.obstacles.clear();
    game.bots.clear();

    // Set koth_zone
    game.koth_zone = Some(Point {
        x: 2,
        y: 2,
    });

    // Set bot
    let mut bot = Snake::new(Point {
        x: 5,
        y: 2,
    });
    bot.direction = Direction::Left;
    game.bots.push(bot);
    game.bots_autopilot_paths.push(vec![]);

    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.bots[0].direction, Direction::Left, "Bot should move towards koth_zone");
}
