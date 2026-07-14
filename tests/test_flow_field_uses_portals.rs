use snake_game::*;

#[test]
fn test_flow_field_uses_portals_debug() {
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
    println!("Flow field value for start: {:?}", flow_field.get(&start));
}
