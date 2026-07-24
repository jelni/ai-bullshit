use snake_game::game::{Boss, BossType, Difficulty, Game, GameMode, GameState, SpellType, Theme};
use snake_game::snake::Point;

#[test]
fn test_chain_lightning_spell() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::BossRush;
    game.state = GameState::Playing;

    game.snake = snake_game::snake::Snake::new(Point { x: 10, y: 10 });

    // Boss 1: closest to snake
    let boss1 = Boss {
        position: Point { x: 12, y: 10 },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Shooter,
        state_timer: 0,
    };

    // Boss 2: further from snake, closest to boss 1
    let boss2 = Boss {
        position: Point { x: 14, y: 10 },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Shooter,
        state_timer: 0,
    };

    // Boss 3: out of range from snake, but close to boss 2
    let boss3 = Boss {
        position: Point { x: 16, y: 10 },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Shooter,
        state_timer: 0,
    };

    // Boss 4: in range of boss 3, should be hit (3rd bounce)
    let boss4 = Boss {
        position: Point { x: 18, y: 10 },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Shooter,
        state_timer: 0,
    };

    // Boss 5: in range of boss 4, but should not be hit (max 3 bounces)
    let boss5 = Boss {
        position: Point { x: 19, y: 10 },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Shooter,
        state_timer: 0,
    };

    game.bosses.push(boss1);
    game.bosses.push(boss2);
    game.bosses.push(boss3);
    game.bosses.push(boss4);
    game.bosses.push(boss5);

    // Cast spell
    game.cast_spell(SpellType::ChainLightning);

    // Bounces:
    // 1. From snake (10,10) -> Boss 1 (12,10)
    // 2. From Boss 1 -> Boss 2 (14,10)
    // 3. From Boss 2 -> Boss 3 (16,10)

    assert_eq!(game.bosses[0].health, 5, "Boss 1 should have taken 5 damage");
    assert_eq!(game.bosses[1].health, 5, "Boss 2 should have taken 5 damage");
    assert_eq!(game.bosses[2].health, 5, "Boss 3 should have taken 5 damage");
    assert_eq!(game.bosses[3].health, 10, "Boss 4 should not have been hit (max 3 chain)");
    assert_eq!(game.bosses[4].health, 10, "Boss 5 should not have been hit (max 3 chain)");

    assert_eq!(SpellType::ChainLightning.cost(), 45);
}

#[test]
fn test_chain_lightning_kills() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = snake_game::snake::Snake::new(Point { x: 10, y: 10 });

    let boss1 = Boss {
        position: Point { x: 12, y: 10 },
        health: 5,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Shooter,
        state_timer: 0,
    };

    game.bosses.push(boss1);

    game.score = 0;
    let initial_boss_count = game.bosses.len();

    // Cast spell
    game.cast_spell(SpellType::ChainLightning);

    assert_eq!(game.bosses.len(), initial_boss_count - 1, "Boss should be removed after health hits 0");
    assert_eq!(game.score, 100, "Should gain 100 score for killing a boss");
}
