use snake_game::game::{Difficulty, Game, Theme};
use snake_game::snake::{Direction, Point, Snake};

#[test]
fn test_black_hole_pulls_food() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);

    // Place snake far away
    game.snake = Snake::new(Point { x: 2, y: 2 });
    game.snake.direction = Direction::Up;

    // Clear obstacles to isolate behavior
    game.obstacles.clear();

    game.food = Point { x: 10, y: 10 };
    game.black_hole = Some(Point { x: 10, y: 15 });

    let initial_y = game.food.y;
    let mut pulled = false;
    for _ in 0..100 {
        game.state = snake_game::game::GameState::Playing;
        // prevent black hole from despawning randomly for test
        let prev_bh = game.black_hole;
        game.update();
        if game.black_hole.is_none() {
            game.black_hole = prev_bh;
        }

        if game.food.y > initial_y {
            pulled = true;
            break;
        }
    }

    assert!(pulled, "Food should have been pulled closer to the black hole");
}

#[test]
fn test_black_hole_kills_snake() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);

    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Down;

    game.black_hole = Some(Point { x: 10, y: 11 });

    game.state = snake_game::game::GameState::Playing;

    let lives_before = game.lives;
    game.update();

    assert_eq!(game.lives, lives_before - 1, "Snake should die when moving into black hole");
}
