use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn crossfire_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Crossfire;
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
    game.portals = Some((Point { x: 27, y: 27 }, Point { x: 28, y: 28 }));
    game.mines.insert(Point { x: 23, y: 1 });
    game.mines.insert(Point { x: 24, y: 1 });
    game.mines.insert(Point { x: 25, y: 1 });
    game.mines.insert(Point { x: 26, y: 1 });
    game.mines.insert(Point { x: 27, y: 1 });
}

// Park the snake at a fixed cell heading right, with the food far away
fn park_snake(game: &mut Game, at: Point) {
    game.snake = Snake::new(at);
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point {
        x: game.width - 3,
        y: game.height - 3,
    };
    game.auto_pilot = false;
}

#[test]
fn test_crossfire_reset_clears_state_and_pins_weather() {
    let mut game = crossfire_game(30, 30);
    game.crossfire_target = Some((7, 9));
    game.crossfire_warn_ticks = 2;
    game.crossfire_beam_ticks = 1;
    game.crossfire_strikes = 5;
    game.weather = Weather::Rain;
    game.reset();

    assert_eq!(game.crossfire_target, None);
    assert_eq!(game.crossfire_warn_ticks, 0);
    assert_eq!(game.crossfire_beam_ticks, 0);
    assert_eq!(game.crossfire_strikes, 0);

    // The mode pins the weather to clear so the telegraph is always visible
    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();
    assert_eq!(game.weather, Weather::Clear);
}

#[test]
fn test_crossfire_schedules_a_fair_cross_on_period_ticks() {
    let mut game = crossfire_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 });

    // update() increments tick_counter first, so tick_counter = 5 makes the
    // next update run tick 6: the first CROSSFIRE_PERIOD tick
    game.tick_counter = 5;
    game.state = GameState::Playing;
    let pre_head = game.snake.head();
    let pre_next = Game::calculate_next_head_dir(pre_head, game.snake.direction);
    game.update();

    assert_eq!(game.tick_counter, 6);
    let (col, row) = game
        .crossfire_target
        .expect("a cross must be scheduled on tick 6");
    assert_eq!(
        game.crossfire_warn_ticks,
        Game::CROSSFIRE_TELEGRAPH_TICKS,
        "the scheduled cross must start in telegraph"
    );
    assert_eq!(game.crossfire_beam_ticks, 0);
    assert_eq!(game.crossfire_strikes, 0, "no beam has fired yet");
    // Fairness: the cross lines never touch the snake's column/row or the cell
    // it is about to enter
    assert_ne!(col, pre_head.x, "the beam column must not be the snake's column");
    assert_ne!(col, pre_next.x, "the beam column must not be the next cell's column");
    assert_ne!(row, pre_head.y, "the beam row must not be the snake's row");
    assert_ne!(row, pre_next.y, "the beam row must not be the next cell's row");
}

#[test]
fn test_crossfire_no_cross_scheduled_off_period() {
    let mut game = crossfire_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 });

    for tick in 0..5_u32 {
        game.tick_counter = tick;
        game.state = GameState::Playing;
        game.update();
        assert_eq!(
            game.crossfire_target, None,
            "no cross may be scheduled before tick 6 (tick {})",
            tick + 1
        );
    }
}

#[test]
fn test_crossfire_telegraph_fires_into_beam_then_expires() {
    let mut game = crossfire_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 15, y: 15 });
    // Pre-arm a cross one telegraph tick away from firing
    game.crossfire_target = Some((5, 25));
    game.crossfire_warn_ticks = 1;
    game.crossfire_beam_ticks = 0;

    game.tick_counter = 0; // tick 1: telegraph hits zero, the beam fires
    game.state = GameState::Playing;
    game.update();
    assert_eq!(game.crossfire_strikes, 1, "the beam must fire once");
    assert_eq!(
        game.crossfire_beam_ticks,
        Game::CROSSFIRE_BEAM_TICKS,
        "the beam must stay active for CROSSFIRE_BEAM_TICKS"
    );
    assert_eq!(game.crossfire_target, Some((5, 25)));
    assert!(
        game.crossfire_beam_cell(Point { x: 5, y: 12 }),
        "a beam column cell must be lethal"
    );
    assert!(
        game.crossfire_beam_cell(Point { x: 20, y: 25 }),
        "a beam row cell must be lethal"
    );
    assert!(
        !game.crossfire_beam_cell(Point { x: 12, y: 12 }),
        "a cell off the cross must be safe"
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the beam must not kill a snake off its lines (lives: {})",
        game.lives
    );

    game.tick_counter = 1; // tick 2: beam counts down
    game.state = GameState::Playing;
    game.update();
    assert_eq!(game.crossfire_beam_ticks, Game::CROSSFIRE_BEAM_TICKS - 1);
    assert_eq!(game.crossfire_strikes, 1);

    game.tick_counter = 2; // tick 3: beam expires, the cross is over
    game.state = GameState::Playing;
    game.update();
    assert_eq!(game.crossfire_beam_ticks, 0);
    assert_eq!(
        game.crossfire_target, None,
        "the cross must clear once the beam expires"
    );
    assert_eq!(game.crossfire_strikes, 1, "the beam fired exactly once");
}

#[test]
fn test_crossfire_beam_kills_a_snake_on_the_line() {
    let mut game = crossfire_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(13);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 });
    // The snake heads right into (11,10): aim the beam column at it. The beam
    // counts down at the start of the tick, so arm it with 2 ticks left: after
    // the countdown it is still active (1) during this tick's movement.
    game.crossfire_target = Some((11, 25));
    game.crossfire_warn_ticks = 0;
    game.crossfire_beam_ticks = 2;
    let lives_before = game.lives;

    game.tick_counter = 1; // tick 2: not a schedule tick
    game.state = GameState::Playing;
    game.update();

    assert!(
        game.lives < lives_before || game.state != GameState::Playing,
        "a snake on the active beam line must die (lives: {} -> {})",
        lives_before,
        game.lives
    );
}

#[test]
fn test_crossfire_telegraph_alone_is_not_lethal() {
    let mut game = crossfire_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(17);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 });
    // A merely telegraphed cross on the snake's next column must not kill yet
    game.crossfire_target = Some((11, 25));
    game.crossfire_warn_ticks = 2;
    game.crossfire_beam_ticks = 0;
    let lives_before = game.lives;

    game.tick_counter = 1; // tick 2
    game.state = GameState::Playing;
    game.update();

    assert!(
        game.lives == lives_before && game.state == GameState::Playing,
        "a telegraphed (not yet fired) cross must not kill (lives: {})",
        game.lives
    );
    assert_eq!(game.crossfire_warn_ticks, 1);
    assert!(
        game.crossfire_danger_cell(Point { x: 11, y: 10 }),
        "a telegraphed cell must be danger for the autopilot"
    );
    assert!(
        !game.crossfire_beam_cell(Point { x: 11, y: 10 }),
        "a telegraphed cell must not be lethal yet"
    );
}

#[test]
fn test_crossfire_fairness_holds_over_many_scheduled_crosses() {
    let mut game = crossfire_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.food = Point { x: 37, y: 37 };
    game.auto_pilot = false;

    for tick in 0..120_u32 {
        game.state = GameState::Playing;
        if game.lives == 0 {
            game.lives = 3;
        }
        park_snake(&mut game, Point { x: 20, y: 20 });
        let pre_head = game.snake.head();
        let pre_next = Game::calculate_next_head_dir(pre_head, game.snake.direction);
        let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
        game.tick_counter = tick;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        // A fresh cross is only scheduled on CROSSFIRE_PERIOD ticks
        if (tick + 1).is_multiple_of(Game::CROSSFIRE_PERIOD)
            && game.crossfire_warn_ticks == Game::CROSSFIRE_TELEGRAPH_TICKS
        {
            let (col, row) = game.crossfire_target.expect("a cross must be scheduled");
            assert!(
                col != pre_head.x && col != pre_next.x && !pre_body.iter().any(|p| p.x == col),
                "tick {tick}: beam column {col} hit a snake column (head {pre_head:?}, next {pre_next:?})"
            );
            assert!(
                row != pre_head.y && row != pre_next.y && !pre_body.iter().any(|p| p.y == row),
                "tick {tick}: beam row {row} hit a snake row (head {pre_head:?}, next {pre_next:?})"
            );
        }
    }
}

#[test]
fn test_crossfire_tick_200_resets_an_active_cross() {
    let mut game = crossfire_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 15, y: 15 });
    game.crossfire_target = Some((5, 25));
    game.crossfire_warn_ticks = 2;
    game.crossfire_beam_ticks = 0;

    game.tick_counter = 199; // next update runs tick 200
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert_eq!(game.crossfire_target, None, "tick 200 must reset the cross");
    assert_eq!(game.crossfire_warn_ticks, 0);
    assert_eq!(game.crossfire_beam_ticks, 0);
}

#[test]
fn test_crossfire_eating_food_dampens_the_cross() {
    let mut game = crossfire_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(23);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 });
    game.food = Point { x: 11, y: 10 }; // right in front of the snake
    game.crossfire_target = Some((5, 25));
    game.crossfire_warn_ticks = 2;
    game.crossfire_beam_ticks = 0;

    game.tick_counter = 1; // tick 2: the snake eats the food
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten");
    assert_eq!(
        game.crossfire_target, None,
        "eating food must dampen the crossfire"
    );
    assert_eq!(game.crossfire_warn_ticks, 0);
    assert_eq!(game.crossfire_beam_ticks, 0);
}

#[test]
fn test_crossfire_autopilot_avoids_danger_lines() {
    let mut game = crossfire_game(30, 30);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 15, y: 15 });
    game.crossfire_target = Some((5, 7));
    game.crossfire_warn_ticks = 2;
    game.crossfire_beam_ticks = 0;

    assert!(
        !game.is_safe_final_p(Point { x: 5, y: 10 }, 1, 1),
        "a telegraphed beam column cell must be rejected by is_safe_final_p"
    );
    assert!(
        !game.is_safe_final_p(Point { x: 10, y: 7 }, 1, 1),
        "a telegraphed beam row cell must be rejected by is_safe_final_p"
    );
    assert!(
        game.is_safe_final_p(Point { x: 10, y: 10 }, 1, 1),
        "a cell off the cross must stay safe"
    );

    game.crossfire_beam_ticks = 2;
    game.crossfire_warn_ticks = 0;
    assert!(
        !game.is_safe_final_p(Point { x: 5, y: 10 }, 1, 1),
        "an active beam column cell must be rejected by is_safe_final_p"
    );
    assert!(
        !game.is_safe_final_p(Point { x: 10, y: 7 }, 1, 1),
        "an active beam row cell must be rejected by is_safe_final_p"
    );
}

#[test]
fn test_crossfire_bot_survives_the_crossfire() {
    let mut game = crossfire_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;

    for _ in 0..250 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
    }

    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "Autopilot should survive Crossfire mode (lives: {}, strikes: {}, score: {})",
        game.lives,
        game.crossfire_strikes,
        game.score
    );
    assert!(
        game.crossfire_strikes > 0,
        "beams should have fired during a 250-tick run"
    );
}

#[test]
fn test_crossfire_persists_through_history_rewind() {
    let mut game = crossfire_game(30, 30);
    game.crossfire_target = Some((8, 12));
    game.crossfire_warn_ticks = 2;
    game.crossfire_beam_ticks = 0;
    game.crossfire_strikes = 7;
    game.save_history_state();
    game.crossfire_target = None;
    game.crossfire_warn_ticks = 0;
    game.crossfire_beam_ticks = 0;
    game.crossfire_strikes = 0;

    game.rewind_time();

    assert_eq!(
        game.crossfire_target,
        Some((8, 12)),
        "crossfire_target must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.crossfire_warn_ticks, 2);
    assert_eq!(game.crossfire_strikes, 7);
}

#[test]
fn test_crossfire_persists_in_save_file() {
    let mut game = crossfire_game(30, 30);
    game.crossfire_target = Some((8, 12));
    game.crossfire_warn_ticks = 2;
    game.crossfire_beam_ticks = 1;
    game.crossfire_strikes = 9;
    let path = std::env::temp_dir().join(format!(
        "crossfire_save_test_{}.json",
        std::process::id()
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"crossfire_target\":[8,12]")
            && json.contains("\"crossfire_warn_ticks\":2")
            && json.contains("\"crossfire_beam_ticks\":1")
            && json.contains("\"crossfire_strikes\":9"),
        "SaveState JSON must contain the crossfire fields, got: {}",
        &json[..json.len().min(200)]
    );
}
