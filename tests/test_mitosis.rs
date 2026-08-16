use snake_game::game::{Difficulty, Game, PowerUp, PowerUpType, Theme};
use snake_game::snake::{Direction, Point, Snake};

#[test]
fn test_mitosis_powerup_spawns_bot() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();

    // Set up a snake
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    // Spawn a Mitosis powerup right in front of the snake
    game.power_up = Some(PowerUp {
        p_type: PowerUpType::Mitosis,
        location: Point {
            x: 6,
            y: 5,
        },
        activation_time: None,
    });

    // Verify initially no bots
    assert_eq!(game.bots.len(), 0, "There should be no bots initially.");

    game.state = snake_game::game::GameState::Playing;
    // Advance tick
    game.update();

    // Verify that a bot spawned
    assert_eq!(game.bots.len(), 1, "Consuming Mitosis should spawn 1 bot.");
    assert!(game.power_up.is_none(), "Power up should be consumed.");
}
