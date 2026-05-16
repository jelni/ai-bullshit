use snake_game::*;

#[test]
fn test_is_safe_final_p_boss() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.snake = snake::Snake::new(snake::Point {
        x: 5,
        y: 5,
    });

    // Boss at x=5, y=7
    game.boss = Some(game::Boss {
        position: snake::Point {
            x: 5,
            y: 7,
        },
        health: 10,
        max_health: 10,
        move_timer: 1, // Will move in 1 step! (threshold 2)
        shoot_timer: 0,
    });

    // is_safe_final_p is called by A* to evaluate a position `steps` ahead.
    // Boss will move to (5,6) to chase the snake at (5,5) in 1 step.
    // If we want to check x=5, y=6 in 1 step, it should be unsafe.
    let safe = game.is_safe_final_p(
        snake::Point {
            x: 5,
            y: 6,
        },
        1,
        1,
    );
    println!("safe: {}", safe);
    assert!(!safe, "Should NOT be safe!");
}
