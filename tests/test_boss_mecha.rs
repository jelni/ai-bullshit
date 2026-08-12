use snake_game::game::{Boss, BossType, Difficulty, Game, GameState, Theme};
use snake_game::snake::Point;

#[test]
fn test_boss_mecha_moves_and_shoots() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();
    game.state = GameState::Playing;

    let bot_pos = Point {
        x: 10,
        y: 15,
    };
    game.snake = snake_game::snake::Snake::new(bot_pos);

    let mecha_pos = Point {
        x: 10,
        y: 5,
    };
    game.bosses.push(Boss {
        position: mecha_pos,
        health: 10,
        max_health: 10,
        move_timer: 10,   // Force move
        shoot_timer: 100, // Force shoot
        kind: BossType::Mecha,
        state_timer: 0,
    });

    // Run one update tick
    game.update();

    if let Some(boss) = game.bosses.first() {
        assert_ne!(boss.position, mecha_pos, "Mecha boss should have moved towards the player");
        assert!(boss.position.y > 5, "Mecha boss should have moved downwards");
    } else {
        panic!("Boss not found");
    }

    // Since shoot_timer was 100, it should have spawned lasers in 4 directions
    assert_eq!(game.lasers.len(), 4, "Mecha boss should have spawned 4 lasers");
}

#[test]
fn test_bot_predicts_mecha_laser() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();
    game.state = GameState::Playing;

    let bot_pos = Point {
        x: 5,
        y: 5,
    };
    game.snake = snake_game::snake::Snake::new(bot_pos);

    // Place Mecha so that it's about to shoot and we are on its X or Y axis
    let mecha_pos = Point {
        x: 10,
        y: 5,
    };
    game.bosses.push(Boss {
        position: mecha_pos,
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 14, // Almost about to shoot (threshold 15)
        kind: BossType::Mecha,
        state_timer: 0,
    });

    // Check if moving right (towards the laser path) is safe
    let right_safe = game.is_safe_final_p(
        Point {
            x: 6,
            y: 5,
        },
        1,
        1,
    );

    // Check if moving up (away from the laser path) is safe
    let up_safe = game.is_safe_final_p(
        Point {
            x: 5,
            y: 4,
        },
        1,
        1,
    );

    assert!(!right_safe, "Moving into the Mecha's laser path should be unsafe.");
    assert!(up_safe, "Moving away from the Mecha's laser path should be safe.");
}
