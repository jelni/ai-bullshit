use snake_game::*;

#[test]
fn test_laser_hits_before_move() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.state = game::GameState::Playing;
    game.snake = snake::Snake::new(snake::Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = snake::Direction::Up; // Snake wants to move Up to (5,4)
    game.snake.direction_queue.push_back(snake::Direction::Up);

    // Laser at (6,5) moving Left.
    game.lasers.push(game::Laser {
        position: snake::Point {
            x: 6,
            y: 5,
        },
        direction: snake::Direction::Left,
        player: 0,
    });

    game.update();

    // Is the snake dead?
    assert!(
        game.just_died || game.lives < 3,
        "Snake should be dead because laser moved first and hit (5,5)"
    );
}
