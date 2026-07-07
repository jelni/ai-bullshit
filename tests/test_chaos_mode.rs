use snake_game::*;

#[test]
fn test_chaos_mode_events() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.mode = game::GameMode::Chaos;

    // Clear initial state
    game.bosses.clear();
    game.weather = game::Weather::Clear;

    // Advance tick to trigger weather
    game.tick_counter = 99;
    game.update();

    // Advance tick to trigger boss
    game.state = game::GameState::Playing;
    game.tick_counter = 499;
    let initial_boss_count = game.bosses.len();
    game.update();
    assert_eq!(game.bosses.len(), initial_boss_count + 1);
}
