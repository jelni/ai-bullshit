use snake_game::game::{Game, GameMode, Difficulty, Theme};

#[test]
fn test_massive_multiplayer_mode() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::MassiveMultiplayer;
    game.reset();

    assert!(game.auto_pilot, "MassiveMultiplayer should enable auto_pilot");
    assert!(!game.chat_log.is_empty(), "MassiveMultiplayer should log simulation text");
    assert_eq!(game.chat_log[0].0, "SYSTEM: Simulating 100 bots entering the arena...");
}
