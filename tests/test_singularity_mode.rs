use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn singularity_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Singularity;
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

fn place_snake(game: &mut Game, head: Point, dir: Direction) {
    game.snake = Snake::new(head);
    game.snake.direction = dir;
    game.snake.direction_queue.push_back(dir);
}

#[test]
fn test_singularity_reset_spawns_permanent_hole_far_from_spawn() {
    let game = singularity_game(30, 30);

    let bh = game
        .black_hole
        .expect("reset must spawn a black hole in Singularity mode");
    let spawn = game.snake.head();
    assert!(
        bh.x.abs_diff(spawn.x).saturating_add(bh.y.abs_diff(spawn.y)) >= 6,
        "the hole must spawn at least 6 cells away from the spawn point"
    );
    assert!(!game.obstacles.contains(&bh));
    assert!(!game.snake.body_map.contains_key(&bh));
    assert_eq!(game.singularity_pulses, 0);
    assert_eq!(game.singularity_drifts, 0);
    assert_eq!(game.singularity_shield, 0);
}

#[test]
fn test_singularity_hole_never_despawns_and_stays_fair() {
    let mut game = singularity_game(30, 30);
    game.obstacles.clear();
    game.auto_pilot = true;

    for _ in 0..120 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        let bh = game
            .black_hole
            .expect("the singularity must be permanent: it must never despawn in Singularity mode");
        assert!(
            bh.x > 0 && bh.x < game.width - 1 && bh.y > 0 && bh.y < game.height - 1,
            "the hole must stay in bounds, got {bh:?}"
        );
        assert!(!game.obstacles.contains(&bh), "the hole must never sit on an obstacle");
        assert!(
            !game.snake.body_map.contains_key(&bh),
            "the hole must never sit on a snake"
        );
    }
}

#[test]
fn test_singularity_pulse_fires_every_6_ticks() {
    let mut game = singularity_game(30, 30);
    game.obstacles.clear();
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must trigger a scheduled gravity pulse.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let pulses_before = game.singularity_pulses;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.singularity_pulses > pulses_before,
            "a scheduled gravity pulse must fire on tick {} (pulses before: {}, after: {})",
            tick + 1,
            pulses_before,
            game.singularity_pulses
        );
    }
    assert!(game.singularity_pulses >= 6);
}

#[test]
fn test_singularity_pulse_pulls_snake_toward_hole() {
    let mut game = singularity_game(30, 30);
    game.obstacles.clear();
    game.black_hole = Some(Point { x: 14, y: 10 });
    game.food = Point { x: 25, y: 25 };
    place_snake(&mut game, Point { x: 10, y: 10 }, Direction::Up);
    game.state = GameState::Playing;
    game.tick_counter = 5; // next update runs tick 6 -> pulse

    game.update();

    assert_eq!(game.singularity_pulses, 1);
    // Pulled one cell toward the hole ((10,10) -> (11,10)), then moved up to
    // (11,9). Without the pull the head would be at (10,9).
    assert_eq!(game.snake.head(), Point { x: 11, y: 9 });
    assert_eq!(game.lives, 3);
}

#[test]
fn test_singularity_pulse_never_pulls_onto_obstacle() {
    let mut game = singularity_game(30, 30);
    game.obstacles.clear();
    game.obstacles.insert(Point { x: 11, y: 10 });
    game.black_hole = Some(Point { x: 14, y: 10 });
    game.food = Point { x: 25, y: 25 };
    place_snake(&mut game, Point { x: 10, y: 10 }, Direction::Up);
    game.state = GameState::Playing;
    game.tick_counter = 5;

    game.update();

    assert_eq!(game.singularity_pulses, 1);
    assert_eq!(
        game.snake.head(),
        Point { x: 10, y: 9 },
        "the pull onto the obstacle cell must be refused; only the normal move happens"
    );
    assert_eq!(game.lives, 3);
}

#[test]
fn test_singularity_pulse_never_pulls_into_hole() {
    let mut game = singularity_game(30, 30);
    game.obstacles.clear();
    game.black_hole = Some(Point { x: 11, y: 10 });
    game.food = Point { x: 25, y: 25 };
    place_snake(&mut game, Point { x: 10, y: 10 }, Direction::Up);
    game.state = GameState::Playing;
    game.tick_counter = 5;

    game.update();

    assert_eq!(game.singularity_pulses, 1);
    assert_eq!(
        game.snake.head(),
        Point { x: 10, y: 9 },
        "the pull into the hole itself must be refused"
    );
    assert_eq!(game.lives, 3);
}

#[test]
fn test_singularity_shield_blocks_pulse_and_counts_down() {
    let mut game = singularity_game(30, 30);
    game.obstacles.clear();
    game.black_hole = Some(Point { x: 14, y: 10 });
    game.food = Point { x: 25, y: 25 };
    place_snake(&mut game, Point { x: 10, y: 10 }, Direction::Up);
    game.singularity_shield = 6;
    game.state = GameState::Playing;
    game.tick_counter = 5;

    game.update();

    assert_eq!(game.singularity_pulses, 1);
    assert_eq!(
        game.singularity_shield, 5,
        "the shield must count down once per tick"
    );
    assert_eq!(
        game.snake.head(),
        Point { x: 10, y: 9 },
        "a shielded snake must not be pulled by the pulse"
    );
}

#[test]
fn test_singularity_eating_food_grants_shield() {
    let mut game = singularity_game(30, 30);
    game.obstacles.clear();
    game.black_hole = Some(Point { x: 25, y: 25 });
    game.food = Point { x: 10, y: 9 };
    place_snake(&mut game, Point { x: 10, y: 10 }, Direction::Up);
    game.state = GameState::Playing;
    game.tick_counter = 0; // next update runs tick 1: no pulse, no drift

    game.update();

    assert_eq!(game.snake.head(), Point { x: 10, y: 9 });
    assert_eq!(
        game.singularity_shield,
        Game::SINGULARITY_SHIELD_TICKS,
        "eating food must grant a gravity shield"
    );
}

#[test]
fn test_singularity_drift_every_8_ticks_moves_hole_one_fair_cell() {
    let mut game = singularity_game(30, 30);
    game.obstacles.clear();
    game.black_hole = Some(Point { x: 15, y: 15 });
    game.food = Point { x: 25, y: 5 };
    place_snake(&mut game, Point { x: 5, y: 25 }, Direction::Left);
    game.state = GameState::Playing;
    game.tick_counter = 7; // next update runs tick 8 -> drift

    game.update();

    assert_eq!(game.singularity_drifts, 1);
    let bh = game.black_hole.expect("the hole must still exist");
    let dist = bh.x.abs_diff(15).saturating_add(bh.y.abs_diff(15));
    assert_eq!(dist, 1, "the hole must drift exactly one cell, got {bh:?}");
    assert!(!game.obstacles.contains(&bh));
    assert!(!game.snake.body_map.contains_key(&bh));
}

#[test]
fn test_singularity_hole_cell_is_lethal() {
    let mut game = singularity_game(30, 30);
    game.obstacles.clear();
    game.black_hole = Some(Point { x: 10, y: 9 });
    game.food = Point { x: 25, y: 25 };
    place_snake(&mut game, Point { x: 10, y: 10 }, Direction::Up); // moves into the hole
    game.state = GameState::Playing;
    game.tick_counter = 0; // next update runs tick 1: no pulse

    game.update();

    assert_eq!(game.lives, 2, "moving into the black hole must cost a life");
}
