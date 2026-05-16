use snake_game::*;

#[test]
fn test_elo_integration() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.mode = game::GameMode::PlayerVsBot;

    // Set base ELO
    game.stats.player_elo = 1000;
    game.stats.bot_elo = 1000;

    // Simulate P1 dying (bot wins)
    // Setup a scenario where P1 walks into a wall next tick
    game.snake = snake::Snake::new(snake::Point {
        x: 1,
        y: 1,
    });
    game.snake.direction = snake::Direction::Left; // into the wall (x=0)

    let mut p2 = snake::Snake::new(snake::Point {
        x: 10,
        y: 10,
    });
    p2.direction = snake::Direction::Right; // safe
    game.player2 = Some(p2);

    // Ensure the game is in playing state so update runs
    game.state = game::GameState::Playing;

    // We expect the bot to win
    game.update();

    // Verify death and game over
    assert!(game.just_died || game.lives < 3, "P1 should have died");

    // The ELO should be updated
    assert!(game.stats.player_elo < 1000, "Player ELO should drop after losing to bot");
    assert!(game.stats.bot_elo > 1000, "Bot ELO should increase after winning against player");
}
