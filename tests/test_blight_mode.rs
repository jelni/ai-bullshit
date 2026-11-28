use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn blight_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Blight;
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

// Block the ambient portal/mine spawners so single-tick spread tests are
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

fn park_snake(game: &mut Game, head: Point, dir: Direction) {
    game.snake = Snake::new(head);
    game.snake.direction = dir;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(dir);
}

#[test]
fn test_blight_reset_seeds_patches_fairly() {
    let game = blight_game(30, 30);

    assert_eq!(game.blight_spreads, 0);
    assert_eq!(game.blight_clears, 0);
    assert!(
        !game.blight_tiles.is_empty(),
        "reset must seed at least one blight patch"
    );
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    let cap = (30usize * 30) / Game::BLIGHT_MAX_DIVISOR;
    assert!(
        game.blight_tiles.len() <= cap,
        "seeded blight must respect the board cap ({cap})"
    );
    for qs in &game.blight_tiles {
        assert!(
            game.obstacles.contains(qs),
            "every blight cell must be mirrored into obstacles: {qs:?}"
        );
        assert!(
            !game.snake.body_map.contains_key(qs),
            "blight must never sit on the snake: {qs:?}"
        );
        assert_ne!(*qs, game.food, "blight must never sit on the food");
        assert!(
            qs.x.abs_diff(spawn.x).saturating_add(qs.y.abs_diff(spawn.y)) >= 5,
            "blight must spawn at least 5 cells from the spawn point: {qs:?}"
        );
    }
}

#[test]
fn test_blight_forces_clear_weather() {
    let mut game = blight_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.weather = Weather::Storm;
    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();
    // A tick where the snake dies short-circuits update_tick before the mode
    // managers run, so run another tick with the game kept alive.
    keep_playing(&mut game);
    game.update();
    keep_playing(&mut game);
    assert_eq!(
        game.weather,
        Weather::Clear,
        "Blight mode must pin clear skies so the blight stays visible"
    );
}

#[test]
fn test_blight_spread_every_6_ticks_stays_fair() {
    let mut game = blight_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = false;
    pin_ambient_spawners(&mut game);
    let cap = (40usize * 40) / Game::BLIGHT_MAX_DIVISOR;

    // update() increments tick_counter first, so setting tick_counter = n - 1
    // makes the next update run tick n. Spread fires on every 6th tick.
    for tick in [5_u32, 11, 17, 23, 29, 35, 41, 47] {
        game.state = GameState::Playing;
        park_snake(&mut game, Point { x: 20, y: 20 }, Direction::Right);
        game.food = Point { x: 35, y: 35 };
        game.tick_counter = tick;
        let pre_head = game.snake.head();
        let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
        let pre_next = Game::calculate_next_head_dir(pre_head, game.snake.direction);
        let pre_food = game.food;
        let pre_tiles: std::collections::HashSet<Point> = game.blight_tiles.clone();
        let spreads_before = game.blight_spreads;

        game.update();

        assert_eq!(game.tick_counter, tick + 1);
        assert!(
            game.blight_spreads > spreads_before,
            "a blight spread must happen on tick {}",
            tick + 1
        );
        assert!(game.blight_tiles.len() <= cap, "cap must hold after spread");
        for qs in &game.blight_tiles {
            assert!(
                game.obstacles.contains(qs),
                "spread blight must be mirrored into obstacles: {qs:?}"
            );
            if pre_tiles.contains(qs) {
                continue; // only newly added cells need the fairness checks
            }
            assert!(
                !pre_body.contains(qs),
                "new blight must not land on the snake body: {qs:?}"
            );
            assert_ne!(
                *qs, pre_next,
                "new blight must not land on the cell the snake is about to enter: {qs:?}"
            );
            assert_ne!(*qs, pre_food, "new blight must not land on the food");
            assert!(
                !game.snake.body_map.contains_key(qs) || pre_body.contains(qs),
                "new blight must not land on the current snake: {qs:?}"
            );
        }
    }
}

#[test]
fn test_blight_never_spreads_onto_next_head_over_many_ticks() {
    let mut game = blight_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 35, y: 35 };
    pin_ambient_spawners(&mut game);

    for tick in 0..120_u32 {
        game.state = GameState::Playing;
        park_snake(&mut game, Point { x: 8, y: 15 }, Direction::Right);
        game.food = Point { x: 35, y: 35 };
        game.tick_counter = tick;
        let pre_head = game.snake.head();
        let pre_next = Game::calculate_next_head_dir(pre_head, game.snake.direction);
        let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
        game.update();
        if game.lives == 0 {
            game.lives = 3;
            game.state = GameState::Playing;
            continue;
        }
        for qs in &game.blight_tiles {
            assert_ne!(
                *qs, pre_next,
                "tick {tick}: blight landed on the about-to-enter cell {pre_next:?}"
            );
            assert!(
                !pre_body.contains(qs),
                "tick {tick}: blight landed on the snake body at {qs:?}"
            );
        }
    }
}

#[test]
fn test_blight_dryup_at_tick_200_clears_tiles_and_obstacles() {
    let mut game = blight_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.obstacles.clear();
    game.blight_tiles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    // Seed a couple of blight cells manually
    game.blight_tiles.insert(Point { x: 15, y: 15 });
    game.obstacles.insert(Point { x: 15, y: 15 });
    game.blight_tiles.insert(Point { x: 16, y: 15 });
    game.obstacles.insert(Point { x: 16, y: 15 });
    assert_eq!(game.blight_tiles.len(), 2);

    // tick 199 -> update runs tick 200, which withers all blight
    game.tick_counter = 199;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.blight_tiles.is_empty(),
        "all blight must dry up at tick 200, got {:?}",
        game.blight_tiles
    );
    assert!(
        !game.obstacles.contains(&Point { x: 15, y: 15 })
            && !game.obstacles.contains(&Point { x: 16, y: 15 }),
        "dried-up blight must be removed from obstacles too"
    );
}

#[test]
fn test_blight_cap_is_never_exceeded() {
    let mut game = blight_game(10, 10);
    game.rng = rand::rngs::StdRng::seed_from_u64(1);
    game.obstacles.clear();
    game.blight_tiles.clear();
    game.auto_pilot = false;
    game.lives = 99;
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 5, y: 5 }, Direction::Right);
    game.food = Point { x: 8, y: 8 };
    let cap = (10usize * 10) / Game::BLIGHT_MAX_DIVISOR; // 20
    // Fill the board up to the cap with blight (mirrored into obstacles),
    // avoiding the snake head.
    let mut placed = 0usize;
    'outer: for y in 1..9u16 {
        for x in 1..9u16 {
            let p = Point { x, y };
            if p == (Point { x: 5, y: 5 }) {
                continue;
            }
            if game.obstacles.insert(p) {
                game.blight_tiles.insert(p);
                placed += 1;
                if placed >= cap {
                    break 'outer;
                }
            }
        }
    }
    assert_eq!(game.blight_tiles.len(), cap);

    // Tick 6 is a spread tick, but the cap guard must prevent any growth.
    game.tick_counter = 5;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 6);
    assert_eq!(
        game.blight_tiles.len(),
        cap,
        "spread must not exceed the cap ({cap})"
    );
    assert_eq!(
        game.blight_spreads, 0,
        "no spread may be counted when already at the cap"
    );
}

#[test]
fn test_blight_eating_food_clears_adjacent_tiles() {
    let mut game = blight_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.blight_tiles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.food = Point { x: 11, y: 10 }; // the snake eats here on tick 1
    // Blight adjacent to the eaten cell (11,10)
    game.blight_tiles.insert(Point { x: 12, y: 10 });
    game.obstacles.insert(Point { x: 12, y: 10 });
    game.blight_tiles.insert(Point { x: 11, y: 11 });
    game.obstacles.insert(Point { x: 11, y: 11 });
    game.auto_pilot = false;

    game.tick_counter = 0; // tick 1: no scheduled spread
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert!(
        !game.blight_tiles.contains(&Point { x: 12, y: 10 })
            && !game.blight_tiles.contains(&Point { x: 11, y: 11 }),
        "eating food must cleanse the adjacent blight, got {:?}",
        game.blight_tiles
    );
    assert!(
        !game.obstacles.contains(&Point { x: 12, y: 10 })
            && !game.obstacles.contains(&Point { x: 11, y: 11 }),
        "cleansed blight must be removed from obstacles too"
    );
    assert_eq!(game.blight_clears, 2, "both adjacent tiles must be counted");
}

#[test]
fn test_blight_kills_on_entry() {
    let mut game = blight_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.obstacles.clear();
    game.blight_tiles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    // Blight (a hard obstacle) sits on the cell the snake is about to enter
    game.blight_tiles.insert(Point { x: 11, y: 10 });
    game.obstacles.insert(Point { x: 11, y: 10 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    let lives_before = game.lives;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert!(
        game.lives < lives_before || game.state != GameState::Playing,
        "entering blight must kill the snake (lives: {}/{})",
        game.lives,
        lives_before
    );
}

#[test]
fn test_blight_is_safe_final_p_rejects_sand() {
    let game = blight_game(30, 30);
    let sand = *game.blight_tiles.iter().next().expect("seeded blight");
    assert!(
        !game.is_safe_final_p(sand, 0, 0),
        "is_safe_final_p must treat a blight cell as unsafe (hard obstacle)"
    );
}

#[test]
fn test_blight_bot_survives_the_spreading_sand() {
    let mut game = blight_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(2024);
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
        "Autopilot should survive Blight mode (lives: {}, spreads: {}, clears: {}, score: {})",
        game.lives,
        game.blight_spreads,
        game.blight_clears,
        game.score
    );
    assert!(
        game.blight_spreads > 0,
        "the blight must actually have spread during the bot run"
    );
}

#[test]
fn test_blight_persists_through_history_rewind() {
    let mut game = blight_game(30, 30);
    game.blight_spreads = 7;
    game.blight_clears = 3;
    game.blight_tiles.insert(Point { x: 12, y: 12 });
    game.save_history_state();
    game.blight_spreads = 0;
    game.blight_clears = 0;
    game.blight_tiles.clear();

    game.rewind_time();

    assert_eq!(
        game.blight_spreads, 7,
        "blight_spreads must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.blight_clears, 3);
    assert!(
        game.blight_tiles.contains(&Point { x: 12, y: 12 }),
        "blight_tiles must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_blight_persists_in_save_file() {
    let mut game = blight_game(30, 30);
    game.blight_spreads = 9;
    game.blight_clears = 4;
    let path = std::env::temp_dir().join(format!(
        "blight_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"blight_spreads\":9")
            && json.contains("\"blight_clears\":4")
            && json.contains("\"blight_tiles\""),
        "SaveState JSON must contain the blight fields, got: {}",
        &json[..json.len().min(200)]
    );
}
