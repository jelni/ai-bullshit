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
        move_timer: 1,
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
    assert_ne!(current_pos, initial_pos, "Teleporter boss should have changed position");
}

#[test]
fn test_splitter_boss_splits_on_death() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Up;
    game.state = snake_game::game::GameState::Playing;

    // Splitter Boss with health 1 and max_health 10
    game.bosses.push(Boss {
        position: Point {
            x: 10,
            y: 10,
        },
        health: 1,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Splitter,
        state_timer: 0,
    });

    // Freeze time so the boss doesn't move and laser moves 1 step
    game.power_up = Some(snake_game::game::PowerUp {
        p_type: snake_game::game::PowerUpType::TimeFreeze,
        location: Point {
            x: 1,
            y: 1,
        },
        activation_time: Some(
            web_time::SystemTime::now()
                .duration_since(web_time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ),
    });

    // Laser to hit the boss
    // Time freeze means laser does not move. It just does collision check.
    // So we spawn it exactly on top of the boss!
    game.lasers.push(snake_game::game::Laser {
        position: Point {
            x: 10,
            y: 10,
        },
        direction: Direction::Right,
        player: 1,
    });

    game.update();

    // The boss should split into two children
    assert_eq!(game.bosses.len(), 2, "Splitter boss should split into 2 smaller bosses");
    assert_eq!(game.bosses[0].kind, BossType::Splitter);
    assert_eq!(game.bosses[0].health, 5);
    assert_eq!(game.bosses[0].max_health, 5);
    assert_eq!(game.bosses[1].kind, BossType::Splitter);
    assert_eq!(game.bosses[1].health, 5);
    assert_eq!(game.bosses[1].max_health, 5);
}

#[test]
fn test_necromancer_summons_goblin() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    // Necromancer Boss
    game.bosses.push(Boss {
        position: Point {
            x: 9,
            y: 5,
        },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 44, // Default threshold is 45 in Normal mode, so next update spawns goblin
        kind: BossType::Necromancer,
        state_timer: 0,
    });

    game.goblin = None; // Ensure no goblin exists
    game.state = snake_game::game::GameState::Playing;

    game.update();

    assert!(game.goblin.is_some(), "Necromancer should summon a goblin");
    assert_eq!(
        game.goblin.unwrap().position,
        Point {
            x: 9,
            y: 5
        },
        "Goblin should be summoned at boss position"
    );
}

#[test]
fn test_shadowclone_moves_towards_snake() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    // ShadowClone Boss
    game.bosses.push(Boss {
        position: Point {
            x: 7,
            y: 7,
        },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::ShadowClone,
        state_timer: 0,
    });

    game.state = snake_game::game::GameState::Playing;
    game.obstacles.clear();
    game.update();

    let new_pos = game.bosses[0].position;
    // Expected movement: boss at (7, 7), snake at (5, 5). dx = -1, dy = -1.
    // So new_pos could be (6, 7) or (7, 6) or (6, 6) depending on rng and axis movement.
    // But it should move closer to the snake.
    assert!(new_pos.x < 7 || new_pos.y < 7, "Shadow clone should move towards the snake");
}
