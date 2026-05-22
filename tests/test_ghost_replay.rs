use snake_game::game::{Difficulty, Game, GameMode, Theme};
use snake_game::snake::{Direction, Point};

#[test]
fn test_ghost_replay_recording_and_playback() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Speedrun;
    game.reset();

    // Emulate some moves
    game.snake.direction_queue.push_back(Direction::Right);
    game.update();
    game.snake.direction_queue.push_back(Direction::Down);
    game.update();

    // Check if moves were recorded
    assert_eq!(game.current_replay.len(), 2);
    assert_eq!(game.current_replay[0], Direction::Right);
    assert_eq!(game.current_replay[1], Direction::Down);

    // Now test ghost playback manually
    game.ghost_moves.push_back(Direction::Left);
    game.ghost_moves.push_back(Direction::Up);
    game.ghost_snake = Some(snake_game::snake::Snake::new(Point { x: 10, y: 10 }));

    game.update();

    // Verify ghost snake moved Left
    let ghost = game.ghost_snake.as_ref().unwrap();
    assert_eq!(ghost.direction, Direction::Left);
    assert_eq!(ghost.head(), Point { x: 9, y: 10 });

    game.update();
    let ghost = game.ghost_snake.as_ref().unwrap();
    assert_eq!(ghost.direction, Direction::Up);
    assert_eq!(ghost.head(), Point { x: 9, y: 9 });
}
