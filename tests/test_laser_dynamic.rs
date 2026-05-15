use snake_game::*;

#[test]
fn test_bot_predicts_laser_collision() {
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
    game.food = snake::Point { x: 5, y: 10 }; // Target food down

    // Laser at x=3, y=6 moving Right.
    // In 1 tick, laser moves to x=5, y=6.
    // If snake moves Down, it will be at x=5, y=6. Collision!
    game.lasers.push(game::Laser {
        position: snake::Point { x: 3, y: 6 },
        direction: snake::Direction::Right,
        player: 0,
    });

    let next_move = game.calculate_autopilot_move();

    // Should avoid Down
    println!("next_move: {:?}", next_move);
    assert_ne!(next_move, Some(snake::Direction::Down));
}
