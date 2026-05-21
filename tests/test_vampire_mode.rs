use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};

#[test]
fn test_vampire_mode_starvation() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Vampire;
    game.state = GameState::Playing;

    let initial_lives = game.lives;

    // Simulate food eaten 16 seconds ago
    let past_time =
        web_time::Instant::now().checked_sub(web_time::Duration::from_secs(16)).unwrap();
    game.last_food_time = Some(past_time);

    game.update();

    assert_eq!(
        game.lives,
        initial_lives.saturating_sub(1),
        "Snake should lose a life from vampire starvation"
    );
    assert!(game.just_died, "Game should mark just_died = true");
}
