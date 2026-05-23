use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point};

#[test]
fn test_gravity_mode() {
    let mut game = Game::new(20, 20, false, '#', Theme::Classic, Difficulty::Normal);

    game.mode = GameMode::Gravity;
    game.reset();

    game.snake.body.clear();
    game.snake.body.push_back(Point {
        x: 10,
        y: 10,
    });
    game.snake.body_map.clear();
    game.snake.body_map.insert(
        Point {
            x: 10,
            y: 10,
        },
        1,
    );
    game.snake.direction = Direction::Right;

    game.state = GameState::Playing;
    game.auto_pilot = false;
    game.obstacles.clear();
    game.food = Point {
        x: 1,
        y: 1,
    };
    game.bonus_food = None;

    let mut fell_down = false;
    for _ in 0..100 {
        let old_y = game.snake.head().y;

        game.update();

        let new_y = game.snake.head().y;

        if game.state == GameState::GameOver {
            break;
        }

        if new_y > old_y {
            fell_down = true;
            break;
        }

        // Avoid hitting right wall
        if game.snake.head().x >= 18 {
            game.snake.body.clear();
            game.snake.body.push_back(Point {
                x: 5,
                y: new_y,
            });
            game.snake.body_map.clear();
            game.snake.body_map.insert(
                Point {
                    x: 5,
                    y: new_y,
                },
                1,
            );
        }
    }

    assert!(fell_down, "Gravity should eventually pull the snake down");
}
