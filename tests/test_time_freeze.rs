use snake_game::*;

#[test]
fn test_time_freeze_pauses_boss_and_lasers() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);

    // Give snake the TimeFreeze powerup
    game.power_up = Some(game::PowerUp {
        p_type: game::PowerUpType::TimeFreeze,
        location: snake::Point {
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

    let boss_initial_pos = snake::Point {
        x: 10,
        y: 10,
    };
    game.boss = Some(game::Boss {
        position: boss_initial_pos,
        health: 10,
        max_health: 10,
        move_timer: 100,  // Should move immediately if time wasn't frozen
        shoot_timer: 100, // Should shoot immediately if time wasn't frozen
    });

    let laser_initial_pos = snake::Point {
        x: 5,
        y: 5,
    };
    game.lasers.push(game::Laser {
        position: laser_initial_pos,
        direction: snake::Direction::Right,
        player: 3, // Boss laser
    });

    // Put snake far away
    game.snake = snake::Snake::new(snake::Point {
        x: 18,
        y: 18,
    });
    game.state = game::GameState::Playing;

    // Simulate several updates where Boss and Lasers would normally move/shoot
    for _ in 0..5 {
        game.update();
    }

    // Assert boss hasn't moved
    let boss = game.boss.as_ref().unwrap();
    assert_eq!(boss.position, boss_initial_pos, "Boss should not move while time is frozen");

    // Assert laser hasn't moved
    assert_eq!(game.lasers.len(), 1, "No new lasers should have spawned");
    assert_eq!(
        game.lasers[0].position, laser_initial_pos,
        "Laser should not move while time is frozen"
    );
}

#[test]
fn test_bot_avoids_frozen_boss() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);

    game.power_up = Some(game::PowerUp {
        p_type: game::PowerUpType::TimeFreeze,
        location: snake::Point {
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

    game.snake = snake::Snake::new(snake::Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = snake::Direction::Right;
    game.food = snake::Point {
        x: 9,
        y: 5,
    };

    // Placing boss right in front of the snake
    game.boss = Some(game::Boss {
        position: snake::Point {
            x: 6,
            y: 5,
        },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
    });

    // Since the direct path (Right) is blocked by the frozen boss, it should choose Up or Down.
    let next_move = game.calculate_autopilot_move();
    assert!(
        next_move == Some(snake::Direction::Up) || next_move == Some(snake::Direction::Down),
        "Bot should pathfind around the frozen boss"
    );
}

#[test]
fn test_bot_avoids_frozen_laser() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);

    game.power_up = Some(game::PowerUp {
        p_type: game::PowerUpType::TimeFreeze,
        location: snake::Point {
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

    game.snake = snake::Snake::new(snake::Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = snake::Direction::Right;
    game.food = snake::Point {
        x: 9,
        y: 5,
    };

    // Placing laser right in front of the snake
    game.lasers.push(game::Laser {
        position: snake::Point {
            x: 6,
            y: 5,
        },
        direction: snake::Direction::Left, // Moving towards player (but frozen)
        player: 3,
    });

    // Since the direct path (Right) is blocked by the frozen laser, it should choose Up or Down.
    let next_move = game.calculate_autopilot_move();
    assert!(
        next_move == Some(snake::Direction::Up) || next_move == Some(snake::Direction::Down),
        "Bot should pathfind around the frozen laser"
    );
}
