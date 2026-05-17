use snake_game::game::Game;
use snake_game::snake::Point;

#[test]
fn test_mine_spawning() {
    let mut game = Game::new(
        20,
        20,
        false,
        'x',
        snake_game::game::Theme::Classic,
        snake_game::game::Difficulty::Normal,
    );

    game.mines.clear();

    // Force rng or loop until a mine spawns
    let mut spawned = false;
    for _ in 0..10000 {
        let old_lives = game.lives;
        game.state = snake_game::game::GameState::Playing;
        game.update();
        game.lives = old_lives;
        if !game.mines.is_empty() {
            spawned = true;
            break;
        }
    }

    assert!(spawned, "A mine should have spawned");
}

#[test]
fn test_mine_explosion() {
    let mut game = Game::new(
        20,
        20,
        false,
        'x',
        snake_game::game::Theme::Classic,
        snake_game::game::Difficulty::Normal,
    );

    game.obstacles.clear();
    game.mines.clear();

    let mine_pos = Point { x: 10, y: 10 };
    game.mines.insert(mine_pos);

    // Add obstacles in 1-tile radius
    let obs1 = Point { x: 9, y: 9 };
    let obs2 = Point { x: 10, y: 11 };
    let obs3 = Point { x: 11, y: 10 };

    // Add an obstacle far away that shouldn't be destroyed
    let safe_obs = Point { x: 2, y: 2 };

    game.obstacles.insert(obs1);
    game.obstacles.insert(obs2);
    game.obstacles.insert(obs3);
    game.obstacles.insert(safe_obs);

    // Trigger explosion by moving snake into it
    game.snake = snake_game::snake::Snake::new(Point { x: 10, y: 9 });
    game.snake.direction = snake_game::snake::Direction::Down;
    game.state = snake_game::game::GameState::Playing;

    game.update(); // Move down to (10, 10), triggering explosion

    assert!(game.just_died, "Snake should die after hitting the mine");
    assert!(!game.mines.contains(&mine_pos), "Mine should be removed");

    // Near obstacles should be removed
    assert!(!game.obstacles.contains(&obs1));
    assert!(!game.obstacles.contains(&obs2));
    assert!(!game.obstacles.contains(&obs3));

    // Safe obstacle should still be there
    assert!(game.obstacles.contains(&safe_obs));
}
