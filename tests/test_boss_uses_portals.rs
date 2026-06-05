use snake_game::*;

#[test]
fn test_boss_uses_portals_to_reach_target() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);

    let start = snake::Point {
        x: 1,
        y: 1,
    };
    let target = snake::Point {
        x: 18,
        y: 18,
    };

    // Portal 1 right next to boss, Portal 2 right next to target
    game.portals = Some((
        snake::Point {
            x: 2,
            y: 1,
        },
        snake::Point {
            x: 17,
            y: 18,
        },
    ));

    let boss_kind = game::BossType::Shooter;

    let path_end = game.get_boss_path(start, target, boss_kind);

    assert_eq!(path_end, Some(snake::Direction::Right));
}
