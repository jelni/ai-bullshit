use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn crumble_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Crumble;
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

// Block the ambient portal/mine spawners so single-tick tests are
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
}

#[test]
fn test_crumble_reset_seeds_cracks_fairly() {
    let game = crumble_game(30, 30);

    assert_eq!(game.crumble_collapses, 0);
    assert_eq!(game.crumble_cracks, 0);
    // crack count = clamp(30*30/50, 4, 10) = 10
    assert_eq!(game.crumble_tiles.len(), 10);
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    for crack in &game.crumble_tiles {
        assert!(
            !game.obstacles.contains(crack),
            "a crack must never sit on an obstacle: {crack:?}"
        );
        assert!(
            !game.snake.body_map.contains_key(crack),
            "a crack must never sit on the snake: {crack:?}"
        );
        assert_ne!(*crack, game.food, "a crack must never sit on the food");
        assert!(
            crack.x.abs_diff(spawn.x).saturating_add(crack.y.abs_diff(spawn.y)) >= 5,
            "a crack must spawn at least 5 cells from the spawn point: {crack:?}"
        );
    }
}

#[test]
fn test_crumble_crossing_a_crack_collapses_it_behind_the_head() {
    let mut game = crumble_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.crumble_tiles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(
        &mut game,
        Point {
            x: 10,
            y: 10,
        },
        Direction::Right,
    );
    // The head starts the tick standing on the crack at (10,10)
    game.crumble_tiles.insert(Point {
        x: 10,
        y: 10,
    });
    game.food = Point {
        x: 25,
        y: 25,
    };
    game.auto_pilot = false;

    game.tick_counter = 0; // tick 1: no scheduled cracking
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 1);
    assert_eq!(game.crumble_collapses, 1, "the crack under the departing head must collapse");
    assert!(
        !game.crumble_tiles.contains(&Point {
            x: 10,
            y: 10
        }),
        "a collapsed crack must leave crumble_tiles"
    );
    assert!(
        game.obstacles.contains(&Point {
            x: 10,
            y: 10
        }),
        "a collapsed crack must become a permanent pit (obstacle)"
    );
    assert_eq!(
        game.snake.head(),
        Point {
            x: 11,
            y: 10
        },
        "the head must move off the crack normally (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "collapsing behind the head must never kill the mover (lives: {})",
        game.lives
    );
}

#[test]
fn test_crumble_collapse_skipped_while_another_snake_enters_the_cell() {
    let mut game = crumble_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.obstacles.clear();
    game.crumble_tiles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(
        &mut game,
        Point {
            x: 10,
            y: 10,
        },
        Direction::Right,
    );
    game.crumble_tiles.insert(Point {
        x: 10,
        y: 10,
    });
    game.food = Point {
        x: 25,
        y: 25,
    };
    game.auto_pilot = false;
    // A bot heading right from (9,10) is about to enter the crack cell (10,10)
    let mut bot = Snake::new(Point {
        x: 9,
        y: 10,
    });
    bot.direction = Direction::Right;
    bot.direction_queue.push_back(Direction::Right);
    game.bots.push(bot);

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.crumble_collapses, 0,
        "a crack must not collapse while another snake is about to enter its cell"
    );
    assert!(
        game.crumble_tiles.contains(&Point {
            x: 10,
            y: 10
        }),
        "the crack must stay live instead of becoming a pit"
    );
    assert!(
        !game.obstacles.contains(&Point {
            x: 10,
            y: 10
        }),
        "the crack must not have become an obstacle"
    );
}

#[test]
fn test_crumble_scheduled_cracks_every_8_ticks_stay_fair() {
    let mut game = crumble_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = false;
    pin_ambient_spawners(&mut game);

    // update() increments tick_counter first, so 7 makes the next update run
    // tick 8. The cracking fires at the start of that tick, so the fairness
    // guarantees must be checked against the pre-update board state.
    // The reset seeds clamp(40*40/50, 4, 10) = 10 cracks and the cap is
    // clamp(40*40/40, 6, 16) = 16, so the first three cracking ticks (8, 16,
    // 24) each seed CRUMBLE_NEW_PER_PERIOD (2) fresh cracks and the later
    // ones (32, 40) find no room left.
    for (i, tick) in [7_u32, 15, 23, 31, 39].iter().enumerate() {
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
        game.tick_counter = *tick;
        let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
        let pre_head = game.snake.head();
        let pre_next =
            snake_game::game::Game::calculate_next_head_dir(pre_head, game.snake.direction);
        let pre_food = game.food;
        let pre_obstacles: std::collections::HashSet<Point> = game.obstacles.clone();
        let cracks_before = game.crumble_cracks;
        game.update();
        // The food can be relocated mid-tick by ambient systems (e.g. the
        // goblin), in which case the pre-update food cell was legitimately
        // free when the cracks were seeded.
        let food_moved = game.food != pre_food;
        assert_eq!(game.tick_counter, tick + 1);
        if i < 3 {
            assert!(
                game.crumble_cracks > cracks_before,
                "fresh cracks must be seeded on tick {}",
                tick + 1
            );
        } else {
            assert_eq!(
                game.crumble_cracks,
                cracks_before,
                "with the cap reached no new cracks may be seeded on tick {}",
                tick + 1
            );
        }
        assert!(
            game.crumble_tiles.len() <= 16,
            "the crack cap clamp(40*40/40, 6, 16) = 16 must hold, got {}",
            game.crumble_tiles.len()
        );
        for crack in &game.crumble_tiles {
            assert!(
                !pre_obstacles.contains(crack),
                "a new crack must not sit on an obstacle: {crack:?}"
            );
            assert!(!pre_body.contains(crack), "a new crack must not sit on the snake: {crack:?}");
            assert_ne!(
                *crack, pre_next,
                "a new crack must not sit on the cell the snake is about to enter"
            );
            assert!(
                food_moved || *crack != pre_food,
                "a new crack must not sit on the food: {crack:?}"
            );
            assert!(
                crack.x.abs_diff(pre_head.x).saturating_add(crack.y.abs_diff(pre_head.y)) >= 3,
                "a new crack must keep its distance from snake heads: {crack:?}"
            );
        }
    }
    assert_eq!(game.crumble_cracks, 6, "3 periods with room x CRUMBLE_NEW_PER_PERIOD (2)");
}

#[test]
fn test_crumble_cap_is_respected() {
    let mut game = crumble_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(
        &mut game,
        Point {
            x: 15,
            y: 15,
        },
        Direction::Right,
    );
    game.food = Point {
        x: 25,
        y: 25,
    };
    game.auto_pilot = false;
    // Fill the board to the cap: clamp(30*30/40, 6, 16) = 16
    game.crumble_tiles.clear();
    for x in 2..18 {
        game.crumble_tiles.insert(Point {
            x,
            y: 3,
        });
    }
    assert_eq!(game.crumble_tiles.len(), 16);

    game.tick_counter = 7; // next update runs tick 8 (a cracking tick)
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 8);
    assert!(
        game.crumble_tiles.len() <= 16,
        "the cap must hold on a cracking tick, got {}",
        game.crumble_tiles.len()
    );
    assert_eq!(game.crumble_cracks, 0, "with no room left no new cracks may be counted");
}

#[test]
fn test_crumble_cracks_fill_in_at_tick_200() {
    let mut game = crumble_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.obstacles.clear();
    game.crumble_tiles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(
        &mut game,
        Point {
            x: 15,
            y: 15,
        },
        Direction::Right,
    );
    game.food = Point {
        x: 25,
        y: 25,
    };
    game.auto_pilot = false;
    game.crumble_tiles.insert(Point {
        x: 5,
        y: 5,
    });
    game.crumble_tiles.insert(Point {
        x: 6,
        y: 5,
    });
    game.crumble_tiles.insert(Point {
        x: 7,
        y: 5,
    });

    game.tick_counter = 199; // next update runs tick 200 of the 250-tick cycle
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.crumble_tiles.is_empty(),
        "at tick 200 of the cycle all remaining cracks must fill in"
    );
    for p in [
        Point {
            x: 5,
            y: 5,
        },
        Point {
            x: 6,
            y: 5,
        },
        Point {
            x: 7,
            y: 5,
        },
    ] {
        assert!(!game.obstacles.contains(&p), "a filled-in crack must never become a pit: {p:?}");
    }
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the fill-in must never kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_crumble_eating_food_reinforces_adjacent_cracks() {
    let mut game = crumble_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.crumble_tiles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(
        &mut game,
        Point {
            x: 10,
            y: 10,
        },
        Direction::Right,
    );
    game.food = Point {
        x: 11,
        y: 10,
    };
    // Adjacent to the eaten cell (11,10): (11,9) and (12,10); (20,20) is far away
    game.crumble_tiles.insert(Point {
        x: 11,
        y: 9,
    });
    game.crumble_tiles.insert(Point {
        x: 12,
        y: 10,
    });
    game.crumble_tiles.insert(Point {
        x: 20,
        y: 20,
    });
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert_eq!(
        game.snake.head(),
        Point {
            x: 11,
            y: 10
        },
        "the head must be on the eaten cell (head: {:?})",
        game.snake.head()
    );
    assert!(
        !game.crumble_tiles.contains(&Point {
            x: 11,
            y: 9
        }) && !game.crumble_tiles.contains(&Point {
            x: 12,
            y: 10
        }),
        "eating food must reinforce the cracks adjacent to the eaten cell"
    );
    assert!(
        game.crumble_tiles.contains(&Point {
            x: 20,
            y: 20
        }),
        "far-away cracks must be untouched"
    );
    for p in [
        Point {
            x: 11,
            y: 9,
        },
        Point {
            x: 12,
            y: 10,
        },
    ] {
        assert!(!game.obstacles.contains(&p), "a reinforced crack must never become a pit: {p:?}");
    }
}

#[test]
fn test_crumble_autopilot_treats_cracks_as_walls() {
    let mut game = crumble_game(30, 30);
    game.obstacles.clear();
    game.crumble_tiles.clear();
    game.crumble_tiles.insert(Point {
        x: 12,
        y: 12,
    });

    assert!(
        !game.is_safe_final_p(
            Point {
                x: 12,
                y: 12
            },
            1,
            1
        ),
        "in Crumble mode a crack must be a hard wall for the verified-safe check"
    );
    assert!(
        game.is_safe_final_p(
            Point {
                x: 13,
                y: 12
            },
            1,
            1
        ),
        "a plain free cell must stay safe"
    );

    game.mode = GameMode::SinglePlayer;
    assert!(
        game.is_safe_final_p(
            Point {
                x: 12,
                y: 12
            },
            1,
            1
        ),
        "outside Crumble mode the same cell must not be blocked by the crack set"
    );
}

#[test]
fn test_crumble_bot_survives_the_collapsing_floor() {
    let mut game = crumble_game(40, 40);
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
        "Autopilot should survive Crumble mode (lives: {}, collapses: {}, cracks: {}, score: {})",
        game.lives,
        game.crumble_collapses,
        game.crumble_cracks,
        game.score
    );
}

#[test]
fn test_crumble_persists_through_history_rewind() {
    let mut game = crumble_game(30, 30);
    game.crumble_collapses = 7;
    game.crumble_cracks = 3;
    game.crumble_tiles.insert(Point {
        x: 12,
        y: 12,
    });
    game.save_history_state();
    game.crumble_collapses = 0;
    game.crumble_cracks = 0;
    game.crumble_tiles.clear();

    game.rewind_time();

    assert_eq!(
        game.crumble_collapses, 7,
        "crumble_collapses must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.crumble_cracks, 3);
    assert!(
        game.crumble_tiles.contains(&Point {
            x: 12,
            y: 12
        }),
        "crumble_tiles must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_crumble_persists_in_save_file() {
    let mut game = crumble_game(30, 30);
    game.crumble_collapses = 9;
    game.crumble_cracks = 4;
    let path = std::env::temp_dir().join(format!(
        "crumble_save_test_{}_{}.json",
        std::process::id(),
        4_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"crumble_collapses\":9")
            && json.contains("\"crumble_cracks\":4")
            && json.contains("\"crumble_tiles\""),
        "SaveState JSON must contain the crumble fields, got: {}",
        &json[..json.len().min(200)]
    );
}
