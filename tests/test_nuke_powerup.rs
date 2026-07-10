use snake_game::game::{Boss, BossType, Game, Laser, Meteor, PowerUp, PowerUpType};
use snake_game::snake::{Direction, Point, Snake};

#[test]
fn test_nuke_powerup_destroys_everything() {
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

    // Clear randomly generated obstacles to prevent flakiness
    game.obstacles.clear();

    // Place a Nuke right in front of the snake
    game.power_up = Some(PowerUp {
        p_type: PowerUpType::Nuke,
        location: Point { x: 11, y: 10 },
        activation_time: None,
    });

    // Add enemies and obstacles
    game.bosses.push(Boss {
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
        player: 3,
    });

    game.mines.insert(Point { x: 15, y: 15 });

    game.meteors.push(Meteor {
        position: Point { x: 8, y: 8 },
        timer: 5,
    });

    game.resources.insert(Point { x: 1, y: 1 }, snake_game::game::Resource::Wood);

    game.obstacles.insert(Point { x: 4, y: 4 });
    game.poison_food = Some((Point { x: 6, y: 6 }, web_time::Instant::now()));
    game.bonus_food = Some((Point { x: 7, y: 7 }, web_time::Instant::now()));

    // Ensure they exist
    assert!(!game.bosses.is_empty());
    assert!(!game.lasers.is_empty());
    assert!(!game.mines.is_empty());
    assert!(!game.meteors.is_empty());
    assert!(!game.resources.is_empty());
    assert!(!game.obstacles.is_empty());
    assert!(game.poison_food.is_some());
    assert!(game.bonus_food.is_some());

    // Move the snake into the Nuke
    game.state = snake_game::game::GameState::Playing;
    game.handle_input(Direction::Right, 1);
    game.update();

    // Verify Nuke effects
    assert!(game.bosses.is_empty(), "Nuke should destroy all bosses");
    assert!(game.lasers.is_empty(), "Nuke should destroy all lasers");
    assert!(game.mines.is_empty(), "Nuke should destroy all mines");
    assert!(game.meteors.is_empty(), "Nuke should destroy all meteors");
    assert!(game.resources.is_empty(), "Nuke should destroy all resources");
    assert!(game.obstacles.is_empty(), "Nuke should clear obstacles");
    assert!(game.poison_food.is_none(), "Nuke should clear poison food");
    assert!(game.bonus_food.is_none(), "Nuke should clear bonus food");
}
