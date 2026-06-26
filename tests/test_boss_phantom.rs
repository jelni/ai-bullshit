use snake_game::game::{Boss, BossType, Difficulty, Game, GameState, Theme};
use snake_game::snake::Point;

#[test]
fn test_boss_phantom_moves_through_obstacles() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();

    let initial_pos = Point {
        x: 10,
        y: 10,
    };
    let target_pos = Point {
        x: 10,
        y: 5,
    }; // Snake is here

    // Place an obstacle right above the Phantom boss
    game.obstacles.insert(Point {
        x: 10,
        y: 9,
    });
    game.obstacles.insert(Point {
        x: 9,
        y: 9,
    });
    game.obstacles.insert(Point {
        x: 11,
        y: 9,
    });

    game.bosses.push(Boss {
        position: initial_pos,
        health: 10,
        max_health: 10,
        move_timer: 10, // Force move
        shoot_timer: 0,
        kind: BossType::Phantom,
        state_timer: 0,
    });

    game.state = GameState::Playing;

    game.snake = snake_game::snake::Snake::new(target_pos);

    // Update once, move_timer reaches threshold, Phantom should move through obstacle
    game.update();

    if let Some(boss) = game.bosses.first() {
        assert_eq!(
            boss.position,
            Point {
                x: 10,
                y: 9
            },
            "Phantom boss should have moved into the obstacle"
        );
    } else {
        panic!("Boss not found");
    }

    // Update again, Phantom should continue moving towards target
    for boss in &mut game.bosses {
        boss.move_timer = 10;
    }
    game.update();

    if let Some(boss) = game.bosses.first() {
        assert_eq!(
            boss.position,
            Point {
                x: 10,
                y: 8
            },
            "Phantom boss should move past the obstacle"
        );
    } else {
        panic!("Boss not found");
    }
}
