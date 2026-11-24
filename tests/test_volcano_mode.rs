use snake_game::game::{Difficulty, Game, GameMode, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn volcano_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Volcano;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_volcano_reset_spawns_vents() {
    let game = volcano_game(30, 30);

    assert_eq!(game.volcano_eruptions, 0);
    assert_eq!(game.volcano_vents.len(), 2, "Volcano mode starts with 2 vents");
    for vent in &game.volcano_vents {
        assert!(
            game.lava_tiles.contains(vent),
            "Vent cells must be lava"
        );
        assert!(
            game.obstacles.contains(vent),
            "Vent cells must also be obstacles so they block movement"
        );
        assert!(!game.snake.body_map.contains_key(vent));
    }
    assert!(!game.lava_tiles.contains(&game.food));
}

#[test]
fn test_volcano_eruption_every_6_ticks() {
    let mut game = volcano_game(40, 40);
    game.obstacles.clear();
    game.lava_tiles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.food = Point { x: 35, y: 35 };
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must erupt.
    for tick in [5_u32, 11, 17, 23] {
        game.state = snake_game::game::GameState::Playing;
        game.tick_counter = tick;
        let eruptions_before = game.volcano_eruptions;
        let lava_before = game.lava_tiles.len();
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.volcano_eruptions > eruptions_before,
            "a scheduled eruption must fire on tick {}",
            tick + 1
        );
        assert!(
            game.lava_tiles.len() > lava_before,
            "an eruption must spread lava onto new cells (before: {}, after: {})",
            lava_before,
            game.lava_tiles.len()
        );
        for lava in &game.lava_tiles {
            assert!(game.obstacles.contains(lava));
            assert!(!game.snake.body_map.contains_key(lava));
        }
    }
    assert_eq!(game.volcano_eruptions, 4);
}

#[test]
fn test_volcano_lava_is_lethal() {
    let mut game = volcano_game(20, 20);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Up;
    game.snake.direction_queue.push_back(Direction::Up);

    let lava_pos = Point { x: 5, y: 4 };
    game.lava_tiles.insert(lava_pos);
    game.obstacles.insert(lava_pos);

    game.update();

    assert!(
        game.just_died || game.lives < 3,
        "Snake should die when moving into a lava tile"
    );
}

#[test]
fn test_volcano_is_safe_final_p_rejects_lava() {
    let mut game = volcano_game(20, 20);
    game.obstacles.clear();
    let lava_pos = Point { x: 5, y: 5 };
    game.lava_tiles.insert(lava_pos);
    game.obstacles.insert(lava_pos);

    assert!(
        !game.is_safe_final_p(lava_pos, 1, 1),
        "is_safe_final_p must reject a lava cell (the autopilot must route around lava)"
    );
    assert!(
        game.is_safe_final_p(Point { x: 6, y: 5 }, 1, 1),
        "a non-lava cell must remain safe"
    );
}

#[test]
fn test_volcano_periodic_cooling_clears_lava_but_keeps_vents() {
    let mut game = volcano_game(30, 30);
    game.obstacles.clear();
    game.lava_tiles.clear();
    let vent = Point { x: 2, y: 2 };
    game.volcano_vents = vec![vent];
    game.lava_tiles.insert(vent);
    game.obstacles.insert(vent);

    let lava_a = Point { x: 10, y: 10 };
    let lava_b = Point { x: 11, y: 10 };
    for p in [lava_a, lava_b] {
        game.lava_tiles.insert(p);
        game.obstacles.insert(p);
    }

    // Next update brings tick_counter to 200, which triggers the cooling
    game.tick_counter = 199;
    game.update();

    assert!(
        !game.lava_tiles.contains(&lava_a) && !game.lava_tiles.contains(&lava_b),
        "Cooling at tick 200 should clear all spread lava"
    );
    assert!(!game.obstacles.contains(&lava_a));
    assert!(!game.obstacles.contains(&lava_b));
    assert!(
        game.lava_tiles.contains(&vent),
        "Vents must survive the cooling"
    );
    assert!(game.obstacles.contains(&vent));
}

#[test]
fn test_volcano_eating_food_cools_adjacent_lava() {
    let mut game = volcano_game(20, 20);
    game.obstacles.clear();
    game.lava_tiles.clear();
    game.volcano_vents.clear();

    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };

    let adjacent_lava = Point { x: 6, y: 6 };
    game.lava_tiles.insert(adjacent_lava);
    game.obstacles.insert(adjacent_lava);
    let far_lava = Point { x: 15, y: 15 };
    game.lava_tiles.insert(far_lava);
    game.obstacles.insert(far_lava);

    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.lava_tiles.contains(&adjacent_lava),
        "Eating food should cool adjacent lava"
    );
    assert!(!game.obstacles.contains(&adjacent_lava));
    assert!(
        game.lava_tiles.contains(&far_lava),
        "Distant lava should be unaffected"
    );
}

#[test]
fn test_volcano_lava_respects_cap_and_never_covers_snakes() {
    let mut game = volcano_game(40, 30);
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
            u32::try_from(game.lava_tiles.len()).unwrap_or(u32::MAX) <= cap,
            "Lava must stay under the 15% board cap (len: {}, cap: {})",
            game.lava_tiles.len(),
            cap
        );
        for lava in &game.lava_tiles {
            assert!(
                game.obstacles.contains(lava),
                "Every lava tile must be mirrored into obstacles"
            );
        }
        assert!(
            !game.lava_tiles.contains(&game.snake.head()),
            "Lava must never spawn on the snake's head"
        );
    }
}

#[test]
fn test_volcano_bot_survives_the_eruptions() {
    let mut game = volcano_game(40, 30);
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
        "Autopilot should survive the volcano (lives: {}, eruptions: {}, lava: {}, score: {})",
        game.lives,
        game.volcano_eruptions,
        game.lava_tiles.len(),
        game.score
    );
    assert!(
        game.volcano_eruptions >= 20,
        "expected at least 20 eruptions over 120 ticks, got {}",
        game.volcano_eruptions
    );
}

#[test]
fn test_volcano_state_persists_through_time_rewind() {
    let mut game = volcano_game(30, 30);
    game.obstacles.clear();
    game.lava_tiles.clear();
    let vent = Point { x: 2, y: 2 };
    game.volcano_vents = vec![vent];
    game.lava_tiles.insert(vent);
    game.obstacles.insert(vent);
    let lava = Point { x: 12, y: 12 };
    game.lava_tiles.insert(lava);
    game.obstacles.insert(lava);
    game.volcano_eruptions = 7;

    game.save_history_state();

    // Simulate later changes: lava cooled away, counter moved on
    game.lava_tiles.remove(&lava);
    game.obstacles.remove(&lava);
    game.volcano_eruptions = 9;

    game.rewind_time();

    assert_eq!(game.volcano_eruptions, 7, "Rewind must restore the eruption counter");
    assert!(
        game.lava_tiles.contains(&lava),
        "Rewind must restore lava tiles"
    );
    assert!(
        game.volcano_vents.contains(&vent),
        "Rewind must restore volcano vents"
    );
}
