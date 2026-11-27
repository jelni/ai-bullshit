use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn gravity_well_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::GravityWell;
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

fn manhattan(a: Point, b: Point) -> u32 {
    u32::from(a.x.abs_diff(b.x)) + u32::from(a.y.abs_diff(b.y))
}

#[test]
fn test_gravity_well_reset_seeds_hole_and_clear_weather() {
    let game = gravity_well_game(30, 30);

    assert_eq!(game.weather, Weather::Clear);
    assert_eq!(game.gravity_well_surges, 0);
    let hole = game.black_hole.expect("Gravity Well mode must seed a black hole on reset");
    assert!(hole.x > 0 && hole.x < game.width - 1 && hole.y > 0 && hole.y < game.height - 1);
    assert!(!game.snake.body_map.contains_key(&hole));
    assert_ne!(game.food, hole);
    assert!(!game.obstacles.contains(&hole));
}

#[test]
fn test_gravity_well_weather_never_breaks() {
    let mut game = gravity_well_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..60 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        assert_eq!(
            game.weather,
            Weather::Clear,
            "Gravity Well mode must pin the weather to Clear on every tick"
        );
    }
}

#[test]
fn test_gravity_well_hole_never_despawns() {
    let mut game = gravity_well_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..240 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        assert!(
            game.black_hole.is_some(),
            "the gravity well is permanent and must never despawn in Gravity Well mode"
        );
    }
}

#[test]
fn test_gravity_well_scheduled_surge_every_6_ticks() {
    let mut game = gravity_well_game(30, 30);
    game.obstacles.clear();
    game.auto_pilot = true;

    // update() increments tick_counter first, so setting it to 6k - 1 makes the
    // next update run tick 6k, which must trigger a scheduled surge.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let surges_before = game.gravity_well_surges;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.gravity_well_surges > surges_before,
            "a scheduled surge must fire on tick {} (surges before: {}, after: {})",
            tick + 1,
            surges_before,
            game.gravity_well_surges
        );
    }
    assert!(game.gravity_well_surges >= 6);
}

#[test]
fn test_gravity_well_surge_moves_hole_at_most_one_cell() {
    let mut game = gravity_well_game(40, 40);
    game.obstacles.clear();
    game.auto_pilot = true;

    for tick in [5_u32, 11, 17, 23, 29, 35, 41, 47] {
        game.state = GameState::Playing;
        game.tick_counter = tick;
        let before = game.black_hole;
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        if let (Some(b), Some(a)) = (before, game.black_hole) {
            assert!(
                manhattan(a, b) <= 1,
                "a surge may only drift the hole one cell (tick {}: {:?} -> {:?})",
                tick + 1,
                b,
                a
            );
        }
    }
}

#[test]
fn test_gravity_well_surge_never_unfair_over_many_ticks() {
    let mut game = gravity_well_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..300 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
        if let Some(hole) = game.black_hole {
            assert!(
                hole.x > 0 && hole.x < game.width - 1 && hole.y > 0 && hole.y < game.height - 1,
                "the hole must stay inside the board, got {hole:?}"
            );
            assert!(
                !game.snake.body_map.contains_key(&hole),
                "a surge must never land on the player's body (hole: {hole:?})"
            );
            assert!(
                game.player2
                    .as_ref()
                    .is_none_or(|p2| !p2.body_map.contains_key(&hole)),
                "a surge must never land on player 2's body"
            );
            assert!(
                game.bots.iter().all(|b| !b.body_map.contains_key(&hole)),
                "a surge must never land on a bot's body"
            );
            assert_ne!(game.food, hole, "a surge must never land on the food");
            assert!(
                game.bonus_food.is_none_or(|(bp, _)| bp != hole),
                "a surge must never land on the bonus food"
            );
            assert!(
                game.poison_food.is_none_or(|(pp, _)| pp != hole),
                "a surge must never land on the poison food"
            );
            assert!(
                game.power_up.as_ref().is_none_or(|pu| pu.location != hole),
                "a surge must never land on the power-up"
            );
            assert!(
                game.portals.is_none_or(|(p1, p2)| hole != p1 && hole != p2),
                "a surge must never land on a portal"
            );
        }
    }
}

#[test]
fn test_gravity_well_pulls_food_toward_hole() {
    let mut game = gravity_well_game(30, 30);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 2, y: 2 });
    game.snake.direction = Direction::Up;
    game.food = Point { x: 8, y: 8 };
    game.black_hole = Some(Point { x: 20, y: 20 });

    let hole = game.black_hole.unwrap();
    let initial_dist = manhattan(game.food, hole);
    let mut closer = false;
    for _ in 0..60 {
        game.state = GameState::Playing;
        game.update();
        // pin the hole to isolate the pull
        game.black_hole = Some(hole);
        if game.lives == 0 {
            break;
        }
        if manhattan(game.food, hole) < initial_dist {
            closer = true;
            break;
        }
    }

    assert!(
        closer,
        "In Gravity Well mode the hole must tug the food towards itself every tick"
    );
}

#[test]
fn test_gravity_well_hole_is_lethal() {
    let mut game = gravity_well_game(20, 20);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 10, y: 10 });
    game.snake.direction = Direction::Down;
    game.black_hole = Some(Point { x: 10, y: 11 });
    game.tick_counter = 1; // not a multiple of 6: no surge can move the hole away

    game.state = GameState::Playing;
    let lives_before = game.lives;
    game.update();

    assert_eq!(
        game.lives,
        lives_before - 1,
        "Snake should die when moving into the gravity well"
    );
}

#[test]
fn test_gravity_well_bot_survives_the_well() {
    let mut game = gravity_well_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..150 {
        game.update();
        keep_playing(&mut game);
        if game.lives == 0 {
            break;
        }
    }

    assert!(
        game.lives > 0,
        "Autopilot should survive the gravity well (lives: {}, surges: {}, score: {})",
        game.lives,
        game.gravity_well_surges,
        game.score
    );
    assert!(
        game.gravity_well_surges >= 20,
        "expected at least 20 surges over 150 ticks, got {}",
        game.gravity_well_surges
    );
}

#[test]
fn test_gravity_well_persists_through_save_load() {
    let file_path = format!("gravity_well_save_test_{}.json", std::process::id());
    let _ = std::fs::remove_file(&file_path);

    let mut game = gravity_well_game(20, 20);
    game.gravity_well_surges = 11;
    game.black_hole = Some(Point { x: 12, y: 9 });
    game.save_game_to_file(&file_path);

    let json = std::fs::read_to_string(&file_path).expect("save file should exist");
    assert!(
        json.contains("\"gravity_well_surges\":11"),
        "the save file must contain the surge counter"
    );
    let _ = std::fs::remove_file(&file_path);

    // Round-trip through the public load path (reads savegame.json)
    let _ = std::fs::remove_file("savegame.json");
    game.save_game_to_file("savegame.json");
    let mut loaded = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    assert!(loaded.load_game());
    assert_eq!(loaded.mode, GameMode::GravityWell);
    assert_eq!(loaded.gravity_well_surges, 11);
    assert_eq!(loaded.black_hole, Some(Point { x: 12, y: 9 }));
    let _ = std::fs::remove_file("savegame.json");
}

#[test]
fn test_gravity_well_persists_through_time_rewind() {
    let mut game = gravity_well_game(30, 30);
    game.auto_pilot = true;

    // save_history_state runs at the start of every tick, so rewinding after
    // tick N restores the state as it was at the end of tick N - 1.
    let mut recorded = None;
    for i in 0..6 {
        game.state = GameState::Playing;
        game.update();
        if game.state != GameState::Playing {
            break;
        }
        if i == 4 {
            recorded = Some((game.black_hole, game.gravity_well_surges));
        }
    }
    let Some((hole_before, surges_before)) = recorded else {
        panic!("a hole should be present after a few ticks");
    };

    // Scramble the live state, then rewind: hole and counter must be restored
    game.black_hole = None;
    game.gravity_well_surges = 999;
    game.rewind_time();

    assert_eq!(game.black_hole, hole_before);
    assert_eq!(game.gravity_well_surges, surges_before);
}
