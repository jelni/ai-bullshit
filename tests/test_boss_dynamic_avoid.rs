use snake_game::*;

#[test]
fn test_boss_avoids_laser() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);

    // Boss at (5, 5)
    let start = snake::Point {
        x: 5,
        y: 5,
    };
    // Target is right of the boss at (10, 5)
    let target = snake::Point {
        x: 10,
        y: 5,
    };

    // Place a laser right in the middle (7, 5)
    game.lasers.push(game::Laser {
        position: snake::Point {
            x: 7,
            y: 5,
        },
        direction: snake::Direction::Up,
        player: 1,
    });

    let boss_kind = game::BossType::Shooter;

    let path_end = game.get_boss_path(start, target, boss_kind);

    // Because of the laser at (7, 5), the boss should pathfind UP or DOWN instead of going straight RIGHT
    // Direct path would be Right
    assert_ne!(
        path_end,
        Some(snake::Direction::Right),
        "Boss should avoid going straight into the laser"
    );
    assert!(
        path_end == Some(snake::Direction::Up) || path_end == Some(snake::Direction::Down),
        "Boss should move up or down to avoid the laser"
    );
}

#[test]
fn test_boss_avoids_mine() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);

    // Boss at (5, 5)
    let start = snake::Point {
        x: 5,
        y: 5,
    };
    // Target is right of the boss at (10, 5)
    let target = snake::Point {
        x: 10,
        y: 5,
    };

    // Place a mine right in the middle (7, 5)
    game.mines.insert(snake::Point {
        x: 7,
        y: 5,
    });

    let boss_kind = game::BossType::Shooter;

    let path_end = game.get_boss_path(start, target, boss_kind);

    // Because of the mine at (7, 5), the boss should pathfind UP or DOWN instead of going straight RIGHT
    // Direct path would be Right
    assert_ne!(
        path_end,
        Some(snake::Direction::Right),
        "Boss should avoid going straight into the mine"
    );
    assert!(
        path_end == Some(snake::Direction::Up) || path_end == Some(snake::Direction::Down),
        "Boss should move up or down to avoid the mine"
    );
}
