use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn mirage_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Mirage;
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
fn test_mirage_reset_clears_state() {
    let game = mirage_game(30, 30);

    assert!(game.mirage_foods.is_empty());
    assert_eq!(game.mirage_spawns, 0);
}

#[test]
fn test_mirage_spawns_every_6_ticks() {
    let mut game = mirage_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(13);
    game.obstacles.clear();
    game.food = Point { x: 28, y: 28 };
    game.auto_pilot = false;

    // Run consecutive ticks; every 6th tick must spawn a scheduled mirage.
    for tick in 1..=36_u32 {
        game.state = GameState::Playing;
        park_snake(&mut game, 20, 20);
        let spawns_before = game.mirage_spawns;
        game.update();
        assert_eq!(
            game.tick_counter, tick,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        if tick % 6 == 0 {
            assert!(
                game.mirage_spawns > spawns_before,
                "a mirage must spawn on tick {tick} (spawns before: {spawns_before}, after: {})",
                game.mirage_spawns
            );
            assert!(
                !game.mirage_foods.is_empty(),
                "a scheduled spawn must leave a mirage on tick {tick}"
            );
        }
    }
    assert_eq!(game.mirage_spawns, 6);
}

#[test]
fn test_mirage_never_spawns_on_snake_next_cell_or_entities() {
    let mut game = mirage_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    let lives_before = game.lives;

    // Park the snake at (8, 15) heading right before every tick: no mirage may
    // ever appear on its body, its next cell (9, 15), the food, or an obstacle.
    let next = Point { x: 9, y: 15 };
    for _ in 1..=60_u32 {
        game.state = GameState::Playing;
        park_snake(&mut game, 8, 15);
        game.update();
        for (cell, _) in &game.mirage_foods {
            assert!(
                !game.snake.body_map.contains_key(cell),
                "a mirage spawned on the snake's body at {cell:?}"
            );
            assert_ne!(*cell, next, "a mirage spawned on the cell the snake is about to enter");
            assert_ne!(*cell, game.food, "a mirage spawned on the real food");
            assert!(
                !game.obstacles.contains(cell),
                "a mirage spawned on an obstacle at {cell:?}"
            );
        }
    }

    assert_eq!(
        game.lives, lives_before,
        "the snake must survive the mirages (spawns: {})",
        game.mirage_spawns
    );
    assert!(
        game.mirage_spawns >= 9,
        "expected most scheduled spawns to fire, got {}",
        game.mirage_spawns
    );
}

#[test]
fn test_mirage_fades_after_mirage_ticks() {
    let mut game = mirage_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.obstacles.clear();
    game.food = Point { x: 28, y: 28 };
    game.auto_pilot = false;
    let cell = Point { x: 20, y: 20 };
    game.mirage_foods.insert(cell, Game::MIRAGE_TICKS);

    for i in 1..=u32::from(Game::MIRAGE_TICKS) {
        game.state = GameState::Playing;
        park_snake(&mut game, 8, 15);
        game.update();
        if i < Game::MIRAGE_TICKS {
            assert!(
                game.mirage_foods.contains_key(&cell),
                "the mirage must still be visible after {i} ticks"
            );
        }
    }
    assert!(
        !game.mirage_foods.contains_key(&cell),
        "the mirage must fade after MIRAGE_TICKS ticks"
    );
}

#[test]
fn test_mirage_clears_at_tick_200() {
    let mut game = mirage_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.obstacles.clear();
    game.food = Point { x: 28, y: 28 };
    game.auto_pilot = false;

    for _ in 1..=199_u32 {
        game.state = GameState::Playing;
        park_snake(&mut game, 20, 20);
        game.update();
    }
    assert!(
        !game.mirage_foods.is_empty(),
        "mirages must be present before tick 200 (spawns: {})",
        game.mirage_spawns
    );

    // Tick 200 of the 250-tick cycle shimmers all mirages away.
    game.state = GameState::Playing;
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.mirage_foods.is_empty(),
        "all mirages must clear at tick 200"
    );
    assert!(
        game.mirage_spawns >= 30,
        "the spawn counter keeps counting through a clearing (got {})",
        game.mirage_spawns
    );
}

#[test]
fn test_mirage_eating_food_dispells_all_mirages() {
    let mut game = mirage_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.mirage_foods.insert(Point { x: 7, y: 5 }, Game::MIRAGE_TICKS);
    game.mirage_foods.insert(Point { x: 12, y: 12 }, Game::MIRAGE_TICKS);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.auto_pilot = false;

    // Tick 5 is not a multiple of 6 and not 200, so no spawn interferes here.
    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        game.mirage_foods.is_empty(),
        "Eating food in Mirage mode should dispel all mirages"
    );
}

#[test]
fn test_mirage_shatters_on_entry_without_growing() {
    let mut game = mirage_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    let mirage = Point { x: 6, y: 5 };
    game.mirage_foods.insert(mirage, Game::MIRAGE_TICKS);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 18, y: 18 };
    game.auto_pilot = false;
    let len_before = game.snake.body.len();

    // Tick 2: the snake steps onto the mirage cell.
    game.state = GameState::Playing;
    game.tick_counter = 1;
    game.update();
    assert_eq!(game.snake.head(), mirage);
    // Tick 3: the mirage shatters at the start of the tick.
    game.state = GameState::Playing;
    game.update();

    assert!(
        !game.mirage_foods.contains_key(&mirage),
        "walking onto a mirage must shatter it"
    );
    assert_eq!(
        game.food_eaten_session, 0,
        "a mirage must never count as food"
    );
    assert_eq!(
        game.snake.body.len(),
        len_before,
        "a mirage must never grow the snake"
    );
}

#[test]
fn test_mirage_bot_eats_real_food_despite_mirages() {
    let mut game = mirage_game(40, 40);
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
        "Autopilot should survive the mirages (lives: {}, spawns: {}, score: {})",
        game.lives,
        game.mirage_spawns,
        game.score
    );
    assert!(
        game.mirage_spawns >= 5,
        "expected several mirage spawns over 120 ticks, got {}",
        game.mirage_spawns
    );
    assert!(
        game.food_eaten_session > 0,
        "the autopilot must still eat real food while mirages are on the board"
    );
}

#[test]
fn test_mirage_persists_through_history_rewind() {
    let mut game = mirage_game(30, 30);
    game.mirage_foods.insert(Point { x: 7, y: 7 }, 12);
    game.mirage_spawns = 7;
    game.save_history_state();
    game.mirage_foods.clear();
    game.mirage_spawns = 0;

    game.rewind_time();

    assert_eq!(
        game.mirage_foods.get(&Point { x: 7, y: 7 }),
        Some(&12),
        "mirage_foods must survive a time-rewind through HistoryState"
    );
    assert_eq!(
        game.mirage_spawns, 7,
        "mirage_spawns must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_mirage_persists_in_save_file() {
    let mut game = mirage_game(30, 30);
    game.mirage_spawns = 9;
    let path = std::env::temp_dir().join(format!(
        "mirage_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"mirage_spawns\":9") && json.contains("\"mirage_foods\""),
        "SaveState JSON must contain the mirage counters, got: {}",
        &json[..json.len().min(200)]
    );
}
