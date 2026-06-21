use snake_game::*;

#[test]
fn test_kraken_pulls_snake() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);

    game.snake = snake::Snake::new(snake::Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = snake::Direction::Right;

    // Spawn Kraken
    game.bosses.push(game::Boss {
        position: snake::Point {
            x: 15,
            y: 5,
        },
        health: 100,
        max_health: 100,
        move_timer: 0,
        shoot_timer: 0,
        kind: game::BossType::Kraken,
        state_timer: 0,
    });

    let _initial_head = game.snake.head();

    // Ensure snake doesn't move forward continuously for normal reasons
    // By giving it a dummy power-up or pausing updates? No, `Game::update` moves the snake if auto_pilot is off and it's time to move.
    // Instead of full `game.update()`, we can just call `process_bosses` (if it existed) or tick the boss manually.
    // Let's just track the direction. Kraken alters input via `game.handle_input`.
    // We can observe `game.snake.direction` or `game.snake.direction_queue` getting modified by Kraken!

    let _initial_dir = game.snake.direction;

    // Tick enough times to trigger pull
    let mut pulled = false;
    for _ in 0..100 {
        game.update();
        if game.snake.direction != _initial_dir {
            // Kraken forced an input!
            pulled = true;
            break;
        }
    }

    assert!(pulled, "Kraken should push a direction into the queue to pull the snake");
}
