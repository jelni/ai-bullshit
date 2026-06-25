use snake_game::*;

fn main() {
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

    let boss_kind = game::BossType::Shooter;

    let path_end = game.get_boss_path(start, target, boss_kind);

    println!("{:?}", path_end);
}
