use snake_game::game::{Difficulty, Game, GameMode, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn wildfire_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Wildfire;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_wildfire_reset_ignites_initial_fires() {
    let game = wildfire_game(30, 30);

    assert_eq!(game.wildfire_ignitions, 2, "Wildfire mode starts with 2 ignitions");
    assert_eq!(game.fire_tiles.len(), 2, "Both ignitions must land on the board");
    for fire in &game.fire_tiles {
        assert!(
            game.obstacles.contains(fire),
            "Burning cells must also be obstacles so they block movement"
        );
        assert_eq!(
            game.fire_ttl.get(fire),
            Some(&Game::FIRE_TTL),
            "Freshly ignited cells burn for FIRE_TTL ticks"
        );
        assert!(!game.snake.body_map.contains_key(fire));
    }
    assert!(!game.fire_tiles.contains(&game.food));
    assert_eq!(game.weather, snake_game::game::Weather::Wildfire);
}

#[test]
fn test_wildfire_blaze_every_6_ticks() {
    let mut game = wildfire_game(40, 40);
    game.obstacles.clear();
    game.fire_tiles.clear();
    game.fire_ttl.clear();
    game.ash_tiles.clear();
    game.wildfire_ignitions = 0;
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.food = Point { x: 35, y: 35 };
    game.auto_pilot = true;
    // Seed one burning cell so the frontier spread has somewhere to start from
    let seed = Point { x: 20, y: 20 };
    game.fire_tiles.insert(seed);
    game.fire_ttl.insert(seed, Game::FIRE_TTL);
    game.obstacles.insert(seed);

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must blaze (spread + new ignition).
    for tick in [5_u32, 11, 17, 23] {
        game.state = snake_game::game::GameState::Playing;
        game.tick_counter = tick;
        let ignitions_before = game.wildfire_ignitions;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.wildfire_ignitions > ignitions_before,
            "a scheduled blaze must start a new ignition on tick {}",
            tick + 1
        );
        assert!(
            game.fire_tiles.len() > 1,
            "a blaze must spread the fire onto new cells (after: {})",
            game.fire_tiles.len()
        );
        for fire in &game.fire_tiles {
            assert!(game.obstacles.contains(fire));
            assert!(!game.snake.body_map.contains_key(fire));
        }
    }
    assert_eq!(game.wildfire_ignitions, 4);
}

#[test]
fn test_wildfire_fire_is_lethal() {
    let mut game = wildfire_game(20, 20);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Up;
    game.snake.direction_queue.push_back(Direction::Up);

    let fire_pos = Point { x: 5, y: 4 };
    game.fire_tiles.insert(fire_pos);
    game.fire_ttl.insert(fire_pos, Game::FIRE_TTL);
    game.obstacles.insert(fire_pos);

    game.update();

    assert!(
        game.just_died || game.lives < 3,
        "Snake should die when moving into a burning cell"
    );
}

#[test]
fn test_wildfire_is_safe_final_p_rejects_fire() {
    let mut game = wildfire_game(20, 20);
    game.obstacles.clear();
    let fire_pos = Point { x: 5, y: 5 };
    game.fire_tiles.insert(fire_pos);
    game.fire_ttl.insert(fire_pos, Game::FIRE_TTL);
    game.obstacles.insert(fire_pos);

    assert!(
        !game.is_safe_final_p(fire_pos, 1, 1),
        "is_safe_final_p must reject a burning cell (the autopilot must route around fire)"
    );
    assert!(
        game.is_safe_final_p(Point { x: 6, y: 5 }, 1, 1),
        "a non-burning cell must remain safe"
    );
}

#[test]
fn test_wildfire_burn_down_turns_fire_into_ash() {
    let mut game = wildfire_game(30, 30);
    game.obstacles.clear();
    game.fire_tiles.clear();
    game.fire_ttl.clear();
    game.ash_tiles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.auto_pilot = true;

    let fire = Point { x: 12, y: 12 };
    game.fire_tiles.insert(fire);
    game.fire_ttl.insert(fire, 1); // burns out on the next tick
    game.obstacles.insert(fire);

    game.tick_counter = 3; // not a blaze tick, not the douse tick
    game.state = snake_game::game::GameState::Playing;
    game.update();

    assert!(
        !game.fire_tiles.contains(&fire),
        "A cell whose TTL expired must stop burning"
    );
    assert!(!game.fire_ttl.contains_key(&fire));
    assert!(
        !game.obstacles.contains(&fire),
        "Burned-out cells must stop being obstacles"
    );
    assert!(
        game.ash_tiles.contains(&fire),
        "Burned-out cells must smolder into ash"
    );
}

#[test]
fn test_wildfire_periodic_douse_clears_fire_and_ash() {
    let mut game = wildfire_game(30, 30);
    game.obstacles.clear();
    game.fire_tiles.clear();
    game.fire_ttl.clear();
    game.ash_tiles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.auto_pilot = true;

    let fire_a = Point { x: 10, y: 10 };
    let fire_b = Point { x: 11, y: 10 };
    for p in [fire_a, fire_b] {
        game.fire_tiles.insert(p);
        game.fire_ttl.insert(p, Game::FIRE_TTL);
        game.obstacles.insert(p);
    }
    let ash = Point { x: 7, y: 7 };
    game.ash_tiles.insert(ash);

    // Next update brings tick_counter to 200, which triggers the rain douse
    game.tick_counter = 199;
    game.state = snake_game::game::GameState::Playing;
    game.update();

    assert!(
        game.fire_tiles.is_empty(),
        "The tick-200 rain must douse all remaining fire"
    );
    assert!(game.fire_ttl.is_empty());
    assert!(!game.obstacles.contains(&fire_a));
    assert!(!game.obstacles.contains(&fire_b));
    assert!(
        game.ash_tiles.is_empty(),
        "The rain must also wash the ash away"
    );
}

#[test]
fn test_wildfire_eating_food_douses_adjacent_fire() {
    let mut game = wildfire_game(20, 20);
    game.obstacles.clear();
    game.fire_tiles.clear();
    game.fire_ttl.clear();
    game.ash_tiles.clear();

    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };

    let adjacent_fire = Point { x: 6, y: 6 };
    game.fire_tiles.insert(adjacent_fire);
    game.fire_ttl.insert(adjacent_fire, Game::FIRE_TTL);
    game.obstacles.insert(adjacent_fire);
    let far_fire = Point { x: 15, y: 15 };
    game.fire_tiles.insert(far_fire);
    game.fire_ttl.insert(far_fire, Game::FIRE_TTL);
    game.obstacles.insert(far_fire);

    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.fire_tiles.contains(&adjacent_fire),
        "Eating food should douse adjacent fire"
    );
    assert!(!game.obstacles.contains(&adjacent_fire));
    assert!(
        game.ash_tiles.contains(&adjacent_fire),
        "Doused cells smolder into ash"
    );
    assert!(
        game.fire_tiles.contains(&far_fire),
        "Distant fire should be unaffected"
    );
}

#[test]
fn test_wildfire_respects_cap_and_never_covers_snakes() {
    let mut game = wildfire_game(40, 30);
    game.auto_pilot = true;
    let cap = (u32::from(game.width) * u32::from(game.height)) * 12 / 100;

    for _ in 0..150 {
        // Pin a harmless power-up so the bot can never eat a Bomb/Nuke/ClearObstacles
        // (those mass-clear obstacles mid-tick; the engine prunes stale fire records on
        // the next tick, matching the lava/sand/snow self-heal pattern).
        game.power_up = Some(snake_game::game::PowerUp {
            p_type: snake_game::game::PowerUpType::SpeedBoost,
            location: Point { x: 1, y: 1 },
            activation_time: None,
        });
        game.update();
        if game.state != snake_game::game::GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = snake_game::game::GameState::Playing;
        }
        assert!(
            u32::try_from(game.fire_tiles.len()).unwrap_or(u32::MAX) <= cap,
            "Fire must stay under the 12% board cap (len: {}, cap: {})",
            game.fire_tiles.len(),
            cap
        );
        for fire in &game.fire_tiles {
            assert!(
                game.obstacles.contains(fire),
                "Every burning cell must be mirrored into obstacles"
            );
        }
        assert!(
            !game.fire_tiles.contains(&game.snake.head()),
            "Fire must never spawn on the snake's head"
        );
    }
}

#[test]
fn test_wildfire_bot_survives_the_blazes() {
    let mut game = wildfire_game(40, 30);
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
        "Autopilot should survive the wildfire (lives: {}, ignitions: {}, fire: {}, score: {})",
        game.lives,
        game.wildfire_ignitions,
        game.fire_tiles.len(),
        game.score
    );
    assert!(
        game.wildfire_ignitions >= 20,
        "expected at least 20 ignitions over 120 ticks, got {}",
        game.wildfire_ignitions
    );
}

#[test]
fn test_wildfire_state_persists_through_time_rewind() {
    let mut game = wildfire_game(30, 30);
    game.obstacles.clear();
    game.fire_tiles.clear();
    game.fire_ttl.clear();
    game.ash_tiles.clear();
    let fire = Point { x: 12, y: 12 };
    game.fire_tiles.insert(fire);
    game.fire_ttl.insert(fire, 9);
    game.obstacles.insert(fire);
    let ash = Point { x: 3, y: 3 };
    game.ash_tiles.insert(ash);
    game.wildfire_ignitions = 7;

    game.save_history_state();

    // Simulate later changes: fire burned out, counter moved on
    game.fire_tiles.remove(&fire);
    game.fire_ttl.remove(&fire);
    game.obstacles.remove(&fire);
    game.ash_tiles.insert(fire);
    game.wildfire_ignitions = 9;

    game.rewind_time();

    assert_eq!(
        game.wildfire_ignitions, 7,
        "Rewind must restore the ignition counter"
    );
    assert!(
        game.fire_tiles.contains(&fire),
        "Rewind must restore burning cells"
    );
    assert_eq!(
        game.fire_ttl.get(&fire),
        Some(&9),
        "Rewind must restore the burn timers"
    );
    assert!(game.obstacles.contains(&fire));
    assert!(
        game.ash_tiles.contains(&ash) && !game.ash_tiles.contains(&fire),
        "Rewind must restore the ash field"
    );
}
