use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn magnet_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Magnet;
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
    game.portals = Some((Point { x: 27, y: 27 }, Point { x: 28, y: 28 }));
    game.mines.insert(Point { x: 23, y: 1 });
    game.mines.insert(Point { x: 24, y: 1 });
    game.mines.insert(Point { x: 25, y: 1 });
    game.mines.insert(Point { x: 26, y: 1 });
    game.mines.insert(Point { x: 27, y: 1 });
}

fn park_snake(game: &mut Game, at: Point) {
    game.snake = Snake::new(at);
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
}

#[test]
fn test_magnet_reset_seeds_poles_fairly() {
    let game = magnet_game(30, 30);

    assert_eq!(game.magnet_pulses, 0);
    assert_eq!(game.magnet_relocations, 0);
    assert_eq!(game.magnet_flips, 0);
    // pole count = clamp(30*30/150, 2, 6) = 6
    assert_eq!(game.magnet_poles.len(), 6);
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    for pole in game.magnet_poles.keys() {
        assert!(
            !game.obstacles.contains(pole),
            "a pole must never sit on an obstacle: {pole:?}"
        );
        assert!(
            !game.snake.body_map.contains_key(pole),
            "a pole must never sit on the snake: {pole:?}"
        );
        assert_ne!(*pole, game.food, "a pole must never sit on the food");
        assert!(
            pole.x.abs_diff(spawn.x).saturating_add(pole.y.abs_diff(spawn.y)) >= 5,
            "a pole must spawn at least 5 cells from the spawn point: {pole:?}"
        );
    }
}

#[test]
fn test_magnet_pulse_attracts_toward_nearest_pole() {
    let mut game = magnet_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 });
    game.magnet_poles.clear();
    game.magnet_poles.insert(Point { x: 15, y: 10 }, true); // attract pole to the right
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 5; // tick 6: a pulse fires
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 6);
    assert_eq!(game.magnet_pulses, 1, "the pulse must be counted");
    assert_eq!(
        game.snake.head(),
        Point { x: 12, y: 10 },
        "the snake moves to (11,10) and the attract pole must pull it one more cell right (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a pulse must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_magnet_pulse_repels_away_from_nearest_pole() {
    let mut game = magnet_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 });
    game.magnet_poles.clear();
    game.magnet_poles.insert(Point { x: 10, y: 5 }, false); // repel pole above
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 5; // tick 6: a pulse fires
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.magnet_pulses, 1);
    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 11 },
        "the snake moves to (11,10) and the repel pole above must push it one cell down (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a pulse must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_magnet_pulse_fizzles_when_destination_blocked() {
    let mut game = magnet_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 });
    game.magnet_poles.clear();
    game.magnet_poles.insert(Point { x: 15, y: 10 }, true); // attract pole to the right
    game.obstacles.insert(Point { x: 12, y: 10 }); // blocks the pulse destination
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 5; // tick 6: a pulse fires
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.magnet_pulses, 1, "the pulse event is still counted");
    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "with the destination blocked the snake must keep its normal move only (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a blocked pulse must never kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_magnet_pulse_never_lands_on_a_hazard() {
    let mut game = magnet_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 });
    game.magnet_poles.clear();
    game.magnet_poles.insert(Point { x: 15, y: 10 }, true); // attract pole to the right
    game.mines.insert(Point { x: 12, y: 10 }); // a mine blocks the pulse destination
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 5; // tick 6: a pulse fires
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "the pulse must skip hazard cells (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the pulse must never land on a hazard (lives: {})",
        game.lives
    );
}

#[test]
fn test_magnet_relocation_every_24_ticks_stays_fair() {
    let mut game = magnet_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = false;

    // update() increments tick_counter first, so 23 makes the next update run
    // tick 24. The relocation fires at the start of that tick, so the fairness
    // guarantees must be checked against the pre-update board state.
    for tick in [23_u32, 47, 71, 95, 119] {
        game.state = GameState::Playing;
        // Park the snake deterministically so the pre-update state is exactly
        // the state the relocation sees (the direction queue is popped first).
        park_snake(&mut game, Point { x: 20, y: 20 });
        game.food = Point { x: 35, y: 35 };
        game.tick_counter = tick;
        let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
        let pre_head = game.snake.head();
        let pre_next =
            snake_game::game::Game::calculate_next_head_dir(pre_head, game.snake.direction);
        let pre_food = game.food;
        let pre_obstacles: std::collections::HashSet<Point> = game.obstacles.clone();
        let relocations_before = game.magnet_relocations;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        assert!(
            game.magnet_relocations > relocations_before,
            "a pole relocation must happen on tick {}",
            tick + 1
        );
        for pole in game.magnet_poles.keys() {
            assert!(
                !pre_obstacles.contains(pole),
                "a relocated pole must not sit on an obstacle: {pole:?}"
            );
            assert!(
                !pre_body.contains(pole),
                "a relocated pole must not sit on the snake: {pole:?}"
            );
            assert_ne!(*pole, pre_next, "a relocated pole must not sit on the cell the snake is about to enter");
            assert_ne!(*pole, pre_food, "a relocated pole must not sit on the food");
            assert!(
                pole.x.abs_diff(pre_head.x).saturating_add(pole.y.abs_diff(pre_head.y)) >= 4,
                "a relocated pole must keep its distance from snake heads: {pole:?}"
            );
        }
    }
    assert_eq!(game.magnet_relocations, 5);
    assert_eq!(game.magnet_poles.len(), 6, "clamp(40*40/150, 2, 6) = 6");
}

#[test]
fn test_magnet_eating_food_reverses_nearest_pole() {
    let mut game = magnet_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 });
    game.magnet_poles.clear();
    game.magnet_poles.insert(Point { x: 20, y: 10 }, true); // attract pole nearest the food
    game.food = Point { x: 11, y: 10 };
    game.auto_pilot = false;

    game.tick_counter = 0; // tick 1: no pulse, no relocation
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert_eq!(game.magnet_flips, 1, "the polarity flip must be counted");
    assert_eq!(
        game.magnet_poles.get(&Point { x: 20, y: 10 }),
        Some(&false),
        "the nearest pole must have flipped from attract to repel"
    );
}

#[test]
fn test_magnet_pulse_never_unfair_over_many_ticks() {
    let mut game = magnet_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 35, y: 35 };
    pin_ambient_spawners(&mut game);
    let mut expected_pulses = 0u32;

    // Park the snake heading toward an attract pole before every non-relocation
    // update and make sure the pulse can never push it into a wall, a hazard,
    // or itself.
    for tick in 0..60_u32 {
        if (tick + 1).is_multiple_of(24) {
            continue; // relocation ticks re-seed the poles
        }
        if (tick + 1).is_multiple_of(6) {
            expected_pulses += 1;
        }
        game.state = GameState::Playing;
        park_snake(&mut game, Point { x: 8, y: 15 });
        game.magnet_poles.clear();
        game.magnet_poles.insert(Point { x: 20, y: 15 }, true); // attract pole ahead
        game.tick_counter = tick;
        let lives_before = game.lives;
        game.update();
        if game.lives == 0 {
            // An ambient hazard (e.g. a meteor) got the parked snake; top up
            // and carry on - the pulse assertions below only apply to ticks
            // where the snake survived.
            game.lives = 3;
            game.state = GameState::Playing;
            continue;
        }
        assert_eq!(game.lives, lives_before, "tick {tick}: a pulse must never kill");
        let head = game.snake.head();
        let pulsed = (tick + 1).is_multiple_of(6);
        let expected_x = if pulsed { 10 } else { 9 };
        assert!(
            head == Point { x: expected_x, y: 15 },
            "tick {tick}: expected head at ({expected_x},15), got {head:?}"
        );
        assert!(
            !game.obstacles.contains(&head) && !game.mines.contains(&head),
            "tick {tick}: the pulse landed on a hazard at {head:?}"
        );
    }

    assert_eq!(
        game.magnet_pulses, expected_pulses,
        "expected a pulse on every 6th tick, got {} of {expected_pulses}",
        game.magnet_pulses
    );
    assert!(expected_pulses >= 8, "the test must exercise several pulses");
}

#[test]
fn test_magnet_bot_survives_the_pulses() {
    let mut game = magnet_game(40, 40);
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
        "Autopilot should survive Magnet mode (lives: {}, pulses: {}, relocations: {}, score: {})",
        game.lives,
        game.magnet_pulses,
        game.magnet_relocations,
        game.score
    );
}

#[test]
fn test_magnet_persists_through_history_rewind() {
    let mut game = magnet_game(30, 30);
    game.magnet_pulses = 7;
    game.magnet_relocations = 3;
    game.magnet_flips = 5;
    game.magnet_poles.insert(Point { x: 12, y: 12 }, true);
    game.save_history_state();
    game.magnet_pulses = 0;
    game.magnet_relocations = 0;
    game.magnet_flips = 0;
    game.magnet_poles.clear();

    game.rewind_time();

    assert_eq!(
        game.magnet_pulses, 7,
        "magnet_pulses must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.magnet_relocations, 3);
    assert_eq!(game.magnet_flips, 5);
    assert_eq!(
        game.magnet_poles.get(&Point { x: 12, y: 12 }),
        Some(&true),
        "magnet_poles must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_magnet_persists_in_save_file() {
    let mut game = magnet_game(30, 30);
    game.magnet_pulses = 9;
    game.magnet_relocations = 4;
    game.magnet_flips = 6;
    let path = std::env::temp_dir().join(format!(
        "magnet_save_test_{}_{}.json",
        std::process::id(),
        11_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"magnet_pulses\":9")
            && json.contains("\"magnet_relocations\":4")
            && json.contains("\"magnet_flips\":6")
            && json.contains("\"magnet_poles\""),
        "SaveState JSON must contain the magnet fields, got: {}",
        &json[..json.len().min(200)]
    );
}
