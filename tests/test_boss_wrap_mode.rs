use snake_game::*;

#[test]
fn test_boss_wrap_mode() {
    let mut game =
        game::Game::new(20, 20, true, 'x', game::Theme::Classic, game::Difficulty::Normal);

    let start = snake::Point {
        x: 1,
        y: 10,
    };
    let target = snake::Point {
        x: 18,
        y: 10,
    };

    let boss_kind = game::BossType::Shooter;

    let dir = game.get_boss_path(start, target, boss_kind);
    assert_eq!(dir, Some(snake::Direction::Left));
}
