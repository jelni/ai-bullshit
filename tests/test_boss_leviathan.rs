use snake_game::game::{Boss, BossType, Game, GameState};
use snake_game::snake::Point;

#[test]
fn test_leviathan_boss_leaves_obstacles() {
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
        kind: BossType::Leviathan,
        state_timer: 0,
    });

    game.state = GameState::Playing;

    game.snake = snake_game::snake::Snake::new(Point {
        x: 10,
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
    assert!(
        game.obstacles.contains(&initial_pos),
        "Leviathan boss should leave an obstacle at its previous position"
    );
}
