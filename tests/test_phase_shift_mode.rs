use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn phase_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::PhaseShift;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_phase_shift_reset_seeds_gates_solid() {
    let game = phase_game(30, 30);

    assert!(
        !game.phase_gates.is_empty(),
        "Phase Shift mode must seed phase gates on reset"
    );
    assert!(game.phase_solid, "the phase must start solid");
    assert_eq!(game.phase_shifts, 0);
    for gate in &game.phase_gates {
        assert!(
            game.obstacles.contains(gate),
            "solid gates must be mirrored into obstacles, gate {gate:?} is missing"
        );
        assert!(
            !game.snake.body_map.contains_key(gate),
            "a gate must never spawn on the snake"
        );
        assert_ne!(*gate, game.food, "a gate must never spawn on the food");
    }
    // The two gate walls sit at the one-third and two-thirds columns and each
    // keeps a gap of two consecutive rows.
    for col in [30 / 3, 30 * 2 / 3] {
        let rows: Vec<u16> = (1..29)
            .filter(|y| game.phase_gates.contains(&Point { x: col, y: *y }))
            .collect();
        assert!(
            !rows.is_empty(),
            "gate column {col} must have gate cells on a 30x30 board"
        );
        let inner_rows = (1..29u16).count();
        assert!(
            inner_rows - rows.len() >= 2,
            "gate column {col} must keep a gap of at least 2 rows (missing {})",
            inner_rows - rows.len()
        );
    }
}

#[test]
fn test_phase_shift_toggles_every_phase_period() {
    let mut game = phase_game(30, 30);
    game.auto_pilot = false;
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);

    // Tick 12: solid -> ghost. update() increments tick_counter first, so
    // setting it to 11 makes the next update run tick 12.
    game.state = GameState::Playing;
    game.tick_counter = Game::PHASE_PERIOD - 1;
    let gates_before: Vec<Point> = game.phase_gates.iter().copied().collect();
    game.update();
    assert_eq!(game.tick_counter, Game::PHASE_PERIOD);
    assert!(!game.phase_solid, "tick 12 must flip the phase to ghost");
    assert_eq!(game.phase_shifts, 1);
    for gate in &gates_before {
        assert!(
            !game.obstacles.contains(gate),
            "ghosted gates must leave obstacles at tick 12 (gate {gate:?})"
        );
    }

    // Tick 24: ghost -> solid again.
    game.state = GameState::Playing;
    game.tick_counter = 2 * Game::PHASE_PERIOD - 1;
    game.update();
    assert_eq!(game.tick_counter, 2 * Game::PHASE_PERIOD);
    assert!(game.phase_solid, "tick 24 must flip the phase back to solid");
    assert_eq!(game.phase_shifts, 2);
    let solid_gates = game
        .phase_gates
        .iter()
        .filter(|g| game.obstacles.contains(*g))
        .count();
    assert!(
        solid_gates > 0,
        "solid gates must be mirrored back into obstacles at tick 24"
    );
}

#[test]
fn test_phase_shift_never_solidifies_on_snake_or_next_cell() {
    let mut game = phase_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.phase_gates.clear();
    // Park the snake at (8, 15) heading right and wall its next cell (9, 15)
    // as a ghost gate, so a solidify must skip exactly that cell.
    game.snake = Snake::new(Point { x: 8, y: 15 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.phase_solid = false;
    game.phase_gates.insert(Point { x: 9, y: 15 });
    game.phase_gates.insert(Point { x: 8, y: 15 }); // on the snake's head
    game.phase_gates.insert(Point { x: 25, y: 25 }); // on the food
    game.auto_pilot = false;
    let lives_before = game.lives;

    for k in 1..=5_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = Game::PHASE_PERIOD * k - 1;
        game.update();
        assert_eq!(
            game.phase_solid,
            k % 2 == 1,
            "tick {} must be {} (k = {k})",
            Game::PHASE_PERIOD * k,
            if k % 2 == 1 { "solid" } else { "ghost" }
        );
        assert!(
            !game.obstacles.contains(&Point { x: 9, y: 15 }),
            "a gate must never solidify on the cell the snake is about to enter"
        );
        assert!(
            !game.obstacles.contains(&Point { x: 8, y: 15 }),
            "a gate must never solidify on the snake's body"
        );
        assert!(
            !game.obstacles.contains(&Point { x: 25, y: 25 }),
            "a gate must never solidify on the food"
        );
    }

    assert_eq!(
        game.lives, lives_before,
        "the snake must survive every solidify (shifts: {})",
        game.phase_shifts
    );
    assert_eq!(game.phase_shifts, 5);
}

#[test]
fn test_phase_shift_ghost_gate_is_passable() {
    let mut game = phase_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.obstacles.clear();
    game.phase_gates.clear();
    let gate_cell = Point { x: 6, y: 5 };
    game.phase_gates.insert(gate_cell);
    game.phase_solid = false; // ghost phase: the gate is not in obstacles
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = false;
    let lives_before = game.lives;

    // Tick 5 is not a multiple of PHASE_PERIOD, so no flip interferes.
    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.snake.head(), gate_cell, "the snake must pass through a ghost gate");
    assert_eq!(game.lives, lives_before, "walking through a ghost gate must not kill");
}

#[test]
fn test_phase_shift_solid_gate_is_lethal() {
    let mut game = phase_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(13);
    game.obstacles.clear();
    game.phase_gates.clear();
    let gate_cell = Point { x: 6, y: 5 };
    game.phase_gates.insert(gate_cell);
    game.phase_solid = true;
    game.obstacles.insert(gate_cell); // solid gate mirrored into obstacles
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = false;
    let lives_before = game.lives;

    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();

    assert!(
        game.lives < lives_before || game.state != GameState::Playing,
        "entering a solid gate must kill the snake (lives: {} -> {})",
        lives_before,
        game.lives
    );
}

#[test]
fn test_phase_shift_eating_food_disrupts_adjacent_gate() {
    let mut game = phase_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.phase_gates.clear();
    let gate_cell = Point { x: 7, y: 5 };
    game.phase_gates.insert(gate_cell);
    game.phase_solid = true;
    game.obstacles.insert(gate_cell);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.auto_pilot = false;

    // Tick 5 is not a multiple of PHASE_PERIOD, so no phase flip interferes.
    game.state = GameState::Playing;
    game.tick_counter = 4;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.phase_gates.contains(&gate_cell),
        "Eating food in Phase Shift mode should disrupt the adjacent gate"
    );
    assert!(
        !game.obstacles.contains(&gate_cell),
        "The disrupted gate must also leave obstacles"
    );
}

#[test]
fn test_phase_shift_bot_survives_the_phasing_gates() {
    let mut game = phase_game(40, 40);
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
        "Autopilot should survive the phasing gates (lives: {}, shifts: {}, score: {})",
        game.lives,
        game.phase_shifts,
        game.score
    );
    assert!(
        game.phase_shifts >= 10,
        "expected many phase shifts over 160 ticks, got {}",
        game.phase_shifts
    );
    // The autopilot treats gates as permanent walls: it never stands on one.
    for seg in &game.snake.body {
        assert!(
            !game.phase_gates.contains(seg),
            "the bot must never occupy a gate cell (seg {seg:?})"
        );
    }
}

#[test]
fn test_phase_shift_persists_through_history_rewind() {
    let mut game = phase_game(30, 30);
    game.phase_shifts = 7;
    game.phase_solid = false;
    let gates = game.phase_gates.clone();
    game.save_history_state();
    game.phase_shifts = 0;
    game.phase_solid = true;
    game.phase_gates.clear();

    game.rewind_time();

    assert_eq!(
        game.phase_shifts, 7,
        "phase_shifts must survive a time-rewind through HistoryState"
    );
    assert!(
        !game.phase_solid,
        "phase_solid must survive a time-rewind through HistoryState"
    );
    assert_eq!(
        game.phase_gates, gates,
        "phase_gates must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_phase_shift_persists_in_save_file() {
    let mut game = phase_game(30, 30);
    game.phase_shifts = 9;
    game.phase_solid = true;
    let path = std::env::temp_dir().join(format!(
        "phase_shift_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"phase_shifts\":9") && json.contains("\"phase_solid\":true"),
        "SaveState JSON must contain the phase counters, got: {}",
        &json[..json.len().min(200)]
    );
}
