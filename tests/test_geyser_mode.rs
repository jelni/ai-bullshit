use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn geyser_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Geyser;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_geyser_reset_seeds_vents_and_clears_state() {
    let game = geyser_game(30, 30);

    assert_eq!(game.geyser_vents.len(), 2, "two geyser vents must be seeded");
    assert_eq!(game.geyser_eruptions, 0);
    for vent in &game.geyser_vents {
        assert!(
            game.obstacles.contains(vent),
            "vents must be mirrored into obstacles"
        );
        assert_eq!(
            game.geyser_jets.get(vent),
            Some(&u32::MAX),
            "vents are permanent fixtures in the jet map"
        );
    }
    assert_eq!(
        game.geyser_jets.len(),
        2,
        "only the vents are in the jet map right after reset"
    );
}

#[test]
fn test_geyser_erupts_every_6_ticks() {
    let mut game = geyser_game(30, 30);
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must trigger a scheduled eruption.
    for tick in [5_u32, 11, 17, 23] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let eruptions_before = game.geyser_eruptions;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert_eq!(
            game.geyser_eruptions,
            eruptions_before + 1,
            "a scheduled eruption must fire on tick {}",
            tick + 1
        );
        for jet in game.geyser_jets.keys() {
            assert!(
                game.obstacles.contains(jet),
                "geyser jets must be mirrored into obstacles"
            );
        }
    }
    assert_eq!(game.geyser_eruptions, 4);
}

#[test]
fn test_geyser_never_jets_snake_or_next_cell() {
    let mut game = geyser_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.geyser_vents.clear();
    game.geyser_jets.clear();
    game.geyser_eruptions = 0;
    // One fixed vent far from the snake so eruptions are deterministic in shape
    game.geyser_vents.push(Point { x: 10, y: 10 });
    game.geyser_jets.insert(Point { x: 10, y: 10 }, u32::MAX);
    game.obstacles.insert(Point { x: 10, y: 10 });
    game.snake = Snake::new(Point { x: 2, y: 2 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = false;

    game.tick_counter = 5;
    game.update();

    assert_eq!(game.geyser_eruptions, 1);
    let next = Point { x: 3, y: 2 };
    assert!(
        !game.geyser_jets.contains_key(&next),
        "an eruption must never jet the cell the snake is about to enter"
    );
    for seg in &game.snake.body {
        assert!(
            !game.geyser_jets.contains_key(seg),
            "an eruption must never jet the snake's body"
        );
    }
    assert!(
        !game.geyser_jets.contains_key(&game.food),
        "an eruption must never jet the food"
    );
}

#[test]
fn test_geyser_fairness_over_many_ticks() {
    let mut game = geyser_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 25, y: 25 };
    let lives_before = game.lives;

    // Park the snake at (8, 15) heading right before every scheduled eruption,
    // so the jet must never land on its next cell (9, 15) or its body.
    let next = Point { x: 9, y: 15 };
    for k in 1..=10_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = 6 * k - 1;
        let eruptions_before = game.geyser_eruptions;
        game.update();
        if game.geyser_eruptions > eruptions_before {
            assert!(
                !game.geyser_jets.contains_key(&next),
                "eruption {} jetted the cell the snake was about to enter ({next:?})",
                game.geyser_eruptions
            );
            for seg in &game.snake.body {
                assert!(
                    !game.geyser_jets.contains_key(seg),
                    "eruption {} jetted the snake's body at {seg:?}",
                    game.geyser_eruptions
                );
            }
        }
    }

    assert_eq!(
        game.lives, lives_before,
        "the snake must survive the erupting geysers (eruptions: {})",
        game.geyser_eruptions
    );
    assert!(
        game.geyser_eruptions >= 7,
        "expected many eruptions over 10 scheduled ticks, got {}",
        game.geyser_eruptions
    );
}

#[test]
fn test_geyser_jets_dissipate() {
    let mut game = geyser_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.auto_pilot = false;
    let jet = Point { x: 12, y: 12 };
    game.geyser_jets.insert(jet, 2);
    game.obstacles.insert(jet);

    // Tick 4: not a multiple of 6 and not 200, so no eruption/rest interferes;
    // the jet burns down from 2 to 1 but stays.
    game.state = GameState::Playing;
    game.tick_counter = 3;
    game.update();
    assert_eq!(game.tick_counter, 4);
    assert!(
        game.geyser_jets.contains_key(&jet),
        "a jet with 2 ticks left must survive one more tick"
    );
    assert!(game.obstacles.contains(&jet));

    // Tick 5: the jet burns out and leaves open ground.
    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();
    assert_eq!(game.tick_counter, 5);
    assert!(
        !game.geyser_jets.contains_key(&jet),
        "a jet must dissipate once its burn ticks run out"
    );
    assert!(
        !game.obstacles.contains(&jet),
        "a dissipated jet must leave obstacles"
    );
}

#[test]
fn test_geyser_rest_at_tick_200() {
    let mut game = geyser_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.auto_pilot = true;

    // Trigger a few eruptions first.
    for tick in [5_u32, 11, 17] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
    }
    assert!(game.geyser_eruptions >= 3);

    // Tick 200 of the 250-tick cycle makes all non-vent jets dissipate.
    game.state = GameState::Playing;
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert_eq!(
        game.geyser_jets.len(),
        game.geyser_vents.len(),
        "only the vents may remain in the jet map at tick 200"
    );
    for vent in &game.geyser_vents {
        assert!(
            game.obstacles.contains(vent),
            "the vents themselves must stay blocked"
        );
    }
}

#[test]
fn test_geyser_eating_food_douses_adjacent_jets() {
    let mut game = geyser_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.geyser_vents.clear();
    game.geyser_jets.clear();
    game.geyser_eruptions = 0;
    let jet = Point { x: 7, y: 5 };
    game.geyser_jets.insert(jet, 5);
    game.obstacles.insert(jet);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.auto_pilot = false;

    // Tick 5 is not a multiple of 6 and not 200, so no eruption/rest interferes.
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.geyser_jets.contains_key(&jet),
        "Eating food in Geyser mode should douse the adjacent jet"
    );
    assert!(
        !game.obstacles.contains(&jet),
        "The doused jet must also leave obstacles"
    );
}

#[test]
fn test_geyser_bot_survives_the_eruptions() {
    let mut game = geyser_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;

    for _ in 0..150 {
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
        "Autopilot should survive the geysers (lives: {}, eruptions: {}, score: {})",
        game.lives,
        game.geyser_eruptions,
        game.score
    );
    assert!(
        game.geyser_eruptions >= 10,
        "expected many eruptions over 150 ticks, got {}",
        game.geyser_eruptions
    );
}

#[test]
fn test_geyser_persists_through_history_rewind() {
    let mut game = geyser_game(30, 30);
    game.geyser_eruptions = 7;
    game.geyser_jets.insert(Point { x: 11, y: 11 }, 4);
    game.save_history_state();
    game.geyser_eruptions = 0;
    game.geyser_jets.clear();

    game.rewind_time();

    assert_eq!(
        game.geyser_eruptions, 7,
        "geyser_eruptions must survive a time-rewind through HistoryState"
    );
    assert_eq!(
        game.geyser_jets.get(&Point { x: 11, y: 11 }),
        Some(&4),
        "geyser_jets must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_geyser_persists_in_save_file() {
    let mut game = geyser_game(30, 30);
    game.geyser_eruptions = 9;
    let path = std::env::temp_dir().join(format!(
        "geyser_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"geyser_eruptions\":9"),
        "SaveState JSON must contain the geyser counter, got: {}",
        &json[..json.len().min(200)]
    );
    assert!(
        json.contains("\"geyser_vents\"") && json.contains("\"geyser_jets\""),
        "SaveState JSON must contain the geyser vents and jets"
    );
}
