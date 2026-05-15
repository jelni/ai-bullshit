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
fn test_bot_predicts_p2_portal_movement() {
    let mut game = game::Game::new(
        20,
        20,
        false,
        'x',
        game::Theme::Classic,
        game::Difficulty::Normal,
    );
    game.mode = game::GameMode::BotVsBot;

    // Add portals at (6,6) and (10,10)
    game.portals = Some((snake::Point { x: 6, y: 6 }, snake::Point { x: 10, y: 10 }));

    // P1 going Right at y=10
    game.snake = snake::Snake::new(snake::Point { x: 9, y: 10 });
    game.snake.direction = snake::Direction::Right;

    // P2 going Down towards the first portal at (6,6)
    let mut p2 = snake::Snake::new(snake::Point { x: 6, y: 5 });
    p2.direction = snake::Direction::Down;
    game.player2 = Some(p2);

    game.food = snake::Point { x: 15, y: 10 }; // Food is to the right

    // P1 wants to go Right (to 10,10).
    // P2 is at (6,5) going Down. In the next step it enters the portal at (6,6),
    // teleporting it instantly to (10,10).
    // So if P1 moves Right to (10,10), they will collide!
    // A smart bot using get_final_p should predict this and NOT move to (10,10).
    let next_move = game.calculate_autopilot_move();
    assert_ne!(next_move, Some(snake::Direction::Right), "P1 should avoid moving into P2's predicted teleport destination");
}
