use rand::SeedableRng;
use snake_game::game::{Boulder, Difficulty, Game, GameMode, GameState, HeroClass, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn avalanche_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Avalanche;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

fn park_snake(game: &mut Game, x: u16, y: u16) {
    game.snake = Snake::new(Point { x, y });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
}

#[test]
fn test_avalanche_reset_clears_state() {
    let game = avalanche_game(30, 30);

    assert!(game.boulders.is_empty());
    assert_eq!(game.avalanche_rolls, 0);
}

#[test]
fn test_avalanche_rockfall_every_6_ticks() {
    let mut game = avalanche_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.auto_pilot = false;
    game.food = Point { x: 25, y: 25 };

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must trigger a scheduled rockfall.
    for k in 1..=6_u32 {
        game.state = GameState::Playing;
        park_snake(&mut game, 8, 15);
        game.tick_counter = 6 * k - 1;
        let rolls_before = game.avalanche_rolls;
        game.update();
        assert_eq!(
            game.tick_counter,
            6 * k,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.avalanche_rolls > rolls_before,
            "a rockfall must fire on tick {} (before: {}, after: {})",
            6 * k,
            rolls_before,
            game.avalanche_rolls
        );
    }
    assert_eq!(game.avalanche_rolls, 6);
    assert!(
        !game.boulders.is_empty(),
        "boulders must be on the board after several rockfalls (cap: {})",
        Game::AVALANCHE_MAX_BOULDERS
    );
    assert!(game.boulders.len() <= Game::AVALANCHE_MAX_BOULDERS);
}

#[test]
fn test_avalanche_never_spawns_in_snake_row() {
    let mut game = avalanche_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 25, y: 25 };
    let lives_before = game.lives;

    // Park the snake at (8, 15) heading right before every scheduled rockfall:
    // no boulder may ever break loose in row 15 (its body or next cell (9, 15)).
    // Clear old boulders each round so every scheduled tick spawns a fresh one.
    for k in 1..=10_u32 {
        game.state = GameState::Playing;
        park_snake(&mut game, 8, 15);
        game.boulders.clear();
        game.tick_counter = 6 * k - 1;
        let rolls_before = game.avalanche_rolls;
        game.update();
        assert!(
            game.avalanche_rolls > rolls_before,
            "a rockfall must fire on tick {}",
            6 * k
        );
        for b in &game.boulders {
            assert_ne!(
                b.position.y, 15,
                "a boulder broke loose in the snake's row, tick {}",
                game.tick_counter
            );
        }
        assert!(
            !game
                .boulders
                .iter()
                .any(|b| b.position == Point { x: 9, y: 15 }),
            "a boulder rolled into the cell the snake was about to enter"
        );
    }

    assert_eq!(
        game.lives, lives_before,
        "the parked snake must survive the rockfalls (rolls: {})",
        game.avalanche_rolls
    );
    assert_eq!(game.avalanche_rolls, 10);
}

#[test]
fn test_avalanche_settles_at_tick_200() {
    let mut game = avalanche_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.auto_pilot = false;
    game.food = Point { x: 25, y: 25 };

    for k in 1..=3_u32 {
        game.state = GameState::Playing;
        park_snake(&mut game, 8, 15);
        game.tick_counter = 6 * k - 1;
        game.update();
    }
    assert!(!game.boulders.is_empty());

    // Tick 200 of the 250-tick cycle settles the avalanche: all boulders crumble.
    game.state = GameState::Playing;
    park_snake(&mut game, 8, 15);
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.boulders.is_empty(),
        "all boulders must crumble at tick 200"
    );
    assert!(
        game.avalanche_rolls >= 3,
        "the roll counter keeps counting through a settling"
    );
}

#[test]
fn test_avalanche_boulder_rolls_one_cell_per_tick() {
    let mut game = avalanche_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.auto_pilot = false;
    park_snake(&mut game, 2, 2);
    game.food = Point { x: 15, y: 15 };
    game.boulders.clear();
    game.boulders.push(Boulder {
        position: Point { x: 5, y: 10 },
        dir: Direction::Right,
    });

    // Tick 4 is not a multiple of 6 and not 200: only the roll runs.
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.boulders.len(), 1);
    assert_eq!(
        game.boulders[0].position,
        Point { x: 6, y: 10 },
        "a boulder must roll exactly one cell in its direction per tick"
    );
}

#[test]
fn test_avalanche_boulder_crushes_obstacles() {
    let mut game = avalanche_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.auto_pilot = false;
    park_snake(&mut game, 2, 2);
    game.food = Point { x: 15, y: 15 };
    game.boulders.clear();
    game.obstacles.insert(Point { x: 6, y: 10 });
    game.boulders.push(Boulder {
        position: Point { x: 5, y: 10 },
        dir: Direction::Right,
    });

    game.tick_counter = 4;
    game.update();

    assert_eq!(game.boulders[0].position, Point { x: 6, y: 10 });
    assert!(
        !game.obstacles.contains(&Point { x: 6, y: 10 }),
        "a rolling boulder must crush the obstacle in its path"
    );
}

#[test]
fn test_avalanche_boulder_crashes_at_the_far_wall() {
    let mut game = avalanche_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.auto_pilot = false;
    park_snake(&mut game, 2, 2);
    game.food = Point { x: 15, y: 15 };
    game.boulders.clear();
    game.boulders.push(Boulder {
        position: Point { x: 19, y: 10 },
        dir: Direction::Right,
    });

    game.tick_counter = 4;
    game.update();

    assert!(
        game.boulders.is_empty(),
        "a boulder rolling off the far edge must crumble"
    );
}

#[test]
fn test_avalanche_boulder_kills_snake() {
    let mut game = avalanche_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.auto_pilot = false;
    game.snake = Snake::new(Point { x: 6, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.boulders.clear();
    // The boulder rolls onto the snake's head cell this tick.
    game.boulders.push(Boulder {
        position: Point { x: 5, y: 10 },
        dir: Direction::Right,
    });
    let lives_before = game.lives;

    game.tick_counter = 4;
    game.update();

    assert!(
        game.lives < lives_before || game.state == GameState::GameOver,
        "a boulder rolling onto the snake must kill it (lives: {} -> {})",
        lives_before,
        game.lives
    );
}

#[test]
fn test_avalanche_eating_food_blasts_adjacent_boulder() {
    let mut game = avalanche_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.auto_pilot = false;
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.boulders.clear();
    // Rolls one cell left this tick, landing on (6, 6) -- adjacent to the eaten cell (6, 5).
    game.boulders.push(Boulder {
        position: Point { x: 7, y: 6 },
        dir: Direction::Left,
    });

    // Tick 4 is not a multiple of 6 and not 200, so no rockfall interferes here.
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        game.boulders.is_empty(),
        "Eating food in Avalanche mode should blast the adjacent boulder, got {:?}",
        game.boulders
            .iter()
            .map(|b| b.position)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_avalanche_boulder_count_is_capped() {
    let mut game = avalanche_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.auto_pilot = false;
    park_snake(&mut game, 8, 15);
    game.food = Point { x: 25, y: 25 };
    game.boulders.clear();
    for i in 0..Game::AVALANCHE_MAX_BOULDERS - 1 {
        game.boulders.push(Boulder {
            position: Point {
                x: 5,
                y: u16::try_from(i).unwrap_or(0) + 1,
            },
            dir: Direction::Right,
        });
    }

    // Below the cap: the rockfall fires and spawns one more boulder (up to the cap).
    game.tick_counter = 5;
    game.update();

    assert_eq!(game.avalanche_rolls, 1, "the rockfall fires below the cap");
    assert_eq!(
        game.boulders.len(),
        Game::AVALANCHE_MAX_BOULDERS,
        "the rockfall spawns up to the cap"
    );

    // At the cap: no new boulder may spawn, and none may exceed the cap.
    game.state = GameState::Playing;
    park_snake(&mut game, 8, 15);
    game.tick_counter = 11;
    game.update();

    assert_eq!(
        game.avalanche_rolls, 1,
        "at the cap no further rockfall is counted"
    );
    assert!(
        game.boulders.len() <= Game::AVALANCHE_MAX_BOULDERS,
        "boulder count must stay capped at {}, got {}",
        Game::AVALANCHE_MAX_BOULDERS,
        game.boulders.len()
    );
}

#[test]
fn test_avalanche_bot_survives_the_rockfall() {
    let mut game = avalanche_game(40, 40);
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
        "Autopilot should survive the avalanche (lives: {}, rolls: {}, score: {})",
        game.lives,
        game.avalanche_rolls,
        game.score
    );
    assert!(
        game.avalanche_rolls >= 5,
        "expected several rockfalls over 120 ticks, got {}",
        game.avalanche_rolls
    );
}

#[test]
fn test_avalanche_persists_through_history_rewind() {
    let mut game = avalanche_game(30, 30);
    game.avalanche_rolls = 7;
    game.boulders.push(Boulder {
        position: Point { x: 12, y: 12 },
        dir: Direction::Left,
    });
    game.save_history_state();
    game.avalanche_rolls = 0;
    game.boulders.clear();

    game.rewind_time();

    assert_eq!(
        game.avalanche_rolls, 7,
        "avalanche_rolls must survive a time-rewind through HistoryState"
    );
    assert_eq!(
        game.boulders.len(),
        1,
        "boulders must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.boulders[0].position, Point { x: 12, y: 12 });
}

#[test]
fn test_avalanche_persists_in_save_file() {
    let mut game = avalanche_game(30, 30);
    game.avalanche_rolls = 9;
    game.boulders.push(Boulder {
        position: Point { x: 4, y: 7 },
        dir: Direction::Right,
    });
    let path = std::env::temp_dir().join(format!(
        "avalanche_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"avalanche_rolls\":9") && json.contains("\"boulders\""),
        "SaveState JSON must contain the avalanche state, got: {}",
        &json[..json.len().min(200)]
    );
}
