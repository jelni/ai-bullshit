use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn domino_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Domino;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_domino_reset_seeds_dominoes_and_clears_counters() {
    let game = domino_game(30, 30);

    assert_eq!(game.domino_waves, 0);
    assert_eq!(game.domino_falls, 0);
    assert_eq!(game.domino_drops, 0);
    assert_eq!(game.domino_reshuffles, 0);
    assert_eq!(game.weather, Weather::Clear);
    // clamp(30*30/80, 4, 10) = 10 initial dominoes (best effort, but a 30x30 board always fits them)
    assert!(
        !game.domino_tiles.is_empty() && game.domino_tiles.len() <= 10,
        "expected up to 10 initial dominoes, got {}",
        game.domino_tiles.len()
    );
    for domino in &game.domino_tiles {
        assert!(
            game.obstacles.contains(domino),
            "dominoes must be mirrored into obstacles"
        );
    }
}

#[test]
fn test_domino_waves_every_8_ticks() {
    let mut game = domino_game(30, 30);
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 8k - 1 makes the
    // next update run tick 8k, which must fire a topple wave.
    for tick in [7_u32, 15, 23, 31, 39, 47] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let waves_before = game.domino_waves;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.domino_waves > waves_before,
            "a wave must fire on tick {} (waves before: {}, after: {})",
            tick + 1,
            waves_before,
            game.domino_waves
        );
        for domino in &game.domino_tiles {
            assert!(
                game.obstacles.contains(domino),
                "dominoes must stay mirrored into obstacles"
            );
        }
    }
    assert_eq!(game.domino_waves, 6);
}

#[test]
fn test_domino_wave_topples_a_chain_without_growing_the_field() {
    let mut game = domino_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.domino_tiles.clear();
    // A solid 2x2 block in the middle of an empty board: whatever domino and
    // direction the wave picks, the chain has a fair landing cell.
    for p in [
        Point { x: 10, y: 10 },
        Point { x: 11, y: 10 },
        Point { x: 10, y: 11 },
        Point { x: 11, y: 11 },
    ] {
        game.domino_tiles.insert(p);
        game.obstacles.insert(p);
    }
    game.snake = Snake::new(Point { x: 2, y: 2 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 17, y: 17 };
    game.auto_pilot = false;
    let count_before = game.domino_tiles.len();

    // Tick 8 is a wave tick (and not a multiple of 24, so no fresh drop).
    game.state = GameState::Playing;
    game.tick_counter = 7;
    game.update();

    assert_eq!(game.domino_waves, 1);
    assert!(
        game.domino_falls >= 1,
        "the wave must topple at least one domino, got {}",
        game.domino_falls
    );
    assert_eq!(
        game.domino_tiles.len(),
        count_before,
        "a topple wave shoves dominoes, it does not add new ones"
    );
    for domino in &game.domino_tiles {
        assert!(
            game.obstacles.contains(domino),
            "toppled dominoes must stay mirrored into obstacles"
        );
    }
}

#[test]
fn test_domino_never_lands_on_a_snake_or_its_next_cell() {
    let mut game = domino_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 25, y: 25 };
    // Clear the random initial dominoes and seed one controlled frontier far away,
    // so the parked snake is never next to a pre-existing domino.
    game.obstacles.clear();
    game.domino_tiles.clear();
    let seed = Point { x: 25, y: 5 };
    game.domino_tiles.insert(seed);
    game.obstacles.insert(seed);
    let lives_before = game.lives;

    // Park the snake at (8, 15) heading right before every scheduled wave, so
    // the wave must never land a domino on its body or the cell it enters.
    for k in 1..=12_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = 8 * k - 1;
        let waves_before = game.domino_waves;
        game.update();
        if game.domino_waves > waves_before {
            let entered = Point { x: 9, y: 15 };
            for p in &game.domino_tiles {
                assert_ne!(
                    *p, entered,
                    "wave {k} landed a domino on the cell the snake was about to enter"
                );
            }
        }
        for seg in &game.snake.body {
            assert!(
                !game.domino_tiles.contains(seg),
                "a domino sits on the snake's body at {seg:?} after wave {k}"
            );
        }
    }

    assert_eq!(
        game.lives, lives_before,
        "the snake must survive the toppling dominoes (waves: {})",
        game.domino_waves
    );
    assert!(
        game.domino_waves >= 10,
        "expected several waves over the ticks, got {}",
        game.domino_waves
    );
}

#[test]
fn test_domino_reshuffles_at_tick_200() {
    let mut game = domino_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.auto_pilot = true;

    // Fire a few waves first.
    for tick in [7_u32, 15, 23] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
    }
    assert!(!game.domino_tiles.is_empty());

    // Tick 200 of the 250-tick cycle stands every domino up on a fresh fair cell.
    game.state = GameState::Playing;
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert_eq!(game.domino_reshuffles, 1);
    assert!(
        !game.domino_tiles.is_empty(),
        "the stand-again must re-seed dominoes"
    );
    for domino in &game.domino_tiles {
        assert!(
            game.obstacles.contains(domino),
            "re-seeded dominoes must be mirrored into obstacles"
        );
    }
    assert!(
        game.domino_waves >= 3,
        "the wave counter keeps counting through a reshuffle"
    );
}

#[test]
fn test_domino_drops_a_fresh_domino_when_the_field_is_empty() {
    let mut game = domino_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.obstacles.clear();
    game.domino_tiles.clear();
    game.snake = Snake::new(Point { x: 2, y: 2 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 17, y: 17 };
    game.auto_pilot = false;

    game.state = GameState::Playing;
    game.tick_counter = 7; // wave tick 8
    game.update();

    assert_eq!(game.domino_waves, 1);
    assert!(
        game.domino_drops >= 1,
        "an empty field must get a fresh domino on a wave tick, drops: {}",
        game.domino_drops
    );
    assert!(!game.domino_tiles.is_empty());
    for domino in &game.domino_tiles {
        assert!(game.obstacles.contains(domino));
    }
}

#[test]
fn test_domino_field_stays_within_the_cap() {
    let mut game = domino_game(12, 12);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(9);
    game.auto_pilot = true;
    // cap = 12*12*15/100 = 21
    for tick in (7..180).step_by(8) {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
        if game.lives == 0 {
            break;
        }
        assert!(
            game.domino_tiles.len() <= 21,
            "domino count must be capped at 21 on a 12x12 board, got {}",
            game.domino_tiles.len()
        );
    }
}

#[test]
fn test_domino_entering_a_domino_costs_a_life() {
    let mut game = domino_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.domino_tiles.clear();
    let domino = Point { x: 6, y: 5 };
    game.domino_tiles.insert(domino);
    game.obstacles.insert(domino);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = false;
    let lives_before = game.lives;

    assert!(!game.is_safe_final_p(Point { x: 6, y: 5 }, 1, 1));
    assert!(game.is_safe_final_p(Point { x: 6, y: 4 }, 1, 1));

    // Tick 4 is not a multiple of 8 and not 200, so no wave interferes here.
    game.tick_counter = 3;
    game.update();

    assert_eq!(
        game.lives,
        lives_before - 1,
        "moving into a standing domino must cost a life"
    );
}

#[test]
fn test_domino_eating_food_braces_adjacent_dominoes() {
    let mut game = domino_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.domino_tiles.clear();
    let domino = Point { x: 7, y: 5 };
    game.domino_tiles.insert(domino);
    game.obstacles.insert(domino);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.auto_pilot = false;

    // Tick 4 is not a multiple of 8 and not 200, so no wave interferes here.
    game.tick_counter = 3;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.domino_tiles.contains(&domino),
        "Eating food in Domino mode should brace the adjacent domino"
    );
    assert!(
        !game.obstacles.contains(&domino),
        "The braced domino must also leave obstacles"
    );
}

#[test]
fn test_domino_bot_survives_the_toppling() {
    let mut game = domino_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;

    for _ in 0..160 {
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
        "Autopilot should survive the dominoes (lives: {}, waves: {}, score: {})",
        game.lives,
        game.domino_waves,
        game.score
    );
    assert!(
        game.domino_waves >= 10,
        "expected several waves over 160 ticks, got {}",
        game.domino_waves
    );
    assert!(
        game.domino_tiles.len() <= 40 * 40 * 15 / 100,
        "domino count must stay within the cap, got {}",
        game.domino_tiles.len()
    );
}

#[test]
fn test_domino_persists_through_history_rewind() {
    let mut game = domino_game(30, 30);
    game.domino_waves = 7;
    game.domino_falls = 13;
    let domino = Point { x: 11, y: 13 };
    game.domino_tiles.insert(domino);
    game.obstacles.insert(domino);
    game.save_history_state();
    game.domino_waves = 0;
    game.domino_falls = 0;
    game.domino_tiles.clear();

    game.rewind_time();

    assert_eq!(
        game.domino_waves, 7,
        "domino_waves must survive a time-rewind through HistoryState"
    );
    assert_eq!(
        game.domino_falls, 13,
        "domino_falls must survive a time-rewind through HistoryState"
    );
    assert!(
        game.domino_tiles.contains(&domino),
        "domino_tiles must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_domino_persists_in_save_file() {
    let mut game = domino_game(30, 30);
    game.domino_waves = 9;
    game.domino_falls = 21;
    let path = std::env::temp_dir().join(format!(
        "domino_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"domino_waves\":9")
            && json.contains("\"domino_falls\":21")
            && json.contains("\"domino_tiles\""),
        "SaveState JSON must contain the domino state, got: {}",
        &json[..json.len().min(200)]
    );
}
