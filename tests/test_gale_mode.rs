use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Meteor, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn gale_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Gale;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while testing
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

// Block the ambient portal/mine spawners so single-tick gust tests are
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

// Park the snake at `head` heading `dir` so the next update is deterministic
fn park_snake(game: &mut Game, head: Point, dir: Direction) {
    game.snake = Snake::new(head);
    game.snake.direction = dir;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(dir);
}

#[test]
fn test_gale_reset_initializes_the_wind() {
    let game = gale_game(30, 30);

    assert_eq!(game.gale_pushes, 0);
    assert_eq!(game.gale_shifts, 0);
    assert_eq!(game.gale_braces, 0);
    assert!(
        matches!(
            game.gale_dir,
            Direction::Up | Direction::Down | Direction::Left | Direction::Right
        ),
        "the wind must start in one of the four directions, got {:?}",
        game.gale_dir
    );
}

#[test]
fn test_gale_wind_pushes_the_snake_one_cell() {
    let mut game = gale_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.gale_dir = Direction::Right;
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0; // tick 1: no scheduled shift (GALE_PERIOD = 18)
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 1);
    assert_eq!(game.gale_pushes, 1, "the wind must gust the snake once");
    assert_eq!(
        game.snake.head(),
        Point { x: 12, y: 10 },
        "move to (11,10) plus one wind cell to (12,10) (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a gust must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_gale_wind_pushes_in_any_direction() {
    // (wind direction, parked direction, expected head after the tick); the
    // snake parks moving along the wind so the gust target is never its own
    // just-vacated head cell
    for (wind, park_dir, expected) in [
        (Direction::Right, Direction::Right, Point { x: 12, y: 10 }),
        (Direction::Left, Direction::Left, Point { x: 8, y: 10 }),
        (Direction::Up, Direction::Right, Point { x: 11, y: 9 }),
        (Direction::Down, Direction::Right, Point { x: 11, y: 11 }),
    ] {
        let mut game = gale_game(30, 30);
        game.rng = rand::rngs::StdRng::seed_from_u64(42);
        game.obstacles.clear();
        pin_ambient_spawners(&mut game);
        park_snake(&mut game, Point { x: 10, y: 10 }, park_dir);
        game.gale_dir = wind;
        game.food = Point { x: 25, y: 25 };
        game.auto_pilot = false;

        game.tick_counter = 0;
        game.state = GameState::Playing;
        game.update();

        assert_eq!(
            game.snake.head(),
            expected,
            "wind {wind:?} must push the head one cell along the wind (head: {:?})",
            game.snake.head()
        );
        assert_eq!(game.gale_pushes, 1);
        assert!(game.lives > 0, "a gust must never kill (wind {wind:?})");
    }
}

#[test]
fn test_gale_wind_fizzles_at_the_wall() {
    let mut game = gale_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    // Head to (28,10): the last playable column; the gust target (29,10) is the wall
    park_snake(&mut game, Point { x: 27, y: 10 }, Direction::Right);
    game.gale_dir = Direction::Right;
    game.food = Point { x: 5, y: 5 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 28, y: 10 },
        "the gust must fizzle at the wall and the snake must stop one cell short (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.gale_pushes, 0, "no gust may be counted at the wall");
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the wall gust must never kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_gale_wind_never_pushes_into_an_obstacle() {
    let mut game = gale_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.gale_dir = Direction::Right;
    game.obstacles.insert(Point { x: 12, y: 10 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "the gust must fizzle before the obstacle (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.gale_pushes, 0);
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the obstacle gust must never kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_gale_wind_never_pushes_into_a_mine() {
    let mut game = gale_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.gale_dir = Direction::Right;
    game.mines.insert(Point { x: 12, y: 10 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "the gust must fizzle before the mine (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.gale_pushes, 0);
    assert!(game.lives > 0, "the mine gust must never kill");
}

#[test]
fn test_gale_wind_never_pushes_into_another_snake() {
    let mut game = gale_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.gale_dir = Direction::Right;
    // A bot sits exactly where the gust would push the player
    let mut bot = Snake::new(Point { x: 12, y: 10 });
    bot.direction = Direction::Up;
    game.bots.push(bot);
    game.bots_autopilot_paths.push(Vec::new());
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "the gust must fizzle instead of pushing into the bot (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.gale_pushes <= 1,
        "the player must not be gusted into the bot (only the bot itself may be pushed)"
    );
    assert!(game.lives > 0, "the gust must never kill the player");
    assert!(!game.bots.is_empty(), "the bot must survive the tick");
}

#[test]
fn test_gale_wind_never_pushes_into_a_meteor_lane() {
    let mut game = gale_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.gale_dir = Direction::Right;
    // A meteor is falling down column 12; (12,10) is in its lane
    game.meteors.push(Meteor {
        position: Point { x: 12, y: 5 },
        timer: 0,
        vx: 0,
    });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "the gust must fizzle instead of dropping the snake into the meteor lane (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.gale_pushes, 0);
    assert!(game.lives > 0, "the meteor-lane gust must never kill");
}

#[test]
fn test_gale_scheduled_shift_changes_the_wind_every_period() {
    let mut game = gale_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = false;
    pin_ambient_spawners(&mut game);

    // update() increments tick_counter first, so setting it to GALE_PERIOD-1
    // makes the next update run tick GALE_PERIOD, where the shift fires
    let mut prev_dir = game.gale_dir;
    for k in 1..=4_u32 {
        let tick = k * Game::GALE_PERIOD - 1;
        game.state = GameState::Playing;
        park_snake(&mut game, Point { x: 20, y: 20 }, Direction::Right);
        game.gale_dir = prev_dir;
        game.food = Point { x: 35, y: 35 };
        game.tick_counter = tick;
        let shifts_before = game.gale_shifts;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        assert_eq!(
            game.gale_shifts,
            shifts_before + 1,
            "a wind shift must happen on tick {}",
            tick + 1
        );
        assert_ne!(
            game.gale_dir, prev_dir,
            "the wind must turn to a new direction on tick {}",
            tick + 1
        );
        assert!(game.lives > 0, "the wind shift must never kill (lives: {})", game.lives);
        prev_dir = game.gale_dir;
    }
    assert_eq!(game.gale_shifts, 4);
}

#[test]
fn test_gale_no_shift_between_periods() {
    let mut game = gale_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    pin_ambient_spawners(&mut game);
    game.gale_dir = Direction::Right;

    // Ticks 1..GALE_PERIOD-1: no shift may fire
    for tick in 0..Game::GALE_PERIOD - 1 {
        game.state = GameState::Playing;
        park_snake(&mut game, Point { x: 20, y: 20 }, Direction::Right);
        game.gale_dir = Direction::Right;
        game.food = Point { x: 35, y: 35 };
        game.tick_counter = tick;
        game.update();
        assert_eq!(game.gale_dir, Direction::Right, "no shift before tick {}", tick + 1);
        assert_eq!(game.gale_shifts, 0);
        if game.lives == 0 {
            game.lives = 3;
        }
    }
}

#[test]
fn test_gale_eating_food_grants_a_brace_and_a_braced_snake_resists_the_wind() {
    let mut game = gale_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.gale_dir = Direction::Right;
    game.food = Point { x: 11, y: 10 }; // right in front of the snake
    // An obstacle behind the food makes the gust of the eating tick fizzle,
    // so the head rests on the food cell and eats it
    game.obstacles.insert(Point { x: 12, y: 10 });
    game.auto_pilot = false;

    // Tick 1: the snake moves onto the food at (11,10), eats it, and braces
    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert_eq!(
        game.gale_braces,
        Game::GALE_BRACE_TICKS,
        "eating food must brace the snake for GALE_BRACE_TICKS"
    );
    assert_eq!(game.gale_pushes, 0, "the gust fizzled at the obstacle");
    assert_eq!(game.snake.head(), Point { x: 11, y: 10 });

    // Tick 2: braced, so the wind cannot push the snake
    game.obstacles.clear();
    park_snake(&mut game, Point { x: 14, y: 10 }, Direction::Right);
    game.gale_dir = Direction::Right;
    game.food = Point { x: 25, y: 25 };
    game.tick_counter = 1;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 15, y: 10 },
        "a braced snake must resist the gust (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.gale_pushes, 0, "a braced snake must not be pushed");
    assert_eq!(
        game.gale_braces,
        Game::GALE_BRACE_TICKS - 1,
        "the brace timer must count down every tick"
    );
    assert!(game.lives > 0);
}

#[test]
fn test_gale_brace_expires_and_the_wind_pushes_again() {
    let mut game = gale_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.gale_dir = Direction::Right;
    game.gale_braces = 2;
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    // Tick 1: brace counts down to 1, still braced -> no push
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();
    assert_eq!(game.gale_braces, 1);
    assert_eq!(game.gale_pushes, 0);
    assert_eq!(game.snake.head(), Point { x: 11, y: 10 });

    // Tick 2: brace counts down to 0, the gust fires again
    park_snake(&mut game, Point { x: 10, y: 12 }, Direction::Right);
    game.tick_counter = 1;
    game.state = GameState::Playing;
    game.update();
    assert_eq!(game.gale_braces, 0);
    assert_eq!(game.gale_pushes, 1, "once the brace expires the wind must push again");
    assert_eq!(game.snake.head(), Point { x: 12, y: 12 });
    assert!(game.lives > 0);
}

#[test]
fn test_gale_gust_never_unfair_over_many_ticks() {
    let mut game = gale_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 35, y: 35 };
    pin_ambient_spawners(&mut game);
    let mut total_pushes = 0u32;

    // Park the snake facing into the wind before every update (avoiding the
    // shift ticks, where the wind direction changes) and make sure the gust
    // machine can never push it into a wall, a hazard, or itself
    for tick in 0..60_u32 {
        if (tick + 1).is_multiple_of(Game::GALE_PERIOD) {
            continue; // shift ticks change the wind direction
        }
        game.state = GameState::Playing;
        park_snake(&mut game, Point { x: 8, y: 15 }, Direction::Right);
        game.gale_dir = Direction::Right;
        game.food = Point { x: 35, y: 35 };
        game.tick_counter = tick;
        let pushes_before = game.gale_pushes;
        let lives_before = game.lives;
        game.update();
        let pushed = game.gale_pushes > pushes_before;
        total_pushes += game.gale_pushes.saturating_sub(pushes_before);
        if game.lives == 0 {
            // An ambient hazard (e.g. a meteor) got the parked snake; top up
            // and carry on - the gust assertions below only apply to ticks
            // where a gust actually happened and the snake survived.
            game.lives = 3;
            game.state = GameState::Playing;
            continue;
        }
        let head = game.snake.head();
        if pushed && game.lives == lives_before {
            assert!(
                head.y == 15 && head.x > 8 && head.x <= 10,
                "tick {tick}: a gust must land one cell past the move on the wind line, got {head:?}"
            );
            assert!(
                !game.obstacles.contains(&head),
                "tick {tick}: the gust landed on an obstacle at {head:?}"
            );
            assert!(
                !game.mines.contains(&head),
                "tick {tick}: the gust landed on a mine at {head:?}"
            );
            assert!(
                head.x > 0 && head.x < game.width - 1 && head.y > 0 && head.y < game.height - 1,
                "tick {tick}: the gust left the board at {head:?}"
            );
        }
    }

    assert!(
        total_pushes >= 40,
        "expected the wind to gust the parked snake on most ticks, got {total_pushes} pushes"
    );
}

#[test]
fn test_gale_bot_survives_the_wind() {
    let mut game = gale_game(40, 40);
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
        "Autopilot should survive Gale mode (lives: {}, pushes: {}, shifts: {}, score: {})",
        game.lives,
        game.gale_pushes,
        game.gale_shifts,
        game.score
    );
    assert!(game.gale_pushes > 0, "the wind should have pushed the bot at least once");
    assert!(game.gale_shifts > 0, "the wind should have shifted at least once in 200 ticks");
}

#[test]
fn test_gale_persists_through_history_rewind() {
    let mut game = gale_game(30, 30);
    game.gale_dir = Direction::Left;
    game.gale_pushes = 7;
    game.gale_shifts = 3;
    game.gale_braces = 5;
    game.save_history_state();
    game.gale_dir = Direction::Up;
    game.gale_pushes = 0;
    game.gale_shifts = 0;
    game.gale_braces = 0;

    game.rewind_time();

    assert_eq!(game.gale_dir, Direction::Left, "gale_dir must survive a time-rewind");
    assert_eq!(
        game.gale_pushes, 7,
        "gale_pushes must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.gale_shifts, 3);
    assert_eq!(game.gale_braces, 5);
}

#[test]
fn test_gale_persists_in_save_file() {
    let mut game = gale_game(30, 30);
    game.gale_dir = Direction::Down;
    game.gale_pushes = 9;
    game.gale_shifts = 4;
    game.gale_braces = 6;
    let path = std::env::temp_dir().join(format!(
        "gale_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"gale_dir\":\"Down\"")
            && json.contains("\"gale_pushes\":9")
            && json.contains("\"gale_shifts\":4")
            && json.contains("\"gale_braces\":6"),
        "SaveState JSON must contain the gale fields, got: {}",
        &json[..json.len().min(200)]
    );
}
