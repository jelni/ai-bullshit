use snake_game::*;

#[test]
fn test_bot_avoids_black_hole() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);

    // Set up snake
    game.snake = snake::Snake::new(snake::Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = snake::Direction::Right;

    // Set normal food far away
    game.food = snake::Point {
        x: 15,
        y: 5,
    };

    // Set black_hole right in front of the bot
    game.black_hole = Some(snake::Point {
        x: 6,
        y: 5,
    });

    game.obstacles.clear();
    // Request a move
    let next_move = game.calculate_autopilot_move();

    // The shortest path is Right (into x=6). But because of black_hole, it should avoid moving Right.
    assert!(
        next_move == Some(snake::Direction::Up) || next_move == Some(snake::Direction::Down),
        "Bot must avoid moving right into the black hole, instead chose {next_move:?}",
    );
}
