use snake_game::*;

#[test]
fn test_flow_field_uses_portals() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);

    let start = snake::Point { x: 1, y: 1 };
    let target = snake::Point { x: 18, y: 18 };

    // Portal 1 right next to start, Portal 2 right next to target
    game.portals = Some((
        snake::Point { x: 2, y: 1 },
        snake::Point { x: 17, y: 18 },
    ));

    let flow_field = game::generate_flow_field(&game, &[target]);

    // Path should point to portal
    assert_eq!(flow_field.get(&start), Some(&snake::Direction::Right));
}
