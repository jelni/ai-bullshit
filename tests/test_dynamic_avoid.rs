use snake_game::*;

#[test]
fn test_bot_predicts_laser() {
    let mut game = game::Game::new(
        20,
        20,
        false,
        'x',
        game::Theme::Classic,
        game::Difficulty::Normal,
    );
    game.snake = snake::Snake::new(snake::Point { x: 5, y: 5 });
    game.snake.direction = snake::Direction::Right;
    game.food = snake::Point { x: 5, y: 10 }; // Food is down

    // Laser at x=5, y=7 heading UP towards snake at x=5, y=5
    // In 1 tick, snake moves Down to 5,6. Laser moves Up 2 units to 5,5. They will cross!
    game.lasers.push(game::Laser {
        position: snake::Point { x: 5, y: 7 },
        direction: snake::Direction::Up,
        player: 0,
    });

    let next_move = game.calculate_autopilot_move();
    // Snake should NOT move Down to 5,6 because the laser will hit it.
    // So it should move Left or Right.
    assert_ne!(next_move, Some(snake::Direction::Down));
}
