use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn maelstrom_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Maelstrom;
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

// Place the vortex at `vortex` with its lethal obstacle mirror, on a cleared board
fn place_vortex(game: &mut Game, vortex: Point) {
    game.obstacles.clear();
    game.maelstrom = Some(vortex);
    game.obstacles.insert(vortex);
}

#[test]
fn test_maelstrom_reset_seeds_vortex_fairly() {
    let game = maelstrom_game(30, 30);

    assert_eq!(game.weather, Weather::Rain);
    assert_eq!(game.maelstrom_swirls, 0);
    assert_eq!(game.maelstrom_drifts, 0);
    assert_eq!(game.maelstrom_calm, 0);
    let vortex = game.maelstrom.expect("Maelstrom mode must spawn a vortex on reset");
    let spawn = Point {
        x: 30 / 2,
        y: 30 / 2,
    };
    assert!(
        !game.snake.body_map.contains_key(&vortex),
        "the vortex must never spawn on the snake"
    );
    assert_ne!(vortex, game.food, "the vortex must never spawn on the food");
    assert!(
        vortex.x.abs_diff(spawn.x).saturating_add(vortex.y.abs_diff(spawn.y)) >= 6,
        "the vortex must spawn at least 6 cells from the spawn point: {vortex:?}"
    );
    assert!(
        game.obstacles.contains(&vortex),
        "the vortex cell must be mirrored into obstacles (lethal wall)"
    );
}

#[test]
fn test_maelstrom_weather_never_breaks() {
    let mut game = maelstrom_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..60 {
        game.update();
        keep_playing(&mut game);
        assert_eq!(
            game.weather,
            Weather::Rain,
            "Maelstrom mode must force rain weather on every tick"
        );
    }
}

#[test]
fn test_maelstrom_vortex_cell_is_lethal_obstacle() {
    let mut game = maelstrom_game(20, 20);
    place_vortex(&mut game, Point { x: 5, y: 4 });

    assert!(
        !game.is_safe_final_p(Point { x: 5, y: 4 }, 1, 1),
        "is_safe_final_p must reject the maelstrom vortex cell"
    );
    assert!(
        game.is_safe_final_p(Point { x: 6, y: 5 }, 1, 1),
        "a different cell must remain safe"
    );
}

#[test]
fn test_maelstrom_kills_snake_entering_vortex() {
    let mut game = maelstrom_game(20, 20);
    place_vortex(&mut game, Point { x: 5, y: 4 });
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Up;
    game.snake.direction_queue.push_back(Direction::Up);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = false;
    game.tick_counter = 1; // tick 2: no scheduled swirl or drift this tick
    let lives_before = game.lives;

    game.update();

    assert!(
        game.lives < lives_before,
        "moving into the maelstrom vortex must cost a life (lives before: {}, after: {})",
        lives_before,
        game.lives
    );
}

#[test]
fn test_maelstrom_swirl_shoves_nearby_snake_tangentially() {
    let mut game = maelstrom_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    place_vortex(&mut game, Point { x: 15, y: 15 });
    pin_ambient_spawners(&mut game);
    // The snake moves right to (13,15) this tick; the vortex sits at (15,15),
    // so the swirl step is perpendicular to the radial direction: upward
    game.snake = Snake::new(Point { x: 12, y: 15 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 5; // tick 6: a scheduled swirl fires
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.tick_counter, 6);
    assert_eq!(game.maelstrom_swirls, 1, "the scheduled swirl must fire on tick 6");
    assert_eq!(
        game.snake.head(),
        Point { x: 13, y: 14 },
        "the swirl must shove the snake one cell tangentially around the vortex (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a swirl must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_maelstrom_swirl_never_shoves_into_hazard() {
    let mut game = maelstrom_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(7);
    place_vortex(&mut game, Point { x: 15, y: 15 });
    pin_ambient_spawners(&mut game);
    // Both tangential destinations from (13,15) are walls: no shove may happen
    game.obstacles.insert(Point { x: 13, y: 14 });
    game.obstacles.insert(Point { x: 13, y: 16 });
    game.snake = Snake::new(Point { x: 12, y: 15 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;

    game.tick_counter = 5; // tick 6: a scheduled swirl fires
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.maelstrom_swirls, 1, "the swirl event still fires on tick 6");
    assert_eq!(
        game.snake.head(),
        Point { x: 13, y: 15 },
        "with both tangential cells blocked the snake must stay put (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.state == GameState::Playing && game.lives > 0,
        "a blocked swirl must never kill the snake (lives: {})",
        game.lives
    );
}

#[test]
fn test_maelstrom_swirl_leaves_far_snakes_alone() {
    let mut game = maelstrom_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(11);
    place_vortex(&mut game, Point { x: 15, y: 15 });
    pin_ambient_spawners(&mut game);
    // The snake is outside the swirl radius: only its normal move happens
    game.snake = Snake::new(Point { x: 25, y: 25 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 5, y: 5 };
    game.auto_pilot = false;

    game.tick_counter = 5; // tick 6: a scheduled swirl fires
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.maelstrom_swirls, 1);
    assert_eq!(
        game.snake.head(),
        Point { x: 26, y: 25 },
        "a snake outside the radius must only make its normal move (head: {:?})",
        game.snake.head()
    );
}

#[test]
fn test_maelstrom_calm_suppresses_swirls() {
    let mut game = maelstrom_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(13);
    place_vortex(&mut game, Point { x: 15, y: 15 });
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 12, y: 15 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game.auto_pilot = false;
    game.maelstrom_calm = 6;

    game.tick_counter = 5; // tick 6: a swirl would fire, but calm suppresses it
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.maelstrom_swirls, 0,
        "no swirl may fire while the waters are calm"
    );
    assert_eq!(
        game.snake.head(),
        Point { x: 13, y: 15 },
        "a calm swirl tick must only make the normal move (head: {:?})",
        game.snake.head()
    );
    assert_eq!(
        game.maelstrom_calm, 5,
        "calm must count down once per tick (6 -> 5)"
    );
}

#[test]
fn test_maelstrom_eating_food_calms_the_waters() {
    let mut game = maelstrom_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(17);
    place_vortex(&mut game, Point { x: 25, y: 25 });
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 11, y: 10 };
    game.auto_pilot = false;

    game.tick_counter = 0; // tick 1: no scheduled swirl or drift
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 11, y: 10 },
        "the snake must reach the food (head: {:?})",
        game.snake.head()
    );
    assert!(
        game.score > 0,
        "the snake must have eaten the food (score: {})",
        game.score
    );
    assert_eq!(
        game.maelstrom_calm,
        Game::MAELSTROM_CALM_TICKS,
        "eating food must calm the waters for MAELSTROM_CALM_TICKS ticks"
    );
}

#[test]
fn test_maelstrom_drift_moves_vortex_fairly_every_8_ticks() {
    let mut game = maelstrom_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    place_vortex(&mut game, Point { x: 15, y: 15 });
    pin_ambient_spawners(&mut game);
    // The snake is far away, so every drift direction is fair
    game.snake = Snake::new(Point { x: 5, y: 25 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 5 };
    game.auto_pilot = false;

    game.tick_counter = 7; // tick 8: a scheduled drift fires (no swirl on tick 8)
    game.state = GameState::Playing;
    let drifts_before = game.maelstrom_drifts;
    game.update();

    assert_eq!(game.tick_counter, 8);
    assert_eq!(
        game.maelstrom_drifts,
        drifts_before + 1,
        "a scheduled drift must move the vortex on tick 8"
    );
    let vortex = game.maelstrom.expect("the vortex must still exist after drifting");
    assert_ne!(
        vortex,
        Point { x: 15, y: 15 },
        "the vortex must have drifted one cell"
    );
    assert!(
        game.obstacles.contains(&vortex),
        "the new vortex cell must be mirrored into obstacles"
    );
    assert!(
        !game.obstacles.contains(&Point { x: 15, y: 15 }),
        "the old vortex cell must no longer be mirrored"
    );
    assert!(
        vortex.x.abs_diff(15).saturating_add(vortex.y.abs_diff(15)) == 1,
        "the vortex must drift exactly one cell: {vortex:?}"
    );
}

#[test]
fn test_maelstrom_swirl_drags_food_towards_vortex() {
    let mut game = maelstrom_game(30, 30);
    game.rng = rand::rngs::StdRng::seed_from_u64(23);
    place_vortex(&mut game, Point { x: 15, y: 15 });
    pin_ambient_spawners(&mut game);
    game.snake = Snake::new(Point { x: 5, y: 25 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 18, y: 15 };
    game.auto_pilot = false;

    game.tick_counter = 5; // tick 6: a scheduled swirl fires
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.maelstrom_swirls, 1);
    assert_eq!(
        game.food,
        Point { x: 17, y: 15 },
        "the swirl must drag the food one cell towards the vortex (food: {:?})",
        game.food
    );
}

#[test]
fn test_maelstrom_bot_survives_the_maelstrom() {
    let mut game = maelstrom_game(40, 30);
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
        "Autopilot should survive the maelstrom (lives: {}, swirls: {}, drifts: {}, score: {})",
        game.lives,
        game.maelstrom_swirls,
        game.maelstrom_drifts,
        game.score
    );
    assert!(
        game.maelstrom_swirls >= 15,
        "expected at least 15 swirls over 150 ticks, got {}",
        game.maelstrom_swirls
    );
    assert!(
        game.maelstrom_drifts >= 10,
        "expected at least 10 drifts over 150 ticks, got {}",
        game.maelstrom_drifts
    );
}

#[test]
fn test_maelstrom_state_persists_through_history_rewind() {
    let mut game = maelstrom_game(30, 30);
    game.maelstrom = Some(Point { x: 12, y: 9 });
    game.maelstrom_swirls = 7;
    game.maelstrom_drifts = 3;
    game.maelstrom_calm = 2;
    game.save_history_state();

    game.maelstrom = Some(Point { x: 20, y: 20 });
    game.maelstrom_swirls = 99;
    game.maelstrom_drifts = 99;
    game.maelstrom_calm = 99;
    game.rewind_time();

    assert_eq!(
        game.maelstrom,
        Some(Point { x: 12, y: 9 }),
        "maelstrom must be restored by time-rewind"
    );
    assert_eq!(game.maelstrom_swirls, 7, "swirls must be restored by time-rewind");
    assert_eq!(game.maelstrom_drifts, 3, "drifts must be restored by time-rewind");
    assert_eq!(game.maelstrom_calm, 2, "calm must be restored by time-rewind");
}

#[test]
fn test_maelstrom_state_persists_through_save_file() {
    let mut game = maelstrom_game(30, 30);
    game.maelstrom = Some(Point { x: 12, y: 9 });
    game.maelstrom_swirls = 12;
    game.maelstrom_drifts = 5;
    game.maelstrom_calm = 4;
    let path = std::env::temp_dir().join(format!(
        "snake_game_maelstrom_save_test_{}.json",
        std::process::id()
    ));
    let path_str = path.to_string_lossy().to_string();
    game.save_game_to_file(&path_str);

    let contents = std::fs::read_to_string(&path).expect("save file must exist");
    assert!(
        contents.contains("\"maelstrom_swirls\":12"),
        "maelstrom_swirls must be persisted in the save file, got: {}",
        &contents[..contents.len().min(200)]
    );
    assert!(
        contents.contains("\"maelstrom_drifts\":5"),
        "maelstrom_drifts must be persisted in the save file"
    );
    assert!(
        contents.contains("\"maelstrom_calm\":4"),
        "maelstrom_calm must be persisted in the save file"
    );

    let _ = std::fs::remove_file(&path);
    // The full load round-trip is covered by the in-crate unit test
    // `test_save_and_load_maelstrom_mode` in src/game/tests.rs
}
