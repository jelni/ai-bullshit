use snake_game::game::{Boss, BossType, Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Point, Direction};

#[test]
fn test_timeweaver_boss_rewinds_time() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::BossRush;
    game.state = GameState::Playing;

    game.snake = snake_game::snake::Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Up;

    // Set some history
    game.score = 500;

    // We add bosses before update so they are part of the state
    game.bosses.push(Boss {
        position: Point { x: 5, y: 5 },
        health: 100,
        max_health: 100,
        move_timer: 0,
        shoot_timer: 49,
        kind: BossType::TimeWeaver,
        state_timer: 0,
    });

    game.update();

    // During update:
    // 1. History captures score=500.
    // 2. Snake moves.
    // 3. TimeWeaver's shoot_timer hits 50, triggers rewind.
    // 4. Rewind restores state from step 1 (score=500).
    // Let's assert that the score remains 500.

    assert_eq!(game.score, 500, "Score should have rewound to 500");
}
