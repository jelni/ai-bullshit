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
    game.player2 = None;
    game.bots.clear();
    game.bots_autopilot_paths.clear();
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

/// P2 at (6,8) heading Up predicts its head along the x=6 column:
/// (6,7)@1, (6,6)@2, (6,5)@3, (6,4)@4, (6,3)@5, (6,2)@6, (6,1)@7.
fn game_with_p2_lane() -> Game {
    let mut game = base_game();
    game.snake = Snake::new(p(3, 5));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(3, 5), p(2, 5), p(1, 5)]);
    let mut p2 = Snake::new(p(6, 8));
    p2.direction = Direction::Up;
    game.player2 = Some(p2);
    game
}

#[test]
fn test_path_is_safe_rejects_predicted_rival_head_at_matching_step() {
    let mut game = game_with_p2_lane();
    // Straight path to the right crosses (6,5) at step 3, exactly where P2's
    // predicted head will be at step 3.
    let straight = vec![p(4, 5), p(5, 5), p(6, 5), p(7, 5), p(8, 5)];
    assert!(
        !game.path_is_safe(game.snake.head(), &straight, &[], 1),
        "a path cell that a rival's predicted head occupies at the same step must be unsafe"
    );

    // Control: without the rival, the same path is safe.
    game.player2 = None;
    assert!(
        game.path_is_safe(game.snake.head(), &straight, &[], 1),
        "the same path must be safe when no rival predicts into it"
    );
}

#[test]
fn test_path_is_safe_accepts_detour_around_rival_lane() {
    let game = game_with_p2_lane();
    // Detour that crosses the x=6 column at step 4 via (6,6), where P2's head
    // is predicted at step 2 instead of step 4.
    let detour = vec![p(4, 5), p(5, 5), p(5, 6), p(6, 6), p(7, 6), p(7, 5), p(8, 5)];
    assert!(
        game.path_is_safe(game.snake.head(), &detour, &[], 1),
        "a detour that crosses the rival lane at a different step should be safe"
    );
}

#[test]
fn test_rival_prediction_stops_at_horizon() {
    let mut game = base_game();
    game.snake = Snake::new(p(3, 5));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(3, 5), p(2, 5), p(1, 5)]);
    // P2 at (3,15) heading Up predicts (3,8) at step 7 (15 - 7 = 8).
    let mut p2 = Snake::new(p(3, 15));
    p2.direction = Direction::Up;
    game.player2 = Some(p2);

    let hit_at_step_7 = vec![p(4, 5), p(5, 5), p(5, 6), p(5, 7), p(5, 8), p(4, 8), p(3, 8)];
    assert!(
        !game.path_is_safe(game.snake.head(), &hit_at_step_7, &[], 1),
        "reaching the predicted cell exactly at the predicted step must be unsafe"
    );

    // The same cell reached later than the horizon is not predicted anymore.
    let late = vec![
        p(4, 5),
        p(5, 5),
        p(5, 6),
        p(5, 7),
        p(5, 8),
        p(5, 9),
        p(4, 9),
        p(4, 8),
        p(3, 8),
    ];
    assert!(
        game.path_is_safe(game.snake.head(), &late, &[], 1),
        "cells beyond the prediction horizon must not be treated as collisions"
    );
    assert!(late.len() as u16 > Game::RIVAL_HEAD_HORIZON);
}

#[test]
fn test_rival_prediction_follows_portals() {
    let mut game = game_with_p2_lane();
    game.snake = Snake::new(p(8, 11));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(8, 11), p(8, 12), p(8, 13)]);
    // P2 at (6,5) heading Down enters portal (6,6) at step 1 and exits at
    // (10,10); its head is then predicted at (10,11) at step 2.
    game.portals = Some((p(6, 6), p(10, 10)));
    let mut p2 = Snake::new(p(6, 5));
    p2.direction = Direction::Down;
    game.player2 = Some(p2);

    let through_exit = vec![p(9, 11), p(10, 11)];
    assert!(
        !game.path_is_safe(game.snake.head(), &through_exit, &[], 1),
        "the portal exit lane of a rival must be predicted"
    );

    game.portals = None;
    assert!(
        game.path_is_safe(game.snake.head(), &through_exit, &[], 1),
        "without portals the rival never reaches the exit cell"
    );
}

#[test]
fn test_rival_lane_mask_excludes_the_mover_itself() {
    let mut game = base_game();
    game.mode = GameMode::MassiveMultiplayer;
    let mut bot1 = Snake::new(p(5, 5));
    bot1.direction = Direction::Right;
    let mut bot2 = Snake::new(p(10, 10));
    bot2.direction = Direction::Left;
    game.bots.push(bot1);
    game.bots.push(bot2);

    let mask = game.rival_head_lane_mask(4, p(5, 5));
    assert!(
        mask.get(&p(9, 10)).is_some_and(|m| m & (1_u32 << 1) != 0),
        "the other bot's lane must be present"
    );
    assert!(
        mask.get(&p(7, 5)).is_none(),
        "the mover's own straight lane must not be treated as a rival threat"
    );
}

#[test]
fn test_invincibility_disables_rival_prediction() {
    let mut game = game_with_p2_lane();
    game.mode = GameMode::Zen;
    let straight = vec![p(4, 5), p(5, 5), p(6, 5), p(7, 5), p(8, 5)];
    assert!(
        game.path_is_safe(game.snake.head(), &straight, &[], 1),
        "an invincible snake ignores predicted head-on collisions"
    );
}

#[test]
fn test_autopilot_detours_around_predicted_rival_head() {
    let mut game = game_with_p2_lane();
    game.food = p(8, 5);

    let head = game.snake.head();
    let next_move = game.calculate_autopilot_move();
    assert!(next_move.is_some(), "bot should still find a move");
    assert_eq!(
        game.autopilot_path.last(),
        Some(&p(8, 5)),
        "bot should still reach the food via a safe detour"
    );

    let mask = game.rival_head_lane_mask(1, head);
    for (idx, &cell) in game.autopilot_path.iter().enumerate() {
        let steps = idx as u16 + 1;
        assert!(
            !(steps > 1
                && steps <= Game::RIVAL_HEAD_HORIZON
                && mask
                    .get(&cell)
                    .is_some_and(|m| m & (1_u32 << steps) != 0)),
            "autopilot path must never share a cell with a rival's predicted head at the same step"
        );
    }
}

#[test]
fn test_arena_bot_predicts_other_bot_lane() {
    let mut game = base_game();
    game.mode = GameMode::MassiveMultiplayer;
    game.weather = Weather::Clear;
    game.auto_pilot = false;
    game.flow_field = None;
    game.flow_field_targets.clear();
    game.snake = Snake::new(p(15, 17));
    game.snake.direction = Direction::Up;

    let mut bot1 = Snake::new(p(3, 5));
    bot1.direction = Direction::Right;
    set_body(&mut bot1, &[p(3, 5), p(2, 5), p(1, 5)]);
    let mut bot2 = Snake::new(p(6, 8));
    bot2.direction = Direction::Up;
    game.bots.push(bot1);
    game.bots_autopilot_paths.push(Vec::new());
    game.bots.push(bot2);
    game.bots_autopilot_paths.push(Vec::new());

    game.food = p(8, 5);

    game.handle_autopilot_moves();
    assert!(
        !game.bots[0].direction_queue.is_empty(),
        "arena bot should queue a move"
    );
    assert_eq!(
        game.bots_autopilot_paths[0].last(),
        Some(&p(8, 5)),
        "arena bot should still reach food around the other bot's predicted lane"
    );

    let mask = game.rival_head_lane_mask(4, p(3, 5));
    for (idx, &cell) in game.bots_autopilot_paths[0].iter().enumerate() {
        let steps = idx as u16 + 1;
        assert!(
            !(steps > 1
                && steps <= Game::RIVAL_HEAD_HORIZON
                && mask
                    .get(&cell)
                    .is_some_and(|m| m & (1_u32 << steps) != 0)),
            "arena bot path must avoid other bots' predicted heads at matching steps"
        );
    }
}
