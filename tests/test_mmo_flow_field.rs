use snake_game::*;

#[test]
fn test_mmo_flow_field() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.mode = game::GameMode::MassiveMultiplayer;
    game.reset();
    game.update();
    assert!(game.flow_field.is_some());
}
