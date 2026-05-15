use snake_game::*;

#[test]
fn test_is_safe_final_p_laser() {
    let mut game = game::Game::new(
        20,
        20,
        false,
        'x',
        game::Theme::Classic,
        game::Difficulty::Normal,
    );
    game.snake = snake::Snake::new(snake::Point { x: 5, y: 5 });

    // Laser at x=3, y=6 moving Right.
    game.lasers.push(game::Laser {
        position: snake::Point { x: 3, y: 6 },
        direction: snake::Direction::Right,
        player: 0,
    });

    // is_safe_final_p is called by A* to evaluate a position `steps` ahead.
    // If we want to check x=5, y=6 in 1 step, it should be unsafe.
    let safe = game.is_safe_final_p(snake::Point { x: 5, y: 6 }, 1, 1);
    println!("safe: {}", safe);
    assert!(!safe, "Should NOT be safe!");
}
