use snake_game::game::{Boss, BossType, Game, Laser, PowerUp, PowerUpType};
use snake_game::snake::{Direction, Point, Snake};

#[test]
fn test_emp_powerup_destroys_mines_lasers_and_stuns_boss() {
    let mut game = Game::new(
        20,
        20,
        false,
        'x',
        snake_game::game::Theme::Classic,
        snake_game::game::Difficulty::Normal,
    );

    // Set up the snake
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;

    // Place an Emp right in front of the snake
    game.power_up = Some(PowerUp {
        p_type: PowerUpType::Emp,
        location: Point { x: 11, y: 10 },
        activation_time: None,
    });

    // Add a boss, a laser, and a mine
    game.boss = Some(Boss {
        position: Point { x: 5, y: 5 },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Charger,
        state_timer: 0,
    });

    game.lasers.push(Laser {
        position: Point { x: 2, y: 2 },
        direction: Direction::Down,
        player: 3, // Boss laser
    });

    game.mines.insert(Point { x: 15, y: 15 });

    // Ensure they exist
    assert!(!game.lasers.is_empty());
    assert!(!game.mines.is_empty());
    assert_eq!(game.boss.unwrap().state_timer, 0);

    // Move the snake into the Emp
    game.state = snake_game::game::GameState::Playing;
    game.handle_input(Direction::Right, 1);
    game.update();

    // Verify Emp effects
    assert!(game.lasers.is_empty(), "Emp should destroy all lasers");
    assert!(game.mines.is_empty(), "Emp should destroy all mines");

    let boss = game.boss.expect("Boss should still exist");
    assert_eq!(boss.state_timer, 30, "Emp should stun boss for 30 ticks");
}
