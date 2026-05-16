use snake_game::*;

#[test]
fn test_boss_blocks_all_paths() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.snake = snake::Snake::new(snake::Point {
        x: 1,
        y: 1,
    });
    game.snake.direction = snake::Direction::Right;
    game.food = snake::Point {
        x: 18,
        y: 18,
    }; // Food is far away

    // Boss is at x=10, y=10
    game.boss = Some(game::Boss {
        position: snake::Point {
            x: 10,
            y: 10,
        },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
    });

    let next_move = game.calculate_autopilot_move();
    // It should find a path!
    assert!(
        next_move.is_some(),
        "Autopilot failed to find a path because boss blocked everything!"
    );
}
