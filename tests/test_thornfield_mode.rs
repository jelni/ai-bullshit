use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn thornfield_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Thornfield;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_thornfield_reset_seeds_thorns_and_clears_blooms() {
    let game = thornfield_game(30, 30);

    assert_eq!(game.thorn_blooms, 0);
    assert_eq!(game.weather, Weather::Clear);
    // max(30*30/100, 2) = 9 initial briars (best effort, but a 30x30 board always fits them)
    assert!(
        !game.thorn_tiles.is_empty() && game.thorn_tiles.len() <= 9,
        "expected up to 9 initial briars, got {}",
        game.thorn_tiles.len()
    );
    for thorn in &game.thorn_tiles {
        assert!(
            game.obstacles.contains(thorn),
            "thorns must be mirrored into obstacles"
        );
    }
}

#[test]
fn test_thornfield_blooms_every_6_ticks() {
    let mut game = thornfield_game(30, 30);
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must spread the briars.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let blooms_before = game.thorn_blooms;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.thorn_blooms > blooms_before,
            "a bloom must fire on tick {} (blooms before: {}, after: {})",
            tick + 1,
            blooms_before,
            game.thorn_blooms
        );
        for thorn in &game.thorn_tiles {
            assert!(
                game.obstacles.contains(thorn),
                "thorns must stay mirrored into obstacles"
            );
        }
    }
    assert_eq!(game.thorn_blooms, 6);
}

#[test]
fn test_thornfield_never_blooms_into_a_snake_lash_zone() {
    let mut game = thornfield_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.thorn_tiles.clear();
    // Seed one briar far from the snake so the frontier can spread.
    let seed = Point { x: 15, y: 15 };
    game.thorn_tiles.insert(seed);
    game.obstacles.insert(seed);
    game.snake = Snake::new(Point { x: 2, y: 2 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 10, y: 10 };
    game.auto_pilot = false;

    // Tick 6 blooms from the frontier; the snake sits at (2, 2) heading right,
    // so its lash zone is (2,2),(3,2),(1,2),(2,1),(2,3) and must stay thorn-free.
    game.tick_counter = 5;
    game.update();

    assert_eq!(game.thorn_blooms, 1);
    let next = Point { x: 3, y: 2 };
    for p in &game.thorn_tiles {
        let dist = p.x.abs_diff(2) + p.y.abs_diff(2);
        assert!(
            dist > 1,
            "a thorn bloomed into the snake's lash zone at {p:?} (dist {dist})"
        );
        assert_ne!(*p, next, "a thorn bloomed on the cell the snake is about to enter");
    }
    for seg in &game.snake.body {
        for p in &game.thorn_tiles {
            let dist = p.x.abs_diff(seg.x) + p.y.abs_diff(seg.y);
            assert!(dist > 1, "a thorn bloomed adjacent to the snake body at {seg:?}");
        }
    }
}

#[test]
fn test_thornfield_fairness_over_many_ticks() {
    let mut game = thornfield_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 25, y: 25 };
    // Clear the random initial briars and seed one controlled frontier far away,
    // so the parked snake is never next to a pre-existing thorn.
    game.obstacles.clear();
    game.thorn_tiles.clear();
    let seed = Point { x: 25, y: 5 };
    game.thorn_tiles.insert(seed);
    game.obstacles.insert(seed);
    let lives_before = game.lives;

    // Park the snake at (8, 15) heading right before every scheduled bloom, so
    // the bloom must never thorn its lash zone or the cell it is about to enter.
    let next = Point { x: 9, y: 15 };
    for k in 1..=10_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = 6 * k - 1;
        let blooms_before = game.thorn_blooms;
        game.update();
        if game.thorn_blooms > blooms_before {
            for p in &game.thorn_tiles {
                let dist_head = p.x.abs_diff(8) + p.y.abs_diff(15);
                assert!(
                    dist_head > 1,
                    "bloom {} thorned the snake's lash zone at {p:?} (dist {dist_head})",
                    k
                );
                assert_ne!(*p, next, "bloom {k} thorned the cell the snake was about to enter");
            }
        }
    }

    assert_eq!(
        game.lives, lives_before,
        "the snake must survive the blooming briars (blooms: {})",
        game.thorn_blooms
    );
    assert!(
        game.thorn_blooms >= 7,
        "expected several blooms over the ticks, got {}",
        game.thorn_blooms
    );
}

#[test]
fn test_thornfield_withers_at_tick_200() {
    let mut game = thornfield_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.auto_pilot = true;

    // Bloom a few times first.
    for tick in [5_u32, 11, 17] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
    }
    assert!(!game.thorn_tiles.is_empty());

    // Tick 200 of the 250-tick cycle withers all thorns.
    game.state = GameState::Playing;
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.thorn_tiles.is_empty(),
        "all thorns must wither at tick 200"
    );
    for obs in &game.obstacles {
        assert!(
            !game.thorn_tiles.contains(obs),
            "withered thorns must leave obstacles"
        );
    }
    assert!(
        game.thorn_blooms >= 3,
        "the bloom counter keeps counting through a wither"
    );
}

#[test]
fn test_thornfield_spread_is_capped() {
    let mut game = thornfield_game(12, 12);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(9);
    game.auto_pilot = true;
    // cap = 12*12/12 = 12
    for tick in (5..180).step_by(6) {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
        if game.lives == 0 {
            break;
        }
        assert!(
            game.thorn_tiles.len() <= 12,
            "thorn count must be capped at 12 on a 12x12 board, got {}",
            game.thorn_tiles.len()
        );
    }
}

#[test]
fn test_thorn_lashes_kills_adjacent_head() {
    let mut game = thornfield_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.thorn_tiles.clear();
    // One briar at (6, 6): the cell (6, 5) is in its lash zone.
    let thorn = Point { x: 6, y: 6 };
    game.thorn_tiles.insert(thorn);
    game.obstacles.insert(thorn);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = false;
    let lives_before = game.lives;

    assert!(game.thorn_lashes(Point { x: 6, y: 5 }));
    assert!(game.thorn_lashes(thorn));
    assert!(!game.thorn_lashes(Point { x: 6, y: 4 }));
    assert!(!game.is_safe_final_p(Point { x: 6, y: 5 }, 1, 1));
    assert!(game.is_safe_final_p(Point { x: 6, y: 4 }, 1, 1));

    // Tick 4 is not a multiple of 6 and not 200, so no bloom interferes here.
    game.tick_counter = 3;
    game.update();

    assert_eq!(
        game.lives,
        lives_before - 1,
        "moving into a thorn's lash zone must cost a life"
    );
}

#[test]
fn test_thornfield_eating_food_prunes_adjacent_thorns() {
    let mut game = thornfield_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.thorn_tiles.clear();
    let thorn = Point { x: 7, y: 5 };
    game.thorn_tiles.insert(thorn);
    game.obstacles.insert(thorn);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.auto_pilot = false;

    // Tick 4 is not a multiple of 6 and not 200, so no bloom interferes here.
    game.tick_counter = 3;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.thorn_tiles.contains(&thorn),
        "Eating food in Thornfield mode should prune the adjacent thorn"
    );
    assert!(
        !game.obstacles.contains(&thorn),
        "The pruned thorn must also leave obstacles"
    );
}

#[test]
fn test_thornfield_bot_survives_the_briars() {
    let mut game = thornfield_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.update();
        if game.state != GameState::Playing {
            if game.lives == 0 {
                break;
            }
            // Non-death interruption (e.g. merchant visit): resume playing
            game.state = GameState::Playing;
        }
    }

    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "Autopilot should survive the thornfield (lives: {}, blooms: {}, score: {})",
        game.lives,
        game.thorn_blooms,
        game.score
    );
    assert!(
        game.thorn_blooms >= 5,
        "expected several blooms over 120 ticks, got {}",
        game.thorn_blooms
    );
    assert!(
        game.thorn_tiles.len() <= 40 * 40 / 12,
        "thorn count must stay within the cap, got {}",
        game.thorn_tiles.len()
    );
}

#[test]
fn test_thornfield_persists_through_history_rewind() {
    let mut game = thornfield_game(30, 30);
    game.thorn_blooms = 7;
    let thorn = Point { x: 11, y: 13 };
    game.thorn_tiles.insert(thorn);
    game.obstacles.insert(thorn);
    game.save_history_state();
    game.thorn_blooms = 0;
    game.thorn_tiles.clear();

    game.rewind_time();

    assert_eq!(
        game.thorn_blooms, 7,
        "thorn_blooms must survive a time-rewind through HistoryState"
    );
    assert!(
        game.thorn_tiles.contains(&thorn),
        "thorn_tiles must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_thornfield_persists_in_save_file() {
    let mut game = thornfield_game(30, 30);
    game.thorn_blooms = 9;
    let path = std::env::temp_dir().join(format!(
        "thornfield_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"thorn_blooms\":9") && json.contains("\"thorn_tiles\""),
        "SaveState JSON must contain the thorn state, got: {}",
        &json[..json.len().min(200)]
    );
}
