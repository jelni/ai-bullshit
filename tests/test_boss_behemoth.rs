use snake_game::game::{Boss, BossType, Game, GameState};
use snake_game::snake::Point;

#[test]
fn test_boss_behemoth() {
    let mut game = Game::new(
        20,
        20,
        false,
        'x',
        snake_game::game::Theme::Classic,
        snake_game::game::Difficulty::Normal,
    );
    game.obstacles.clear();

    let initial_pos = Point {
        x: 10,
        y: 10,
    };
    game.bosses.push(Boss {
        position: initial_pos,
        health: 10,
        max_health: 10,
        move_timer: 1,
        shoot_timer: 0,
        kind: BossType::Behemoth,
        state_timer: 0,
    });

    // Place obstacles all around the boss so it has to step on one
    game.obstacles.insert(Point {
        x: 9,
        y: 10,
    });
    game.obstacles.insert(Point {
        x: 10,
        y: 9,
    });
    game.obstacles.insert(Point {
        x: 11,
        y: 10,
    });
    game.obstacles.insert(Point {
        x: 10,
        y: 11,
    });

    game.state = GameState::Playing;

    game.snake = snake_game::snake::Snake::new(Point {
        x: 5,
        y: 5,
    });

    let mut moved = false;
    for _ in 0..10 {
        game.update();
        if let Some(boss) = game.bosses.first() {
            if boss.position != initial_pos {
                moved = true;
                break;
            }
        }
    }

    assert!(moved, "Boss should have moved");
    let boss_pos = game.bosses[0].position;
    assert!(
        !game.obstacles.contains(&boss_pos),
        "Behemoth boss should destroy the obstacle at its new position {:?}",
        boss_pos
    );
}
