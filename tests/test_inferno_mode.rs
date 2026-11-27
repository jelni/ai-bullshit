use snake_game::game::{Difficulty, Game, GameMode, SaveState, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn inferno_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Inferno;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_inferno_reset_spawns_hotspots() {
    let game = inferno_game(30, 30);

    assert_eq!(game.inferno_flares, 0);
    assert_eq!(game.inferno_burned, 0);
    assert_eq!(
        game.inferno_hotspots.len(),
        3,
        "Inferno mode starts with 3 hotspots"
    );
    assert_eq!(game.weather, Weather::Inferno, "Inferno weather must be set");
    for hotspot in &game.inferno_hotspots {
        assert!(
            game.inferno_tiles.contains(hotspot),
            "Hotspot cells must be burning"
        );
        assert!(
            game.obstacles.contains(hotspot),
            "Hotspot cells must also be obstacles so they block movement"
        );
        assert!(!game.snake.body_map.contains_key(hotspot));
        assert!(
            !game.inferno_burn.contains_key(hotspot),
            "Hotspots never burn out, so they must have no burn timer"
        );
    }
    assert!(!game.inferno_tiles.contains(&game.food));
}

#[test]
fn test_inferno_flare_up_every_6_ticks() {
    let mut game = inferno_game(40, 40);
    game.obstacles.clear();
    game.inferno_tiles.clear();
    // Hotspot in the middle of a clear board so the fire blob always has free neighbors
    game.inferno_hotspots = vec![Point { x: 20, y: 20 }];
    game.inferno_tiles.insert(Point { x: 20, y: 20 });
    game.obstacles.insert(Point { x: 20, y: 20 });
    game.snake = Snake::new(Point { x: 5, y: 35 });
    game.snake.direction = Direction::Right;
    game.food = Point { x: 35, y: 5 };
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must flare up.
    let fire_at_start = game.inferno_tiles.len();
    for tick in [5_u32, 11, 17, 23] {
        game.state = snake_game::game::GameState::Playing;
        game.tick_counter = tick;
        let flares_before = game.inferno_flares;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.inferno_flares > flares_before,
            "a scheduled flare-up must fire on tick {}",
            tick + 1
        );
        for fire in &game.inferno_tiles {
            assert!(game.obstacles.contains(fire));
            assert!(!game.snake.body_map.contains_key(fire));
        }
    }
    assert_eq!(game.inferno_flares, 4);
    assert!(
        game.inferno_tiles.len() > fire_at_start,
        "flare-ups must spread fire onto new cells (start: {}, now: {})",
        fire_at_start,
        game.inferno_tiles.len()
    );
}

#[test]
fn test_inferno_flames_burn_out() {
    let mut game = inferno_game(30, 30);
    game.obstacles.clear();
    game.inferno_tiles.clear();
    game.inferno_hotspots = vec![Point { x: 2, y: 2 }];
    game.inferno_tiles.insert(Point { x: 2, y: 2 });
    game.obstacles.insert(Point { x: 2, y: 2 });

    let flame = Point { x: 10, y: 10 };
    game.inferno_tiles.insert(flame);
    game.obstacles.insert(flame);
    game.inferno_burn.insert(flame, 1);
    let burned_before = game.inferno_burned;

    // Next update runs tick 1: no flare-up, no rain, just burn-down
    game.tick_counter = 0;
    game.update();

    assert!(
        !game.inferno_tiles.contains(&flame),
        "A flame whose timer ran out must burn out"
    );
    assert!(!game.obstacles.contains(&flame));
    assert!(!game.inferno_burn.contains_key(&flame));
    assert_eq!(game.inferno_burned, burned_before + 1);
    assert!(
        game.inferno_tiles.contains(&Point { x: 2, y: 2 }),
        "The hotspot must keep burning"
    );
}

#[test]
fn test_inferno_fire_is_lethal() {
    let mut game = inferno_game(20, 20);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Up;
    game.snake.direction_queue.push_back(Direction::Up);

    let fire_pos = Point { x: 5, y: 4 };
    game.inferno_tiles.insert(fire_pos);
    game.obstacles.insert(fire_pos);

    game.update();

    assert!(
        game.just_died || game.lives < 3,
        "Snake should die when moving into a fire tile"
    );
}

#[test]
fn test_inferno_is_safe_final_p_rejects_fire() {
    let mut game = inferno_game(20, 20);
    game.obstacles.clear();
    let fire_pos = Point { x: 5, y: 5 };
    game.inferno_tiles.insert(fire_pos);
    game.obstacles.insert(fire_pos);

    assert!(
        !game.is_safe_final_p(fire_pos, 1, 1),
        "is_safe_final_p must reject a fire cell (the autopilot must route around fire)"
    );
    assert!(
        game.is_safe_final_p(Point { x: 6, y: 5 }, 1, 1),
        "a non-fire cell must remain safe"
    );
}

#[test]
fn test_inferno_periodic_rain_douses_fire_but_keeps_hotspots() {
    let mut game = inferno_game(30, 30);
    game.obstacles.clear();
    game.inferno_tiles.clear();
    let hotspot = Point { x: 2, y: 2 };
    game.inferno_hotspots = vec![hotspot];
    game.inferno_tiles.insert(hotspot);
    game.obstacles.insert(hotspot);

    let flame_a = Point { x: 10, y: 10 };
    let flame_b = Point { x: 11, y: 10 };
    for p in [flame_a, flame_b] {
        game.inferno_tiles.insert(p);
        game.obstacles.insert(p);
        game.inferno_burn.insert(p, 30);
    }

    // Next update brings tick_counter to 200, which triggers the rain
    game.tick_counter = 199;
    game.update();

    assert!(
        !game.inferno_tiles.contains(&flame_a) && !game.inferno_tiles.contains(&flame_b),
        "Rain at tick 200 should douse all spread fire"
    );
    assert!(!game.obstacles.contains(&flame_a));
    assert!(!game.obstacles.contains(&flame_b));
    assert!(!game.inferno_burn.contains_key(&flame_a));
    assert!(
        game.inferno_tiles.contains(&hotspot),
        "Hotspots must survive the rain"
    );
    assert!(game.obstacles.contains(&hotspot));
}

#[test]
fn test_inferno_eating_food_clears_adjacent_fire() {
    let mut game = inferno_game(20, 20);
    game.obstacles.clear();
    game.inferno_tiles.clear();
    game.inferno_hotspots.clear();

    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };

    let adjacent_flame = Point { x: 6, y: 6 };
    game.inferno_tiles.insert(adjacent_flame);
    game.obstacles.insert(adjacent_flame);
    game.inferno_burn.insert(adjacent_flame, 30);
    let far_flame = Point { x: 15, y: 15 };
    game.inferno_tiles.insert(far_flame);
    game.obstacles.insert(far_flame);
    game.inferno_burn.insert(far_flame, 30);

    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.inferno_tiles.contains(&adjacent_flame),
        "Eating food should clear adjacent fire (firebreak)"
    );
    assert!(!game.obstacles.contains(&adjacent_flame));
    assert!(!game.inferno_burn.contains_key(&adjacent_flame));
    assert!(
        game.inferno_tiles.contains(&far_flame),
        "Distant fire should be unaffected"
    );
}

#[test]
fn test_inferno_fire_respects_cap_and_never_covers_snakes() {
    let mut game = inferno_game(40, 30);
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
            u32::try_from(game.inferno_tiles.len()).unwrap_or(u32::MAX) <= cap,
            "Fire must stay under the 15% board cap (len: {}, cap: {})",
            game.inferno_tiles.len(),
            cap
        );
        assert!(
            !game.inferno_tiles.contains(&game.snake.head()),
            "Fire must never spawn on the snake's head"
        );
        for (p, timer) in &game.inferno_burn {
            assert!(
                game.inferno_tiles.contains(p),
                "Burn timers must only exist for burning cells"
            );
            assert!(*timer > 0 && *timer <= Game::INFERNO_BURN_TICKS);
        }
    }
}

#[test]
fn test_inferno_bot_survives_the_flames() {
    let mut game = inferno_game(40, 30);
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
        "Autopilot should survive the inferno (lives: {}, flares: {}, fire: {}, score: {})",
        game.lives,
        game.inferno_flares,
        game.inferno_tiles.len(),
        game.score
    );
    assert!(
        game.inferno_flares >= 20,
        "expected at least 20 flare-ups over 120 ticks, got {}",
        game.inferno_flares
    );
}

#[test]
fn test_inferno_state_persists_through_time_rewind() {
    let mut game = inferno_game(30, 30);
    game.obstacles.clear();
    game.inferno_tiles.clear();
    let hotspot = Point { x: 2, y: 2 };
    game.inferno_hotspots = vec![hotspot];
    game.inferno_tiles.insert(hotspot);
    game.obstacles.insert(hotspot);
    let flame = Point { x: 12, y: 12 };
    game.inferno_tiles.insert(flame);
    game.obstacles.insert(flame);
    game.inferno_burn.insert(flame, 17);
    game.inferno_flares = 7;

    game.save_history_state();

    // Simulate later changes: flame burned out, counter moved on
    game.inferno_tiles.remove(&flame);
    game.obstacles.remove(&flame);
    game.inferno_burn.remove(&flame);
    game.inferno_flares = 9;

    game.rewind_time();

    assert_eq!(game.inferno_flares, 7, "Rewind must restore the flare counter");
    assert!(
        game.inferno_tiles.contains(&flame),
        "Rewind must restore fire tiles"
    );
    assert_eq!(
        game.inferno_burn.get(&flame),
        Some(&17),
        "Rewind must restore burn timers"
    );
    assert!(
        game.inferno_hotspots.contains(&hotspot),
        "Rewind must restore inferno hotspots"
    );
}

#[test]
fn test_inferno_state_persists_through_save_load() {
    let mut game = inferno_game(30, 30);
    game.obstacles.clear();
    game.inferno_tiles.clear();
    let hotspot = Point { x: 2, y: 2 };
    game.inferno_hotspots = vec![hotspot];
    game.inferno_tiles.insert(hotspot);
    game.obstacles.insert(hotspot);
    let flame = Point { x: 12, y: 9 };
    game.inferno_tiles.insert(flame);
    game.obstacles.insert(flame);
    game.inferno_burn.insert(flame, 17);
    game.inferno_flares = 42;
    game.inferno_burned = 5;

    let path = std::env::temp_dir().join(format!(
        "snake_inferno_save_{}.json",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    game.save_game_to_file(path.to_str().unwrap());
    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("save file must exist after save_game_to_file: {e}"));
    let state: SaveState = serde_json::from_str(&json).expect("saved JSON must parse as SaveState");
    let _ = std::fs::remove_file(&path);

    assert_eq!(state.mode, GameMode::Inferno);
    assert!(state.inferno_tiles.contains(&flame));
    assert!(state.inferno_tiles.contains(&hotspot));
    assert_eq!(state.inferno_burn.get(&flame), Some(&17));
    assert!(state.inferno_hotspots.contains(&hotspot));
    assert_eq!(state.inferno_flares, 42);
    assert_eq!(state.inferno_burned, 5);
}
