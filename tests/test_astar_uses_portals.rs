use snake_game::*;

#[test]
fn test_flow_field_uses_portals() {
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

    // Portal 1 right next to start, Portal 2 right next to target
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

    let flow_field = game::generate_flow_field(&game, &[target]);

    // Path should point to portal

    // Since the flow field traverses backwards, depending on how it pushes portal candidates,
    // it might map it differently depending on exact distances. The test checks if flow field uses portals correctly.
    // If we're at (1, 1), target is (18, 18), and Portal 1 is at (2, 1), the optimal move is Right towards Portal 1.
    assert_eq!(flow_field.get(&start), Some(&snake::Direction::Right));

}
