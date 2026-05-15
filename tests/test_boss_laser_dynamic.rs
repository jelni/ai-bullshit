use snake_game::*;

#[test]
fn test_bot_predicts_boss_laser() {
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
    game.food = snake::Point { x: 5, y: 10 };

    // Boss at x=5, y=7
    game.boss = Some(game::Boss {
        position: snake::Point { x: 5, y: 7 },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 14, // Normal threshold is 15. So it will shoot next tick!
    });

    let next_move = game.calculate_autopilot_move();
    println!("next_move: {:?}", next_move);
    assert_ne!(next_move, Some(snake::Direction::Down));
}
