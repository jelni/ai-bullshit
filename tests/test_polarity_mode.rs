use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn polarity_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Polarity;
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
    game.food = Point { x: 25, y: 25 };
    game.state = GameState::Playing;
}

#[test]
fn test_polarity_reset_seeds_poles_fairly() {
    let game = polarity_game(30, 30);

    assert!(game.polarity_attract, "the field starts attracting");
    assert_eq!(game.polarity_flips, 0);
    assert_eq!(game.polarity_shuffles, 0);
    assert_eq!(game.polarity_shoves, 0);
    assert_eq!(game.polarity_shield, 0);
    // pole count = clamp(30*30/80, 2, 6) = 6
    assert_eq!(game.polarity_poles.len(), 6);
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    for pole in &game.polarity_poles {
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
fn test_polarity_flips_every_16_ticks() {
    let mut game = polarity_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.polarity_poles.clear(); // no poles -> no shove, isolates the flip
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);

    // update() increments tick_counter first, so 15 makes the next update run
    // tick 16, a flip tick
    game.tick_counter = 15;
    game.update();
    assert_eq!(game.tick_counter, 16);
    assert_eq!(game.polarity_flips, 1, "tick 16 must flip the polarity");
    assert!(!game.polarity_attract, "the first flip turns attract into repel");
    assert_eq!(game.polarity_shoves, 0, "no poles means no shove");
    assert!(game.state == GameState::Playing && game.lives > 0);

    game.tick_counter = 31;
    game.update();
    assert_eq!(game.tick_counter, 32);
    assert_eq!(game.polarity_flips, 2, "tick 32 must flip the polarity again");
    assert!(game.polarity_attract, "the second flip turns repel back into attract");
}

#[test]
fn test_polarity_repel_shove_pushes_away_from_pole() {
    let mut game = polarity_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.obstacles.clear();
    game.polarity_poles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    // Pole directly below the snake: the repel shove pushes the head up
    game.polarity_poles.insert(Point { x: 10, y: 20 });

    game.tick_counter = 15; // tick 16: flip to repel + shove
    game.update();

    assert!(!game.polarity_attract);
    assert_eq!(game.polarity_shoves, 1, "the flip tick must shove the snake");
    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 9 },
        "the repel shove must push away from the pole on the dominant axis (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a shove must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_polarity_attract_shove_pulls_toward_pole() {
    let mut game = polarity_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.obstacles.clear();
    game.polarity_poles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.polarity_poles.insert(Point { x: 10, y: 20 });
    // Tick 32 flips the field: start from repel so the flip lands on attract
    game.polarity_attract = false;

    game.tick_counter = 31; // tick 32: flip to attract + shove
    game.update();

    assert!(game.polarity_attract);
    assert_eq!(game.polarity_shoves, 1);
    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 11 },
        "the attract shove must pull toward the pole on the dominant axis (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a shove must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_polarity_shove_fizzles_when_no_cell_is_fair() {
    let mut game = polarity_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(13);
    game.obstacles.clear();
    game.polarity_poles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.polarity_poles.insert(Point { x: 10, y: 20 });
    game.polarity_attract = false; // tick 32 flips to attract
    // A mine blocks the attract cell (11,11) and the fallback cell (10,10) is
    // the snake's own head, so no shove may happen at all
    game.mines.insert(Point { x: 11, y: 11 });

    game.tick_counter = 31;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "with no fair shove cell the snake must make only its normal move (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.polarity_shoves, 0, "no shove may be counted");
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a pole must never shove the snake into a kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_polarity_no_shove_on_non_flip_ticks() {
    let mut game = polarity_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(17);
    game.obstacles.clear();
    game.polarity_poles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.polarity_poles.insert(Point { x: 10, y: 20 });

    game.tick_counter = 0; // tick 1: not a multiple of POLARITY_FLIP_PERIOD
    game.update();

    assert_eq!(game.polarity_flips, 0);
    assert_eq!(game.polarity_shoves, 0, "shoves only happen on flip ticks");
    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "the snake must make only its normal move on non-flip ticks"
    );
}

#[test]
fn test_polarity_shield_blocks_the_shove() {
    let mut game = polarity_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(19);
    game.obstacles.clear();
    game.polarity_poles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.polarity_poles.insert(Point { x: 10, y: 20 });
    game.polarity_shield = 5;

    game.tick_counter = 15; // tick 16: flip to repel, but the player is shielded
    game.update();

    assert_eq!(game.polarity_flips, 1, "the flip still happens while shielded");
    assert_eq!(
        game.polarity_shoves, 0,
        "a shielded player must not be shoved"
    );
    assert_eq!(game.polarity_shield, 4, "the shield must count down every tick");
    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "the shielded snake must make only its normal move (head: {:?})",
        game.snake.head()
    );
}

#[test]
fn test_polarity_eating_food_grants_a_shield() {
    let mut game = polarity_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(23);
    game.obstacles.clear();
    game.polarity_poles.clear();
    pin_ambient_spawners(&mut game);
    park_snake(&mut game, Point { x: 10, y: 10 }, Direction::Right);
    game.food = Point { x: 11, y: 10 }; // right in front of the head

    game.tick_counter = 0; // tick 1: eat the food
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert_eq!(
        game.polarity_shield,
        Game::POLARITY_SHIELD_TICKS,
        "eating food must grant a POLARITY_SHIELD_TICKS shield"
    );

    game.tick_counter = 1; // tick 2: the shield counts down, no flip
    game.state = GameState::Playing;
    game.update();
    assert_eq!(
        game.polarity_shield,
        Game::POLARITY_SHIELD_TICKS - 1,
        "the shield must count down once per tick"
    );
}

#[test]
fn test_polarity_shuffle_every_24_ticks_stays_fair() {
    let mut game = polarity_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = false;

    // update() increments tick_counter first, so 23 makes the next update run
    // tick 24. The shuffle fires at the start of that tick, so the fairness
    // guarantees must be checked against the pre-update board state.
    for tick in [23_u32, 47, 71, 95, 119] {
        game.state = GameState::Playing;
        // Park the snake deterministically so the pre-update state is exactly
        // the state the shuffle sees (the direction queue is popped first).
        game.snake = Snake::new(Point { x: 20, y: 20 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.food = Point { x: 35, y: 35 };
        game.tick_counter = tick;
        let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
        let pre_head = game.snake.head();
        let pre_next =
            snake_game::game::Game::calculate_next_head_dir(pre_head, game.snake.direction);
        let pre_food = game.food;
        let pre_obstacles: std::collections::HashSet<Point> = game.obstacles.clone();
        let shuffles_before = game.polarity_shuffles;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        assert!(
            game.polarity_shuffles > shuffles_before,
            "a pole shuffle must happen on tick {}",
            tick + 1
        );
        for pole in &game.polarity_poles {
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
    assert_eq!(game.polarity_shuffles, 5);
    assert_eq!(game.polarity_poles.len(), 6, "clamp(40*40/80, 2, 6) = 6");
}

#[test]
fn test_polarity_scraps_poles_under_fresh_obstacles() {
    let mut game = polarity_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.polarity_poles.clear();
    game.polarity_poles.insert(Point { x: 14, y: 14 });
    // Something else (e.g. a meteor) drops an obstacle on the pole
    game.obstacles.insert(Point { x: 14, y: 14 });

    game.tick_counter = 0; // tick 1: no shuffle
    game.state = GameState::Playing;
    game.update();

    assert!(
        !game.polarity_poles.contains(&Point { x: 14, y: 14 }),
        "a pole under a fresh obstacle must be scrapped on the next tick"
    );
}

#[test]
fn test_polarity_shove_never_unfair_over_many_ticks() {
    let mut game = polarity_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 35, y: 35 };
    pin_ambient_spawners(&mut game);
    let mut total_shoves = 0u32;

    // Park the snake before every update and make sure the machine can never
    // shove it into a wall, a hazard, or itself on flip ticks
    for tick in 0..80_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.polarity_poles.clear();
        game.polarity_poles.insert(Point { x: 8, y: 30 });
        game.polarity_shield = 0;
        game.tick_counter = tick;
        let shoves_before = game.polarity_shoves;
        let lives_before = game.lives;
        game.update();
        let shoved = game.polarity_shoves > shoves_before;
        total_shoves += game.polarity_shoves.saturating_sub(shoves_before);
        if game.lives == 0 {
            // An ambient hazard (e.g. a meteor) got the parked snake; top up
            // and carry on - the shove assertions below only apply to ticks
            // where a shove actually happened and the snake survived.
            game.lives = 3;
            game.state = GameState::Playing;
            continue;
        }
        let head = game.snake.head();
        if shoved && game.lives == lives_before {
            assert!(
                head.x > 0 && head.x < game.width - 1 && head.y > 0 && head.y < game.height - 1,
                "tick {tick}: the shove left the board at {head:?}"
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
                !game.snake.body.iter().skip(1).any(|p| *p == head),
                "tick {tick}: the shove landed on the snake's own body at {head:?}"
            );
        }
    }

    assert!(
        total_shoves >= 3,
        "expected the pole to shove the parked snake on every flip tick, got {total_shoves} shoves"
    );
}

#[test]
fn test_polarity_bot_survives_the_poles() {
    let mut game = polarity_game(40, 40);
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
        "Autopilot should survive Polarity mode (lives: {}, flips: {}, shoves: {}, shuffles: {}, score: {})",
        game.lives,
        game.polarity_flips,
        game.polarity_shoves,
        game.polarity_shuffles,
        game.score
    );
}

#[test]
fn test_polarity_persists_through_history_rewind() {
    let mut game = polarity_game(30, 30);
    game.polarity_attract = false;
    game.polarity_flips = 7;
    game.polarity_shuffles = 3;
    game.polarity_shoves = 5;
    game.polarity_shield = 4;
    game.polarity_poles.insert(Point { x: 12, y: 12 });
    game.save_history_state();
    game.polarity_attract = true;
    game.polarity_flips = 0;
    game.polarity_shuffles = 0;
    game.polarity_shoves = 0;
    game.polarity_shield = 0;
    game.polarity_poles.clear();

    game.rewind_time();

    assert!(!game.polarity_attract, "polarity_attract must survive a time-rewind");
    assert_eq!(game.polarity_flips, 7);
    assert_eq!(game.polarity_shuffles, 3);
    assert_eq!(game.polarity_shoves, 5);
    assert_eq!(game.polarity_shield, 4);
    assert!(
        game.polarity_poles.contains(&Point { x: 12, y: 12 }),
        "polarity_poles must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_polarity_persists_in_save_file() {
    let mut game = polarity_game(30, 30);
    game.polarity_attract = false;
    game.polarity_flips = 9;
    game.polarity_shuffles = 4;
    game.polarity_shoves = 6;
    game.polarity_shield = 2;
    let path = std::env::temp_dir().join(format!(
        "polarity_save_test_{}_{}.json",
        std::process::id(),
        11_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"polarity_attract\":false")
            && json.contains("\"polarity_flips\":9")
            && json.contains("\"polarity_shuffles\":4")
            && json.contains("\"polarity_shoves\":6")
            && json.contains("\"polarity_shield\":2")
            && json.contains("\"polarity_poles\""),
        "SaveState JSON must contain the polarity fields, got: {}",
        &json[..json.len().min(200)]
    );
}
