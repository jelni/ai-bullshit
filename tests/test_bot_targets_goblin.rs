use snake_game::game::{Difficulty, Game, Goblin, Theme};
use snake_game::snake::Point;

#[test]
fn test_bot_targets_goblin() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();
    game.snake = snake_game::snake::Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = snake_game::snake::Direction::Right;

    // Place goblin right in front of the snake
    game.goblin = Some(Goblin {
        position: Point {
            x: 6,
            y: 5,
        },
        move_timer: 0,
        food_eaten: 0,
    });

    // Move food away
    game.food = Point {
        x: 18,
        y: 18,
    };

    let next_move = game.calculate_autopilot_move();

    // The bot should target the goblin, so it should move Right towards it
    assert_eq!(
        next_move,
        Some(snake_game::snake::Direction::Right)
    );
}
