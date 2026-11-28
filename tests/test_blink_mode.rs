use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn blink_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Blink;
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

// Block the ambient portal/mine spawners so single-tick blink tests are
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

// A blink destination must always be a fair cell: in bounds within the safe
// zone, free of obstacles, mines, snake bodies, pads, and the food.
fn assert_fair_destination(game: &Game, head: Point, pre_body: &[Point]) {
    assert!(
        head.x > 0 && head.x < game.width - 1 && head.y > 0 && head.y < game.height - 1,
        "a blink must stay on the board, got {head:?}"
    );
    assert!(
        !game.obstacles.contains(&head),
        "a blink must never land on an obstacle at {head:?}"
    );
    assert!(
        !game.mines.contains(&head),
        "a blink must never land on a mine at {head:?}"
    );
    assert!(
        !pre_body.contains(&head),
        "a blink must never land on the snake's own body at {head:?}"
    );
    assert!(
        !game.blink_pads.contains(&head),
        "a blink must never land on another pad at {head:?}"
    );
    assert_ne!(head, game.food, "a blink must never land on the food");
}

#[test]
fn test_blink_reset_seeds_pads_fairly() {
    let game = blink_game(30, 30);

    assert_eq!(game.blink_teleports, 0);
    assert_eq!(game.blink_shuffles, 0);
    assert_eq!(game.blink_overcharge, 0);
    // pad count = clamp(30*30/60, 3, 8) = 8
    assert_eq!(game.blink_pads.len(), 8);
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    for pad in &game.blink_pads {
        assert!(
            !game.obstacles.contains(pad),
            "a pad must never sit on an obstacle: {pad:?}"
        );
        assert!(
            !game.snake.body_map.contains_key(pad),
            "a pad must never sit on the snake: {pad:?}"
        );
        assert_ne!(*pad, game.food, "a pad must never sit on the food");
        assert!(
            pad.x.abs_diff(spawn.x).saturating_add(pad.y.abs_diff(spawn.y)) >= 5,
            "a pad must spawn at least 5 cells from the spawn point: {pad:?}"
        );
    }
}

#[test]
fn test_blink_teleports_snake_to_a_fair_cell() {
    let mut game = blink_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.blink_pads.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    // The snake heads right into the pad at (11,10)
    game.blink_pads.insert(Point { x: 11, y: 10 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0; // tick 1: no scheduled scatter
    game.state = GameState::Playing;
    let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
    game.update();

    assert_eq!(game.tick_counter, 1);
    assert_eq!(game.blink_teleports, 1, "the pad must blink the snake");
    let head = game.snake.head();
    assert_ne!(
        head,
        Point { x: 11, y: 10 },
        "a blink must leave the pad cell (head: {head:?})"
    );
    assert_fair_destination(&game, head, &pre_body);
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a blink must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_blink_rests_on_the_pad_when_no_fair_destination_exists() {
    let mut game = blink_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(9);
    game.blink_pads.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.blink_pads.insert(Point { x: 11, y: 10 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    // Wall off the entire board except the pad itself: no fair destination exists
    for y in 0..game.height {
        for x in 0..game.width {
            let p = Point { x, y };
            if p != (Point { x: 11, y: 10 }) {
                game.obstacles.insert(p);
            }
        }
    }

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "with no fair destination the snake must simply rest on the walkable pad (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.blink_teleports, 0, "no blink may be counted");
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a pad must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_blink_never_lands_on_a_hazard() {
    let mut game = blink_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    game.obstacles.clear();
    game.blink_pads.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.blink_pads.insert(Point { x: 11, y: 10 });
    // Mines scattered around the likely landing zone
    game.mines.insert(Point { x: 14, y: 10 });
    game.mines.insert(Point { x: 15, y: 12 });
    game.mines.insert(Point { x: 8, y: 8 });
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
    game.update();

    assert_eq!(game.blink_teleports, 1);
    assert_fair_destination(&game, game.snake.head(), &pre_body);
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the blink must never land on a hazard (lives: {})",
        game.lives
    );
}

#[test]
fn test_blink_shuffle_every_24_ticks_stays_fair() {
    let mut game = blink_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = false;

    // update() increments tick_counter first, so 23 makes the next update run
    // tick 24. The scatter fires at the start of that tick, so the fairness
    // guarantees must be checked against the pre-update board state.
    for tick in [23_u32, 47, 71, 95, 119] {
        game.state = GameState::Playing;
        // Park the snake deterministically so the pre-update state is exactly
        // the state the scatter sees (the direction queue is popped first).
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
        let shuffles_before = game.blink_shuffles;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        assert!(
            game.blink_shuffles > shuffles_before,
            "a pad scatter must happen on tick {}",
            tick + 1
        );
        for pad in &game.blink_pads {
            assert!(
                !pre_obstacles.contains(pad),
                "a relocated pad must not sit on an obstacle: {pad:?}"
            );
            assert!(
                !pre_body.contains(pad),
                "a relocated pad must not sit on the snake: {pad:?}"
            );
            assert_ne!(*pad, pre_next, "a relocated pad must not sit on the cell the snake is about to enter");
            assert_ne!(*pad, pre_food, "a relocated pad must not sit on the food");
            assert!(
                pad.x.abs_diff(pre_head.x).saturating_add(pad.y.abs_diff(pre_head.y)) >= 4,
                "a relocated pad must keep its distance from snake heads: {pad:?}"
            );
        }
    }
    assert_eq!(game.blink_shuffles, 5);
    assert_eq!(game.blink_pads.len(), 8, "clamp(40*40/60, 3, 8) = 8");
}

#[test]
fn test_blink_eating_food_overcharges_the_pads() {
    let mut game = blink_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.blink_pads.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 11, y: 10 };
    game.blink_pads.insert(Point { x: 20, y: 20 });
    game.auto_pilot = false;

    // Tick 1: the snake eats the food at (11,10)
    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert_eq!(
        game.blink_overcharge,
        Game::BLINK_OVERCHARGE_TICKS,
        "eating food must overcharge the pads for BLINK_OVERCHARGE_TICKS"
    );
    assert!(
        game.blink_overcharge > 0 && game.blink_overcharge <= Game::BLINK_OVERCHARGE_TICKS,
        "the overcharge timer must count down every tick"
    );
}

#[test]
fn test_blink_overcharge_homes_in_on_food() {
    let mut game = blink_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.blink_pads.clear();
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 11, y: 10 };
    game.auto_pilot = false;

    // Tick 1: eat the food to overcharge the pads
    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();
    assert_eq!(game.food_eaten_session, 1);
    assert!(game.blink_overcharge > 0);
    let food_after_eat = game.food;

    // Tick 2: head into a pad while overcharged - the blink lands next to the food
    game.snake = Snake::new(Point { x: 10, y: 12 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
    game.blink_pads.clear();
    game.blink_pads.insert(Point { x: 11, y: 12 });
    game.tick_counter = 1;
    game.state = GameState::Playing;
    let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
    game.update();

    assert_eq!(game.blink_teleports, 1, "the overcharged pad must blink the snake");
    let head = game.snake.head();
    assert_eq!(
        head.x.abs_diff(food_after_eat.x).saturating_add(head.y.abs_diff(food_after_eat.y)),
        1,
        "an overcharged blink must land adjacent to the food (head: {head:?}, food: {food_after_eat:?})"
    );
    assert_fair_destination(&game, head, &pre_body);
    assert!(
        game.blink_overcharge < Game::BLINK_OVERCHARGE_TICKS,
        "the overcharge timer must count down every tick"
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "an overcharged blink must never kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_blink_never_unfair_over_many_ticks() {
    let mut game = blink_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 35, y: 35 };
    pin_ambient_spawners(&mut game);
    let mut total_blinks = 0u32;

    // Park the snake heading into a pad before every non-scatter update and
    // make sure the machine can never blink it into a wall, a hazard, or itself
    for tick in 0..60_u32 {
        if (tick + 1).is_multiple_of(Game::BLINK_PERIOD) {
            continue; // scatter ticks relocate the pads
        }
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.blink_pads.clear();
        game.blink_pads.insert(Point { x: 9, y: 15 });
        game.tick_counter = tick;
        let pre_body: Vec<Point> = game.snake.body.iter().copied().collect();
        let blinks_before = game.blink_teleports;
        let lives_before = game.lives;
        game.update();
        let blinked = game.blink_teleports > blinks_before;
        total_blinks += game.blink_teleports.saturating_sub(blinks_before);
        if game.lives == 0 {
            // An ambient hazard (e.g. a meteor) got the parked snake; top up
            // and carry on - the blink assertions below only apply to ticks
            // where a blink actually happened and the snake survived.
            game.lives = 3;
            game.state = GameState::Playing;
            continue;
        }
        let head = game.snake.head();
        if blinked && game.lives == lives_before {
            assert_fair_destination(&game, head, &pre_body);
        }
    }

    assert!(
        total_blinks >= 10,
        "expected the pad to blink the parked snake on most ticks, got {total_blinks} blinks"
    );
}

#[test]
fn test_blink_scraps_pads_under_fresh_obstacles() {
    let mut game = blink_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.blink_pads.clear();
    game.blink_pads.insert(Point { x: 14, y: 14 });
    // Something else (e.g. a meteor) drops an obstacle on the pad
    game.obstacles.insert(Point { x: 14, y: 14 });

    game.tick_counter = 0; // tick 1: no scatter
    game.state = GameState::Playing;
    game.update();

    assert!(
        !game.blink_pads.contains(&Point { x: 14, y: 14 }),
        "a pad under a fresh obstacle must be scrapped on the next tick"
    );
}

#[test]
fn test_blink_bot_survives_the_pads() {
    let mut game = blink_game(40, 40);
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
        "Autopilot should survive Blink mode (lives: {}, blinks: {}, shuffles: {}, score: {})",
        game.lives,
        game.blink_teleports,
        game.blink_shuffles,
        game.score
    );
}

#[test]
fn test_blink_persists_through_history_rewind() {
    let mut game = blink_game(30, 30);
    game.blink_teleports = 7;
    game.blink_shuffles = 3;
    game.blink_overcharge = 5;
    game.blink_pads.insert(Point { x: 12, y: 12 });
    game.save_history_state();
    game.blink_teleports = 0;
    game.blink_shuffles = 0;
    game.blink_overcharge = 0;
    game.blink_pads.clear();

    game.rewind_time();

    assert_eq!(
        game.blink_teleports, 7,
        "blink_teleports must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.blink_shuffles, 3);
    assert_eq!(game.blink_overcharge, 5);
    assert!(
        game.blink_pads.contains(&Point { x: 12, y: 12 }),
        "blink_pads must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_blink_persists_in_save_file() {
    let mut game = blink_game(30, 30);
    game.blink_teleports = 9;
    game.blink_shuffles = 4;
    game.blink_overcharge = 6;
    let path = std::env::temp_dir().join(format!(
        "blink_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"blink_teleports\":9")
            && json.contains("\"blink_shuffles\":4")
            && json.contains("\"blink_overcharge\":6")
            && json.contains("\"blink_pads\""),
        "SaveState JSON must contain the blink fields, got: {}",
        &json[..json.len().min(200)]
    );
}
