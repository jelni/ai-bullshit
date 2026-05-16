
use snake_game::game::{Game};
use snake_game::snake::{Point, Direction, Snake};
use web_time::Instant;

#[test]
fn test_poison_food_collision() {
    let mut game = Game::new(20, 20, false, 'x', snake_game::game::Theme::Classic, snake_game::game::Difficulty::Normal);

    // Set up the snake
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;

    // Initial length is 3 according to snake.rs. Let's add some more so shrink_tail can reduce it.
    // target_len = max(3, len / 2). If len is 3, target_len is 3.
    // Let's add 3 more segments to make len 6. target_len will be 3.
    game.snake.move_to(Point { x: 11, y: 10 }, true);
    game.snake.move_to(Point { x: 12, y: 10 }, true);
    game.snake.move_to(Point { x: 13, y: 10 }, true);

    assert_eq!(game.snake.body.len(), 6);

    // Initial score
    game.score = 50;

    // Place poison food right in front of the snake
    game.poison_food = Some((Point { x: 14, y: 10 }, Instant::now()));

    // clear obstacles so it doesn't hit one randomly
    game.obstacles.clear();

    // Move the snake into the poison food
    game.state = snake_game::game::GameState::Playing;
    game.handle_input(Direction::Right, 1);
    game.update();

    // Snake should have shrunk from 6 to 3
    assert_eq!(game.snake.body.len(), 3);

    // Score should have decreased by 10
    assert_eq!(game.score, 40);

    // Poison food should have despawned
    assert!(game.poison_food.is_none());
}

#[test]
fn test_bot_avoids_poison_food() {
    let mut game = Game::new(20, 20, false, 'x', snake_game::game::Theme::Classic, snake_game::game::Difficulty::Normal);

    // Set up snake
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;

    // Set normal food far away
    game.food = Point { x: 15, y: 5 };

    // Put poison food exactly where the bot would normally go (Right)
    game.poison_food = Some((Point { x: 6, y: 5 }, Instant::now()));

    // Request a move
    let next_move = game.calculate_autopilot_move();

    // It should avoid moving right
    assert!(next_move == Some(Direction::Up) || next_move == Some(Direction::Down));
}
