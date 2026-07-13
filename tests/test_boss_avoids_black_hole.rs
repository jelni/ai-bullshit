use snake_game::*;

#[test]
fn test_boss_avoids_black_hole() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);

    let start = snake::Point {
        x: 5,
        y: 5,
    };
    let target = snake::Point {
        x: 10,
        y: 5,
    };

    game.black_hole = Some(snake::Point {
        x: 7,
        y: 5,
    });

    let boss_kind = game::BossType::Shooter;

    let path_end = game.get_boss_path(start, target, boss_kind);

    assert_ne!(
        path_end,
        Some(snake::Direction::Right),
        "Boss should avoid going straight into the black hole"
    );
    assert!(
        path_end == Some(snake::Direction::Up) || path_end == Some(snake::Direction::Down),
        "Boss should move up or down to avoid the black hole"
    );
}
