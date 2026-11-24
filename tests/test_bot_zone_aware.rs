use snake_game::game::{
    Difficulty, Game, GameMode, GameState, PowerUp, PowerUpType, Theme, Weather,
};
use snake_game::snake::{Direction, Point, Snake};
use rand::SeedableRng;
use std::time::Duration;

fn base_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::BattleRoyale;
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
    game.merchant = None;
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
    Point {
        x,
        y,
    }
}

#[test]
fn test_zone_margin_helpers_match_engine_wall_rule() {
    let mut game = base_game(30, 30);
    game.safe_zone_margin = 4;
    assert_eq!(game.zone_margin_now(), 4);
    // The engine kills at x <= margin or x >= width-1-margin (same for y).
    assert!(!game.within_safe_zone(p(4, 15), 0));
    assert!(game.within_safe_zone(p(5, 15), 0));
    assert!(!game.within_safe_zone(p(25, 15), 0));
    assert!(game.within_safe_zone(p(24, 15), 0));
    assert!(!game.within_safe_zone(p(15, 4), 0));
    assert!(game.within_safe_zone(p(15, 5), 0));

    // Outside Battle Royale there is no zone: the margin is always 0 and
    // within_safe_zone never rejects (the map edge is handled by get_final_p).
    game.mode = GameMode::SinglePlayer;
    game.safe_zone_margin = 7;
    assert_eq!(game.zone_margin_now(), 0);
    assert_eq!(game.zone_margin_at(1000), 0);
    assert!(game.within_safe_zone(p(1, 1), 0));
    assert!(game.within_safe_zone(p(0, 1), 0));
    assert!(game.within_safe_zone(p(29, 15), 0));
}

#[test]
fn test_zone_margin_at_projects_scheduled_shrink() {
    let mut game = base_game(30, 30);
    game.safe_zone_margin = 1;
    // The zone shrinks every 10s of wall-clock time; pretend the last shrink
    // was 9s ago, so the next one lands after ~1s (~34 minimum-length ticks).
    game.last_shrink_time -= Duration::from_secs(9);
    assert_eq!(game.zone_margin_at(0), 1);
    assert_eq!(
        game.zone_margin_at(1),
        1,
        "9s + 1*500ms < 10s: no shrink projected within 1 step"
    );
    assert_eq!(
        game.zone_margin_at(2),
        2,
        "9s + 2*500ms >= 10s: one shrink must be projected within 2 steps"
    );
    // The projection is capped at the engine's maximum margin (30x30 -> 13).
    game.safe_zone_margin = 13;
    assert_eq!(game.zone_margin_at(1000), 13);
}

#[test]
fn test_is_safe_final_p_rejects_cells_outside_zone() {
    let mut game = base_game(30, 30);
    game.safe_zone_margin = 3;
    game.snake = Snake::new(p(10, 10));
    set_body(&mut game.snake, &[p(10, 10)]);

    assert!(!game.is_safe_final_p(p(3, 10), 1, 1), "x <= margin must be lethal");
    assert!(game.is_safe_final_p(p(4, 10), 1, 1));
    assert!(!game.is_safe_final_p(p(26, 10), 1, 1), "x >= width-1-margin must be lethal");
    assert!(game.is_safe_final_p(p(25, 10), 1, 1));
}

#[test]
fn test_is_safe_final_p_uses_projected_margin_at_arrival_step() {
    let mut game = base_game(30, 30);
    game.safe_zone_margin = 1;
    game.snake = Snake::new(p(10, 10));
    set_body(&mut game.snake, &[p(10, 10)]);
    game.last_shrink_time -= Duration::from_secs(9);

    assert!(
        game.is_safe_final_p(p(2, 10), 1, 1),
        "after 1 step (9.5s) the zone has not shrunk yet, so x=2 is still inside"
    );
    assert!(
        !game.is_safe_final_p(p(2, 10), 2, 1),
        "after 2 steps (10.0s) the zone shrinks to margin 2, so x=2 becomes lethal"
    );
}

#[test]
fn test_verified_search_stays_inside_zone_and_skips_out_of_zone_targets() {
    let mut game = base_game(30, 30);
    game.safe_zone_margin = 5;
    game.snake = Snake::new(p(6, 15));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(6, 15), p(6, 16), p(6, 17)]);
    game.food = p(23, 15);

    let (_, path) = game
        .astar_search_verified(p(6, 15), Some(Direction::Right), &[p(23, 15)], 1)
        .expect("bot should find a verified path to the in-zone food");
    assert_eq!(path.last(), Some(&p(23, 15)));
    for (i, &cell) in path.iter().enumerate() {
        let steps = u16::try_from(i + 1).unwrap_or(u16::MAX);
        assert!(
            game.within_safe_zone(cell, steps),
            "path cell {cell:?} at step {steps} leaves the safe zone"
        );
    }

    assert!(
        game.astar_search_verified(p(6, 15), Some(Direction::Right), &[p(26, 15)], 1).is_none(),
        "a target outside the safe zone (x >= 24) must be unreachable, even as a target exception"
    );
}

#[test]
fn test_autopilot_ignores_food_outside_the_zone() {
    let mut game = base_game(30, 30);
    game.safe_zone_margin = 5;
    game.snake = Snake::new(p(10, 15));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(10, 15), p(9, 15), p(8, 15)]);
    game.food = p(27, 15); // outside the zone (x >= 30-1-5 = 24)
    game.power_up = Some(PowerUp {
        p_type: PowerUpType::ScoreMultiplier,
        location: p(10, 20),
        activation_time: None,
    });

    let dir = game.calculate_autopilot_move().expect("bot should find a move");
    assert_eq!(
        game.autopilot_path.last(),
        Some(&p(10, 20)),
        "the bot must go for the in-zone power-up instead of the out-of-zone food"
    );
    let next = Game::calculate_next_head_dir(p(10, 15), dir);
    assert!(
        game.within_safe_zone(next, 1),
        "the chosen move must keep the head inside the safe zone"
    );
}

#[test]
fn test_path_is_safe_rejects_paths_crossing_the_zone_border() {
    let mut game = base_game(30, 30);
    game.safe_zone_margin = 5;
    game.snake = Snake::new(p(10, 15));
    set_body(&mut game.snake, &[p(10, 15)]);

    let inside = vec![p(11, 15), p(12, 15)];
    assert!(
        game.path_is_safe(p(10, 15), &inside, &[p(12, 15)], 1),
        "a path staying inside the zone must verify"
    );
    let crossing = vec![p(6, 15), p(5, 15)];
    assert!(
        !game.path_is_safe(p(10, 15), &crossing, &[p(5, 15)], 1),
        "a path entering the lethal border (x <= 5) must be rejected at its arrival step"
    );
}

#[test]
fn test_reachable_space_counts_only_cells_inside_zone() {
    let mut game = base_game(30, 30);
    game.snake = Snake::new(p(15, 15));
    set_body(&mut game.snake, &[p(15, 15)]);
    let path = vec![p(16, 15)];

    game.safe_zone_margin = 0;
    let open_space = game.reachable_space_after_path(p(15, 15), &path, 1);
    game.safe_zone_margin = 10;
    let boxed_space = game.reachable_space_after_path(p(15, 15), &path, 1);

    assert!(
        boxed_space < open_space,
        "with margin 10 the reachable space ({boxed_space}) must be smaller than with margin 0 ({open_space})"
    );
    // 30x30 with margin 10 leaves an 8x8 interior.
    assert!(boxed_space <= 8 * 8, "reachable space must not count cells outside the safe zone");
}

#[test]
fn test_autopilot_survives_in_battle_royale_without_leaving_the_zone() {
    let mut game = Game::new(30, 30, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::BattleRoyale;
    game.reset();
    game.auto_pilot = true;
    game.obstacles.clear();
    game.safe_zone_margin = 3;
    game.player2 = None;
    game.merchant = None;
    game.goblin = None;
    game.portals = None;
    game.xp_to_next_level = u32::MAX;
    game.state = GameState::Playing;
    game.rng = rand::rngs::StdRng::seed_from_u64(42);

    let mut ticks = 0;
    let mut deaths = 0;
    for _ in 0..150 {
        if game.state != GameState::Playing {
            break;
        }
        game.weather = Weather::Clear;
        game.merchant = None;
        game.goblin = None;
        game.update();
        ticks += 1;
        if game.just_died {
            deaths += 1;
            game.just_died = false;
        }
        if game.state == GameState::Playing {
            assert!(
                game.within_safe_zone(game.snake.head(), 0),
                "tick {ticks}: bot head {:?} left the safe zone (margin {})",
                game.snake.head(),
                game.safe_zone_margin
            );
        } else {
            // A merchant/goblin shop or similar may pause the game; resume.
            game.state = GameState::Playing;
        }
    }
    assert!(
        ticks == 150 && deaths == 0,
        "the zone-aware autopilot should survive 150 ticks on an empty board (tick {ticks}, deaths {deaths})"
    );
}
