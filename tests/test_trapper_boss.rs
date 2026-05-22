use snake_game::game::{Game, GameState, Boss, BossType};
use snake_game::snake::Point;

#[test]
fn test_trapper_boss_leaves_obstacles() {
    let mut game = Game::new(20, 20, false, 'x', snake_game::game::Theme::Classic, snake_game::game::Difficulty::Normal);
    game.obstacles.clear();

    let initial_pos = Point { x: 10, y: 10 };
    game.bosses.push(Boss {
        position: initial_pos,
        health: 10,
        max_health: 10,
        move_timer: 1, // Ready to move immediately based on move_threshold being 2 normally
        shoot_timer: 0,
        kind: BossType::Trapper,
        state_timer: 0,
    });

    game.state = GameState::Playing;

    // Trapper boss moves towards snake head (at 10, 10 or similar default center).
    // Let's place snake at 10, 5 to pull the boss up.
    game.snake = snake_game::snake::Snake::new(Point { x: 10, y: 5 });

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
    assert!(game.obstacles.contains(&initial_pos), "Trapper boss should leave an obstacle at its previous position");
}
