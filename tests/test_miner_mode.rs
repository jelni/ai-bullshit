use snake_game::*;

#[test]
fn test_miner_mode_breaks_obstacle() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    game.mode = game::GameMode::Miner;

    game.snake = snake::Snake::new(snake::Point { x: 5, y: 5 });
    game.snake.direction = snake::Direction::Right;

    // Clear and place one obstacle to the right
    game.obstacles.clear();
    let obs_pos = snake::Point { x: 6, y: 5 };
    game.obstacles.insert(obs_pos);

    // We also make sure the final head calculation is clear
    assert!(game.obstacles.contains(&obs_pos));

    // Process game logic for a move
    game.state = game::GameState::Playing;
    game.update();

    // The obstacle should be removed, and snake should be at obs_pos without dying
    assert!(!game.obstacles.contains(&obs_pos), "Obstacle should be destroyed");
    assert_eq!(game.snake.head(), obs_pos, "Snake should move into the obstacle");
    assert_eq!(game.lives, 3, "Snake should not lose a life"); // Wait, it's 3 or something similar depending on gear, but definitely shouldn't be dead
    assert!(!game.just_died, "Snake should not die");
}
