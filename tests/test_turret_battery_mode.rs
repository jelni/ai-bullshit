use snake_game::game::{Difficulty, Game, GameMode, GameState, Laser, Theme, Turret, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn battery_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::TurretBattery;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

fn keep_playing(game: &mut Game) {
    if game.state != GameState::Playing {
        if game.lives == 0 {
            return;
        }
        game.state = GameState::Playing;
    }
}

#[test]
fn test_turret_battery_reset_seeds_battery_and_clear_weather() {
    let game = battery_game(30, 30);

    let enemy_turrets = game.turrets.iter().filter(|t| t.is_enemy).count();
    assert_eq!(
        enemy_turrets,
        Game::TURRET_BATTERY_COUNT,
        "Turret Battery mode must seed a full enemy battery on reset"
    );
    assert_eq!(game.weather, Weather::Clear, "skies must be pinned clear");
    assert_eq!(game.turret_volleys, 0);
    assert_eq!(game.turret_jammed, 0);
    let spawn = Point {
        x: game.width / 2,
        y: game.height / 2,
    };
    for turret in &game.turrets {
        assert!(
            turret.position.x.abs_diff(spawn.x).saturating_add(turret.position.y.abs_diff(spawn.y))
                >= 5,
            "turrets must spawn away from the player spawn"
        );
        assert!(
            !game.snake.body_map.contains_key(&turret.position),
            "no turret may spawn on the snake"
        );
        assert_ne!(turret.position, game.food, "no turret may spawn on the food");
    }
}

#[test]
fn test_turret_battery_weather_stays_clear() {
    let mut game = battery_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..60 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        assert_eq!(
            game.weather,
            Weather::Clear,
            "Turret Battery mode must force clear weather on every tick"
        );
    }
}

#[test]
fn test_turret_battery_scheduled_volley_every_6_ticks() {
    let mut game = battery_game(30, 30);
    game.obstacles.clear();
    game.state = GameState::Playing;
    game.snake = Snake::new(Point { x: 3, y: 3 });
    game.snake.direction = Direction::Left;
    game.snake.direction_queue.push_back(Direction::Left);
    game.food = Point { x: 25, y: 25 };

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must trigger a scheduled volley.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let volleys_before = game.turret_volleys;
        game.update();
        assert_eq!(game.tick_counter, tick + 1, "update must run exactly one tick");
        assert!(
            game.turret_volleys > volleys_before,
            "a scheduled volley must fire on tick {}",
            tick + 1
        );
    }
    assert!(game.turret_volleys >= 6);
}

#[test]
fn test_turret_battery_volley_fires_enemy_beam() {
    let mut game = battery_game(30, 30);
    game.obstacles.clear();
    game.lasers.clear();
    game.state = GameState::Playing;
    game.snake = Snake::new(Point { x: 3, y: 3 });
    game.snake.direction = Direction::Left;
    game.snake.direction_queue.push_back(Direction::Left);
    game.food = Point { x: 25, y: 25 };
    game.turrets.clear();
    game.turrets.push(Turret {
        position: Point { x: 15, y: 15 },
        shoot_timer: 0,
        is_enemy: true,
    });

    game.tick_counter = 5; // next update runs tick 6: scheduled volley
    game.update();

    assert_eq!(game.tick_counter, 6);
    assert_eq!(game.turret_volleys, 1, "the volley must be counted");
    assert!(
        !game.lasers.is_empty(),
        "a fair volley from a lone turret must fire at least one beam"
    );
    assert!(
        game.lasers.iter().all(|l| l.player == 3),
        "volley beams must be enemy lasers"
    );
    assert!(
        game.lasers
            .iter()
            .any(|l| l.position == Point { x: 15, y: 15 }),
        "the placed turret must have fired a beam from its own cell"
    );
}

#[test]
fn test_turret_battery_volley_fizzles_when_every_direction_unfair() {
    let mut game = battery_game(30, 30);
    game.obstacles.clear();
    game.lasers.clear();
    game.state = GameState::Playing;
    // Snake heads Up into (1,2); the only turret sits in the corner (1,1), so
    // Up/Left are out of bounds, Down enters the snake's next cell, and Right
    // enters the food cell: every direction is unfair and must fizzle.
    game.snake = Snake::new(Point { x: 1, y: 3 });
    game.snake.direction = Direction::Up;
    game.snake.direction_queue.push_back(Direction::Up);
    game.food = Point { x: 2, y: 1 };
    game.turrets.clear();
    game.turrets.push(Turret {
        position: Point { x: 1, y: 1 },
        shoot_timer: 0,
        is_enemy: true,
    });

    game.tick_counter = 5; // next update runs tick 6: scheduled volley
    game.update();

    assert_eq!(game.tick_counter, 6);
    assert_eq!(game.turret_volleys, 1, "the volley is counted even when beams fizzle");
    assert!(
        game.lasers
            .iter()
            .all(|l| l.position != Point { x: 1, y: 1 }),
        "the corner turret must fizzle every beam when all directions are unfair (got {:?})",
        game.lasers
    );
}

#[test]
fn test_turret_battery_eating_food_jams_volleys() {
    let mut game = battery_game(30, 30);
    game.obstacles.clear();
    game.lasers.clear();
    game.state = GameState::Playing;
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.turrets.clear();
    game.turrets.push(Turret {
        position: Point { x: 20, y: 20 },
        shoot_timer: 0,
        is_enemy: true,
    });
    game.tick_counter = 100; // next tick 101: no scheduled volley

    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 6, y: 5 },
        "the snake must have moved onto the food"
    );
    assert_eq!(
        game.turret_jammed, Game::TURRET_JAM_TICKS,
        "eating food must jam the battery"
    );

    // While jammed, scheduled volleys are suppressed: jump to a volley tick
    // (102 is a multiple of 6) and verify nothing fires.
    game.state = GameState::Playing;
    game.tick_counter = 101;
    let volleys_before = game.turret_volleys;
    let lasers_before = game.lasers.len();
    game.update();

    assert_eq!(game.tick_counter, 102);
    assert_eq!(
        game.turret_volleys, volleys_before,
        "no volley may fire while the battery is jammed"
    );
    assert_eq!(
        game.lasers.len(),
        lasers_before,
        "no volley beam may fire while the battery is jammed"
    );
    assert_eq!(
        game.turret_jammed,
        Game::TURRET_JAM_TICKS - 1,
        "the jam timer must tick down once per tick"
    );
}

#[test]
fn test_turret_battery_enemy_laser_is_lethal() {
    let mut game = battery_game(30, 30);
    game.obstacles.clear();
    game.lasers.clear();
    game.state = GameState::Playing;
    game.lives = 1;
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    // Lasers move before the collision check, so a beam stepping into (6,5)
    // this tick meets the snake head there.
    game.lasers.push(Laser {
        position: Point { x: 7, y: 5 },
        direction: Direction::Left,
        player: 3,
    });
    game.tick_counter = 100;

    game.update();

    assert!(
        game.just_died || game.lives == 0 || game.state != GameState::Playing,
        "an enemy beam hitting the head must kill the snake"
    );
}

#[test]
fn test_turret_battery_battery_stays_armed() {
    let mut game = battery_game(30, 30);
    game.obstacles.clear();
    game.auto_pilot = true;
    game.turrets.clear(); // simulate the whole battery being destroyed

    for _ in 0..30 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        let enemy_turrets = game.turrets.iter().filter(|t| t.is_enemy).count();
        assert!(
            enemy_turrets > 0,
            "manage_turret_battery must re-seed enemy turrets to keep the battery armed"
        );
    }
}
