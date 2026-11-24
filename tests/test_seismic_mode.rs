use snake_game::game::{Difficulty, Game, GameMode, HeroClass, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn seismic_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Seismic;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_seismic_reset_forces_weather_and_initial_rubble() {
    let game = seismic_game(30, 30);

    assert_eq!(game.weather, Weather::Earthquake);
    assert_eq!(game.seismic_quakes, 0);
    assert!(
        !game.seismic_rubble.is_empty(),
        "Seismic mode should start with initial rubble"
    );
    for rubble in &game.seismic_rubble {
        assert!(
            game.obstacles.contains(rubble),
            "Rubble tiles must also be obstacles so they block movement"
        );
        assert!(!game.snake.body_map.contains_key(rubble));
    }
    assert!(!game.seismic_rubble.contains(&game.food));
}

#[test]
fn test_seismic_weather_never_breaks() {
    let mut game = seismic_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..60 {
        game.update();
        if game.state != snake_game::game::GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = snake_game::game::GameState::Playing;
        }
        assert_eq!(
            game.weather,
            Weather::Earthquake,
            "Seismic mode must force earthquake weather on every tick"
        );
    }
}

#[test]
fn test_seismic_scheduled_tremor_every_6_ticks() {
    let mut game = seismic_game(30, 30);
    game.obstacles.clear();
    game.seismic_rubble.clear();
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must trigger a scheduled tremor.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = snake_game::game::GameState::Playing;
        game.tick_counter = tick;
        let quakes_before = game.seismic_quakes;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.seismic_quakes > quakes_before,
            "a scheduled tremor must fire on tick {} (quakes before: {}, after: {})",
            tick + 1,
            quakes_before,
            game.seismic_quakes
        );
    }
    assert!(game.seismic_quakes >= 6);
}

#[test]
fn test_seismic_tremor_never_shakes_snake_row() {
    let mut game = seismic_game(20, 20);
    game.obstacles.clear();
    game.seismic_rubble.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = true;

    let mut saw_fresh_rubble = false;
    for _ in 0..60 {
        // The tremor runs after the autopilot picks this tick's direction but before
        // the snake moves, so the rows it must avoid are exactly the pre-update head
        // row and the post-update head row (the cell the snake moved into this tick)
        let head_before = game.snake.head();
        let quakes_before = game.seismic_quakes;
        let rubble_before: std::collections::HashSet<Point> = game.seismic_rubble.clone();
        game.update();
        if game.state != snake_game::game::GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = snake_game::game::GameState::Playing;
        }
        if game.seismic_quakes > quakes_before {
            let fresh: Vec<Point> = game
                .seismic_rubble
                .iter()
                .copied()
                .filter(|p| !rubble_before.contains(p))
                .collect();
            // A tremor may spawn nothing once the 10%-of-board rubble cap is reached
            if !fresh.is_empty() {
                saw_fresh_rubble = true;
                let head_after = game.snake.head();
                for rubble in &fresh {
                    assert_ne!(
                        rubble.y, head_before.y,
                        "a tremor must never spawn rubble on the row the snake occupied"
                    );
                    assert_ne!(
                        rubble.y, head_after.y,
                        "a tremor must never spawn rubble on the row the snake is about to enter"
                    );
                }
            }
        }
    }
    assert!(
        saw_fresh_rubble,
        "expected at least one tremor spawning fresh rubble over 60 ticks"
    );

    assert!(
        game.seismic_quakes >= 10,
        "expected at least 10 tremors over 60 ticks, got {}",
        game.seismic_quakes
    );
}

#[test]
fn test_seismic_rubble_is_lethal() {
    let mut game = seismic_game(20, 20);
    game.obstacles.clear();
    game.seismic_rubble.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Up;
    game.snake.direction_queue.push_back(Direction::Up);
    // Druid ignores ambient weather effects, so the ambient quake cannot remove the rubble first
    game.stats.equipped_class = Some(HeroClass::Druid);

    let rubble_pos = Point { x: 5, y: 4 };
    game.seismic_rubble.insert(rubble_pos);
    game.obstacles.insert(rubble_pos);

    game.update();

    assert!(
        game.just_died || game.lives < 3,
        "Snake should die when moving into a rubble tile"
    );
}

#[test]
fn test_seismic_rubble_is_hard_obstacle_for_autopilot() {
    let mut game = seismic_game(20, 20);
    game.obstacles.clear();
    game.seismic_rubble.clear();
    let rubble_pos = Point { x: 6, y: 5 };
    game.seismic_rubble.insert(rubble_pos);
    game.obstacles.insert(rubble_pos);

    assert!(
        !game.is_safe_final_p(rubble_pos, 1, 1),
        "is_safe_final_p must reject a rubble cell"
    );
    assert!(
        game.is_safe_final_p(Point { x: 7, y: 5 }, 1, 1),
        "a different cell must remain safe"
    );
}

#[test]
fn test_seismic_periodic_settling_clears_rubble() {
    let mut game = seismic_game(20, 20);
    game.obstacles.clear();
    game.seismic_rubble.clear();

    let rubble_a = Point { x: 2, y: 2 };
    let rubble_b = Point { x: 3, y: 3 };
    for p in [rubble_a, rubble_b] {
        game.seismic_rubble.insert(p);
        game.obstacles.insert(p);
    }

    // Next update brings tick_counter to 200, which triggers the settling
    game.tick_counter = 199;
    game.update();

    assert!(
        game.seismic_rubble.is_empty(),
        "Settling at tick 200 should clear all rubble"
    );
    assert!(!game.obstacles.contains(&rubble_a));
    assert!(!game.obstacles.contains(&rubble_b));
}

#[test]
fn test_seismic_eating_food_braces_adjacent_rubble() {
    let mut game = seismic_game(20, 20);
    game.obstacles.clear();
    game.seismic_rubble.clear();

    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    // Druid ignores ambient weather effects, so only the food-brace clears rubble
    game.stats.equipped_class = Some(HeroClass::Druid);

    let adjacent_rubble = Point { x: 6, y: 6 };
    game.seismic_rubble.insert(adjacent_rubble);
    game.obstacles.insert(adjacent_rubble);
    let far_rubble = Point { x: 15, y: 15 };
    game.seismic_rubble.insert(far_rubble);
    game.obstacles.insert(far_rubble);

    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.seismic_rubble.contains(&adjacent_rubble),
        "Eating food should brace (clear) adjacent rubble"
    );
    assert!(!game.obstacles.contains(&adjacent_rubble));
    assert!(
        game.seismic_rubble.contains(&far_rubble),
        "Distant rubble should be unaffected"
    );
}

#[test]
fn test_seismic_tremor_crumbles_obstacles_in_row() {
    let mut game = seismic_game(20, 20);
    game.obstacles.clear();
    game.seismic_rubble.clear();
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 2, y: 2 };
    // Druid ignores ambient weather effects, so only the scheduled tremor moves obstacles
    game.stats.equipped_class = Some(HeroClass::Druid);

    // Pre-place a probe obstacle on every row except the snake's row
    for y in 1..game.height - 1 {
        if y != game.snake.head().y {
            game.obstacles.insert(Point { x: 3, y });
        }
    }
    let rubble_before: std::collections::HashSet<Point> = game.seismic_rubble.clone();

    // Next update runs tick 6, which triggers a scheduled tremor
    game.tick_counter = 5;
    game.update();

    assert_eq!(game.seismic_quakes, 1, "a scheduled tremor should have fired on tick 6");
    let fresh: Vec<Point> = game
        .seismic_rubble
        .iter()
        .copied()
        .filter(|p| !rubble_before.contains(p))
        .collect();
    assert!(
        !fresh.is_empty(),
        "a tremor should spawn fresh rubble on the shaken row"
    );
    let quake_y = fresh[0].y;
    for p in &fresh {
        assert_eq!(p.y, quake_y, "all fresh rubble must lie in the shaken row");
        assert_ne!(quake_y, game.snake.head().y, "the snake's row is never shaken");
    }
    let probe = Point { x: 3, y: quake_y };
    assert!(
        !game.obstacles.contains(&probe) || game.seismic_rubble.contains(&probe),
        "the probe obstacle in the shaken row must crumble (or be replaced by fresh rubble)"
    );
    let other_y = if quake_y == 1 { 2 } else { 1 };
    let other_probe = Point { x: 3, y: other_y };
    assert!(
        game.obstacles.contains(&other_probe) && !game.seismic_rubble.contains(&other_probe),
        "obstacles in unshaken rows must survive the tremor untouched"
    );
}

#[test]
fn test_seismic_bot_survives_the_quake() {
    let mut game = seismic_game(40, 30);
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
        "Autopilot should survive the earthquake (lives: {}, quakes: {}, score: {})",
        game.lives,
        game.seismic_quakes,
        game.score
    );
    assert!(
        game.seismic_quakes >= 20,
        "expected at least 20 tremors over 120 ticks, got {}",
        game.seismic_quakes
    );
}
