use snake_game::game::{Game, Meteor};
use snake_game::snake::Point;

#[test]
fn test_meteor_spawning_and_falling() {
    let mut game = Game::new(
        20,
        20,
        false,
        'x',
        snake_game::game::Theme::Classic,
        snake_game::game::Difficulty::Normal,
    );

    game.meteors.clear();

    // Force a meteor to spawn
    game.meteors.push(Meteor {
        position: Point { x: 10, y: 5 },
        timer: 0,
    });

    game.state = snake_game::game::GameState::Playing;

    // After 1 tick, timer should be 1, y should be 5
    game.update();
    assert_eq!(game.meteors.len(), 1);
    assert_eq!(game.meteors[0].timer, 1);
    assert_eq!(game.meteors[0].position.y, 5);

    // After 2 ticks, timer resets to 0, y should be 6
    game.update();
    assert_eq!(game.meteors.len(), 1);
    assert_eq!(game.meteors[0].timer, 0);
    assert_eq!(game.meteors[0].position.y, 6);
}

#[test]
fn test_meteor_kills_snake() {
    let mut game = Game::new(
        20,
        20,
        false,
        'x',
        snake_game::game::Theme::Classic,
        snake_game::game::Difficulty::Normal,
    );

    // Place snake
    game.snake = snake_game::snake::Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = snake_game::snake::Direction::Right; // moving to (11,10)
    game.obstacles.clear();

    // Place meteor that will hit the snake's next head position
    game.meteors.push(Meteor {
        position: Point { x: 11, y: 10 },
        timer: 0,
    });

    game.state = snake_game::game::GameState::Playing;

    let lives_before = game.lives;
    game.update(); // Snake moves to (11,10), colliding with meteor

    assert_eq!(game.lives, lives_before - 1, "Snake should die from meteor collision");
}

#[test]
fn test_meteor_destroys_obstacle() {
    let mut game = Game::new(
        20,
        20,
        false,
        'x',
        snake_game::game::Theme::Classic,
        snake_game::game::Difficulty::Normal,
    );

    game.obstacles.clear();
    let obs_pos = Point { x: 5, y: 5 };
    game.obstacles.insert(obs_pos);

    // Meteor falls exactly on the obstacle next tick
    game.meteors.push(Meteor {
        position: Point { x: 5, y: 4 },
        timer: 1, // Ready to move down on next update
    });

    game.state = snake_game::game::GameState::Playing;

    game.update();

    // The meteor should move down to (5,5), hit the obstacle, destroy it, and despawn itself
    assert!(!game.obstacles.contains(&obs_pos), "Meteor should destroy the obstacle");
    assert!(game.meteors.is_empty(), "Meteor should despawn after hitting obstacle");
}
