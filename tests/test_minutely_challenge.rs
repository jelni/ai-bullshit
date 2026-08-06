use snake_game::game::{Game, GameMode, Theme, Difficulty};

#[test]
fn test_minutely_challenge_is_deterministic_within_minute() {
    let mut game1 = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game1.mode = GameMode::MinutelyChallenge;
    game1.reset();

    let mut game2 = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game2.mode = GameMode::MinutelyChallenge;
    game2.reset();

    // Since they were both restarted in the same minute, their PRNG and initial state should match.
    assert_eq!(game1.food, game2.food, "Food position should be identical for the same minute");
    assert_eq!(game1.obstacles, game2.obstacles, "Obstacles should be identical for the same minute");
    assert_eq!(game1.portals, game2.portals, "Portals should be identical for the same minute");
}
