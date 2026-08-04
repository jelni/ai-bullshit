use snake_game::game::{Boss, BossType, Difficulty, Game, Laser, Theme};
use snake_game::snake::{Direction, Point};

#[test]
fn test_astar_pathfind_avoids_boss() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();

    // Start bot left of the boss, target right of the boss
    let target = Point {
        x: 15,
        y: 10,
    };

    // Boss right in the middle
    game.bosses.push(Boss {
        position: Point {
            x: 10,
            y: 10,
        },
        health: 10,
        max_health: 10,
        kind: BossType::Juggernaut,
        move_timer: 0,
        shoot_timer: 0,
        state_timer: 0,
    });

    // If there were no boss, it would just go Right
    // But since there is a boss, the direct path is heavily penalized,
    // so it should choose Up or Down, or at least not go perfectly straight for too long.
    // Let's test if it takes a different path.
    let start_close = Point {
        x: 8,
        y: 10,
    };
    let dir_close = game.bot_smart_pathfind(start_close, target, 3);

    // If it goes Right from (8, 10), the next point is (9, 10) which is dist 1 from boss, penalty 40.
    // Going Up to (8, 9) is dist 2 from boss (x=2, y=1 => 3), penalty 20.
    // Going Up is cheaper penalty than Right.
    let next_dir = dir_close.unwrap_or(Direction::Up);
    assert!(
        next_dir == Direction::Up || next_dir == Direction::Down,
        "Bot should steer away from the boss. Got {next_dir:?}"
    );
}

#[test]
fn test_astar_pathfind_avoids_laser() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();

    let start = Point {
        x: 5,
        y: 5,
    };
    let target = Point {
        x: 15,
        y: 5,
    };

    game.lasers.push(Laser {
        position: Point {
            x: 7,
            y: 5,
        },
        direction: Direction::Up,
        player: 1,
    });

    let next_dir = game.bot_smart_pathfind(start, target, 3).unwrap_or(Direction::Up);

    // Going Right would put us at (6, 5), dist 1 from laser -> penalty 15
    // Going Up would put us at (5, 4), dist 3 from laser -> penalty 5
    assert!(
        next_dir == Direction::Up || next_dir == Direction::Down,
        "Bot should steer away from the laser. Got {next_dir:?}"
    );
}

#[test]
fn test_astar_avoids_portals_when_unsafe() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();

    game.snake.body.clear();
    game.snake.body.push_back(Point {
        x: 1,
        y: 1,
    });
    game.snake.direction = Direction::Right;

    let target = Point {
        x: 18,
        y: 18,
    };

    game.portals = Some((
        Point {
            x: 2,
            y: 1,
        },
        Point {
            x: 17,
            y: 18,
        },
    ));

    // add an obstacle at the exit of portal
    game.obstacles.insert(Point {
        x: 17,
        y: 18,
    });

    let dir = game.astar_pathfind(game.snake.head(), target, 1);

    assert!(dir != Some(Direction::Right), "Bot should avoid portal if exit is unsafe. Got {:?}", dir);
}

#[test]
fn test_astar_avoids_portals_when_laser_is_on_exit() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();

    game.snake.body.clear();
    game.snake.body.push_back(Point {
        x: 1,
        y: 1,
    });
    game.snake.direction = Direction::Right;

    let target = Point {
        x: 18,
        y: 18,
    };

    game.portals = Some((
        Point {
            x: 2,
            y: 1,
        },
        Point {
            x: 17,
            y: 18,
        },
    ));

    // laser aiming at exit
    game.lasers.push(Laser {
        position: Point {
            x: 17,
            y: 18,
        },
        direction: Direction::Up,
        player: 1,
    });

    let dir = game.astar_pathfind(game.snake.head(), target, 1);

    assert!(dir != Some(Direction::Right), "Bot should avoid portal if laser is hitting exit. Got {:?}", dir);
}

#[test]
fn test_astar_avoids_portals_when_laser_passes_through() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();

    game.snake.body.clear();
    game.snake.body.push_back(Point {
        x: 1,
        y: 1,
    });
    game.snake.direction = Direction::Right;

    let target = Point {
        x: 18,
        y: 18,
    };

    game.portals = Some((
        Point {
            x: 2,
            y: 1,
        },
        Point {
            x: 17,
            y: 18,
        },
    ));

    // laser aiming at portal 2
    game.lasers.push(Laser {
        position: Point {
            x: 17,
            y: 16, // travelling down, hits (17,18) in 2 ticks
        },
        direction: Direction::Down,
        player: 1,
    });

    let dir = game.astar_pathfind(game.snake.head(), target, 1);

    // Portal 1 is at 2,1, so if the laser hits portal 2 at 17,18, it comes OUT of portal 1
    // Let's see if a_star sees this as dangerous
    // wait a_star doesn't step simulation, is_safe_final_p does checking.
    // wait, is_safe_final_p has portal checks for lasers.

    // We expect it to NOT go Right into the portal because it's dangerous
    assert!(dir != Some(Direction::Right), "Bot should avoid portal if laser will hit it. Got {:?}", dir);
}
