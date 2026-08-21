use snake_game::*;

#[test]
fn test_bot_predicts_meteor() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.snake = snake::Snake::new(snake::Point { x: 5, y: 5 });
    game.snake.direction = snake::Direction::Right; // current direction is Right
    game.food = snake::Point { x: 5, y: 10 }; // Food is down

    game.meteors.clear();
    game.obstacles.clear();

    // Meteor at x=5, y=3
    // It falls 1 unit every 2 ticks. It is at timer=0.
    // In 1 tick: timer=1, y=3. Snake moves Down to (5,6).
    // In 2 ticks: timer=0, y=4. Snake moves Down to (5,7).
    // In 3 ticks: timer=1, y=4. Snake moves Down to (5,8).
    // Wait, the meteor falls onto the snake's path.
    // Let's make a test where moving Right hits the meteor in EXACTLY `steps` ticks.
    // Let's say snake wants to move Right (to x=6, y=5).
    // Meteor is at x=6, y=4, timer=1.
    // Next tick: snake moves to (6,5). Meteor timer becomes 2 -> resets to 0, moves to y=5.
    // Collision!
    game.meteors.push(game::Meteor {
        position: snake::Point { x: 6, y: 4 },
        timer: 1,
    });

    let safe_right = game.is_safe_final_p(snake::Point { x: 6, y: 5 }, 1, 1);
    assert!(!safe_right, "Moving Right to (6,5) should be unsafe because meteor falls on it!");
}
