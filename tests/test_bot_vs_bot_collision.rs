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
