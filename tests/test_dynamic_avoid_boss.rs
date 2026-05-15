use snake_game::*;

#[test]
fn test_bot_predicts_boss() {
    let mut game = game::Game::new(
        20,
        20,
        false,
        'x',
        game::Theme::Classic,
        game::Difficulty::Normal,
    );
    game.snake = snake::Snake::new(snake::Point { x: 5, y: 5 });
    game.snake.direction = snake::Direction::Right;
    game.food = snake::Point { x: 5, y: 10 }; // Food is down

    // Boss at x=5, y=7
    // Next turn we move Down to 5,6. Boss moves closer to us. Will it hit us?
    game.boss = Some(game::Boss {
        position: snake::Point { x: 5, y: 7 },
        health: 10,
        max_health: 10,
        move_timer: 1, // Will move this turn
        shoot_timer: 0,
    });

    // In a real scenario, the A* needs to simulate the boss moving towards the snake.
    // It shouldn't just check if the spot is currently empty.
    let next_move = game.calculate_autopilot_move();

    // Right now, the A* in Game::is_safe_final_p only checks `self.boss.position == final_p`, which is the CURRENT position, not where it will be in `steps` steps.

    // We should ensure the snake doesn't go straight down if the boss is going to move up to 5,6.
    println!("next_move: {:?}", next_move);
}
