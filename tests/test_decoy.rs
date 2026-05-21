use snake_game::game::{Boss, BossType, Difficulty, Game, Theme};
use snake_game::snake::{Direction, Point, Snake};
use web_time::SystemTime;

#[test]
fn test_boss_targets_decoy() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);

    // Set up the snake at (5, 5)
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    // Boss at (10, 10)
    game.bosses.push(Boss {
        position: Point {
            x: 10,
            y: 10,
        },
        health: 10,
        max_health: 10,
        move_timer: 100, // force move immediately
        shoot_timer: 0,
        kind: BossType::Charger, // Charger uses A* to ram the player
        state_timer: 0,
    });

    // Decoy at (15, 15) - opposite direction from snake
    game.decoy = Some((
        Point {
            x: 15,
            y: 15,
        },
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs(),
    ));

    // Clear obstacles
    game.obstacles.clear();

    // Store the distance to decoy before update
    let initial_dist_to_decoy = game.bosses[0].position.x.abs_diff(game.decoy.unwrap().0.x)
        + game.bosses[0].position.y.abs_diff(game.decoy.unwrap().0.y);

    // Play a frame
    game.state = snake_game::game::GameState::Playing;
    game.update();

    let new_boss_pos = game.bosses[0].position;

    let new_dist_to_decoy = new_boss_pos.x.abs_diff(game.decoy.unwrap().0.x)
        + new_boss_pos.y.abs_diff(game.decoy.unwrap().0.y);

    // The boss should have moved closer to the decoy (15, 15), meaning the distance should have decreased
    assert!(new_dist_to_decoy < initial_dist_to_decoy);
}
