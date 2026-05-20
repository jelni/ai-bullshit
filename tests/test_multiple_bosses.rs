use snake_game::game::{Boss, BossType, Difficulty, Game, GameMode, Theme};
use snake_game::snake::Point;

#[test]
fn test_multiple_bosses_can_exist_and_move() {
    let mut game = Game::new(20, 20, false, '█', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::BossRush;
    game.obstacles.clear();
    game.bosses.clear();

    // Add Boss 1 (Shooter)
    game.bosses.push(Boss {
        position: Point { x: 5, y: 5 },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Shooter,
        state_timer: 0,
    });

    // Add Boss 2 (Charger)
    game.bosses.push(Boss {
        position: Point { x: 15, y: 15 },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Charger,
        state_timer: 0,
    });

    // Snake far away
    game.snake = snake_game::snake::Snake::new(Point { x: 2, y: 2 });
    game.state = snake_game::game::GameState::Playing;

    let initial_b1_pos = game.bosses[0].position;
    let initial_b2_pos = game.bosses[1].position;

    // Simulate several ticks to allow both bosses to move and act
    for _ in 0..10 {
        game.update();
    }

    // Both bosses should still be alive
    assert_eq!(game.bosses.len(), 2, "Both bosses should remain alive");

    // They should have moved or acted based on their respective logic
    // A charger moves quickly towards the player
    let new_b1_pos = game.bosses[0].position;
    let new_b2_pos = game.bosses[1].position;

    // Check that at least one of them moved (Charger definitely should have)
    assert!(
        new_b1_pos != initial_b1_pos || new_b2_pos != initial_b2_pos,
        "Bosses should be capable of movement independently"
    );
}
