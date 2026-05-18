use snake_game::game::{Difficulty, Game, GameMode, Theme};
use snake_game::snake::Point;

#[test]
fn test_flood_mode() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Flood;
    game.reset();

    // Verify game starts with some initial obstacles (if difficulty is Normal, there are 3 random obstacles).
    // Let's clear them so we can test the flood logic reliably.
    game.obstacles.clear();

    // Trigger rise_flood manually
    game.rise_flood();

    // Check if the bottom row (y = 19) is filled with obstacles
    for x in 0..game.width {
        assert!(
            game.obstacles.contains(&Point {
                x,
                y: 19
            }),
            "Bottom row should be flooded"
        );
    }

    // Trigger rise_flood again
    game.rise_flood();

    // Check if the next row (y = 18) is filled
    for x in 0..game.width {
        assert!(
            game.obstacles.contains(&Point {
                x,
                y: 18
            }),
            "Second row from bottom should be flooded"
        );
    }

    // Ensure the rest is still clear
    for y in 0..18 {
        for x in 0..game.width {
            assert!(
                !game.obstacles.contains(&Point {
                    x,
                    y
                }),
                "Upper rows should remain clear"
            );
        }
    }
}
