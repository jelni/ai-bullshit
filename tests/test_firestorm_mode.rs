use snake_game::game::{Difficulty, Game, GameMode, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn firestorm_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Firestorm;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_firestorm_reset_spawns_sources() {
    let game = firestorm_game(30, 30);

    assert_eq!(game.firestorm_blazes, 0);
    assert_eq!(game.weather, Weather::Firestorm);
    assert_eq!(game.blaze_sources.len(), 2, "Firestorm mode starts with 2 sources");
    for source in &game.blaze_sources {
        assert!(
            game.blaze_tiles.contains_key(source),
            "Source cells must be burning"
        );
        assert!(
            game.obstacles.contains(source),
            "Source cells must also be obstacles so they block movement"
        );
        assert!(!game.snake.body_map.contains_key(source));
    }
    assert!(!game.blaze_tiles.contains_key(&game.food));
}

#[test]
fn test_firestorm_blaze_every_6_ticks() {
    let mut game = firestorm_game(40, 40);
    game.obstacles.clear();
    game.blaze_tiles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.food = Point { x: 35, y: 35 };
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must blaze.
    for (i, tick) in [5_u32, 11, 17, 23].iter().enumerate() {
        game.state = snake_game::game::GameState::Playing;
        game.tick_counter = *tick;
        let blazes_before = game.firestorm_blazes;
        let fire_before = game.blaze_tiles.len();
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.firestorm_blazes > blazes_before,
            "a scheduled blaze must fire on tick {}",
            tick + 1
        );
        if i < 2 {
            assert!(
                game.blaze_tiles.len() > fire_before,
                "an early blaze must spread fire onto new cells (before: {}, after: {})",
                fire_before,
                game.blaze_tiles.len()
            );
        }
        for blaze in game.blaze_tiles.keys() {
            assert!(game.obstacles.contains(blaze));
            assert!(!game.snake.body_map.contains_key(blaze));
        }
    }
    assert_eq!(game.firestorm_blazes, 4);
}

#[test]
fn test_firestorm_is_lethal() {
    let mut game = firestorm_game(20, 20);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Up;
    game.snake.direction_queue.push_back(Direction::Up);

    let blaze_pos = Point { x: 5, y: 4 };
    game.blaze_tiles.insert(blaze_pos, Game::BLAZE_BURN_TICKS);
    game.obstacles.insert(blaze_pos);

    game.update();

    assert!(
        game.just_died || game.lives < 3,
        "Snake should die when moving into a burning tile"
    );
}

#[test]
fn test_firestorm_is_safe_final_p_rejects_fire() {
    let mut game = firestorm_game(20, 20);
    game.obstacles.clear();
    let blaze_pos = Point { x: 5, y: 5 };
    game.blaze_tiles.insert(blaze_pos, Game::BLAZE_BURN_TICKS);
    game.obstacles.insert(blaze_pos);

    assert!(
        !game.is_safe_final_p(blaze_pos, 1, 1),
        "is_safe_final_p must reject a burning cell (the autopilot must route around fire)"
    );
    assert!(
        game.is_safe_final_p(Point { x: 6, y: 5 }, 1, 1),
        "a non-burning cell must remain safe"
    );
}

#[test]
fn test_firestorm_blazes_burn_out_and_leave_open_ground() {
    let mut game = firestorm_game(20, 20);
    game.obstacles.clear();
    game.blaze_tiles.clear();
    game.blaze_sources.clear();
    let source = Point { x: 2, y: 2 };
    game.blaze_sources.push(source);
    game.blaze_tiles.insert(source, u32::MAX);
    game.obstacles.insert(source);

    let expiring = Point { x: 15, y: 15 };
    game.blaze_tiles.insert(expiring, 1);
    game.obstacles.insert(expiring);
    let burning = Point { x: 17, y: 17 };
    game.blaze_tiles.insert(burning, 5);
    game.obstacles.insert(burning);

    game.tick_counter = 1; // next update runs tick 2: no blaze, no die-down
    game.update();

    assert!(
        !game.blaze_tiles.contains_key(&expiring),
        "A blaze with 1 tick left must burn out"
    );
    assert!(
        !game.obstacles.contains(&expiring),
        "Burned-out fire must leave open ground behind"
    );
    assert_eq!(
        game.blaze_tiles.get(&burning),
        Some(&4),
        "A burning blaze must tick down one per tick"
    );
    assert!(
        game.blaze_tiles.contains_key(&source) && game.obstacles.contains(&source),
        "Blaze sources must keep burning"
    );
}

#[test]
fn test_firestorm_periodic_die_down_keeps_sources() {
    let mut game = firestorm_game(30, 30);
    game.obstacles.clear();
    game.blaze_tiles.clear();
    let source = Point { x: 2, y: 2 };
    game.blaze_sources = vec![source];
    game.blaze_tiles.insert(source, u32::MAX);
    game.obstacles.insert(source);

    let flame_a = Point { x: 10, y: 10 };
    let flame_b = Point { x: 11, y: 10 };
    for p in [flame_a, flame_b] {
        game.blaze_tiles.insert(p, 8);
        game.obstacles.insert(p);
    }

    // Next update brings tick_counter to 200, which triggers the die-down
    game.tick_counter = 199;
    game.update();

    assert!(
        !game.blaze_tiles.contains_key(&flame_a) && !game.blaze_tiles.contains_key(&flame_b),
        "Die-down at tick 200 should clear all spread blazes"
    );
    assert!(!game.obstacles.contains(&flame_a));
    assert!(!game.obstacles.contains(&flame_b));
    assert!(
        game.blaze_tiles.contains_key(&source),
        "Blaze sources must survive the die-down"
    );
    assert!(game.obstacles.contains(&source));
}

#[test]
fn test_firestorm_eating_food_douses_adjacent_blazes() {
    let mut game = firestorm_game(20, 20);
    game.obstacles.clear();
    game.blaze_tiles.clear();
    game.blaze_sources.clear();

    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };

    let adjacent_blaze = Point { x: 6, y: 6 };
    game.blaze_tiles.insert(adjacent_blaze, 8);
    game.obstacles.insert(adjacent_blaze);
    let far_blaze = Point { x: 15, y: 15 };
    game.blaze_tiles.insert(far_blaze, 8);
    game.obstacles.insert(far_blaze);

    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.blaze_tiles.contains_key(&adjacent_blaze),
        "Eating food should douse adjacent blazes"
    );
    assert!(!game.obstacles.contains(&adjacent_blaze));
    assert!(
        game.blaze_tiles.contains_key(&far_blaze),
        "Distant blazes should be unaffected"
    );
}

#[test]
fn test_firestorm_never_spreads_onto_the_next_head_cell() {
    let mut game = firestorm_game(20, 20);
    game.obstacles.clear();
    game.blaze_tiles.clear();
    game.blaze_sources.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 18, y: 18 };
    // Blaze source right next to the cell the snake is about to enter (6, 5)
    let source = Point { x: 6, y: 6 };
    game.blaze_sources.push(source);
    game.blaze_tiles.insert(source, u32::MAX);
    game.obstacles.insert(source);

    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = snake_game::game::GameState::Playing;
        game.tick_counter = tick;
        let next_head = Point {
            x: game.snake.head().x + 1,
            y: game.snake.head().y,
        };
        game.update();
        assert!(
            !game.blaze_tiles.contains_key(&next_head),
            "Fire must never spread onto the cell the snake was about to enter (tick {})",
            tick + 1
        );
    }
    assert!(game.firestorm_blazes >= 6);
}

#[test]
fn test_firestorm_blaze_respects_cap_and_never_covers_snakes() {
    let mut game = firestorm_game(40, 30);
    game.auto_pilot = true;
    let cap = (u32::from(game.width) * u32::from(game.height)) * 15 / 100;

    for _ in 0..150 {
        game.update();
        if game.state != snake_game::game::GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = snake_game::game::GameState::Playing;
        }
        assert!(
            u32::try_from(game.blaze_tiles.len()).unwrap_or(u32::MAX) <= cap,
            "Fire must stay under the 15% board cap (len: {}, cap: {})",
            game.blaze_tiles.len(),
            cap
        );
        for blaze in game.blaze_tiles.keys() {
            assert!(
                game.obstacles.contains(blaze),
                "Every burning tile must be mirrored into obstacles"
            );
        }
        assert!(
            !game.blaze_tiles.contains_key(&game.snake.head()),
            "Fire must never spawn on the snake's head"
        );
        assert_eq!(game.weather, Weather::Firestorm, "The firestorm weather stays pinned");
    }
}

#[test]
fn test_firestorm_bot_survives_the_blazes() {
    let mut game = firestorm_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.update();
        if game.state != snake_game::game::GameState::Playing {
            if game.lives == 0 {
                break;
            }
            // Non-death interruption (e.g. merchant visit): resume playing
            game.state = snake_game::game::GameState::Playing;
        }
    }

    assert!(
        game.state == snake_game::game::GameState::Playing && game.lives > 0,
        "Autopilot should survive the firestorm (lives: {}, blazes: {}, fire: {}, score: {})",
        game.lives,
        game.firestorm_blazes,
        game.blaze_tiles.len(),
        game.score
    );
    assert!(
        game.firestorm_blazes >= 20,
        "expected at least 20 blazes over 120 ticks, got {}",
        game.firestorm_blazes
    );
}

#[test]
fn test_firestorm_state_persists_through_time_rewind() {
    let mut game = firestorm_game(30, 30);
    game.obstacles.clear();
    game.blaze_tiles.clear();
    let source = Point { x: 2, y: 2 };
    game.blaze_sources = vec![source];
    game.blaze_tiles.insert(source, u32::MAX);
    game.obstacles.insert(source);
    let flame = Point { x: 12, y: 12 };
    game.blaze_tiles.insert(flame, 8);
    game.obstacles.insert(flame);
    game.firestorm_blazes = 7;

    game.save_history_state();

    // Simulate later changes: blaze burned out, counter moved on
    game.blaze_tiles.remove(&flame);
    game.obstacles.remove(&flame);
    game.firestorm_blazes = 9;

    game.rewind_time();

    assert_eq!(game.firestorm_blazes, 7, "Rewind must restore the blaze counter");
    assert!(
        game.blaze_tiles.contains_key(&flame),
        "Rewind must restore burning tiles"
    );
    assert!(
        game.blaze_sources.contains(&source),
        "Rewind must restore blaze sources"
    );
}
