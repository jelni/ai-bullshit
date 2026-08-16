use snake_game::*;

#[test]
fn test_asteroids_mode() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.mode = game::GameMode::Asteroids;
    game.reset();

    for _ in 0..100 {
        game.update();
    }

    assert!(game.meteors.len() > 0, "Asteroids mode should spawn meteors frequently");
}
