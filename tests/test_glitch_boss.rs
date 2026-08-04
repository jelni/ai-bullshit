use snake_game::game::{Boss, BossType, Difficulty, Game, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

#[test]
fn test_glitch_boss_scrambles_controls() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();
    game.bosses.clear();

    let start_pos = Point { x: 5, y: 5 };
    game.bosses.push(Boss {
        position: start_pos,
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 19, // One tick before threshold (20)
        kind: BossType::Glitch,
        state_timer: 0,
    });

    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Up;
    game.snake.direction_queue.clear();

    game.state = GameState::Playing;

    assert_eq!(game.controls_scrambled_timer, 0);

    // Update game, the Glitch boss shoot_timer will hit 20
    game.update();

    assert_eq!(game.controls_scrambled_timer, 50, "Glitch boss should set controls_scrambled_timer to 50");
    assert_eq!(game.bosses[0].shoot_timer, 0, "Glitch boss shoot_timer should reset");
    assert_ne!(game.bosses[0].position, start_pos, "Glitch boss should teleport");

    // Test input handling
    game.handle_input(Direction::Right, 1);

    // Because controls are scrambled, pushing Right should result in Left
    assert_eq!(game.snake.direction_queue.len(), 1);
    assert_eq!(game.snake.direction_queue.front().copied(), Some(Direction::Left), "Input should be scrambled from Right to Left");

    game.handle_input(Direction::Up, 1);
    assert_eq!(game.snake.direction_queue.len(), 2);
    assert_eq!(game.snake.direction_queue.back().copied(), Some(Direction::Down), "Input should be scrambled from Up to Down");

    // Clear queue
    game.snake.direction_queue.clear();

    // Test decrement
    game.update();
    // After next update, timer should be current_timer - 1, BUT wait, when timer hits 20 it was 50.
    // In update, timer is decremented at the very beginning, then boss fires and sets to 50.
    // So if we run another update, it will decrement to 49.
    assert_eq!(game.controls_scrambled_timer, 49, "Timer should decrement on update");
}
