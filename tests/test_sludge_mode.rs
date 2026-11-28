use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn sludge_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Sludge;
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

// Block the ambient portal/mine spawners so single-tick sink tests are
// deterministic: an existing portal pair stops manage_portals from spawning,
// and a full mine rack (len >= 5) stops manage_mines from dropping new mines.
fn pin_ambient_spawners(game: &mut Game) {
    game.portals = Some((Point { x: 27, y: 27 }, Point { x: 28, y: 28 }));
    game.mines.insert(Point { x: 23, y: 1 });
    game.mines.insert(Point { x: 24, y: 1 });
    game.mines.insert(Point { x: 25, y: 1 });
    game.mines.insert(Point { x: 26, y: 1 });
    game.mines.insert(Point { x: 27, y: 1 });
}

#[test]
fn test_sludge_reset_seeds_fairly() {
    let game = sludge_game(30, 30);

    assert_eq!(game.sludge_spills, 0);
    assert_eq!(game.sludge_settles, 0);
    // seed count = max(2, 30*30/100) = 9
    assert_eq!(game.sludge_tiles.len(), 9);
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    for qs in &game.sludge_tiles {
        assert!(
            game.obstacles.contains(qs),
            "sludge must be mirrored into obstacles: {qs:?}"
        );
        assert!(
            !game.snake.body_map.contains_key(qs),
            "sludge must never sit on the snake: {qs:?}"
        );
        assert_ne!(*qs, game.food, "sludge must never sit on the food");
        assert!(
            qs.x.abs_diff(spawn.x).saturating_add(qs.y.abs_diff(spawn.y)) >= 5,
            "sludge must spawn at least 5 cells from the spawn point: {qs:?}"
        );
    }
}

#[test]
fn test_sludge_spills_every_6_ticks_and_stays_mirrored() {
    let mut game = sludge_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.auto_pilot = true;
    pin_ambient_spawners(&mut game);

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must trigger a scheduled sink.
    for tick in [5_u32, 11, 17, 23] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let sinks_before = game.sludge_spills;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert_eq!(
            game.sludge_spills,
            sinks_before + 1,
            "a scheduled sink must fire on tick {}",
            tick + 1
        );
        for qs in &game.sludge_tiles {
            assert!(
                game.obstacles.contains(qs),
                "sludge must stay mirrored into obstacles: {qs:?}"
            );
        }
    }
    assert_eq!(game.sludge_spills, 4);
    assert!(
        game.sludge_tiles.len() > 9,
        "the scheduled sinks must grow the field, got {}",
        game.sludge_tiles.len()
    );
}

#[test]
fn test_sludge_never_spills_snake_next_cell_or_food() {
    let mut game = sludge_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.sludge_tiles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 2, y: 2 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = false;

    game.tick_counter = 5;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.sludge_spills, 1);
    let next = Point { x: 3, y: 2 };
    assert!(
        !game.sludge_tiles.contains(&next),
        "a sink must never swallow the cell the snake is about to enter"
    );
    for seg in &game.snake.body {
        assert!(
            !game.sludge_tiles.contains(seg),
            "a sink must never swallow the snake's body"
        );
    }
    assert!(
        !game.sludge_tiles.contains(&game.food),
        "a sink must never swallow the food"
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the sink must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_sludge_fairness_over_many_ticks() {
    let mut game = sludge_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 25, y: 25 };
    pin_ambient_spawners(&mut game);
    let lives_before = game.lives;

    // Park the snake at (8, 15) heading right before every scheduled sink, so
    // the sink must never land on its next cell (9, 15) or its body.
    let next = Point { x: 9, y: 15 };
    for k in 1..=10_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = 6 * k - 1;
        let sinks_before = game.sludge_spills;
        game.update();
        if game.sludge_spills > sinks_before {
            assert!(
                !game.sludge_tiles.contains(&next),
                "sink {} swallowed the cell the snake was about to enter ({next:?})",
                game.sludge_spills
            );
            for seg in &game.snake.body {
                assert!(
                    !game.sludge_tiles.contains(seg),
                    "sink {} swallowed the snake's body at {seg:?}",
                    game.sludge_spills
                );
            }
        }
        assert!(
            !game.sludge_tiles.contains(&game.snake.head()),
            "tick {}: the snake's head must never end up in sludge (head: {:?})",
            6 * k,
            game.snake.head()
        );
    }

    assert_eq!(
        game.lives, lives_before,
        "the snake must survive the sinking field (sinks: {})",
        game.sludge_spills
    );
    assert!(
        game.sludge_spills >= 7,
        "expected many sinks over 10 scheduled ticks, got {}",
        game.sludge_spills
    );
}

#[test]
fn test_sludge_settles_over_at_tick_200() {
    let mut game = sludge_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.auto_pilot = true;
    pin_ambient_spawners(&mut game);

    // Trigger a few scheduled sinks first.
    for tick in [5_u32, 11, 17] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
    }
    assert!(game.sludge_spills >= 3);
    assert!(!game.sludge_tiles.is_empty());
    let doomed: Vec<Point> = game.sludge_tiles.iter().copied().collect();

    // Tick 200 of the 250-tick cycle dries the whole field up.
    game.state = GameState::Playing;
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert_eq!(
        game.sludge_tiles.len(),
        0,
        "all sludge must dry up at tick 200"
    );
    assert_eq!(game.sludge_settles, 1);
    for p in &doomed {
        assert!(
            !game.obstacles.contains(p),
            "a dried-up sludge cell must leave obstacles too: {p:?}"
        );
    }
}

#[test]
fn test_sludge_is_lethal_on_entry() {
    let mut game = sludge_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(13);
    game.obstacles.clear();
    game.sludge_tiles.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    // A sludge cell right on the snake's path (mirrored into obstacles)
    let qs = Point { x: 11, y: 10 };
    game.sludge_tiles.insert(qs);
    game.obstacles.insert(qs);
    let lives_before = game.lives;

    game.tick_counter = 0; // tick 1: no scheduled sink
    game.state = GameState::Playing;
    game.update();

    assert!(
        game.lives < lives_before || game.state != GameState::Playing,
        "entering sludge must be lethal (lives: {} -> {})",
        lives_before,
        game.lives
    );
}

#[test]
fn test_sludge_eating_food_shakes_adjacent_loose() {
    let mut game = sludge_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.sludge_tiles.clear();
    pin_ambient_spawners(&mut game);
    let qs1 = Point { x: 6, y: 4 };
    let qs2 = Point { x: 7, y: 5 };
    game.sludge_tiles.insert(qs1);
    game.sludge_tiles.insert(qs2);
    game.obstacles.insert(qs1);
    game.obstacles.insert(qs2);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.auto_pilot = false;

    // Tick 5 is not a multiple of 6 and not 200, so no sink/dry-up interferes.
    game.tick_counter = 4;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.sludge_tiles.contains(&qs1),
        "eating food in Sludge mode must shake the adjacent sludge loose"
    );
    assert!(
        !game.sludge_tiles.contains(&qs2),
        "eating food in Sludge mode must shake the adjacent sludge loose"
    );
    assert!(
        !game.obstacles.contains(&qs1) && !game.obstacles.contains(&qs2),
        "the shaken-loose sludge must also leave obstacles"
    );
}

#[test]
fn test_sludge_bot_survives_the_sludge_field() {
    let mut game = sludge_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;

    for _ in 0..200 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
    }

    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "Autopilot should survive Sludge mode (lives: {}, sinks: {}, dries: {}, score: {})",
        game.lives,
        game.sludge_spills,
        game.sludge_settles,
        game.score
    );
    assert!(
        game.sludge_spills >= 20,
        "expected many sinks over 200 ticks, got {}",
        game.sludge_spills
    );
}

#[test]
fn test_sludge_persists_through_history_rewind() {
    let mut game = sludge_game(30, 30);
    game.sludge_spills = 7;
    game.sludge_settles = 2;
    game.sludge_tiles.insert(Point { x: 12, y: 12 });
    game.save_history_state();
    game.sludge_spills = 0;
    game.sludge_settles = 0;
    game.sludge_tiles.clear();

    game.rewind_time();

    assert_eq!(
        game.sludge_spills, 7,
        "sludge_spills must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.sludge_settles, 2);
    assert!(
        game.sludge_tiles.contains(&Point { x: 12, y: 12 }),
        "sludge_tiles must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_sludge_persists_in_save_file() {
    let mut game = sludge_game(30, 30);
    game.sludge_spills = 9;
    game.sludge_settles = 1;
    let path = std::env::temp_dir().join(format!(
        "sludge_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"sludge_spills\":9")
            && json.contains("\"sludge_settles\":1")
            && json.contains("\"sludge_tiles\""),
        "SaveState JSON must contain the sludge fields, got: {}",
        &json[..json.len().min(200)]
    );
}
