use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn pinball_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Pinball;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_pinball_reset_seeds_bumpers_mirrored_into_obstacles() {
    let game = pinball_game(30, 30);

    assert!(!game.pinball_bumpers.is_empty(), "reset must seed bumpers");
    // cap = clamp(min(30, 30) / 5, 4, 8) = 6
    assert_eq!(game.pinball_bumpers.len(), 6);
    assert_eq!(game.pinball_bumps, 0);
    assert_eq!(game.pinball_relocations, 0);
    for bumper in &game.pinball_bumpers {
        assert!(
            game.obstacles.contains(bumper),
            "bumpers must be mirrored into obstacles"
        );
    }
    // Bumpers must spawn inside the playable area, never on the snake or the food
    for bumper in &game.pinball_bumpers {
        assert!(bumper.x > 0 && bumper.x < game.width - 1);
        assert!(bumper.y > 0 && bumper.y < game.height - 1);
        assert!(!game.snake.body_map.contains_key(bumper));
        assert_ne!(*bumper, game.food);
    }
}

#[test]
fn test_pinball_shuffles_every_6_ticks() {
    let mut game = pinball_game(30, 30);
    game.auto_pilot = true;
    let cap = game.pinball_bumpers.len();

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must shuffle the bumpers.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let relocs_before = game.pinball_relocations;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.pinball_relocations > relocs_before,
            "a bumper shuffle must happen on tick {}",
            tick + 1
        );
        assert_eq!(
            game.pinball_bumpers.len(),
            cap,
            "the bumper count must stay at the cap once seeded"
        );
        for bumper in &game.pinball_bumpers {
            assert!(
                game.obstacles.contains(bumper),
                "bumpers must stay mirrored into obstacles"
            );
        }
    }
    assert_eq!(game.pinball_relocations, 6);
}

#[test]
fn test_pinball_never_walls_snake_or_next_cell() {
    let mut game = pinball_game(20, 20);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.obstacles.clear();
    game.pinball_bumpers.clear();
    game.snake = Snake::new(Point { x: 2, y: 2 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = false;

    // Tick 6 shuffles a bumper, which must never land on the snake's next cell.
    game.tick_counter = 5;
    game.update();

    assert_eq!(game.pinball_relocations, 1);
    let next = Point { x: 3, y: 2 };
    assert!(
        !game.pinball_bumpers.contains(&next),
        "a shuffle must never place a bumper on the cell the snake is about to enter"
    );
    for seg in &game.snake.body {
        assert!(
            !game.pinball_bumpers.contains(seg),
            "a shuffle must never place a bumper on the snake's body"
        );
    }
    assert!(
        !game.pinball_bumpers.contains(&game.food),
        "a shuffle must never place a bumper on the food"
    );
}

#[test]
fn test_pinball_fairness_over_many_ticks() {
    let mut game = pinball_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    game.auto_pilot = false;
    game.food = Point { x: 25, y: 25 };
    let lives_before = game.lives;

    // Deterministic: park the snake at (8, 15) heading right before every
    // scheduled shuffle, so the shuffle must never bumper its next cell (9, 15).
    let next = Point { x: 9, y: 15 };
    for k in 1..=10_u32 {
        game.state = GameState::Playing;
        game.snake = Snake::new(Point { x: 8, y: 15 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.tick_counter = 6 * k - 1;
        game.update();
        assert!(
            !game.pinball_bumpers.contains(&next),
            "shuffle {} placed a bumper on the cell the snake was about to enter ({next:?})",
            k
        );
        for seg in &game.snake.body {
            assert!(
                !game.pinball_bumpers.contains(seg),
                "shuffle {} placed a bumper on the snake's body at {seg:?}",
                k
            );
        }
    }

    assert_eq!(
        game.lives, lives_before,
        "the snake must survive the shuffles (relocations: {})",
        game.pinball_relocations
    );
    assert!(
        game.pinball_relocations >= 10,
        "expected a shuffle every 6 ticks, got {}",
        game.pinball_relocations
    );
}

#[test]
fn test_pinball_bumpers_reset_at_tick_200() {
    let mut game = pinball_game(30, 30);
    game.stats.equipped_class = Some(HeroClass::Druid);
    game.rng = rand::rngs::StdRng::seed_from_u64(5);
    game.auto_pilot = true;

    // Shuffle a few times first.
    for tick in [5_u32, 11, 17] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        game.update();
    }
    assert!(!game.pinball_bumpers.is_empty());
    let walls: Vec<Point> = game.pinball_bumpers.iter().copied().collect();

    // Tick 200 of the 250-tick cycle resets all bumpers.
    game.state = GameState::Playing;
    game.tick_counter = 199;
    game.update();

    assert_eq!(game.tick_counter, 200);
    assert!(
        game.pinball_bumpers.is_empty(),
        "all bumpers must be cleared at tick 200"
    );
    for bumper in &walls {
        assert!(
            !game.obstacles.contains(bumper),
            "cleared bumpers must leave obstacles too"
        );
    }
    assert!(
        game.pinball_relocations >= 3,
        "the shuffle counter keeps counting through a reset"
    );
}

#[test]
fn test_pinball_bumper_deflects_snake_sideways() {
    let mut game = pinball_game(20, 20);
    game.obstacles.clear();
    game.pinball_bumpers.clear();
    let bumper = Point { x: 6, y: 5 };
    game.pinball_bumpers.insert(bumper);
    game.obstacles.insert(bumper);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = false;
    game.state = GameState::Playing;
    game.tick_counter = 1; // not a shuffle tick
    let score_before = game.score;

    game.update();

    assert!(
        game.lives == 3 && !game.just_died,
        "the snake must survive the bumper deflection (lives: {}, just_died: {})",
        game.lives,
        game.just_died
    );
    assert_eq!(game.pinball_bumps, 1, "the bump must be counted");
    assert_eq!(
        game.score,
        score_before + 5,
        "a bump must award +5 score"
    );
    assert!(
        !game.pinball_bumpers.contains(&game.snake.head()),
        "the snake must never end up on the bumper cell"
    );
    assert_ne!(
        game.snake.head(),
        bumper,
        "the deflected head must not be the bumper cell"
    );
    assert_eq!(
        game.snake.direction,
        Direction::Up,
        "a horizontal approach must deflect onto a free perpendicular lane"
    );
    assert_eq!(game.snake.head(), Point { x: 5, y: 4 });
}

#[test]
fn test_pinball_bumper_is_lethal_when_every_lane_is_blocked() {
    let mut game = pinball_game(20, 20);
    game.obstacles.clear();
    game.pinball_bumpers.clear();
    // Snake of length 2 moving right: head (5,5), neck (4,5).
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.body.clear();
    game.snake.body.push_back(Point { x: 5, y: 5 });
    game.snake.body.push_back(Point { x: 4, y: 5 });
    game.snake.rebuild_map();
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
    // Bumper straight ahead, both perpendicular lanes walled off; the only
    // remaining lane is the snake's own neck, so the bumper stays lethal.
    let bumper = Point { x: 6, y: 5 };
    game.pinball_bumpers.insert(bumper);
    game.obstacles.insert(bumper);
    game.obstacles.insert(Point { x: 5, y: 4 });
    game.obstacles.insert(Point { x: 5, y: 6 });
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = false;
    game.state = GameState::Playing;
    game.tick_counter = 1;
    let lives_before = game.lives;

    game.update();

    assert_eq!(
        game.lives,
        lives_before - 1,
        "a bumper with every deflection lane blocked must kill the snake"
    );
    assert_eq!(game.pinball_bumps, 0, "a lethal hit must not count as a bump");
}

#[test]
fn test_pinball_eating_food_clears_adjacent_bumper() {
    let mut game = pinball_game(20, 20);
    game.obstacles.clear();
    game.pinball_bumpers.clear();
    let bumper = Point { x: 7, y: 5 };
    game.pinball_bumpers.insert(bumper);
    game.obstacles.insert(bumper);
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 6, y: 5 };
    game.auto_pilot = false;
    game.state = GameState::Playing;
    // Tick 5 is not a multiple of 6 and not 200, so no shuffle interferes here.
    game.tick_counter = 4;

    game.update();

    assert_eq!(game.food_eaten_session, 1, "Snake should have eaten the food");
    assert!(
        !game.pinball_bumpers.contains(&bumper),
        "Eating food in Pinball mode should clear the adjacent bumper"
    );
    assert!(
        !game.obstacles.contains(&bumper),
        "The cleared bumper must also leave obstacles"
    );
}

#[test]
fn test_pinball_bot_survives_the_bumpers() {
    let mut game = pinball_game(40, 40);
    game.rng = rand::rngs::StdRng::seed_from_u64(99);
    game.auto_pilot = true;

    for _ in 0..150 {
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
        "Autopilot should survive the pinball machine (lives: {}, bumps: {}, score: {})",
        game.lives,
        game.pinball_bumps,
        game.score
    );
    assert!(
        game.pinball_relocations >= 10,
        "expected many shuffles over 150 ticks, got {}",
        game.pinball_relocations
    );
    assert!(
        game.pinball_bumpers.len() <= 8,
        "the bumper count must stay within the cap (min(40, 40) / 5 = 8), got {}",
        game.pinball_bumpers.len()
    );
}

#[test]
fn test_pinball_persists_through_history_rewind() {
    let mut game = pinball_game(30, 30);
    game.pinball_bumps = 7;
    game.pinball_relocations = 3;
    game.pinball_bumpers.insert(Point { x: 10, y: 10 });
    game.save_history_state();
    game.pinball_bumps = 0;
    game.pinball_relocations = 0;
    game.pinball_bumpers.clear();

    game.rewind_time();

    assert_eq!(
        game.pinball_bumps, 7,
        "pinball_bumps must survive a time-rewind through HistoryState"
    );
    assert_eq!(
        game.pinball_relocations, 3,
        "pinball_relocations must survive a time-rewind through HistoryState"
    );
    assert!(
        game.pinball_bumpers.contains(&Point { x: 10, y: 10 }),
        "pinball_bumpers must survive a time-rewind through HistoryState"
    );
}

#[test]
fn test_pinball_persists_in_save_file() {
    let mut game = pinball_game(30, 30);
    game.pinball_bumps = 9;
    game.pinball_relocations = 4;
    let path = std::env::temp_dir().join(format!(
        "pinball_save_test_{}_{}.json",
        std::process::id(),
        9_u32
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);
    let json = std::fs::read_to_string(&path).expect("save file should exist");
    let _ = std::fs::remove_file(&path);

    assert!(
        json.contains("\"pinball_bumps\":9") && json.contains("\"pinball_relocations\":4"),
        "SaveState JSON must contain the pinball counters, got: {}",
        &json[..json.len().min(200)]
    );
}
