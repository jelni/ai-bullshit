use snake_game::game::{Difficulty, Game, GameMode, GameState, HeroClass, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn warp_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Warp;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    // The Druid ignores ambient weather effects, so only the portals themselves
    // matter in these tests (keeps the assertions deterministic).
    game.stats.equipped_class = Some(HeroClass::Druid);
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

fn manhattan(a: Point, b: Point) -> u16 {
    a.x.abs_diff(b.x).saturating_add(a.y.abs_diff(b.y))
}

/// Pins the snake in a small 2x2 loop at (3,3)-(4,4) so it cannot run into
/// walls during longer scheduled-event tests. Call `setup_snake_loop` once,
/// then `queue_loop_dir` before every update.
fn setup_snake_loop(game: &mut Game) {
    game.snake = Snake::new(Point { x: 3, y: 3 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.clear();
    game.snake.direction_queue.push_back(Direction::Right);
}

fn queue_loop_dir(game: &mut Game, dir_cycle: &mut usize) {
    const DIRS: [Direction; 4] = [
        Direction::Right,
        Direction::Down,
        Direction::Left,
        Direction::Up,
    ];
    let dir = DIRS[*dir_cycle % 4];
    *dir_cycle += 1;
    game.snake.direction_queue.push_back(dir);
}

/// Neutralizes unrelated random hazards so long-running tests stay
/// deterministic (portals are intentionally left alone in Warp mode).
fn calm_the_board(game: &mut Game) {
    game.weather = Weather::Clear;
    game.mines.clear();
    game.poison_food = None;
    game.bonus_food = None;
    game.power_up = None;
}

#[test]
fn test_warp_reset_spawns_permanent_fair_portal_pair() {
    let game = warp_game(30, 30);

    let (p1, p2) = game
        .portals
        .expect("Warp mode must spawn a portal pair on reset");
    let head = game.snake.head();
    assert!(
        manhattan(p1, head) >= 8 && manhattan(p2, head) >= 8,
        "both portals must spawn at least 8 cells away from the snake head"
    );
    assert!(manhattan(p1, p2) >= 6, "the pair must be far enough apart");
    assert!(!game.obstacles.contains(&p1) && !game.obstacles.contains(&p2));
    assert_ne!(game.food, p1);
    assert_ne!(game.food, p2);
    assert!(game.warp_targets.is_none());
    assert_eq!(game.warp_relocations, 0);
}

#[test]
fn test_warp_teleports_snake_through_portal() {
    let mut game = warp_game(30, 30);
    game.obstacles.clear();
    // Snake::new builds the body at +y from the head, so moving Right is safe.
    game.snake = Snake::new(Point { x: 9, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.portals = Some((Point { x: 10, y: 10 }, Point { x: 20, y: 20 }));
    game.food = Point { x: 2, y: 2 };
    let lives_before = game.lives;

    game.state = GameState::Playing;
    game.weather = Weather::Clear;
    game.update();

    assert_eq!(
        game.snake.head(),
        Point { x: 20, y: 20 },
        "moving into the first portal must teleport the head to the second"
    );
    assert_eq!(
        game.lives, lives_before,
        "teleporting through a fair portal pair must not kill the snake"
    );
}

#[test]
fn test_warp_scheduled_relocation_is_telegraphed_then_moves() {
    let mut game = warp_game(40, 30);
    game.obstacles.clear();
    game.portals = Some((Point { x: 30, y: 20 }, Point { x: 35, y: 25 }));
    game.food = Point { x: 20, y: 5 };
    game.tick_counter = Game::WARP_RELOCATE_INTERVAL - 1;
    let mut dir_cycle = 0usize;

    setup_snake_loop(&mut game);
    game.state = GameState::Playing;
    calm_the_board(&mut game);
    game.update(); // tick 45: the relocation is scheduled and telegraphed

    let targets = game
        .warp_targets
        .expect("tick 45 must telegraph a relocation target pair");
    assert_eq!(
        game.warp_relocate_countdown,
        Game::WARP_TELEGRAPH,
        "the telegraph countdown must start at WARP_TELEGRAPH"
    );
    let head_after = game.snake.head();
    assert!(
        manhattan(targets.0, head_after) >= 6 && manhattan(targets.1, head_after) >= 6,
        "relocation targets must be fair: at least 6 cells from the snake"
    );
    assert_eq!(
        game.portals,
        Some((Point { x: 30, y: 20 }, Point { x: 35, y: 25 })),
        "the portals must stay put while the relocation is telegraphed"
    );

    for _ in 0..Game::WARP_TELEGRAPH {
        queue_loop_dir(&mut game, &mut dir_cycle);
        calm_the_board(&mut game);
        game.state = GameState::Playing;
        game.update();
    }

    assert_eq!(
        game.warp_relocations, 1,
        "exactly one relocation must have completed after the telegraph countdown"
    );
    assert_eq!(
        game.portals,
        Some(targets),
        "the portals must move to the telegraphed target pair"
    );
    assert!(
        game.warp_targets.is_none(),
        "the telegraph marker must clear once the relocation lands"
    );
}

#[test]
fn test_warp_portals_never_randomly_spawn_or_despawn() {
    let mut game = warp_game(40, 30);
    game.obstacles.clear();
    game.food = Point { x: 20, y: 5 };
    let mut dir_cycle = 0usize;
    setup_snake_loop(&mut game);

    for _ in 0..100 {
        queue_loop_dir(&mut game, &mut dir_cycle);
        calm_the_board(&mut game);
        game.state = GameState::Playing;
        game.update();
        assert!(
            game.portals.is_some(),
            "the portal pair is permanent in Warp mode and must never despawn"
        );
    }
}

#[test]
fn test_warp_state_persists_in_history_state() {
    let mut game = warp_game(30, 30);
    game.portals = Some((Point { x: 20, y: 20 }, Point { x: 8, y: 8 }));
    game.warp_targets = Some((Point { x: 12, y: 9 }, Point { x: 25, y: 25 }));
    game.warp_relocate_countdown = 3;
    game.warp_relocations = 2;

    game.save_history_state();

    let last = game.history.back().expect("history should have one entry");
    assert_eq!(
        last.portals,
        Some((Point { x: 20, y: 20 }, Point { x: 8, y: 8 }))
    );
    assert_eq!(
        last.warp_targets,
        Some((Point { x: 12, y: 9 }, Point { x: 25, y: 25 }))
    );
    assert_eq!(last.warp_relocate_countdown, 3);
    assert_eq!(last.warp_relocations, 2);
}

#[test]
fn test_warp_persists_through_save_and_load() {
    let mut game = warp_game(30, 30);
    game.portals = Some((Point { x: 20, y: 20 }, Point { x: 8, y: 8 }));
    game.warp_targets = Some((Point { x: 12, y: 9 }, Point { x: 25, y: 25 }));
    game.warp_relocate_countdown = 4;
    game.warp_relocations = 7;

    let file_path = "savegame_test_warp_integration.json";
    game.save_game_to_file(file_path);
    let json = std::fs::read_to_string(file_path).expect("save file should exist");
    assert!(
        json.contains("\"warp_relocations\":7"),
        "the save file must contain the relocation counter"
    );
    let _ = std::fs::remove_file(file_path);

    // Round-trip through the public load path (reads savegame.json)
    let _ = std::fs::remove_file("savegame.json");
    game.save_game_to_file("savegame.json");
    let mut loaded = Game::new(30, 30, false, 'x', Theme::Classic, Difficulty::Normal);
    assert!(loaded.load_game());
    assert_eq!(loaded.mode, GameMode::Warp);
    assert_eq!(
        loaded.portals,
        Some((Point { x: 20, y: 20 }, Point { x: 8, y: 8 }))
    );
    assert_eq!(
        loaded.warp_targets,
        Some((Point { x: 12, y: 9 }, Point { x: 25, y: 25 }))
    );
    assert_eq!(loaded.warp_relocate_countdown, 4);
    assert_eq!(loaded.warp_relocations, 7);
    let _ = std::fs::remove_file("savegame.json");
}

#[test]
fn test_warp_autopilot_runs_cleanly() {
    let mut game = warp_game(40, 30);
    game.obstacles.clear();
    game.auto_pilot = true;

    for _ in 0..120 {
        calm_the_board(&mut game);
        if game.state != GameState::Playing {
            keep_playing(&mut game);
            if game.state != GameState::Playing {
                break;
            }
        }
        game.update();
        if game.state == GameState::Playing {
            let head = game.snake.head();
            assert!(
                !game.obstacles.contains(&head),
                "the autopilot must never end a tick on an obstacle"
            );
        }
    }

    assert!(
        game.warp_relocations >= 1,
        "over 120 ticks at least one scheduled relocation should have completed (interval 45)"
    );
}
