use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn electro_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Electro;
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

// Block the ambient portal/mine spawners so single-tick pulse tests are
// deterministic: an existing portal pair stops manage_portals from spawning,
// and a full mine rack (len >= 5) stops manage_mines from dropping new mines.
fn pin_ambient_spawners(game: &mut Game) {
    game.portals = Some((
        Point {
            x: 27,
            y: 27,
        },
        Point {
            x: 28,
            y: 28,
        },
    ));
    game.mines.insert(Point {
        x: 23,
        y: 1,
    });
    game.mines.insert(Point {
        x: 24,
        y: 1,
    });
    game.mines.insert(Point {
        x: 25,
        y: 1,
    });
    game.mines.insert(Point {
        x: 26,
        y: 1,
    });
    game.mines.insert(Point {
        x: 27,
        y: 1,
    });
}

fn park_snake(game: &mut Game, head: Point, dir: Direction) {
    game.snake = Snake::new(head);
    game.snake.direction = dir;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(dir);
    game.auto_pilot = false;
}

#[test]
fn test_electro_reset_seeds_magnets_fairly() {
    let game = electro_game(30, 30);

    assert!(game.electro_attract, "electro must start in attract mode");
    assert_eq!(game.electro_pulses, 0);
    assert_eq!(game.electro_flips, 0);
    assert_eq!(game.electro_shuffles, 0);
    assert!(game.electro_grounded.is_empty());
    // magnet count = clamp(30*30/60, 3, 8) = 8
    assert_eq!(game.electro_magnets.len(), 8);
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    for magnet in &game.electro_magnets {
        assert!(
            !game.obstacles.contains(magnet),
            "a magnet must never sit on an obstacle: {magnet:?}"
        );
        assert!(
            !game.snake.body_map.contains_key(magnet),
            "a magnet must never sit on the snake: {magnet:?}"
        );
        assert_ne!(*magnet, game.food, "a magnet must never sit on the food");
        assert!(!game.mines.contains(magnet), "a magnet must never sit on a mine: {magnet:?}");
        assert!(
            magnet.x.abs_diff(spawn.x).saturating_add(magnet.y.abs_diff(spawn.y)) >= 5,
            "a magnet must spawn at least 5 cells from the spawn point: {magnet:?}"
        );
    }
}

#[test]
fn test_electro_pulse_pulls_head_toward_nearest_magnet() {
    let mut game = electro_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.electro_magnets.clear();
    pin_ambient_spawners(&mut game);
    park_snake(
        &mut game,
        Point {
            x: 10,
            y: 10,
        },
        Direction::Right,
    );
    // Magnet straight ahead: the pulse drags the final head one more cell toward it
    game.electro_magnets.insert(Point {
        x: 20,
        y: 10,
    });
    game.food = Point {
        x: 25,
        y: 25,
    };

    game.tick_counter = 5; // update() runs tick 6: a pulse tick
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 6);
    assert_eq!(game.electro_pulses, 1, "the pulse must be counted");
    assert_eq!(
        game.snake.head(),
        Point {
            x: 12,
            y: 10
        },
        "the pulse must drag the head one cell toward the magnet (normal step to (11,10) plus the drag to (12,10)), head: {:?}",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a pulse must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_electro_repel_electro_pushes_head_away() {
    let mut game = electro_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.electro_magnets.clear();
    pin_ambient_spawners(&mut game);
    park_snake(
        &mut game,
        Point {
            x: 10,
            y: 10,
        },
        Direction::Right,
    );
    // Magnet behind the snake: repelling pushes the final head (11,10) further away to (12,10)
    game.electro_magnets.insert(Point {
        x: 5,
        y: 10,
    });
    game.electro_attract = false;
    game.food = Point {
        x: 25,
        y: 25,
    };

    game.tick_counter = 5; // tick 6: pulse tick
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.electro_pulses, 1);
    assert_eq!(
        game.snake.head(),
        Point {
            x: 12,
            y: 10
        },
        "a repel pulse must push the head away from the magnet, head: {:?}",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a repel pulse must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_electro_no_pulse_on_non_pulse_ticks() {
    let mut game = electro_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.electro_magnets.clear();
    pin_ambient_spawners(&mut game);
    park_snake(
        &mut game,
        Point {
            x: 10,
            y: 10,
        },
        Direction::Right,
    );
    game.electro_magnets.insert(Point {
        x: 20,
        y: 10,
    });
    game.food = Point {
        x: 25,
        y: 25,
    };

    game.tick_counter = 0; // tick 1: not a multiple of ELECTRO_PULSE_PERIOD
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.electro_pulses, 0, "no pulse may fire off-schedule");
    assert_eq!(
        game.snake.head(),
        Point {
            x: 11,
            y: 10
        },
        "on a non-pulse tick the snake must move normally, head: {:?}",
        game.snake.head()
    );
}

#[test]
fn test_electro_pulse_fizzles_when_destination_is_blocked() {
    let mut game = electro_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.obstacles.clear();
    game.electro_magnets.clear();
    pin_ambient_spawners(&mut game);
    park_snake(
        &mut game,
        Point {
            x: 10,
            y: 10,
        },
        Direction::Right,
    );
    game.electro_magnets.insert(Point {
        x: 20,
        y: 10,
    });
    // The drag destination (12,10) is blocked, so the pulse must fizzle and
    // the snake just makes its normal step to (11,10)
    game.obstacles.insert(Point {
        x: 12,
        y: 10,
    });
    game.food = Point {
        x: 25,
        y: 25,
    };

    game.tick_counter = 5; // tick 6: pulse tick
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.electro_pulses, 0, "a blocked pulse must fizzle without counting");
    assert_eq!(
        game.snake.head(),
        Point {
            x: 11,
            y: 10
        },
        "a fizzled pulse must leave the normal move untouched, head: {:?}",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a fizzled pulse must never kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_electro_pulse_never_lands_on_a_hazard() {
    let mut game = electro_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.obstacles.clear();
    game.electro_magnets.clear();
    pin_ambient_spawners(&mut game);
    park_snake(
        &mut game,
        Point {
            x: 10,
            y: 10,
        },
        Direction::Right,
    );
    game.electro_magnets.insert(Point {
        x: 20,
        y: 10,
    });
    // A mine on the drag destination (12,10): the pulse must fizzle rather than land on it
    game.mines.insert(Point {
        x: 12,
        y: 10,
    });
    game.mines.insert(Point {
        x: 23,
        y: 2,
    });
    game.mines.insert(Point {
        x: 24,
        y: 2,
    });
    game.mines.insert(Point {
        x: 25,
        y: 2,
    });
    game.mines.insert(Point {
        x: 26,
        y: 2,
    });
    game.food = Point {
        x: 25,
        y: 25,
    };

    game.tick_counter = 5; // tick 6: pulse tick
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.electro_pulses, 0, "a pulse onto a mine must fizzle");
    assert!(
        game.snake.head()
            == Point {
                x: 11,
                y: 10
            },
        "the snake must make only its normal step, head: {:?}",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the pulse must never land on a hazard (lives: {})",
        game.lives
    );
}

#[test]
fn test_electro_flips_every_12_ticks() {
    let mut game = electro_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.obstacles.clear();
    game.auto_pilot = false;
    game.food = Point {
        x: 35,
        y: 35,
    };
    pin_ambient_spawners(&mut game);

    // tick 12: first flip (attract -> repel)
    park_snake(
        &mut game,
        Point {
            x: 20,
            y: 20,
        },
        Direction::Right,
    );
    game.tick_counter = 11;
    game.state = GameState::Playing;
    game.update();
    assert_eq!(game.tick_counter, 12);
    assert_eq!(game.electro_flips, 1);
    assert!(!game.electro_attract, "tick 12 must flip attract -> repel");

    // tick 24: second flip (repel -> attract) plus a shuffle
    park_snake(
        &mut game,
        Point {
            x: 20,
            y: 20,
        },
        Direction::Right,
    );
    game.tick_counter = 23;
    game.state = GameState::Playing;
    game.update();
    assert_eq!(game.tick_counter, 24);
    assert_eq!(game.electro_flips, 2);
    assert!(game.electro_attract, "tick 24 must flip repel -> attract");
    assert_eq!(game.electro_shuffles, 1, "tick 24 must also shuffle the magnets");
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "flips and pulses must never kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_electro_shuffle_every_24_ticks_stays_fair() {
    let mut game = electro_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = false;

    for tick in [23_u32, 47, 71, 95, 119] {
        game.state = GameState::Playing;
        park_snake(
            &mut game,
            Point {
                x: 20,
                y: 20,
            },
            Direction::Right,
        );
        game.food = Point {
            x: 35,
            y: 35,
        };
        game.tick_counter = tick;
        let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
        let pre_head = game.snake.head();
        let pre_next = Game::calculate_next_head_dir(pre_head, game.snake.direction);
        let pre_food = game.food;
        let pre_obstacles: std::collections::HashSet<Point> = game.obstacles.clone();
        let shuffles_before = game.electro_shuffles;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        assert!(
            game.electro_shuffles > shuffles_before,
            "a magnet shuffle must happen on tick {}",
            tick + 1
        );
        for magnet in &game.electro_magnets {
            assert!(
                !pre_obstacles.contains(magnet),
                "a relocated magnet must not sit on an obstacle: {magnet:?}"
            );
            assert!(
                !pre_body.contains(magnet),
                "a relocated magnet must not sit on the snake: {magnet:?}"
            );
            assert_ne!(
                *magnet, pre_next,
                "a relocated magnet must not sit on the cell the snake is about to enter"
            );
            assert_ne!(*magnet, pre_food, "a relocated magnet must not sit on the food");
            assert!(
                magnet.x.abs_diff(pre_head.x).saturating_add(magnet.y.abs_diff(pre_head.y)) >= 4,
                "a relocated magnet must keep its distance from snake heads: {magnet:?}"
            );
        }
    }
    assert_eq!(game.electro_shuffles, 5);
    assert_eq!(game.electro_magnets.len(), 8, "clamp(40*40/60, 3, 8) = 8");
}

#[test]
fn test_electro_eating_food_grounds_the_nearest_magnet() {
    let mut game = electro_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.electro_magnets.clear();
    pin_ambient_spawners(&mut game);
    park_snake(
        &mut game,
        Point {
            x: 10,
            y: 10,
        },
        Direction::Right,
    );
    game.electro_magnets.insert(Point {
        x: 20,
        y: 10,
    });
    game.food = Point {
        x: 11,
        y: 10,
    };

    // Tick 1: the snake eats the food at (11,10)
    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert_eq!(
        game.electro_grounded.get(&Point {
            x: 20,
            y: 10
        }),
        Some(&Game::ELECTRO_GROUND_TICKS),
        "eating food must ground the nearest magnet for ELECTRO_GROUND_TICKS"
    );

    // Tick 6 with the only magnet grounded: no active magnet, so no pulse
    game.obstacles.clear();
    park_snake(
        &mut game,
        Point {
            x: 10,
            y: 12,
        },
        Direction::Right,
    );
    game.food = Point {
        x: 25,
        y: 25,
    };
    game.tick_counter = 5;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.electro_pulses, 0, "a grounded magnet must not pulse (no active magnets left)");
    assert_eq!(
        game.snake.head(),
        Point {
            x: 11,
            y: 12
        },
        "with no active magnet the snake must move normally, head: {:?}",
        game.snake.head()
    );
    assert!(
        game.electro_grounded
            .get(&Point {
                x: 20,
                y: 10
            })
            .is_some_and(|t| *t < Game::ELECTRO_GROUND_TICKS),
        "the grounded timer must count down every tick"
    );
}

#[test]
fn test_electro_grounded_magnet_reactivates() {
    let mut game = electro_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.obstacles.clear();
    game.electro_magnets.clear();
    game.electro_magnets.insert(Point {
        x: 14,
        y: 14,
    });
    game.electro_grounded.insert(
        Point {
            x: 14,
            y: 14,
        },
        1,
    );
    pin_ambient_spawners(&mut game);

    game.tick_counter = 0; // tick 1: no flip, no shuffle
    game.state = GameState::Playing;
    game.update();

    assert!(
        game.electro_grounded.is_empty(),
        "a grounded magnet with 1 tick left must reactivate on the next tick"
    );
    assert!(
        game.electro_magnets.contains(&Point {
            x: 14,
            y: 14
        }),
        "the magnet itself must stay on the board"
    );
}

#[test]
fn test_electro_scraps_magnets_under_fresh_obstacles() {
    let mut game = electro_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.electro_magnets.clear();
    game.electro_magnets.insert(Point {
        x: 14,
        y: 14,
    });
    // Something else (e.g. a meteor) drops an obstacle on the magnet
    game.obstacles.insert(Point {
        x: 14,
        y: 14,
    });

    game.tick_counter = 0; // tick 1: no shuffle
    game.state = GameState::Playing;
    game.update();

    assert!(
        !game.electro_magnets.contains(&Point {
            x: 14,
            y: 14
        }),
        "a magnet under a fresh obstacle must be scrapped on the next tick"
    );
}

#[test]
fn test_electro_pulse_never_unfair_over_many_ticks() {
    let mut game = electro_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point {
        x: 35,
        y: 35,
    };
    pin_ambient_spawners(&mut game);
    let mut total_pulses = 0u32;

    for tick in 0..60_u32 {
        if (tick + 1).is_multiple_of(24) {
            continue; // shuffle ticks relocate the magnets
        }
        game.state = GameState::Playing;
        park_snake(
            &mut game,
            Point {
                x: 8,
                y: 15,
            },
            Direction::Right,
        );
        game.electro_magnets.clear();
        // Magnet straight below the normal step: both attract (down to
        // (9,16)) and repel (up to (9,14)) have a fair destination, so every
        // pulse tick must actually pulse
        game.electro_magnets.insert(Point {
            x: 9,
            y: 25,
        });
        game.tick_counter = tick;
        let pulses_before = game.electro_pulses;
        let lives_before = game.lives;
        game.update();
        let pulsed = game.electro_pulses > pulses_before;
        total_pulses += game.electro_pulses.saturating_sub(pulses_before);
        if game.lives == 0 {
            // An ambient hazard got the parked snake; top up and carry on
            game.lives = 3;
            game.state = GameState::Playing;
            continue;
        }
        let head = game.snake.head();
        if pulsed && game.lives == lives_before {
            assert!(
                head.x == 9 && (head.y == 14 || head.y == 15 || head.y == 16),
                "tick {tick}: a pulse must land within one cell of the normal step (9,15), got {head:?}"
            );
            assert!(
                !game.obstacles.contains(&head),
                "tick {tick}: the pulse landed on an obstacle at {head:?}"
            );
            assert!(
                head.x > 0 && head.x < game.width - 1 && head.y > 0 && head.y < game.height - 1,
                "tick {tick}: the pulse left the board at {head:?}"
            );
        }
    }

    assert!(
        total_pulses >= 6,
        "expected the magnet to pulse the parked snake on most pulse ticks, got {total_pulses} pulses"
    );
}

#[test]
fn test_electro_bot_survives_the_magnets() {
    let mut game = electro_game(40, 40);
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
        "Autopilot should survive Electro mode (lives: {}, pulses: {}, flips: {}, shuffles: {}, score: {})",
        game.lives,
        game.electro_pulses,
        game.electro_flips,
        game.electro_shuffles,
        game.score
    );
}

#[test]
fn test_electro_persists_through_history_rewind() {
    let mut game = electro_game(30, 30);
    game.electro_pulses = 7;
    game.electro_flips = 3;
    game.electro_shuffles = 2;
    game.electro_attract = false;
    game.electro_magnets.insert(Point {
        x: 12,
        y: 12,
    });
    game.electro_grounded.insert(
        Point {
            x: 13,
            y: 13,
        },
        9,
    );
    game.save_history_state();
    game.electro_pulses = 0;
    game.electro_flips = 0;
    game.electro_shuffles = 0;
    game.electro_attract = true;
    game.electro_magnets.clear();
    game.electro_grounded.clear();

    game.rewind_time();

    assert_eq!(
        game.electro_pulses, 7,
        "electro_pulses must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.electro_flips, 3);
    assert_eq!(game.electro_shuffles, 2);
    assert!(!game.electro_attract);
    assert!(
        game.electro_magnets.contains(&Point {
            x: 12,
            y: 12
        }),
        "electro_magnets must survive a time-rewind through HistoryState"
    );
    assert_eq!(
        game.electro_grounded.get(&Point {
            x: 13,
            y: 13
        }),
        Some(&9),
        "electro_grounded must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_electro_persists_in_save_file() {
    let mut game = electro_game(30, 30);
    game.electro_pulses = 9;
    game.electro_flips = 4;
    game.electro_shuffles = 2;
    game.electro_attract = false;
    let path = std::env::temp_dir().join(format!(
        "electro_save_test_{}_{}.json",
        std::process::id(),
        11_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"electro_pulses\":9")
            && json.contains("\"electro_flips\":4")
            && json.contains("\"electro_shuffles\":2")
            && json.contains("\"electro_attract\":false")
            && json.contains("\"electro_magnets\"")
            && json.contains("\"electro_grounded\""),
        "SaveState JSON must contain the electro fields, got: {}",
        &json[..json.len().min(200)]
    );
}
