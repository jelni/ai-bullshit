use snake_game::game::{Difficulty, Game, GameMode, Theme};

#[test]
fn test_bullet_hell_spawns_lasers() {
    let mut game = Game::new(100, 100, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::BulletHell;
    game.reset();

    // Verify initial laser count
    let initial_lasers = game.lasers.len();

    // Fast-forward time significantly by shifting the start time
    game.start_time = game.start_time.checked_sub(web_time::Duration::from_secs(200)).unwrap();

    game.state = snake_game::game::GameState::Playing;

    // Update the game loop multiple times to trigger the probability-based laser spawning
    for _ in 0..100 {
        game.update();
    }

    // Since spawn rate goes up over time and we hit the random chances 100 times,
    // we expect lasers to be generated.
    assert!(
        game.lasers.len() > initial_lasers,
        "BulletHell should have spawned multiple lasers over time."
    );

    // Also assert that scores are awarded based on survival ticks
    assert!(game.score > 0, "BulletHell should award points simply for surviving ticks.");
}
