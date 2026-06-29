use snake_game::*;

#[test]
fn test_zombie_mode_spawns_zombie_on_eat() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.mode = game::GameMode::Zombie;

    // Place snake and food
    game.snake = snake::Snake::new(snake::Point {
        x: 5,
        y: 5,
    });
    game.food = snake::Point {
        x: 6,
        y: 5,
    };
    game.snake.direction = snake::Direction::Right;

    // Clear initial bots if any (must be done after generating game, which might add them)
    game.bots.clear();
    game.bots_autopilot_paths.clear();

    assert_eq!(game.bots.len(), 0);

    game.update();

    // Ensure the snake is moving
    game.state = game::GameState::Playing;

    // Update might require multiple ticks due to speed differences, let's call it a few times to ensure snake moves and eats the food
    for _ in 0..10 {
        game.update();
        if game.bots.len() == 1 {
            break;
        }
    }

    // Snake ate food, a zombie should have spawned
    assert_eq!(game.bots.len(), 1);
}

#[test]
fn test_zombie_targets_player() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.mode = game::GameMode::Zombie;

    // Clear initial bots if any
    game.bots.clear();
    game.bots_autopilot_paths.clear();

    // Place snake
    game.snake = snake::Snake::new(snake::Point {
        x: 2,
        y: 2,
    });
    game.snake.direction = snake::Direction::Right; // moving away or standing still

    // Place zombie bot
    let bot_pos = snake::Point {
        x: 10,
        y: 10,
    };
    game.bots.push(snake::Snake::new(bot_pos));
    game.bots_autopilot_paths.push(Vec::new());

    // Ensure the snake is moving
    game.state = game::GameState::Playing;

    // Run game update
    game.update();

    // bots update every 3 ticks or so, let's run up to 10 ticks
    let mut moved = false;
    for _ in 0..15 {
        game.update();
        let new_head = game.bots[0].head();
        if new_head != bot_pos {
            moved = true;
            assert!(new_head.x < 10 || new_head.y < 10);
            break;
        }
    }
    assert!(moved);
}
