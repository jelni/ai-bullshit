use snake_game::*;

#[test]
fn test_bot_predicts_p2_movement() {
    let mut game = game::Game::new(
        20,
        20,
        false,
        'x',
        game::Theme::Classic,
        game::Difficulty::Normal,
    );
    game.mode = game::GameMode::BotVsBot;

    // P1 going Right
    game.snake = snake::Snake::new(snake::Point { x: 5, y: 5 });
    game.snake.direction = snake::Direction::Right;

    // P2 going Up, right towards the path of P1
    let mut p2 = snake::Snake::new(snake::Point { x: 6, y: 6 });
    p2.direction = snake::Direction::Up;
    game.player2 = Some(p2);

    game.food = snake::Point { x: 10, y: 5 }; // Food is to the right

    // P1 wants to go Right. But if it goes Right (to 6,5), and P2 goes Up (to 6,5), they will collide!
    // A smart bot should predict P2's movement and avoid (6,5).
    let next_move = game.calculate_autopilot_move();
    assert_ne!(next_move, Some(snake::Direction::Right), "P1 should avoid moving into P2's predicted next head position");
}

#[test]
fn test_bot_predicts_p2_turning() {
    let mut game = game::Game::new(
        20,
        20,
        false,
        'x',
        game::Theme::Classic,
        game::Difficulty::Normal,
    );
    game.mode = game::GameMode::BotVsBot;

    // P1 going Right, head at (5, 5). Target food is at (10, 5).
    // If P1 moves Right, it will go to (6, 5).
    game.snake = snake::Snake::new(snake::Point { x: 5, y: 5 });
    game.snake.direction = snake::Direction::Right;

    // P2 is at (6, 4).
    // P2 is facing Right.
    // P2's possible moves: Right (7, 4), Down (6, 5), Up (6, 3).
    // P2 could turn Down to (6, 5)!
    // We want P1 to avoid (6, 5) because P2 COULD turn there.
    let mut p2 = snake::Snake::new(snake::Point { x: 6, y: 4 });
    p2.direction = snake::Direction::Right;
    game.player2 = Some(p2);

    game.food = snake::Point { x: 10, y: 5 }; // Food is to the right

    // P1 wants to go Right to (6, 5). But P2 could turn Down to (6, 5).
    // The bot should predict this potential turn and avoid (6, 5).
    let next_move = game.calculate_autopilot_move();
    assert_ne!(next_move, Some(snake::Direction::Right), "P1 should avoid moving into P2's potential turn space");
}
