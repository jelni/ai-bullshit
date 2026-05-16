use snake_game::*;

#[test]
fn test_boss_rage_phase() {
    let mut game = game::Game::new(
        20,
        20,
        false,
        'x',
        game::Theme::Classic,
        game::Difficulty::Normal,
    );
    game.snake = snake::Snake::new(snake::Point { x: 5, y: 5 });

    let boss_pos = snake::Point { x: 5, y: 15 };
    game.boss = Some(game::Boss {
        position: boss_pos,
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
    });

    // At full health (10/10), thresholds are normal
    game.state = game::GameState::Playing;

    // Simulate updating game to check movement
    // move_threshold is usually 2. So in 1 update, move_timer becomes 1, no move.
    game.update();

    let boss = game.boss.as_ref().unwrap();
    assert_eq!(boss.move_timer, 1, "Boss should not have moved yet");
    assert_eq!(boss.position, boss_pos, "Boss position should be unchanged");

    // After 2nd update, move_timer reaches 2 (threshold) and resets to 0, Boss moves
    game.update();
    let boss = game.boss.as_ref().unwrap();
    assert_eq!(boss.move_timer, 0, "Boss move_timer should reset");
    assert_ne!(boss.position, boss_pos, "Boss should have moved towards player");

    let new_pos = boss.position;

    // Now trigger rage phase: health <= 5 (50%)
    let mut boss_mut = game.boss.take().unwrap();
    boss_mut.health = 5;
    boss_mut.move_timer = 0;
    game.boss = Some(boss_mut);

    // In rage phase, move_threshold halves from 2 to 1.
    // This means every update should cause the boss to move!
    game.update();
    let boss = game.boss.as_ref().unwrap();
    assert_eq!(boss.move_timer, 0, "Boss move_timer should reset immediately in rage phase");
    assert_ne!(boss.position, new_pos, "Boss should have moved immediately in rage phase");
}

#[test]
fn test_boss_death_nova() {
    let mut game = game::Game::new(
        20,
        20,
        false,
        'x',
        game::Theme::Classic,
        game::Difficulty::Normal,
    );

    let boss_pos = snake::Point { x: 10, y: 10 };
    game.boss = Some(game::Boss {
        position: boss_pos,
        health: 1, // 1 HP left
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
    });

    // Spawn a laser that will hit the boss next tick
    game.lasers.push(game::Laser {
        position: snake::Point { x: 10, y: 12 }, // Dist 2, will move 2 and hit 10,10
        direction: snake::Direction::Up,
        player: 1, // Player's laser
    });

    game.state = game::GameState::Playing;

    let initial_laser_count = game.lasers.len();

    // Trigger update -> laser moves to 10,10 -> hits boss -> boss health 0 -> Nova
    game.update();

    // Boss should be dead
    assert!(game.boss.is_none(), "Boss should be defeated");

    // The laser that hit the boss is destroyed.
    // Death nova spawns 4 new lasers.
    assert_eq!(game.lasers.len(), initial_laser_count + 3, "Death nova should spawn 4 lasers");

    // Verify the lasers are positioned around the boss
    let mut dirs = vec![];
    for laser in &game.lasers {
        assert_eq!(laser.player, 3, "Nova lasers should belong to the boss (player 3)");
        dirs.push(laser.direction);
    }

    assert!(dirs.contains(&snake::Direction::Up));
    assert!(dirs.contains(&snake::Direction::Down));
    assert!(dirs.contains(&snake::Direction::Left));
    assert!(dirs.contains(&snake::Direction::Right));
}
