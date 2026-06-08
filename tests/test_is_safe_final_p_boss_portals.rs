use snake_game::game::{Boss, BossType, Difficulty, Game, Theme};
use snake_game::snake::{Direction, Point, Snake};

#[test]
fn test_is_safe_final_p_boss_portals() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);

    // Set up a boss at (1, 1) and portal at (2, 1) and (18, 18)
    // The player evaluates moving to (17, 18)
    // Distance from (17, 18) to boss (1, 1) is 16 + 17 = 33
    // But distance via portal is:
    // (17, 18) -> (18, 18) = 1
    // (18, 18) teleport to (2, 1)
    // (2, 1) to (1, 1) = 1
    // Total distance = 2.
    // If the steps we are looking ahead is say 5, the boss can definitely reach (17, 18) in 5 moves (2 <= moves)

    game.snake = Snake::new(Point { x: 17, y: 17 });
    game.snake.direction = Direction::Up; // So we evaluate (17, 18)

    game.bosses.push(Boss {
        position: Point { x: 1, y: 1 },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Charger, // moves fast
        state_timer: 0,
    });

    game.portals = Some((Point { x: 2, y: 1 }, Point { x: 18, y: 18 }));

    // Evaluate if (17, 18) is safe in 5 steps.
    // Boss moves 1 tile per 1 tick for charger (move_threshold=1).
    // In 5 steps, moves = 5.
    // distance via portal = 2.
    // 2 <= 5 is true. Thus it's not safe.

    let is_safe = game.is_safe_final_p(Point { x: 17, y: 18 }, 5, 1);
    assert!(!is_safe, "Final point should not be safe due to portal proximity to boss");
}
