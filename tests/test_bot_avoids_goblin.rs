use snake_game::game::{Game, GameMode, Difficulty, Theme, Goblin};
use snake_game::snake::Point;

#[test]
fn test_bot_avoids_goblin_when_directed() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();
    game.snake = snake_game::snake::Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = snake_game::snake::Direction::Right;

    // Place goblin right in front of the snake
    game.goblin = Some(Goblin {
        position: Point { x: 6, y: 5 },
        move_timer: 0,
        food_eaten: 0,
    });

    // Move food away
    game.food = Point { x: 18, y: 18 };

    // Autopilot should not move right into the goblin
    let next_move = game.calculate_autopilot_move();

    // Since it's at (5,5) and goblin at (6,5), it should NOT move Right
    assert!(next_move == Some(snake_game::snake::Direction::Up) || next_move == Some(snake_game::snake::Direction::Down));
}
