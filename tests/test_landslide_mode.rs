use rand::SeedableRng;
use snake_game::game::{LandslideRock, Difficulty, Game, GameMode, GameState, HeroClass, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn landslide_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Landslide;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_landslide_reset_clears_state() {
    let game = landslide_game(30, 30);

    assert!(game.rocks.is_empty());
    assert_eq!(game.landslide_rolls, 0);
    assert_eq!(game.weather, Weather::Snow);
}

#[test]
fn test_landslide_scheduled_roll_every_6_ticks() {
    let mut game = landslide_game(30, 30);
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must roll a rock in.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let rolls_before = game.landslide_rolls;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.landslide_rolls > rolls_before,
            "a rock must roll in on tick {} (rolls before: {}, after: {})",
            tick + 1,
            rolls_before,
            game.landslide_rolls
        );
    }
    assert!(game.landslide_rolls >= 6);
    assert!(!game.rocks.is_empty());
}

#[test]
fn test_landslide_never_rolls_down_snake_row_or_next_cell() {
    let mut game = landslide_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.auto_pilot = false;
    game.snake = Snake::new(Point {
        x: 8,
        y: 10,
    });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point {
        x: 15,
        y: 15,
    };

    // Tick 6 spawns a rock; the snake occupies rows 10..=12 and is about
    // to enter (9, 10), so the rock must roll down some other row.
    game.tick_counter = 5;
    game.update();

    assert_eq!(game.landslide_rolls, 1);
    assert_eq!(game.rocks.len(), 1);
    let rock = game.rocks[0];
    assert!(
        rock.position.y != 10 && rock.position.y != 11 && rock.position.y != 12,
        "a rock must never roll down a row the snake occupies or is about to enter, got row {}",
        rock.position.y
    );
    assert!(
        (rock.direction == Direction::Right && rock.position.x <= 1)
            || (rock.direction == Direction::Left && rock.position.x >= game.width - 2),
        "a rock must spawn at the edge it rolls in from (it rolls once on the spawn tick), got {:?}",
        rock.position
    );
}

#[test]
fn test_landslide_fairness_over_many_ticks() {
    let mut game = landslide_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point {
        x: 25,
        y: 25,
    };
    let lives_before = game.lives;

    // Park the snake at (8, 15) heading right before every scheduled roll.
    for k in 1..=10_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point {
            x: 8,
            y: 15,
        });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = 6 * k - 1;
        game.update();
        for rock in &game.rocks {
            assert!(
                rock.position.y != 15 && rock.position.y != 16 && rock.position.y != 17,
                "rock rolled down snake row {} on tick {}",
                rock.position.y,
                game.tick_counter
            );
        }
    }

    assert_eq!(
        game.lives, lives_before,
        "the parked snake must never be hit by a fairly spawned rock"
    );
    assert!(game.landslide_rolls >= 10);
}

#[test]
fn test_rock_rolls_one_cell_per_tick_and_crushes_obstacles() {
    let mut game = landslide_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.obstacles.clear();
    game.rocks.clear();
    game.landslide_rolls = 0;
    game.auto_pilot = false;
    game.rocks.push(LandslideRock {
        position: Point {
            x: 2,
            y: 5,
        },
        direction: Direction::Right,
    });
    game.obstacles.insert(Point {
        x: 4,
        y: 5,
    });

    game.state = GameState::Playing;
    game.tick_counter = 4; // tick 5: no scheduled spawn
    game.update();
    assert!(
        game.rocks.iter().any(|b| b.position
            == Point {
                x: 3,
                y: 5
            }),
        "the rock must roll exactly one cell per tick, got {:?}",
        game.rocks
    );
    assert!(
        game.obstacles.contains(&Point {
            x: 4,
            y: 5
        }),
        "the obstacle at (4, 5) must survive until the rock reaches it"
    );

    game.state = GameState::Playing;
    game.tick_counter = 5; // tick 6: scheduled spawn may add more, fine
    game.update();
    assert!(
        game.rocks.iter().any(|b| b.position
            == Point {
                x: 4,
                y: 5
            }),
        "the rock must have rolled onto (4, 5), got {:?}",
        game.rocks
    );
    assert!(
        !game.obstacles.contains(&Point {
            x: 4,
            y: 5
        }),
        "the rock must crush the obstacle in its path"
    );
}

#[test]
fn test_rock_position_after_predicts_the_roll() {
    let game = landslide_game(30, 30);
    let b = LandslideRock {
        position: Point {
            x: 2,
            y: 5,
        },
        direction: Direction::Right,
    };
    assert_eq!(
        game.rock_position_after(b, 3),
        Some(Point {
            x: 5,
            y: 5
        })
    );
    assert_eq!(
        game.rock_position_after(b, 27),
        Some(Point {
            x: 29,
            y: 5
        })
    );
    assert_eq!(game.rock_position_after(b, 28), None, "off the board");
    let left = LandslideRock {
        position: Point {
            x: 10,
            y: 5,
        },
        direction: Direction::Left,
    };
    assert_eq!(
        game.rock_position_after(left, 4),
        Some(Point {
            x: 6,
            y: 5
        })
    );
    assert_eq!(game.rock_position_after(left, 11), None);
}

#[test]
fn test_rock_kills_snake_in_its_path() {
    let mut game = landslide_game(20, 20);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.obstacles.clear();
    game.rocks.clear();
    game.landslide_rolls = 0;
    game.auto_pilot = false;
    game.lives = 3;
    game.snake = Snake::new(Point {
        x: 10,
        y: 10,
    });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point {
        x: 2,
        y: 2,
    };
    // The rock rolls from (10, 10) to (11, 10) this tick, exactly where the
    // snake's head is moving.
    game.rocks.push(LandslideRock {
        position: Point {
            x: 10,
            y: 10,
        },
        direction: Direction::Right,
    });

    game.state = GameState::Playing;
    game.tick_counter = 4; // tick 5: no scheduled spawn
    game.update();

    assert_eq!(
        game.lives, 2,
        "a rock landing on the snake's head must cost a life (lives: {})",
        game.lives
    );
}

#[test]
fn test_landslide_eating_food_braces_nearest_rock() {
    let mut game = landslide_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.rocks.clear();
    game.landslide_rolls = 0;
    game.auto_pilot = false;
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point {
        x: 6,
        y: 5,
    };
    game.rocks.push(LandslideRock {
        position: Point {
            x: 20,
            y: 5,
        },
        direction: Direction::Left,
    });

    game.state = GameState::Playing;
    game.tick_counter = 4; // tick 5: no scheduled spawn
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.rocks.iter().any(|b| b.position.y == 5),
        "eating food in Landslide mode must brace (remove) the nearest rock, got {:?}",
        game.rocks
    );
}

#[test]
fn test_landslide_bot_survives_the_rolling_rocks() {
    let mut game = landslide_game(40, 40);
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
        "Autopilot should survive the landslide (lives: {}, rolls: {}, score: {})",
        game.lives,
        game.landslide_rolls,
        game.score
    );
    assert!(
        game.landslide_rolls >= 10,
        "expected many rock rolls over 150 ticks, got {}",
        game.landslide_rolls
    );
}

#[test]
fn test_landslide_persists_through_history_rewind() {
    let mut game = landslide_game(30, 30);
    game.rocks.push(LandslideRock {
        position: Point {
            x: 4,
            y: 7,
        },
        direction: Direction::Right,
    });
    game.landslide_rolls = 7;
    game.save_history_state();
    game.rocks.clear();
    game.landslide_rolls = 0;

    game.rewind_time();

    assert_eq!(
        game.landslide_rolls, 7,
        "landslide_rolls must survive a time-rewind through HistoryState"
    );
    assert_eq!(
        game.rocks,
        vec![LandslideRock {
            position: Point {
                x: 4,
                y: 7
            },
            direction: Direction::Right,
        }],
        "rocks must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_landslide_persists_in_save_file() {
    let mut game = landslide_game(30, 30);
    game.rocks.push(LandslideRock {
        position: Point {
            x: 4,
            y: 7,
        },
        direction: Direction::Right,
    });
    game.landslide_rolls = 9;
    let path = std::env::temp_dir().join(format!(
        "landslide_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"landslide_rolls\":9") && json.contains("\"rocks\":["),
        "SaveState JSON must contain the landslide state, got: {}",
        &json[..json.len().min(200)]
    );
}
