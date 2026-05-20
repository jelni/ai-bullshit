use snake_game::game::{Boss, BossType, Difficulty, Game, Theme};
use snake_game::snake::{Direction, Point, Snake};

#[test]
fn test_charger_moves_faster() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    // Charger Boss
    game.bosses.push(Boss {
        position: Point {
            x: 9,
            y: 5,
        },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Charger,
        state_timer: 0,
    });

    game.state = snake_game::game::GameState::Playing;

    game.update();
    // Move timer for charger increments by 1. Threshold is (2/2) = 1.
    // So it should move in 1 update call.
    let current_pos = game.bosses[0].position;
    assert_ne!(
        current_pos,
        Point {
            x: 9,
            y: 5
        },
        "Charger boss should have moved"
    );
}

#[test]
fn test_spawner_drops_mines() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    // Spawner Boss
    game.bosses.push(Boss {
        position: Point {
            x: 9,
            y: 5,
        },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 29, // Threshold is 30, so next update it should drop a mine
        kind: BossType::Spawner,
        state_timer: 0,
    });

    game.mines.clear();
    game.state = snake_game::game::GameState::Playing;
    game.update();

    assert_eq!(game.mines.len(), 1, "Spawner boss should drop a mine");
    assert!(
        game.mines.contains(&Point {
            x: 9,
            y: 5
        }),
        "Mine should be dropped at boss position"
    );
}

#[test]
fn test_teleporter_boss_moves() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    let initial_pos = Point {
        x: 9,
        y: 5,
    };
    // Teleporter Boss
    game.bosses.push(Boss {
        position: initial_pos,
        health: 10,
        max_health: 10,
        move_timer: 29, // Threshold is 30, so next update it should teleport
        shoot_timer: 0,
        kind: BossType::Teleporter,
        state_timer: 0,
    });

    game.state = snake_game::game::GameState::Playing;

    game.update();
    // Move timer hits threshold, boss should teleport to a random empty point.
    let current_pos = game.bosses[0].position;
    assert_ne!(
        current_pos, initial_pos,
        "Teleporter boss should have changed position"
    );
}
