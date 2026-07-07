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
        "Bot should steer away from the boss. Got {:?}",
        next_dir
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
        "Bot should steer away from the laser. Got {:?}",
        next_dir
    );
}
