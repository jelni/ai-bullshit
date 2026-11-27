use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn spike_pit_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::SpikePit;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_spike_pit_reset_seeds_vents_and_clears_state() {
    let game = spike_pit_game(30, 30);

    assert!(game.spike_tiles.is_empty());
    assert!(game.spike_warnings.is_empty());
    assert_eq!(game.spike_eruptions, 0);
    assert_eq!(game.spike_vents.len(), Game::SPIKE_PIT_VENTS);
    let spawn = Point {
        x: game.width / 2,
        y: game.height / 2,
    };
    for vent in &game.spike_vents {
        assert!(
            !game.obstacles.contains(vent),
            "a vent must not spawn on an obstacle"
        );
        assert!(
            !game.snake.body_map.contains_key(vent),
            "a vent must not spawn on the snake"
        );
        assert!(vent != &game.food, "a vent must not spawn on the food");
        assert!(
            vent.x.abs_diff(spawn.x).saturating_add(vent.y.abs_diff(spawn.y)) >= 5,
            "a vent must spawn at least 5 cells from the spawn point"
        );
    }
}

#[test]
fn test_spike_pit_telegraphs_then_erupts_every_6_ticks() {
    let mut game = spike_pit_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.auto_pilot = false;

    // update() increments tick_counter first, so tick_counter = 2 runs tick 3,
    // which telegraphs the cells that will erupt on tick 6.
    game.state = GameState::Playing;
    game.tick_counter = 2;
    game.update();
    assert_eq!(game.tick_counter, 3);
    assert!(
        !game.spike_warnings.is_empty(),
        "tick 3 must telegraph cracked cells around the vents"
    );
    assert!(game.spike_tiles.is_empty(), "no spikes before the eruption");
    for crack in &game.spike_warnings {
        let near_vent = game
            .spike_vents
            .iter()
            .any(|v| v.x.abs_diff(crack.x).saturating_add(v.y.abs_diff(crack.y)) <= 1);
        assert!(near_vent, "a warning must be on or next to a vent: {crack:?}");
    }

    // Tick 6 erupts: the warnings are consumed and spikes come up.
    game.state = GameState::Playing;
    game.tick_counter = 5;
    game.update();
    assert_eq!(game.tick_counter, 6);
    assert_eq!(game.spike_eruptions, 1);
    assert!(
        game.spike_warnings.is_empty(),
        "the telegraph is consumed by the eruption"
    );
    assert!(
        !game.spike_tiles.is_empty(),
        "tick 6 must erupt spikes around the vents"
    );
    for (spike, ticks) in &game.spike_tiles {
        assert!(
            game.obstacles.contains(spike),
            "spikes must be mirrored into obstacles"
        );
        assert_eq!(*ticks, Game::SPIKE_UP_TICKS);
    }

    // Ticks 12 and 18 erupt again.
    for tick in [11_u32, 17] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
    }
    assert_eq!(game.spike_eruptions, 3);
}

#[test]
fn test_spike_pit_never_spikes_snake_or_next_cell() {
    let mut game = spike_pit_game(20, 20);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.spike_vents.clear();
    // One vent right in front of the snake: the snake heads Left into the vent cell.
    game.spike_vents.push(Point { x: 4, y: 5 });
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Left;
    game.snake.direction_queue.push_back(Direction::Left);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = false;

    // Tick 6 erupts around every vent (the pit re-seeds the missing vents fairly).
    game.state = GameState::Playing;
    game.tick_counter = 5;
    game.update();

    assert_eq!(game.spike_eruptions, 1);
    let next = Point { x: 4, y: 5 };
    assert!(
        !game.spike_tiles.contains_key(&next),
        "an eruption must never spike the cell the snake is about to enter"
    );
    for seg in &game.snake.body {
        assert!(
            !game.spike_tiles.contains_key(seg),
            "an eruption must never spike the snake's body"
        );
    }
    assert!(
        !game.spike_tiles.contains_key(&game.food),
        "an eruption must never spike the food"
    );
    for spike in game.spike_tiles.keys() {
        assert!(game.obstacles.contains(spike));
    }
}

#[test]
fn test_spike_pit_fairness_over_many_ticks() {
    let mut game = spike_pit_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 25, y: 25 };
    let lives_before = game.lives;

    // Deterministic: park the snake at (8, 15) heading right before every
    // scheduled eruption, so the eruption must never spike its next cell (9, 15) or body.
    let next = Point { x: 9, y: 15 };
    for k in 1..=10_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = 6 * k - 1;
        game.update();
        assert!(
            !game.spike_tiles.contains_key(&next),
            "eruption {} spiked the cell the snake was about to enter ({next:?})",
            game.spike_eruptions
        );
        for seg in &game.snake.body {
            assert!(
                !game.spike_tiles.contains_key(seg),
                "eruption {} spiked the snake's body at {seg:?}",
                game.spike_eruptions
            );
        }
    }

    assert_eq!(
        game.lives, lives_before,
        "the snake must survive the eruptions (eruptions: {})",
        game.spike_eruptions
    );
    assert!(
        game.spike_eruptions >= 7,
        "expected several eruptions over 60 ticks, got {}",
        game.spike_eruptions
    );
}

#[test]
fn test_spike_pit_spikes_retract_after_up_ticks() {
    let mut game = spike_pit_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.auto_pilot = false;
    game.obstacles.clear();
    game.spike_vents.clear();
    let dying = Point { x: 5, y: 5 };
    let staying = Point { x: 6, y: 6 };
    game.spike_tiles.insert(dying, 1);
    game.obstacles.insert(dying);
    game.spike_tiles.insert(staying, 3);
    game.obstacles.insert(staying);

    // Tick 11 is not a multiple of 6 and not 200, so no eruption interferes.
    game.state = GameState::Playing;
    game.tick_counter = 10;
    game.update();

    assert!(
        !game.spike_tiles.contains_key(&dying),
        "a spike with one tick left must retract this tick"
    );
    assert!(
        !game.obstacles.contains(&dying),
        "a retracted spike must leave obstacles"
    );
    assert_eq!(
        game.spike_tiles.get(&staying),
        Some(&2),
        "a spike with more ticks left must stay up and count down"
    );
}

#[test]
fn test_spike_pit_retracts_all_at_tick_200() {
    let mut game = spike_pit_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.auto_pilot = true;

    // Erupt a few times first.
    for tick in [5_u32, 11, 17] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
    }
    assert!(!game.spike_tiles.is_empty());
    assert!(game.spike_eruptions >= 3);

    // Tick 200 of the 250-tick cycle retracts all spikes.
    game.state = GameState::Playing;
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.spike_tiles.is_empty(),
        "all spikes must retract at tick 200"
    );
    assert!(
        game.spike_warnings.is_empty(),
        "all telegraph cracks must clear at tick 200"
    );
    for spike in game.obstacles.iter() {
        assert!(
            !game.spike_tiles.contains_key(spike),
            "no obstacle may remain a recorded spike"
        );
    }
    assert!(
        game.spike_eruptions >= 3,
        "the eruption counter keeps counting through a retraction"
    );
}

#[test]
fn test_spike_pit_eating_food_retracts_adjacent_spikes() {
    let mut game = spike_pit_game(20, 20);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.spike_vents.clear();
    let spike = Point { x: 7, y: 5 };
    game.spike_tiles.insert(spike, 5);
    game.obstacles.insert(spike);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.auto_pilot = false;

    // Tick 5 is not a multiple of 6 and not 200, so no eruption interferes here.
    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.spike_tiles.contains_key(&spike),
        "Eating food in Spike Pit mode should retract the adjacent spike"
    );
    assert!(
        !game.obstacles.contains(&spike),
        "The retracted spike must also leave obstacles"
    );
}

#[test]
fn test_spike_pit_bot_survives_the_eruptions() {
    let mut game = spike_pit_game(40, 40);
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
        "Autopilot should survive the spike pit (lives: {}, eruptions: {}, score: {})",
        game.lives,
        game.spike_eruptions,
        game.score
    );
    assert!(
        game.spike_eruptions >= 5,
        "expected several eruptions over 150 ticks, got {}",
        game.spike_eruptions
    );
}

#[test]
fn test_spike_pit_persists_through_history_rewind() {
    let mut game = spike_pit_game(30, 30);
    game.spike_vents = vec![Point { x: 10, y: 10 }];
    game.spike_tiles.insert(Point { x: 10, y: 11 }, 4);
    game.spike_warnings.insert(Point { x: 11, y: 10 });
    game.spike_eruptions = 7;
    game.save_history_state();
    game.spike_vents.clear();
    game.spike_tiles.clear();
    game.spike_warnings.clear();
    game.spike_eruptions = 0;

    game.rewind_time();

    assert_eq!(game.spike_vents, vec![Point { x: 10, y: 10 }]);
    assert_eq!(game.spike_tiles.get(&Point { x: 10, y: 11 }), Some(&4));
    assert!(game.spike_warnings.contains(&Point { x: 11, y: 10 }));
    assert_eq!(
        game.spike_eruptions, 7,
        "spike_eruptions must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_spike_pit_persists_in_save_file() {
    let mut game = spike_pit_game(30, 30);
    game.spike_eruptions = 9;
    game.spike_vents = vec![Point { x: 12, y: 9 }];
    game.spike_tiles.insert(Point { x: 12, y: 10 }, 3);
    game.spike_warnings.insert(Point { x: 11, y: 9 });
    let path = std::env::temp_dir().join(format!(
        "spike_pit_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"spike_eruptions\":9"),
        "SaveState JSON must contain the eruption counter, got: {}",
        &json[..json.len().min(200)]
    );
    assert!(
        json.contains("spike_vents") && json.contains("spike_tiles") && json.contains("spike_warnings"),
        "SaveState JSON must contain the spike pit state, got: {}",
        &json[..json.len().min(400)]
    );
}
