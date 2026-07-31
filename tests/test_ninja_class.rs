use snake_game::*;

#[test]
fn test_ninja_class_passes_through_walls() {
    let mut game = game::Game::new(
        20,
        20,
        false, // wrap_mode = false
        'x',
        game::Theme::Classic,
        game::Difficulty::Normal,
    );
    game.stats.equipped_class = Some(game::HeroClass::Ninja);

    game.snake = snake::Snake::new(snake::Point { x: 1, y: 10 });
    game.snake.direction = snake::Direction::Left;
    game.snake.direction_queue.push_back(snake::Direction::Left);
    game.state = game::GameState::Playing;

    // Use update() to process a single tick of the game state
    // In our wrap calculation: x=0 wraps to x=18 since width is 20.
    game.update();

    let head = game.snake.head();
    assert_eq!(head.x, 18);
    assert_eq!(head.y, 10);
    assert!(!game.just_died && game.lives >= 3, "Ninja should not die on wall collision");
}
