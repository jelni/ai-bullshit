use snake_game::*;

fn main() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
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

    // According to tests/test_astar_uses_portals.rs, flow_field uses targets.
    let start = snake::Point { x: 1, y: 1 };
    let target = snake::Point { x: 18, y: 18 };
    let flow_field = game::generate_flow_field(&game, &[target]);
    println!("flow field for start: {:?}", flow_field.get(&start));
}
