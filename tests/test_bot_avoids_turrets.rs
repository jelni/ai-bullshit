use snake_game::game::turret::Turret;
use snake_game::game::{Difficulty, Game, GameMode, Theme};
use snake_game::snake::{Direction, Point};

#[test]
fn test_bot_avoids_turrets() {
    let mut game = Game::new(20, 20, false, ' ', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::PlayerVsBot;

    // Set bot to autopilot
    game.auto_pilot = true;

    // Set snake starting position
    game.snake.body.clear();
    game.snake.body.push_back(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    // Ensure the player is far away, so it doesn't try to move towards player
    // Autopilot moves towards targets: food, etc.
    game.food = Point { x: 10, y: 5 };

    // Place a turret directly in front of the bot
    game.turrets.push(Turret {
        position: Point {
            x: 6,
            y: 5,
        },
        shoot_timer: 0,
    });

    // The bot should change direction to avoid the turret
    let next_dir = game.calculate_autopilot_move();

    assert!(next_dir.is_some(), "Autopilot should find a move");
    // It should not go Right into the turret
    assert_ne!(next_dir.unwrap(), Direction::Right);
}
