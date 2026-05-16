use snake_game::game::{Boss, Difficulty, Game, GameMode, Theme};
use snake_game::snake::{Direction, Point, Snake};

#[test]
fn test_bot_shoots_boss_in_line_of_sight() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    // Boss right in front
    game.boss = Some(Boss {
        position: Point {
            x: 9,
            y: 5,
        },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
    });

    assert!(game.should_bot_shoot(1));
}

#[test]
fn test_bot_shoots_p2_in_line_of_sight() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::BotVsBot;

    // Create snake facing down. Make sure it has only 1 body part so it doesn't overlap
    // Note: Snake::new creates body facing UP, so x:5, y:5, y:6, y:7
    // So if it's facing down, it will shoot towards y:8, y:9, y:10.
    // However, since we created it at y:5, its body extends to y:6 and y:7.
    // If it casts ray down, it hits its own body first!
    // That's why it didn't shoot. Let's fix the setup to avoid self-hit.

    game.snake.body.clear();
    game.snake.body.push_back(Point {
        x: 5,
        y: 5,
    });
    game.snake.rebuild_map();
    game.snake.direction = Direction::Down;

    let mut p2 = Snake::new(Point {
        x: 5,
        y: 10,
    });
    p2.body.clear();
    p2.body.push_back(Point {
        x: 5,
        y: 10,
    });
    p2.rebuild_map();
    p2.direction = Direction::Up;

    game.player2 = Some(p2);

    // Player 1 facing Player 2 should shoot
    assert!(game.should_bot_shoot(1));
    // Player 2 facing Player 1 should shoot
    assert!(game.should_bot_shoot(2));
}

#[test]
fn test_bot_shoots_obstacle_in_line_of_sight() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    game.obstacles.clear();
    // Obstacle at x: 8, y: 5 (distance 3)
    game.obstacles.insert(Point {
        x: 8,
        y: 5,
    });

    assert!(game.should_bot_shoot(1));

    game.obstacles.clear();
    // Obstacle at x: 15, y: 5 (distance 10, > 5)
    game.obstacles.insert(Point {
        x: 15,
        y: 5,
    });

    assert!(!game.should_bot_shoot(1));
}

#[test]
fn test_bot_does_not_shoot_empty_space() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    game.obstacles.clear();
    game.boss = None;
    game.player2 = None;

    assert!(!game.should_bot_shoot(1));
}

#[test]
fn test_bot_does_not_shoot_itself() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    // Make the snake very long and curl back into its own line of sight
    game.snake.body.push_back(Point {
        x: 5,
        y: 4,
    });
    game.snake.body.push_back(Point {
        x: 6,
        y: 4,
    });
    game.snake.body.push_back(Point {
        x: 7,
        y: 4,
    });
    game.snake.body.push_back(Point {
        x: 7,
        y: 5,
    });
    game.snake.rebuild_map();

    game.snake.direction = Direction::Right; // Facing x: 6, 7 where the body is at 7

    assert!(!game.should_bot_shoot(1));
}
