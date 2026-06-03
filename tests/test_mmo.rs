use snake_game::game::{Difficulty, Game, GameMode, Theme};

#[test]
fn test_massive_multiplayer_mode() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::MassiveMultiplayer;
    game.reset();

    assert!(game.auto_pilot, "MassiveMultiplayer should enable auto_pilot");
    assert!(!game.chat_log.is_empty(), "MassiveMultiplayer should log simulation text");
    assert_eq!(game.chat_log[0].0, "SYSTEM: Simulating 100 bots entering the arena...");
}

#[test]
fn test_mmo_bots_avoid_each_other() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::MassiveMultiplayer;
    game.reset();

    // Clear all existing bots to set up a specific scenario
    game.bots.clear();
    game.bots_autopilot_paths.clear();

    use snake_game::snake::{Point, Snake, Direction};

    // Bot 1 going Right
    let mut bot1 = Snake::new(Point { x: 5, y: 5 });
    bot1.direction = Direction::Right;

    // Bot 2 going Up, directly into the path of Bot 1
    let mut bot2 = Snake::new(Point { x: 6, y: 6 });
    bot2.direction = Direction::Up;

    game.bots.push(bot1);
    game.bots_autopilot_paths.push(Vec::new());

    game.bots.push(bot2);
    game.bots_autopilot_paths.push(Vec::new());

    game.food = Point { x: 10, y: 5 }; // Food is to the right

    // The point (6,5) is right in front of both bot1 (5,5 -> Right) and bot2 (6,6 -> Up).
    // If they both pathfind, is_safe_final_p should return false for (6,5) because they
    // would both try to enter it, or at least one of them will see the other can enter it.

    // We use the same prediction logic from `is_safe_final_p` with checking_player = 4
    let is_safe = game.is_safe_final_p(Point { x: 6, y: 5 }, 1, 4);
    assert!(!is_safe, "Point (6,5) should not be safe since both bots can move there simultaneously");
}
