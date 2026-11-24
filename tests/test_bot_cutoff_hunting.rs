use snake_game::game::{Difficulty, Game, GameMode, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn base_game() -> Game {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();
    game.bosses.clear();
    game.lasers.clear();
    game.mines.clear();
    game.meteors.clear();
    game.black_hole = None;
    game.lightning_column = None;
    game.poison_food = None;
    game.bonus_food = None;
    game.power_up = None;
    game.goblin = None;
    game.portals = None;
    game.bots.clear();
    game.bots_autopilot_paths.clear();
    game.weather = Weather::Clear;
    game
}

fn set_body(snake: &mut Snake, cells: &[Point]) {
    snake.body = cells.iter().copied().collect();
    snake.body_map.clear();
    for p in &snake.body {
        *snake.body_map.entry(*p).or_insert(0) += 1;
    }
}

fn p(x: u16, y: u16) -> Point {
    Point { x, y }
}

// Player bot at (8,8) facing Right with length 5; p2 at (10,5) facing Down
// with length 3, so its head lane runs down the x=10 column: (10,6), (10,7),
// (10,8), ... Reaching (10,8) in two steps puts the player's head on p2's
// step-3 cell one tick early, so p2 crashes into it.
fn hunt_setup() -> Game {
    let mut game = base_game();
    game.mode = GameMode::BotVsBot;
    game.snake = Snake::new(p(8, 8));
    game.snake.direction = Direction::Right;
    set_body(
        &mut game.snake,
        &[p(8, 8), p(7, 8), p(6, 8), p(5, 8), p(4, 8)],
    );
    let mut p2 = Snake::new(p(10, 5));
    p2.direction = Direction::Down;
    set_body(&mut p2, &[p(10, 5), p(10, 4), p(10, 3)]);
    game.player2 = Some(p2);
    game.food = p(18, 18);
    game
}

#[test]
fn test_cutoff_targets_lists_rival_head_lane_cells() {
    let game = hunt_setup();
    let targets = game.cutoff_targets(1, p(8, 8), 5);
    assert!(
        targets.contains(&p(10, 7)),
        "step-2 rival head cell must be a cut-off candidate"
    );
    assert!(
        targets.contains(&p(10, 8)),
        "step-3 rival head cell must be a cut-off candidate"
    );
    assert!(
        targets.contains(&p(10, 9)),
        "later rival head cells within the horizon must be candidates"
    );
    assert!(
        !targets.contains(&p(10, 6)),
        "step-1 cell cannot be intercepted (the mover must arrive by step k-1 >= 1)"
    );
    assert!(
        !targets.contains(&p(8, 8)),
        "the mover's own head is never a cut-off target"
    );
}

#[test]
fn test_cutoff_targets_empty_when_rival_not_shorter() {
    let mut game = hunt_setup();
    // Equal length: no hunting.
    set_body(
        &mut game.player2.as_mut().unwrap(),
        &[p(10, 5), p(10, 4), p(10, 3), p(10, 2), p(10, 1)],
    );
    assert!(
        game.cutoff_targets(1, p(8, 8), 5).is_empty(),
        "rivals at least as long as the mover are never hunted"
    );

    // Shorter rival but non-hunting mode: no targets.
    let mut game = hunt_setup();
    game.mode = GameMode::SinglePlayer;
    assert!(
        game.cutoff_targets(1, p(8, 8), 5).is_empty(),
        "hunting must be disabled outside rival modes"
    );

    // Rival too far away to intercept within the horizon.
    let mut game = hunt_setup();
    let mut p2 = Snake::new(p(18, 18));
    p2.direction = Direction::Down;
    set_body(&mut p2, &[p(18, 18), p(18, 17), p(18, 16)]);
    game.player2 = Some(p2);
    assert!(
        game.cutoff_targets(1, p(8, 8), 5).is_empty(),
        "rivals beyond the intercept distance must be ignored"
    );
}

#[test]
fn test_path_cuts_off_rival_detects_intercept() {
    let game = hunt_setup();
    let start = game.snake.head();
    // Arrive at p2's step-3 head cell (10,8) on step 2: p2 dies on tick 3.
    let killing = vec![p(9, 8), p(10, 8)];
    assert!(
        game.path_cuts_off_rival(start, &killing, 1),
        "landing on the rival's predicted head cell one step early must count as a cut-off"
    );
    // A path that misses the rival's lane must not count.
    let missing = vec![p(9, 8), p(9, 9)];
    assert!(
        !game.path_cuts_off_rival(start, &missing, 1),
        "a path away from the rival's head lane must not count as a cut-off"
    );
    assert!(
        !game.path_cuts_off_rival(start, &[], 1),
        "an empty path can never cut anyone off"
    );
}

#[test]
fn test_path_cuts_off_rival_respects_length_gate() {
    let mut game = hunt_setup();
    // Make p2 as long as the player: the same path must no longer count.
    set_body(
        &mut game.player2.as_mut().unwrap(),
        &[p(10, 5), p(10, 4), p(10, 3), p(10, 2), p(10, 1)],
    );
    let killing = vec![p(9, 8), p(10, 8)];
    assert!(
        !game.path_cuts_off_rival(game.snake.head(), &killing, 1),
        "rivals at least as long as the mover are never considered cut off"
    );
}

#[test]
fn test_autopilot_hunts_shorter_rival() {
    let mut game = hunt_setup();
    let mv = game
        .calculate_autopilot_move()
        .expect("bot should find a hunting move");
    assert_eq!(
        mv,
        Direction::Right,
        "bot should step toward the intercept cell (10,8)"
    );
    assert!(
        game.path_cuts_off_rival(game.snake.head(), &game.autopilot_path, 1),
        "the chosen autopilot path must actually cut off the rival"
    );
    assert_eq!(
        game.autopilot_path.last(),
        Some(&p(10, 8)),
        "the hunting path should end on the rival's predicted lane"
    );
}

#[test]
fn test_autopilot_does_not_hunt_equal_length_rival() {
    let mut game = hunt_setup();
    set_body(
        &mut game.player2.as_mut().unwrap(),
        &[p(10, 5), p(10, 4), p(10, 3), p(10, 2), p(10, 1)],
    );
    game.food = p(12, 8);
    let mv = game
        .calculate_autopilot_move()
        .expect("bot should still find a move");
    assert_eq!(
        mv,
        Direction::Right,
        "bot should head for the food instead of hunting an equal-length rival"
    );
    assert_eq!(
        game.autopilot_path.last(),
        Some(&p(12, 8)),
        "bot should path to the food, not to a cut-off"
    );
    assert!(
        !game.path_cuts_off_rival(game.snake.head(), &game.autopilot_path, 1),
        "no cut-off may be reported against an equal-length rival"
    );
}

#[test]
fn test_multi_tick_hunt_kills_rival() {
    let mut game = hunt_setup();
    let mut p2_dead = false;
    for tick in 1..=5 {
        let mv = game
            .calculate_autopilot_move()
            .expect("bot should always find a move");
        let head = game.snake.head();
        let next = Game::calculate_next_head_dir(head, mv);

        // Enemy p2 is committed to moving Down this tick.
        let p2_head = game.player2.as_ref().unwrap().head();
        let p2_next = Game::calculate_next_head_dir(p2_head, Direction::Down);

        // Engine collision rule: collision checks use pre-move bodies, so p2
        // dies if its final head lands on the player's current body (except a
        // tail that vacates this tick).
        let player_tail = game.snake.body.back().copied();
        if game.snake.body_map.contains_key(&p2_next) && player_tail != Some(p2_next) {
            p2_dead = true;
            break;
        }
        // The player must never run into p2's body either.
        assert!(
            !game.player2.as_ref().unwrap().body_map.contains_key(&next)
                || game.player2.as_ref().unwrap().body.back() == Some(&next),
            "tick {tick}: player must not crash into p2's body"
        );

        game.snake.direction = mv;
        game.snake.move_to(next, false);
        let p2 = game.player2.as_mut().unwrap();
        p2.direction = Direction::Down;
        p2.move_to(p2_next, false);
    }
    assert!(p2_dead, "the hunting bot must cut off p2 within a few ticks");
}

#[test]
fn test_p2_autopilot_hunts_shorter_player() {
    let mut game = base_game();
    game.mode = GameMode::PlayerVsBot;
    // Player (victim) at (10,5) moving Down, length 3.
    game.snake = Snake::new(p(10, 5));
    game.snake.direction = Direction::Down;
    set_body(&mut game.snake, &[p(10, 5), p(10, 4), p(10, 3)]);
    // p2 (hunter) at (8,8) moving Right, length 5.
    let mut p2 = Snake::new(p(8, 8));
    p2.direction = Direction::Right;
    set_body(&mut p2, &[p(8, 8), p(7, 8), p(6, 8), p(5, 8), p(4, 8)]);
    game.player2 = Some(p2);
    game.food = p(18, 18);

    let mv = game
        .calculate_p2_autopilot_move()
        .expect("p2 bot should find a hunting move");
    assert_eq!(mv, Direction::Right);
    assert!(
        game.path_cuts_off_rival(
            game.player2.as_ref().unwrap().head(),
            &game.p2_autopilot_path,
            2
        ),
        "p2's hunting path must cut off the shorter player"
    );
}

#[test]
fn test_arena_bot_hunts_shorter_rival() {
    let mut game = base_game();
    game.mode = GameMode::MassiveMultiplayer;
    game.auto_pilot = false;
    game.flow_field = None;
    game.flow_field_targets.clear();
    // The human player is far away.
    game.snake = Snake::new(p(15, 17));
    game.snake.direction = Direction::Up;
    set_body(&mut game.snake, &[p(15, 17), p(15, 18), p(15, 19)]);

    // Hunter bot at (8,8) facing Right, length 5.
    let mut hunter = Snake::new(p(8, 8));
    hunter.direction = Direction::Right;
    set_body(&mut hunter, &[p(8, 8), p(7, 8), p(6, 8), p(5, 8), p(4, 8)]);
    game.bots.push(hunter);
    game.bots_autopilot_paths.push(Vec::new());

    // Victim bot at (10,5) facing Down, length 3.
    let mut victim = Snake::new(p(10, 5));
    victim.direction = Direction::Down;
    set_body(&mut victim, &[p(10, 5), p(10, 4), p(10, 3)]);
    game.bots.push(victim);
    game.bots_autopilot_paths.push(Vec::new());

    game.food = p(1, 1);

    // The victim's committed direction is straight Down (empty queue), so the
    // hunter must find a verified cut-off path onto that lane.
    let hunt = game
        .hunting_move(p(8, 8), Some(Direction::Right), 4)
        .expect("hunter arena bot should find a cut-off");
    assert_eq!(hunt.0, Direction::Right);
    assert!(
        game.path_cuts_off_rival(p(8, 8), &hunt.1, 4),
        "arena bot's hunting path must cut off the shorter rival"
    );

    game.handle_autopilot_moves();
    assert!(
        !game.bots[0].direction_queue.is_empty(),
        "hunter arena bot should queue a move"
    );
    assert_eq!(
        *game.bots[0].direction_queue.front().unwrap(),
        Direction::Right,
        "hunter arena bot should step toward the intercept cell"
    );
    assert_eq!(
        game.bots_autopilot_paths[0], hunt.1,
        "the queued hunting move must be the verified cut-off path"
    );
}

#[test]
fn test_arena_bot_does_not_hunt_when_not_longer() {
    let mut game = base_game();
    game.mode = GameMode::MassiveMultiplayer;
    game.auto_pilot = false;
    game.flow_field = None;
    game.flow_field_targets.clear();
    game.snake = Snake::new(p(15, 17));
    game.snake.direction = Direction::Up;
    set_body(&mut game.snake, &[p(15, 17), p(15, 18), p(15, 19)]);

    // Hunter bot at (8,8) facing Right, length 3 (not longer than the victim).
    let mut hunter = Snake::new(p(8, 8));
    hunter.direction = Direction::Right;
    set_body(&mut hunter, &[p(8, 8), p(7, 8), p(6, 8)]);
    game.bots.push(hunter);
    game.bots_autopilot_paths.push(Vec::new());

    let mut victim = Snake::new(p(10, 5));
    victim.direction = Direction::Down;
    set_body(&mut victim, &[p(10, 5), p(10, 4), p(10, 3)]);
    game.bots.push(victim);
    game.bots_autopilot_paths.push(Vec::new());

    game.food = p(12, 8);

    game.handle_autopilot_moves();
    assert!(
        !game.bots[0].direction_queue.is_empty(),
        "arena bot should still queue a normal move"
    );
    assert!(
        !game.path_cuts_off_rival(game.bots[0].head(), &game.bots_autopilot_paths[0], 4),
        "a bot that is not longer than its rival must not hunt"
    );
    assert_eq!(
        game.bots_autopilot_paths[0].last(),
        Some(&p(12, 8)),
        "the bot should path to the food instead"
    );
}
