use rand::SeedableRng;
use snake_game::game::{Comet, Difficulty, Game, GameMode, GameState, HeroClass, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn comet_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Comet;
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

// Walk the full diagonal path a comet launched at `start` would take.
fn comet_path(game: &Game, start: Point, dx: i8, dy: i8) -> Vec<Point> {
    let mut cells = Vec::new();
    let mut p = start;
    loop {
        cells.push(p);
        let nx = i32::from(p.x) + i32::from(dx);
        let ny = i32::from(p.y) + i32::from(dy);
        if nx < 0 || ny < 0 || nx >= i32::from(game.width) || ny >= i32::from(game.height) {
            break;
        }
        p = Point {
            x: u16::try_from(nx).unwrap_or(0),
            y: u16::try_from(ny).unwrap_or(0),
        };
    }
    cells
}

#[test]
fn test_comet_reset_clears_state() {
    let game = comet_game(30, 30);

    assert!(game.comets.is_empty());
    assert_eq!(game.comet_launches, 0);
    assert_eq!(game.comet_deflections, 0);
    assert_eq!(game.weather, Weather::Clear);
}

#[test]
fn test_comet_scheduled_launch_every_period() {
    let mut game = comet_game(30, 30);
    game.auto_pilot = true;
    game.rng = rand::rngs::StdRng::seed_from_u64(11);

    // update() increments tick_counter first, so setting it to 18k - 1 makes
    // the next update run tick 18k, which must launch a comet.
    for k in 1..=6_u32 {
        game.state = GameState::Playing;
        game.comets.clear(); // keep the MAX_COMETS cap out of the way
        game.tick_counter = Game::COMET_PERIOD * k - 1;
        let launches_before = game.comet_launches;
        game.update();
        assert_eq!(
            game.tick_counter,
            Game::COMET_PERIOD * k,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.comet_launches > launches_before,
            "a comet must launch on tick {} (launches before: {}, after: {})",
            Game::COMET_PERIOD * k,
            launches_before,
            game.comet_launches
        );
        assert!(!game.comets.is_empty(), "a comet must be in flight");
    }
    assert!(game.comet_launches >= 6);
}

#[test]
fn test_comet_never_launches_through_snake_or_next_cell() {
    let mut game = comet_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.auto_pilot = false;
    game.food = Point { x: 25, y: 25 };
    let lives_before = game.lives;

    for k in 1..=8_u32 {
        game.state = GameState::Playing;
        park_snake(&mut game, 8, 15);
        let body_before: Vec<Point> = game.snake.body.iter().copied().collect();
        let next_before = Point { x: 9, y: 15 };
        game.comets.clear();
        game.tick_counter = Game::COMET_PERIOD * k - 1;
        game.update();

        assert!(
            game.comet_launches >= k,
            "at least one comet must have launched by tick {} (launches: {})",
            Game::COMET_PERIOD * k,
            game.comet_launches
        );
        for comet in &game.comets {
            // The comet moved one cell on its launch tick, so its launch cell
            // is one step back along its diagonal.
            let launch = Point {
                x: u16::try_from(i32::from(comet.position.x) - i32::from(comet.dx)).unwrap_or(0),
                y: u16::try_from(i32::from(comet.position.y) - i32::from(comet.dy)).unwrap_or(0),
            };
            for cell in comet_path(&game, launch, comet.dx, comet.dy) {
                assert!(
                    !body_before.contains(&cell),
                    "comet launched through snake body cell {cell:?} on tick {}",
                    game.tick_counter
                );
                assert_ne!(
                    cell, next_before,
                    "comet launched through the cell the snake was about to enter on tick {}",
                    game.tick_counter
                );
            }
        }
    }

    assert_eq!(
        game.lives, lives_before,
        "the parked snake must never be hit by a fairly launched comet"
    );
}

#[test]
fn test_comet_moves_one_cell_per_tick_and_leaves_a_burning_trail() {
    let mut game = comet_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.obstacles.clear();
    game.comets.clear();
    game.comet_launches = 0;
    game.auto_pilot = false;
    park_snake(&mut game, 15, 25);
    game.comets.push(Comet {
        position: Point { x: 2, y: 5 },
        dx: 1,
        dy: 1,
    });

    game.state = GameState::Playing;
    game.tick_counter = 4; // tick 5: not a scheduled launch tick
    game.update();

    assert_eq!(game.comets.len(), 1);
    assert_eq!(
        game.comets[0].position,
        Point { x: 3, y: 6 },
        "the comet must streak exactly one cell diagonally per tick, got {:?}",
        game.comets[0].position
    );
    assert!(
        game.fire_tiles.contains(&Point { x: 2, y: 5 })
            && game.fire_tiles.contains(&Point { x: 3, y: 6 }),
        "both the launch cell and the head cell must burn, fire_tiles: {:?}",
        game.fire_tiles
    );
    // The burning trail is mirrored into obstacles, so it is lethal
    assert!(game.obstacles.contains(&Point { x: 2, y: 5 }));
    assert!(game.obstacles.contains(&Point { x: 3, y: 6 }));
    assert_eq!(
        game.fire_ttl.get(&Point { x: 3, y: 6 }),
        Some(&Game::COMET_TAIL_TICKS),
        "the head cell's trail timer must be refreshed while the comet occupies it"
    );
}

#[test]
fn test_comet_streaks_off_the_board() {
    let mut game = comet_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.obstacles.clear();
    game.comets.clear();
    game.auto_pilot = false;
    park_snake(&mut game, 15, 25);
    game.comets.push(Comet {
        position: Point { x: 29, y: 29 },
        dx: 1,
        dy: 1,
    });

    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();

    assert!(
        game.comets.is_empty(),
        "the comet must be gone once it streaks off the board, got {:?}",
        game.comets
    );
    assert!(
        game.fire_tiles.contains(&Point { x: 29, y: 29 }),
        "the last in-bounds cell must still burn after the comet leaves"
    );
}

#[test]
fn test_comet_trail_burns_down_and_clears_obstacles() {
    let mut game = comet_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(9);
    game.obstacles.clear();
    game.comets.clear();
    game.auto_pilot = false;
    park_snake(&mut game, 15, 25);
    game.comets.push(Comet {
        position: Point { x: 2, y: 5 },
        dx: 1,
        dy: 1,
    });

    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();
    assert!(!game.fire_tiles.is_empty());

    // Let the whole trail burn out: no comets left to re-ignite anything
    game.comets.clear();
    for ttl in game.fire_ttl.values_mut() {
        *ttl = 1;
    }
    game.tick_counter = 5;
    game.update();

    assert!(
        game.fire_tiles.is_empty(),
        "the trail must burn out once its timer expires, fire_tiles: {:?}",
        game.fire_tiles
    );
    assert!(
        !game.obstacles.contains(&Point { x: 2, y: 5 })
            && !game.obstacles.contains(&Point { x: 3, y: 6 }),
        "burned-out trail cells must stop blocking movement"
    );
}

#[test]
fn test_comet_smashes_obstacles_and_mines() {
    let mut game = comet_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(13);
    game.obstacles.clear();
    game.mines.clear();
    game.comets.clear();
    game.auto_pilot = false;
    park_snake(&mut game, 15, 25);
    game.comets.push(Comet {
        position: Point { x: 2, y: 5 },
        dx: 1,
        dy: 1,
    });
    game.obstacles.insert(Point { x: 3, y: 6 });
    game.mines.insert(Point { x: 4, y: 7 });

    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();
    assert_eq!(
        game.comets[0].position,
        Point { x: 3, y: 6 },
        "the obstacle must not stop the comet"
    );
    assert!(
        game.fire_tiles.contains(&Point { x: 3, y: 6 }),
        "the smashed cell burns as the comet passes through it"
    );
    assert!(game.mines.contains(&Point { x: 4, y: 7 }));

    game.state = GameState::Playing;
    game.tick_counter = 5;
    game.update();
    assert_eq!(game.comets[0].position, Point { x: 4, y: 7 });
    assert!(
        !game.mines.contains(&Point { x: 4, y: 7 }),
        "the comet must smash the mine in its path"
    );
}

#[test]
fn test_comet_tick_200_clears_the_sky() {
    let mut game = comet_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(17);
    game.obstacles.clear();
    game.comets.clear();
    game.auto_pilot = false;
    park_snake(&mut game, 15, 25);
    game.comets.push(Comet {
        position: Point { x: 2, y: 5 },
        dx: 1,
        dy: 1,
    });

    game.state = GameState::Playing;
    game.tick_counter = 199; // tick 200: the sky clears
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.comets.is_empty(),
        "all comets must burn out at tick 200, got {:?}",
        game.comets
    );
}

#[test]
fn test_comet_deflected_by_eating_food() {
    let mut game = comet_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(23);
    game.obstacles.clear();
    game.comets.clear();
    game.auto_pilot = false;
    park_snake(&mut game, 10, 10);
    game.food = Point { x: 11, y: 10 }; // right in front of the head
    game.comets.push(Comet {
        position: Point { x: 2, y: 2 },
        dx: 1,
        dy: 1,
    });

    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();

    assert!(
        game.comets.is_empty(),
        "eating food must deflect the in-flight comet, got {:?}",
        game.comets
    );
    assert_eq!(game.comet_deflections, 1);
}

#[test]
fn test_comet_kills_snake_on_contact() {
    let mut game = comet_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(29);
    game.obstacles.clear();
    game.comets.clear();
    game.auto_pilot = false;
    park_snake(&mut game, 10, 10); // final head this tick: (11, 10)
    game.food = Point { x: 25, y: 25 };
    // Streaks into (11, 10) on this tick
    game.comets.push(Comet {
        position: Point { x: 10, y: 9 },
        dx: 1,
        dy: 1,
    });
    let lives_before = game.lives;

    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();

    assert!(
        game.lives < lives_before || game.state != GameState::Playing,
        "a comet landing on the snake's head must kill it (lives before: {lives_before}, after: {})",
        game.lives
    );
}

#[test]
fn test_comet_lane_blocks_verified_safe_paths() {
    let mut game = comet_game(20, 20);
    game.obstacles.clear();
    game.goblin = None;
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.food = Point { x: 16, y: 10 };
    game.comets.clear();
    game.comets.push(Comet {
        position: Point { x: 12, y: 9 },
        dx: 1,
        dy: 1,
    });

    // (13, 10) is exactly where the comet streaks on the next tick
    assert!(
        !game.is_safe_final_p(Point { x: 13, y: 10 }, 1, 1),
        "is_safe_final_p must reject the comet's next cell"
    );
    assert!(
        !game.is_safe_final_p(Point { x: 15, y: 12 }, 3, 1),
        "is_safe_final_p must reject a cell on the comet's predicted diagonal"
    );
    // A cell off the diagonal stays safe
    assert!(
        game.is_safe_final_p(Point { x: 13, y: 9 }, 1, 1),
        "a cell off the comet's path must stay safe"
    );

    // comet_position_after predicts the diagonal and reports off-board as None
    assert_eq!(
        game.comet_position_after(game.comets[0], 2),
        Some(Point { x: 14, y: 11 })
    );
    assert_eq!(game.comet_position_after(game.comets[0], 20), None);
}

#[test]
fn test_comet_fairness_over_many_ticks_with_autopilot() {
    let mut game = comet_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(77);
    game.auto_pilot = true;
    game.food = Point { x: 25, y: 25 };

    let mut launches = 0u32;
    for tick in 1..=120_u32 {
        game.state = GameState::Playing;
        game.tick_counter = tick - 1;
        game.update();
        launches = game.comet_launches;
        if game.lives == 0 || game.state != GameState::Playing {
            break;
        }
    }

    assert!(
        launches >= 6,
        "expected at least 6 scheduled launches over 120 ticks, got {launches}"
    );
    assert!(
        game.lives > 0,
        "the autopilot must survive the comet storm (lives: {})",
        game.lives
    );
}
