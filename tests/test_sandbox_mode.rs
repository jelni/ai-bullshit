use snake_game::*;

#[test]
fn test_sandbox_mode_infinite_lives() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.mode = game::GameMode::Sandbox;
    game.reset();

    game.lives = 1; // Start with 1 life

    // Trigger death manually by changing state internally
    let head = game.snake.head();
    let wall_pos = snake::Point {
        x: 0,
        y: head.y,
    };
    game.snake.move_to(wall_pos, false);

    game.update();

    // In Sandbox mode, we should have triggered a death
    assert!(game.just_died);
    assert_eq!(game.lives, 1); // Lives should not decrease
    // We can't directly check GameState::GameOver easily because it doesn't derive Debug,
    // but we can check it indirectly by pattern matching
    assert!(matches!(game.state, game::GameState::Playing | game::GameState::Paused));
}
