use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn induction_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Induction;
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

// Block the ambient portal/mine spawners so single-tick tug tests are
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

// Park the snake at (10,10) heading right with two poles placed by hand (two
// poles means manage_induction never tops up a random third one).
fn park_snake(game: &mut Game, poles: [Point; 2]) {
    game.obstacles.clear();
    game.induction_poles.clear();
    game.induction_poles.extend(poles);
    pin_ambient_spawners(game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 28, y: 3 };
    game.auto_pilot = false;
}

#[test]
fn test_induction_reset_seeds_two_poles_fairly() {
    let game = induction_game(30, 30);

    assert!(game.induction_attract, "the poles must start attracting");
    assert_eq!(game.induction_tugs, 0);
    assert_eq!(game.induction_flips, 0);
    assert_eq!(game.induction_relocs, 0);
    assert_eq!(game.induction_shield, 0);
    assert_eq!(game.induction_poles.len(), 2, "exactly two magnet poles");
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    for pole in &game.induction_poles {
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
fn test_induction_attract_tugs_aligned_head_toward_pole() {
    let mut game = induction_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    park_snake(&mut game, [Point { x: 16, y: 10 }, Point { x: 25, y: 25 }]);
    // The snake heads right to (11,10); the pole at (16,10) is aligned on the
    // same row 5 cells away, so attract tugs the head one cell toward it
    game.induction_attract = true;

    game.tick_counter = 0; // tick 1: no flip, no relocation
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 1);
    assert_eq!(game.induction_tugs, 1, "the aligned pole must tug the head");
    assert_eq!(
        game.snake.head(),
        Point { x: 12, y: 10 },
        "an attract tug must pull the head one cell toward the pole (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a tug must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_induction_repel_tugs_aligned_head_away_from_pole() {
    let mut game = induction_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    park_snake(&mut game, [Point { x: 5, y: 10 }, Point { x: 25, y: 25 }]);
    // The snake heads right to (11,10); the pole at (5,10) is aligned on the
    // same row 6 cells away, so repel tugs the head one cell away from it
    game.induction_attract = false;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.induction_tugs, 1, "the aligned pole must tug the head");
    assert_eq!(
        game.snake.head(),
        Point { x: 12, y: 10 },
        "a repel tug must push the head one cell away from the pole (head: {:?})",
        game.snake.head()
    );
}

#[test]
fn test_induction_vertical_alignment_tugs_along_column() {
    let mut game = induction_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    park_snake(&mut game, [Point { x: 10, y: 4 }, Point { x: 25, y: 25 }]);
    // Snake::new trails the body below the head, so face up: the snake heads
    // up to (10,9); the pole at (10,4) is aligned on the same column 5 cells
    // away, so attract tugs the head one cell toward it
    game.snake.direction = Direction::Up;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Up);
    game.induction_attract = true;

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.induction_tugs, 1);
    assert_eq!(
        game.snake.head(),
        Point { x: 10, y: 8 },
        "a vertical attract tug must pull the head toward the pole (head: {:?})",
        game.snake.head()
    );
}

#[test]
fn test_induction_tug_fizzles_when_destination_blocked() {
    let mut game = induction_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(9);
    park_snake(&mut game, [Point { x: 16, y: 10 }, Point { x: 25, y: 25 }]);
    game.induction_attract = true;
    // The tug destination (12,10) is an obstacle, so the tug must fizzle and
    // the snake simply moves to (11,10)
    game.obstacles.insert(Point { x: 12, y: 10 });

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "a blocked tug destination must fizzle (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.induction_tugs, 0, "a fizzled tug must not be counted");
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a fizzled tug must never kill (lives: {})",
        game.lives
    );
}

#[test]
fn test_induction_tug_never_lands_on_a_hazard() {
    let mut game = induction_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    park_snake(&mut game, [Point { x: 16, y: 10 }, Point { x: 25, y: 25 }]);
    game.induction_attract = true;
    // A mine sits on the tug destination (12,10): the tug must fizzle
    game.mines.insert(Point { x: 12, y: 10 });

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "the tug must never land on a hazard (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.induction_tugs, 0);
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "the tug must never land on a mine (lives: {})",
        game.lives
    );
}

#[test]
fn test_induction_no_tug_beyond_range() {
    let mut game = induction_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(13);
    park_snake(&mut game, [Point { x: 20, y: 10 }, Point { x: 3, y: 25 }]);
    game.induction_attract = true;
    // The pole at (20,10) is 9 cells away from the head's destination (11,10):
    // beyond INDUCTION_RANGE, so no tug
    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "a pole beyond INDUCTION_RANGE must not tug (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.induction_tugs, 0);
}

#[test]
fn test_induction_flip_every_12_ticks() {
    let mut game = induction_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.obstacles.clear();
    game.auto_pilot = false;
    game.food = Point { x: 25, y: 3 };
    pin_ambient_spawners(&mut game);

    // update() increments tick_counter first, so 11 makes the next update run
    // tick 12, which flips the induction
    for (tick, expected_attract, expected_flips) in
        [(11_u32, false, 1_u32), (23, true, 2), (35, false, 3)]
    {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 20, y: 20 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = tick;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        assert_eq!(
            game.induction_attract, expected_attract,
            "induction must flip on tick {}",
            tick + 1
        );
        assert_eq!(game.induction_flips, expected_flips);
        assert!(
            game.state == GameState::Playing && game.lives > 0,
            "the flip must never kill the parked snake (lives: {})",
            game.lives
        );
    }
}

#[test]
fn test_induction_reloc_every_30_ticks_stays_fair() {
    let mut game = induction_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = false;

    // update() increments tick_counter first, so 29 makes the next update run
    // tick 30. The relocation fires at the start of that tick, so the
    // fairness guarantees must be checked against the pre-update board state.
    for tick in [29_u32, 59, 89] {
        game.state = GameState::Playing;
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
        let relocs_before = game.induction_relocs;
        game.update();
        assert_eq!(game.tick_counter, tick + 1);
        assert!(
            game.induction_relocs > relocs_before,
            "a pole relocation must happen on tick {}",
            tick + 1
        );
        assert_eq!(game.induction_poles.len(), 2, "both poles must be re-placed");
        for pole in &game.induction_poles {
            assert!(
                !pre_obstacles.contains(pole),
                "a relocated pole must not sit on an obstacle: {pole:?}"
            );
            assert!(
                !pre_body.contains(pole),
                "a relocated pole must not sit on the snake: {pole:?}"
            );
            assert_ne!(
                *pole, pre_next,
                "a relocated pole must not sit on the cell the snake is about to enter"
            );
            assert_ne!(*pole, pre_food, "a relocated pole must not sit on the food");
            assert!(
                pole.x.abs_diff(pre_head.x).saturating_add(pole.y.abs_diff(pre_head.y)) >= 4,
                "a relocated pole must keep its distance from snake heads: {pole:?}"
            );
        }
    }
    assert_eq!(game.induction_relocs, 3);
}

#[test]
fn test_induction_eating_food_shields_against_tugs() {
    let mut game = induction_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    park_snake(&mut game, [Point { x: 20, y: 20 }, Point { x: 21, y: 21 }]);
    // Tick 1: the snake eats the food at (11,10)
    game.food = Point { x: 11, y: 10 };

    game.tick_counter = 0;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.food_eaten_session, 1, "the snake should have eaten the food");
    assert_eq!(
        game.induction_shield,
        Game::INDUCTION_SHIELD_TICKS,
        "eating food must shield the snakes for INDUCTION_SHIELD_TICKS"
    );

    // Tick 2: head to (11,12), aligned with the pole at (16,12) - but the
    // shield is up, so no tug may happen
    game.snake = Snake::new(Point { x: 10, y: 12 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
    game.induction_poles = vec![Point { x: 16, y: 12 }, Point { x: 25, y: 25 }];
    game.food = Point { x: 28, y: 3 };
    game.tick_counter = 1;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 12 },
        "a shielded snake must not be tugged (head: {:?})",
        game.snake.head()
    );
    assert_eq!(game.induction_tugs, 0, "no tug may count while shielded");
    assert!(
        game.induction_shield < Game::INDUCTION_SHIELD_TICKS,
        "the shield timer must count down every tick"
    );
}

#[test]
fn test_induction_tug_never_unfair_over_many_ticks() {
    let mut game = induction_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 35, y: 35 };
    pin_ambient_spawners(&mut game);
    let mut total_tugs = 0u32;

    // Park the snake heading right with a pole aligned on its row before
    // every non-relocation update and make sure the machine can never tug it
    // into a wall, a hazard, or itself
    for tick in 0..60_u32 {
        if (tick + 1).is_multiple_of(30) {
            continue; // relocation ticks move the poles
        }
        game.state = GameState::Playing;
        game.obstacles.clear();
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.induction_poles = vec![Point { x: 14, y: 15 }, Point { x: 30, y: 30 }];
        game.tick_counter = tick;
        let tugs_before = game.induction_tugs;
        let lives_before = game.lives;
        game.update();
        let tugged = game.induction_tugs > tugs_before;
        total_tugs += game.induction_tugs.saturating_sub(tugs_before);
        if game.lives == 0 {
            // An ambient hazard (e.g. a meteor) got the parked snake; top up
            // and carry on - the tug assertions below only apply to ticks
            // where a tug actually happened and the snake survived.
            game.lives = 3;
            game.state = GameState::Playing;
            continue;
        }
        let head = game.snake.head();
        if tugged && game.lives == lives_before {
            assert!(
                head.y == 15 && head.x > 8 && head.x <= 10,
                "tick {tick}: a tug must land one cell along the alignment axis, got {head:?}"
            );
            assert!(
                !game.obstacles.contains(&head),
                "tick {tick}: the tug landed on an obstacle at {head:?}"
            );
            assert!(
                head.x > 0 && head.x < game.width - 1 && head.y > 0 && head.y < game.height - 1,
                "tick {tick}: the tug left the board at {head:?}"
            );
        }
    }

    assert!(
        total_tugs >= 10,
        "expected the pole to tug the parked snake on most ticks, got {total_tugs} tugs"
    );
}

#[test]
fn test_induction_scraps_poles_under_fresh_obstacles() {
    let mut game = induction_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(3);
    game.induction_poles = vec![Point { x: 14, y: 14 }, Point { x: 15, y: 15 }];
    // Something else (e.g. a meteor) drops an obstacle on a pole
    game.obstacles.insert(Point { x: 14, y: 14 });

    game.tick_counter = 0; // tick 1: no relocation
    game.state = GameState::Playing;
    game.update();

    assert!(
        !game.induction_poles.contains(&Point { x: 14, y: 14 }),
        "a pole under a fresh obstacle must be scrapped on the next tick"
    );
    assert_eq!(
        game.induction_poles.len(),
        2,
        "the scrapped pole must be topped back up to a fair cell"
    );
    for pole in &game.induction_poles {
        assert!(
            !game.obstacles.contains(pole),
            "no pole may sit on an obstacle: {pole:?}"
        );
    }
}

#[test]
fn test_induction_bot_survives_the_magnet_poles() {
    let mut game = induction_game(40, 40);
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
        "Autopilot should survive Induction mode (lives: {}, tugs: {}, flips: {}, relocs: {}, score: {})",
        game.lives,
        game.induction_tugs,
        game.induction_flips,
        game.induction_relocs,
        game.score
    );
}

#[test]
fn test_induction_persists_through_history_rewind() {
    let mut game = induction_game(30, 30);
    game.induction_poles = vec![Point { x: 12, y: 12 }, Point { x: 13, y: 13 }];
    game.induction_attract = false;
    game.induction_tugs = 7;
    game.induction_flips = 3;
    game.induction_relocs = 2;
    game.induction_shield = 5;
    game.save_history_state();
    game.induction_poles.clear();
    game.induction_attract = true;
    game.induction_tugs = 0;
    game.induction_flips = 0;
    game.induction_relocs = 0;
    game.induction_shield = 0;

    game.rewind_time();

    assert_eq!(
        game.induction_tugs, 7,
        "induction_tugs must survive a time-rewind through HistoryState"
    );
    assert_eq!(game.induction_flips, 3);
    assert_eq!(game.induction_relocs, 2);
    assert_eq!(game.induction_shield, 5);
    assert!(!game.induction_attract);
    assert!(
        game.induction_poles.contains(&Point { x: 12, y: 12 })
            && game.induction_poles.contains(&Point { x: 13, y: 13 }),
        "induction_poles must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_induction_persists_in_save_file() {
    let mut game = induction_game(30, 30);
    game.induction_tugs = 9;
    game.induction_flips = 4;
    game.induction_relocs = 2;
    game.induction_shield = 6;
    game.induction_attract = false;
    let path = std::env::temp_dir().join(format!(
        "induction_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"induction_tugs\":9")
            && json.contains("\"induction_flips\":4")
            && json.contains("\"induction_relocs\":2")
            && json.contains("\"induction_shield\":6")
            && json.contains("\"induction_attract\":false")
            && json.contains("\"induction_poles\""),
        "SaveState JSON must contain the induction fields, got: {}",
        &json[..json.len().min(200)]
    );
}
