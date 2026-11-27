use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn hole_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::BlackHole;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

fn solo_game() -> Game {
    let mut game = hole_game(30, 30);
    game.obstacles.clear();
    game.mines.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 5 };
    game.black_hole = Some(Point { x: 15, y: 15 });
    game
}

fn manhattan(a: Point, b: Point) -> u16 {
    a.x.abs_diff(b.x).saturating_add(a.y.abs_diff(b.y))
}

#[test]
fn test_black_hole_mode_reset_spawns_permanent_singularity() {
    let game = hole_game(30, 30);

    assert!(
        game.black_hole.is_some(),
        "Black Hole mode must guarantee a singularity on reset"
    );
    assert_eq!(game.singularity_wanders, 0);
    assert_eq!(game.singularity_feeds, 0);
    assert!(game.player2.is_none(), "Black Hole mode is a solo arena");
    assert_eq!(
        game.weather,
        Weather::Clear,
        "ambient weather randomization is disabled in Black Hole mode"
    );
}

#[test]
fn test_black_hole_mode_singularity_never_collapses() {
    let mut game = solo_game();
    game.auto_pilot = true;

    let mut saw_wander = false;
    for _ in 0..240 {
        game.state = GameState::Playing;
        game.update();
        assert!(
            game.black_hole.is_some(),
            "the singularity must never despawn in Black Hole mode"
        );
        if game.singularity_wanders > 0 {
            saw_wander = true;
        }
        if game.state != GameState::Playing {
            break;
        }
    }

    assert!(
        saw_wander,
        "the singularity must wander on its 6-tick schedule (wanders: {})",
        game.singularity_wanders
    );
}

#[test]
fn test_black_hole_mode_wander_moves_at_most_one_cell_and_counts() {
    let mut game = solo_game();
    game.tick_counter = 5; // becomes 6 inside update() -> scheduled wander
    let before = game.black_hole.expect("hole present");

    game.state = GameState::Playing;
    game.update();

    let after = game.black_hole.expect("hole still present");
    assert_eq!(game.singularity_wanders, 1, "a scheduled wander must be counted");
    assert!(
        manhattan(before, after) <= 1,
        "the singularity drifts at most one cell per wander ({before:?} -> {after:?})"
    );
}

#[test]
fn test_black_hole_mode_no_wander_off_schedule() {
    let mut game = solo_game();
    game.tick_counter = 1; // becomes 2 inside update() -> no wander
    let before = game.black_hole.expect("hole present");

    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.singularity_wanders, 0);
    assert_eq!(
        game.black_hole,
        Some(before),
        "off-schedule ticks must not move the singularity"
    );
}

#[test]
fn test_black_hole_mode_pulls_food_every_tick() {
    let mut game = solo_game();
    game.tick_counter = 1; // no wander this tick
    game.food = Point { x: 15, y: 18 }; // 3 cells below the hole at (15,15)

    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.food,
        Point { x: 15, y: 17 },
        "in Black Hole mode the food is dragged one cell towards the singularity every tick"
    );
}

#[test]
fn test_black_hole_mode_devours_adjacent_obstacles_and_mines() {
    let mut game = solo_game();
    game.tick_counter = 1; // no wander this tick
    game.obstacles.insert(Point { x: 15, y: 16 }); // adjacent to the hole
    game.obstacles.insert(Point { x: 10, y: 10 }); // far away
    game.mines.insert(Point { x: 14, y: 15 }); // adjacent to the hole

    game.state = GameState::Playing;
    game.update();

    assert!(
        !game.obstacles.contains(&Point { x: 15, y: 16 }),
        "obstacles adjacent to the singularity are devoured"
    );
    assert!(
        game.obstacles.contains(&Point { x: 10, y: 10 }),
        "distant obstacles are untouched"
    );
    assert!(
        !game.mines.contains(&Point { x: 14, y: 15 }),
        "mines adjacent to the singularity are devoured"
    );
}

#[test]
fn test_black_hole_mode_eating_food_triggers_feeding_frenzy() {
    let mut game = solo_game();
    game.tick_counter = 1; // no wander this tick
    // The hole sits left of the snake: the food at (6,5) cannot be pulled further
    // (the snake's head blocks the way), so the snake moving Right eats it.
    game.black_hole = Some(Point { x: 3, y: 5 });
    game.food = Point { x: 6, y: 5 };
    game.obstacles.insert(Point { x: 3, y: 3 }); // distance 2 from the hole -> frenzy only
    game.obstacles.insert(Point { x: 5, y: 4 }); // distance 3 from the hole -> survives
    game.mines.insert(Point { x: 4, y: 5 }); // distance 1 -> devoured anyway
    let body_len_before = game.snake.body.len();

    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.body.len(),
        body_len_before + 1,
        "the snake must have eaten the food"
    );
    assert_eq!(
        game.singularity_feeds, 1,
        "eating food in Black Hole mode feeds the singularity"
    );
    assert!(
        !game.obstacles.contains(&Point { x: 3, y: 3 }),
        "the feeding frenzy devours obstacles within reach (distance <= 2)"
    );
    assert!(
        game.obstacles.contains(&Point { x: 5, y: 4 }),
        "obstacles outside the frenzy reach survive"
    );
    assert!(
        !game.mines.contains(&Point { x: 4, y: 5 }),
        "the frenzy devours mines within reach"
    );
}

#[test]
fn test_black_hole_mode_wander_fairness_never_lands_on_next_head() {
    let mut game = solo_game();
    game.auto_pilot = true;

    for _ in 0..120 {
        game.state = GameState::Playing;
        // Cells every snake is about to enter this tick (fairness contract)
        let mut next_heads: Vec<Point> = Vec::new();
        let p1_next = Game::calculate_next_head_dir(game.snake.head(), game.snake.direction);
        next_heads.push(p1_next);
        if let Some(p2) = &game.player2 {
            next_heads.push(Game::calculate_next_head_dir(p2.head(), p2.direction));
        }
        for b in &game.bots {
            next_heads.push(Game::calculate_next_head_dir(b.head(), b.direction));
        }
        let prev_hole = game.black_hole;

        game.update();

        if game.state != GameState::Playing {
            break;
        }
        if let Some(new_hole) = game.black_hole
            && prev_hole != Some(new_hole)
        {
            assert!(
                !next_heads.contains(&new_hole),
                "a wander must never land on the cell a snake is about to enter (wandered to {new_hole:?})"
            );
            assert!(
                !game.snake.body_map.contains_key(&new_hole),
                "a wander must never land on a snake body"
            );
        }
    }
}

#[test]
fn test_black_hole_mode_autopilot_avoids_the_singularity() {
    let mut game = solo_game();
    game.auto_pilot = true;
    // Put the singularity right in front of the bot: it must not move into it
    game.black_hole = Some(Point { x: 6, y: 5 });

    let next_move = game.calculate_autopilot_move();

    assert!(
        next_move == Some(Direction::Up) || next_move == Some(Direction::Down),
        "the autopilot must route around the singularity, chose {next_move:?}"
    );
}

#[test]
fn test_black_hole_mode_state_persists_in_history_state() {
    let mut game = hole_game(30, 30);
    game.black_hole = Some(Point { x: 12, y: 9 });
    game.singularity_wanders = 17;
    game.singularity_feeds = 5;

    game.save_history_state();

    let last = game.history.back().expect("history should have one entry");
    assert_eq!(last.black_hole, Some(Point { x: 12, y: 9 }));
    assert_eq!(last.singularity_wanders, 17);
    assert_eq!(
        last.singularity_feeds, 5,
        "feed counter must be stored in HistoryState for time-rewind consistency"
    );
}

#[test]
fn test_black_hole_mode_autopilot_game_runs_cleanly() {
    let mut game = hole_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.state = GameState::Playing;
        game.update();
        assert!(
            game.black_hole.is_some(),
            "the singularity must exist on every tick of a Black Hole run"
        );
        if game.state != GameState::Playing {
            break;
        }
    }

    assert!(
        game.black_hole.is_some(),
        "the singularity must still exist after the run"
    );
}
