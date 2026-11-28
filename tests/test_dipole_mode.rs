use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, DipolePol, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn dipole_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Dipole;
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

// Block the ambient portal/mine spawners so single-tick shove tests are
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

fn park_snake(game: &mut Game, head: Point, dir: Direction) {
    game.snake = Snake::new(head);
    game.snake.direction = dir;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(dir);
    game.auto_pilot = false;
    game.state = GameState::Playing;
}

#[test]
fn test_dipole_reset_seeds_poles_fairly() {
    let game = dipole_game(30, 30);

    assert_eq!(game.dipole_shoves, 0);
    assert_eq!(game.dipole_flips, 0);
    assert_eq!(game.dipole_shifts, 0);
    // dipole count = clamp(30*30/80, 3, 6) = 6
    assert_eq!(game.dipole_poles.len(), 6);
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    for dipole in game.dipole_poles.keys() {
        assert!(
            !game.obstacles.contains(dipole),
            "a dipole must never sit on an obstacle: {dipole:?}"
        );
        assert!(
            !game.snake.body_map.contains_key(dipole),
            "a dipole must never sit on the snake: {dipole:?}"
        );
        assert_ne!(*dipole, game.food, "a dipole must never sit on the food");
        assert!(
            dipole.x.abs_diff(spawn.x).saturating_add(dipole.y.abs_diff(spawn.y)) >= 5,
            "a dipole must spawn at least 5 cells from the spawn point: {dipole:?}"
        );
    }
}

#[test]
fn test_dipole_attract_pulls_head_onto_the_pole() {
    let mut game = dipole_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.dipole_poles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    // The snake heads right to (11,10); the attract dipole at (12,10) is
    // adjacent to that cell and pulls the head onto the walkable dipole cell
    game.dipole_poles.insert(Point { x: 12, y: 10 }, DipolePol::Attract);
    game.food = Point { x: 25, y: 25 };

    game.tick_counter = 0; // tick 1: no flip, no relocation, no reset
    game.update();

    assert_eq!(game.tick_counter, 1);
    assert_eq!(game.dipole_shoves, 1, "the attract dipole must shove the snake");
    assert_eq!(
        game.snake.head(),
        Point { x: 12, y: 10 },
        "the attract shove must pull the head onto the dipole cell (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "an attract shove must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_dipole_repel_pushes_head_away_from_the_pole() {
    let mut game = dipole_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(43);
    game.obstacles.clear();
    game.dipole_poles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    // The snake heads right to (11,10); the repel dipole at (11,9) sits above
    // that cell and pushes the head one cell further down, away from itself
    game.dipole_poles.insert(Point { x: 11, y: 9 }, DipolePol::Repel);
    game.food = Point { x: 25, y: 25 };

    game.tick_counter = 0;
    game.update();

    assert_eq!(game.dipole_shoves, 1, "the repel dipole must shove the snake");
    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 11 },
        "the repel shove must push the head away from the dipole (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a repel shove must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_dipole_head_on_repel_fizzles_on_the_snakes_own_body() {
    let mut game = dipole_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(44);
    game.obstacles.clear();
    game.dipole_poles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    // The snake heads right to (11,10); the repel dipole at (12,10) would push
    // the head back onto (10,10), its own current head cell, so the shove
    // must fizzle and the snake simply steps onto (11,10)
    game.dipole_poles.insert(Point { x: 12, y: 10 }, DipolePol::Repel);
    game.food = Point { x: 25, y: 25 };

    game.tick_counter = 0;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "a shove onto the snake's own body must fizzle (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.dipole_shoves, 0, "no shove may be counted");
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a fizzled shove must never kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_dipole_shove_never_lands_on_a_hazard() {
    let mut game = dipole_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(45);
    game.obstacles.clear();
    game.dipole_poles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    // The attract dipole at (12,10) would pull the head onto (12,10), but a
    // mine sits there, so the shove must fizzle instead of landing on it
    game.dipole_poles.insert(Point { x: 12, y: 10 }, DipolePol::Attract);
    game.mines.insert(Point { x: 12, y: 10 });
    game.food = Point { x: 25, y: 25 };

    game.tick_counter = 0;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "a shove onto a hazard must fizzle (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.dipole_shoves, 0);
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a dipole must never shove the snake onto a hazard (lives: {})",
        game.lives
    );
}

#[test]
fn test_dipole_flips_every_6_ticks_and_relocates_every_12() {
    let mut game = dipole_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.obstacles.clear();
    game.dipole_poles.clear();
    pin_ambient_spawners(&mut game);
    game.dipole_poles.insert(Point { x: 20, y: 20 }, DipolePol::Attract);
    game.dipole_poles.insert(Point { x: 22, y: 22 }, DipolePol::Repel);
    game.food = Point { x: 25, y: 25 };
    park_snake(&mut game, Point { x: 5, y: 5 }, Direction::Right);

    // update() increments tick_counter first, so 5 makes the next update run
    // tick 6: every dipole flips dipole
    game.tick_counter = 5;
    game.update();
    assert_eq!(game.tick_counter, 6);
    assert_eq!(game.dipole_flips, 1, "tick 6 must flip every dipole");
    assert_eq!(
        game.dipole_poles.get(&Point { x: 20, y: 20 }),
        Some(&DipolePol::Repel),
        "Attract must flip to Repel"
    );
    assert_eq!(
        game.dipole_poles.get(&Point { x: 22, y: 22 }),
        Some(&DipolePol::Attract),
        "Repel must flip to Attract"
    );

    // 11 makes the next update run tick 12: another flip plus one relocation
    let pre_head = game.snake.head();
    let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
    let pre_food = game.food;
    let pre_obstacles: std::collections::HashSet<Point> = game.obstacles.clone();
    let shifts_before = game.dipole_shifts;
    game.tick_counter = 11;
    game.update();
    assert_eq!(game.tick_counter, 12);
    assert_eq!(game.dipole_flips, 2, "tick 12 must flip every dipole again");
    assert!(
        game.dipole_shifts > shifts_before,
        "tick 12 must relocate one dipole"
    );
    assert_eq!(
        game.dipole_poles.len(),
        2,
        "a relocation moves a dipole, it does not add one"
    );
    for dipole in game.dipole_poles.keys() {
        assert!(
            !pre_obstacles.contains(dipole),
            "a relocated dipole must not sit on an obstacle: {dipole:?}"
        );
        assert!(
            !pre_body.contains(dipole),
            "a relocated dipole must not sit on the snake: {dipole:?}"
        );
        assert_ne!(*dipole, pre_food, "a relocated dipole must not sit on the food");
        assert!(
            dipole.x.abs_diff(pre_head.x).saturating_add(dipole.y.abs_diff(pre_head.y)) >= 4,
            "a relocated dipole must keep its distance from snake heads: {dipole:?}"
        );
    }
}

#[test]
fn test_dipole_discharges_and_reseeds_at_tick_200() {
    let mut game = dipole_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.obstacles.clear();
    pin_ambient_spawners(&mut game);
    game.food = Point { x: 25, y: 25 };
    park_snake(&mut game, Point { x: 5, y: 5 }, Direction::Right);

    let pre_head = game.snake.head();
    let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
    let pre_next =
        snake_game::game::Game::calculate_next_head_dir(pre_head, game.snake.direction);
    let pre_food = game.food;
    let pre_obstacles: std::collections::HashSet<Point> = game.obstacles.clone();
    let shifts_before = game.dipole_shifts;

    // 199 makes the next update run tick 200: the dipoles discharge and reseed
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert_eq!(game.dipole_flips, 0, "tick 200 is not a multiple of 6");
    assert!(
        game.dipole_shifts > shifts_before,
        "the tick-200 discharge must count as a shift"
    );
    assert_eq!(
        game.dipole_poles.len(),
        6,
        "the discharge must reseed the full dipole layout (clamp(30*30/80, 3, 6) = 6)"
    );
    for dipole in game.dipole_poles.keys() {
        assert!(!pre_obstacles.contains(dipole), "reseeded dipole on obstacle: {dipole:?}");
        assert!(!pre_body.contains(dipole), "reseeded dipole on the snake: {dipole:?}");
        assert_ne!(*dipole, pre_food, "reseeded dipole on the food");
        assert_ne!(
            *dipole, pre_next,
            "reseeded dipole on the cell the snake is about to enter"
        );
        assert!(
            dipole.x.abs_diff(pre_head.x).saturating_add(dipole.y.abs_diff(pre_head.y)) >= 5,
            "reseeded dipole too close to a snake head: {dipole:?}"
        );
    }
}

#[test]
fn test_dipole_eating_food_degausses_adjacent_poles() {
    let mut game = dipole_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.dipole_poles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.food = Point { x: 11, y: 10 };
    // The attract dipole at (12,10) is adjacent to the eaten cell, but a mine on
    // the dipole cell makes its shove fizzle so the snake really eats the food
    game.dipole_poles.insert(Point { x: 12, y: 10 }, DipolePol::Attract);
    game.mines.insert(Point { x: 12, y: 10 });

    game.tick_counter = 0;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert_eq!(game.dipole_shoves, 0, "the mined dipole cell must fizzle the shove");
    assert!(
        !game.dipole_poles.contains_key(&Point { x: 12, y: 10 }),
        "eating food must degauss the dipoles adjacent to the eaten cell"
    );
    assert_eq!(
        game.dipole_shifts, 1,
        "the degaussed dipole must count as a shift"
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "eating must never kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_dipole_shove_never_unfair_over_many_ticks() {
    let mut game = dipole_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.food = Point { x: 35, y: 35 };
    pin_ambient_spawners(&mut game);
    let mut total_shoves = 0u32;

    // Park the snake heading toward an attract dipole before every update and
    // make sure the dipole can never shove it into a wall, a hazard, or itself
    for tick in 0..60_u32 {
        if (tick + 1) % 6 == 0 || (tick + 1) % 12 == 0 || tick + 1 == 200 {
            continue; // flip/reloc ticks move the dipoles around
        }
        park_snake(&mut game, Point { x: 8, y: 15 }, Direction::Right);
        game.dipole_poles.clear();
        game.dipole_poles.insert(Point { x: 10, y: 15 }, DipolePol::Attract);
        game.tick_counter = tick;
        let shoves_before = game.dipole_shoves;
        let lives_before = game.lives;
        game.update();
        let shoved = game.dipole_shoves > shoves_before;
        total_shoves += game.dipole_shoves.saturating_sub(shoves_before);
        if game.lives == 0 {
            // An ambient hazard got the parked snake; top up and carry on -
            // the shove assertions below only apply to ticks where a shove
            // actually happened and the snake survived.
            game.lives = 3;
            game.state = GameState::Playing;
            continue;
        }
        let head = game.snake.head();
        if shoved && game.lives == lives_before {
            assert!(
                head == Point { x: 10, y: 15 },
                "tick {tick}: the only fair attract shove lands on the dipole cell, got {head:?}"
            );
            assert!(
                !game.obstacles.contains(&head),
                "tick {tick}: the shove landed on an obstacle at {head:?}"
            );
            assert!(
                !game.mines.contains(&head),
                "tick {tick}: the shove landed on a mine at {head:?}"
            );
            assert!(
                head.x > 0 && head.x < game.width - 1 && head.y > 0 && head.y < game.height - 1,
                "tick {tick}: the shove left the board at {head:?}"
            );
        }
    }

    assert!(
        total_shoves >= 40,
        "expected the dipole to shove the parked snake on most ticks, got {total_shoves} shoves"
    );
}

#[test]
fn test_dipole_scraps_poles_under_fresh_obstacles() {
    let mut game = dipole_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.dipole_poles.clear();
    game.dipole_poles.insert(Point { x: 14, y: 14 }, DipolePol::Attract);
    // Something else (e.g. a meteor) drops an obstacle on the dipole
    game.obstacles.insert(Point { x: 14, y: 14 });

    game.tick_counter = 0; // tick 1: no flip, no relocation
    game.state = GameState::Playing;
    game.update();

    assert!(
        !game.dipole_poles.contains_key(&Point { x: 14, y: 14 }),
        "a dipole under a fresh obstacle must be scrapped on the next tick"
    );
}

#[test]
fn test_dipole_bot_survives_the_poles() {
    let mut game = dipole_game(40, 40);
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
        "Autopilot should survive Dipole mode (lives: {}, shoves: {}, flips: {}, shifts: {}, score: {})",
        game.lives,
        game.dipole_shoves,
        game.dipole_flips,
        game.dipole_shifts,
        game.score
    );
}

#[test]
fn test_dipole_persists_through_history_rewind() {
    let mut game = dipole_game(30, 30);
    game.dipole_shoves = 7;
    game.dipole_flips = 3;
    game.dipole_shifts = 5;
    game.dipole_poles.insert(Point { x: 12, y: 12 }, DipolePol::Repel);
    game.save_history_state();
    game.dipole_shoves = 0;
    game.dipole_flips = 0;
    game.dipole_shifts = 0;
    game.dipole_poles.clear();

    game.rewind_time();

    assert_eq!(
        game.dipole_shoves, 7,
        "dipole_shoves must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.dipole_flips, 3);
    assert_eq!(game.dipole_shifts, 5);
    assert_eq!(
        game.dipole_poles.get(&Point { x: 12, y: 12 }),
        Some(&DipolePol::Repel),
        "dipole_poles must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_dipole_persists_in_save_file() {
    let mut game = dipole_game(30, 30);
    game.dipole_shoves = 9;
    game.dipole_flips = 4;
    game.dipole_shifts = 6;
    game.dipole_poles.clear();
    game.dipole_poles.insert(Point { x: 12, y: 12 }, DipolePol::Attract);
    let path = std::env::temp_dir().join(format!(
        "dipole_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"dipole_shoves\":9")
            && json.contains("\"dipole_flips\":4")
            && json.contains("\"dipole_shifts\":6")
            && json.contains("\"dipole_poles\""),
        "SaveState JSON must contain the dipole fields, got: {}",
        &json[..json.len().min(200)]
    );
}
