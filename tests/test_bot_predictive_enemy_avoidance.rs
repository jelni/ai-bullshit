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

// Player bot at (5,5) facing Right, enemy p2 at (9,5) committed to moving Left.
fn head_on_setup() -> Game {
    let mut game = base_game();
    game.snake = Snake::new(p(5, 5));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(5, 5), p(4, 5), p(3, 5)]);
    let mut p2 = Snake::new(p(9, 5));
    p2.direction = Direction::Left;
    set_body(&mut p2, &[p(9, 5), p(10, 5), p(11, 5)]);
    game.player2 = Some(p2);
    game
}

#[test]
fn test_path_is_safe_rejects_predicted_head_on_at_step_two() {
    let game = head_on_setup();
    // p2's head will be at (8,5) after 1 step and (7,5) after 2 steps, so
    // arriving at (7,5) on step 2 is a head-on collision.
    let colliding = vec![p(6, 5), p(7, 5)];
    assert!(
        !game.path_is_safe(game.snake.head(), &colliding, &[], 1),
        "a path whose step-2 cell is the enemy's predicted head must be unsafe"
    );

    let clear = vec![p(6, 5), p(6, 4)];
    assert!(
        game.path_is_safe(game.snake.head(), &clear, &[], 1),
        "a path away from the enemy's predicted head path must stay safe"
    );
}

#[test]
fn test_path_is_safe_rejects_entering_enemy_body_path() {
    let mut game = base_game();
    game.snake = Snake::new(p(5, 10));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(5, 10), p(4, 10), p(3, 10)]);
    let mut p2 = Snake::new(p(10, 10));
    p2.direction = Direction::Left;
    set_body(&mut p2, &[p(10, 10), p(11, 10), p(12, 10)]);
    game.player2 = Some(p2);

    // p2's head passes through (8,10) on step 2, so its body still occupies
    // (8,10) on step 3: stepping there on step 3 is a body collision.
    let colliding = vec![p(6, 10), p(7, 10), p(8, 10)];
    assert!(
        !game.path_is_safe(game.snake.head(), &colliding, &[], 1),
        "arriving on a cell the enemy's body still occupies must be unsafe"
    );

    let clear = vec![p(6, 10), p(6, 9), p(7, 9), p(8, 9)];
    assert!(
        game.path_is_safe(game.snake.head(), &clear, &[], 1),
        "a parallel path one row away from the enemy must stay safe"
    );
}

#[test]
fn test_prediction_follows_committed_direction_queue() {
    let mut game = base_game();
    game.snake = Snake::new(p(10, 5));
    game.snake.direction = Direction::Down;
    set_body(&mut game.snake, &[p(10, 5), p(10, 4), p(10, 3)]);
    let mut p2 = Snake::new(p(10, 10));
    p2.direction = Direction::Right;
    set_body(&mut p2, &[p(10, 10), p(9, 10), p(8, 10)]);
    p2.direction_queue.push_back(Direction::Up);
    p2.direction_queue.push_back(Direction::Up);
    game.player2 = Some(p2);

    // p2 turns up: its head is at (10,9) on step 1 and (10,8) on step 2.
    let colliding = vec![p(10, 6), p(10, 7), p(10, 8)];
    assert!(
        !game.path_is_safe(game.snake.head(), &colliding, &[], 1),
        "the prediction must follow the enemy's queued turns"
    );

    let clear = vec![p(10, 6), p(11, 6), p(11, 7), p(11, 8)];
    assert!(
        game.path_is_safe(game.snake.head(), &clear, &[], 1),
        "a path beside the enemy's turned head path must stay safe"
    );
}

#[test]
fn test_path_is_safe_ignores_enemy_moving_away() {
    let mut game = base_game();
    game.snake = Snake::new(p(5, 5));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(5, 5), p(4, 5), p(3, 5)]);
    let mut p2 = Snake::new(p(9, 5));
    p2.direction = Direction::Right;
    set_body(&mut p2, &[p(9, 5), p(8, 5), p(7, 5)]);
    game.player2 = Some(p2);

    let path = vec![p(6, 5), p(7, 5)];
    assert!(
        game.path_is_safe(game.snake.head(), &path, &[], 1),
        "an enemy moving away must not make a clear path unsafe"
    );
}

fn assert_path_avoids_enemy(
    game: &Game,
    start: Point,
    path: &[Point],
    checking_player: u8,
    enemy_head: Point,
    enemy_dir: Direction,
) {
    let mut enemy = enemy_head;
    for (idx, &cell) in path.iter().enumerate() {
        enemy = Game::calculate_next_head_dir(enemy, enemy_dir);
        assert_ne!(
            cell, enemy,
            "step {}: path cell must not be the enemy's predicted head",
            idx + 1
        );
    }
    assert!(
        game.path_is_safe(start, path, &[], checking_player),
        "the chosen path must be verified safe against predicted enemy occupancy"
    );
}

#[test]
fn test_autopilot_avoids_predicted_head_on() {
    let mut game = head_on_setup();
    game.food = p(7, 5);

    let mv = game
        .calculate_autopilot_move()
        .expect("bot should still find a survival move");
    let next = Game::calculate_next_head_dir(game.snake.head(), mv);
    assert_ne!(
        next,
        p(6, 5),
        "bot must not step along the path that head-ons into the enemy on step 2"
    );
    assert_path_avoids_enemy(&game, game.snake.head(), &game.autopilot_path, 1, p(9, 5), Direction::Left);
}

#[test]
fn test_autopilot_still_takes_food_path_when_enemy_moves_away() {
    let mut game = base_game();
    game.snake = Snake::new(p(5, 5));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(5, 5), p(4, 5), p(3, 5)]);
    let mut p2 = Snake::new(p(9, 5));
    p2.direction = Direction::Right;
    set_body(&mut p2, &[p(9, 5), p(8, 5), p(7, 5)]);
    game.player2 = Some(p2);
    game.food = p(7, 5);

    let mv = game.calculate_autopilot_move();
    assert_eq!(mv, Some(Direction::Right), "bot should path straight to food when safe");
    assert_eq!(game.autopilot_path.last(), Some(&p(7, 5)));
}

#[test]
fn test_p2_autopilot_avoids_predicted_player_head_on() {
    let mut game = base_game();
    game.snake = Snake::new(p(9, 5));
    game.snake.direction = Direction::Left;
    set_body(&mut game.snake, &[p(9, 5), p(10, 5), p(11, 5)]);
    let mut p2 = Snake::new(p(5, 5));
    p2.direction = Direction::Right;
    set_body(&mut p2, &[p(5, 5), p(4, 5), p(3, 5)]);
    game.player2 = Some(p2);
    game.food = p(7, 5);

    let mv = game
        .calculate_p2_autopilot_move()
        .expect("p2 bot should still find a survival move");
    let head = game.player2.as_ref().unwrap().head();
    let next = Game::calculate_next_head_dir(head, mv);
    assert_ne!(
        next,
        p(6, 5),
        "p2 bot must not step along the path that head-ons into the player on step 2"
    );
    assert_path_avoids_enemy(&game, head, &game.p2_autopilot_path, 2, p(9, 5), Direction::Left);
}

#[test]
fn test_arena_bot_avoids_predicted_enemy_head_on() {
    let mut game = base_game();
    game.mode = GameMode::MassiveMultiplayer;
    game.weather = Weather::Clear;
    game.auto_pilot = false;
    game.flow_field = None;
    game.flow_field_targets.clear();
    game.snake = Snake::new(p(15, 17));
    game.snake.direction = Direction::Up;
    set_body(&mut game.snake, &[p(15, 17), p(15, 18), p(15, 19)]);

    let mut bot = Snake::new(p(5, 5));
    bot.direction = Direction::Right;
    set_body(&mut bot, &[p(5, 5), p(4, 5), p(3, 5)]);
    game.bots.push(bot);
    game.bots_autopilot_paths.push(Vec::new());

    let mut enemy = Snake::new(p(9, 5));
    enemy.direction = Direction::Left;
    set_body(&mut enemy, &[p(9, 5), p(10, 5), p(11, 5)]);
    game.bots.push(enemy);
    game.bots_autopilot_paths.push(Vec::new());

    game.food = p(7, 5);

    game.handle_autopilot_moves();
    assert!(
        !game.bots[0].direction_queue.is_empty(),
        "arena bot should queue a survival move"
    );
    let dir = *game.bots[0].direction_queue.front().unwrap();
    assert_ne!(
        dir,
        Direction::Right,
        "arena bot must not move along the path that head-ons into the other bot"
    );
    assert_path_avoids_enemy(
        &game,
        game.bots[0].head(),
        &game.bots_autopilot_paths[0],
        4,
        p(9, 5),
        Direction::Left,
    );
}

#[test]
fn test_multi_tick_autopilot_never_head_ons_enemy() {
    let mut game = head_on_setup();
    game.food = p(7, 5);

    for tick in 1..=6 {
        let mv = game
            .calculate_autopilot_move()
            .expect("bot should always find a survival move");
        let head = game.snake.head();
        let next = Game::calculate_next_head_dir(head, mv);

        // Simulate the enemy's committed move for this tick.
        let enemy_head = game.player2.as_ref().unwrap().head();
        let enemy_next = Game::calculate_next_head_dir(enemy_head, Direction::Left);
        assert_ne!(
            next, enemy_next,
            "tick {tick}: bot head must never share a cell with the enemy head"
        );

        game.snake.direction = mv;
        game.snake.move_to(next, false);
        let p2 = game.player2.as_mut().unwrap();
        p2.direction = Direction::Left;
        p2.move_to(enemy_next, false);
    }
}
