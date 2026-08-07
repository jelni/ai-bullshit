use snake_game::game::{Difficulty, Game, GameMode, Theme};

#[test]
fn test_pacman_mode_generates_maze() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::PacMan;

    // obstacles are initially empty before calling reset/update logic that triggers it
    // let's just trigger reset to populate it
    game.reset();

    assert!(!game.obstacles.is_empty(), "PacMan mode should generate maze obstacles");
}
