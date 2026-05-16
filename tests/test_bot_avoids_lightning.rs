use snake_game::*;

#[test]
fn test_bot_avoids_lightning() {
    let mut game = game::Game::new(
        20,
        20,
        false,
        'x',
        game::Theme::Classic,
        game::Difficulty::Normal,
    );

    // Set up snake
    game.snake = snake::Snake::new(snake::Point { x: 5, y: 5 });
    game.snake.direction = snake::Direction::Right;

    // Set normal food far away
    game.food = snake::Point { x: 15, y: 5 };

    // Set weather to storm and spawn a lightning column exactly where the bot would normally go (Right to x=6)
    game.weather = game::Weather::Storm;
    game.lightning_column = Some(6);

    // Request a move
    let next_move = game.calculate_autopilot_move();

    // The shortest path is Right (into x=6). But because of lightning, it should avoid moving Right.
    assert!(next_move == Some(snake::Direction::Up) || next_move == Some(snake::Direction::Down),
            "Bot must avoid moving right into the active lightning column, instead chose {:?}", next_move);
}
