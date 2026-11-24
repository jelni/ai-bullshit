use snake_game::game::{Difficulty, Game, PowerUp, PowerUpType, Theme};
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

/// Player bot at (3,10) heading Right; food at (5,10) (2 steps away); rival
/// P2 at (6,10) heading Left, so P2 reaches the food in 1 step and wins it.
/// A power-up at (3,18) is 8 steps for the bot but 11 for P2, so it is free.
fn game_with_rival_racing_for_food() -> Game {
    let mut game = base_game();
    game.snake = Snake::new(p(3, 10));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(3, 10), p(2, 10), p(1, 10)]);
    game.food = p(5, 10);
    game.power_up = Some(PowerUp {
        p_type: PowerUpType::ScoreMultiplier,
        location: p(3, 18),
        activation_time: None,
    });
    let mut p2 = Snake::new(p(6, 10));
    p2.direction = Direction::Left;
    set_body(&mut p2, &[p(6, 10)]);
    game.player2 = Some(p2);
    game
}

#[test]
fn test_contested_targets_marks_food_rival_reaches_first() {
    let game = game_with_rival_racing_for_food();
    let contested = game.contested_targets(p(3, 10), 1, &[p(5, 10), p(3, 18)]);
    assert!(
        contested.contains(&p(5, 10)),
        "food that P2 reaches in 1 step vs the bot's 2 must be contested"
    );
    assert!(
        !contested.contains(&p(3, 18)),
        "the far power-up is closer to the bot than to P2 and must stay uncontested"
    );
}

#[test]
fn test_contested_targets_empty_without_rivals() {
    let mut game = game_with_rival_racing_for_food();
    game.player2 = None;
    let contested = game.contested_targets(p(3, 10), 1, &[p(5, 10), p(3, 18)]);
    assert!(
        contested.is_empty(),
        "with no rival snakes nothing can be contested"
    );
}

#[test]
fn test_contested_targets_tie_is_not_contested() {
    let mut game = game_with_rival_racing_for_food();
    let mut p2 = Snake::new(p(7, 10));
    p2.direction = Direction::Left;
    set_body(&mut p2, &[p(7, 10)]);
    game.player2 = Some(p2);
    // P2 is 2 steps from the food, exactly like the bot: a tie is a fair race.
    let contested = game.contested_targets(p(3, 10), 1, &[p(5, 10)]);
    assert!(
        contested.is_empty(),
        "a rival arriving at the same step must not mark the target contested"
    );
}

#[test]
fn test_contested_targets_never_marks_rival_head_hunt_target() {
    let game = game_with_rival_racing_for_food();
    // Zombie-style hunt target: the cell the rival currently occupies.
    let contested = game.contested_targets(p(3, 10), 1, &[p(6, 10)]);
    assert!(
        contested.is_empty(),
        "a target on a rival's head is a hunt target, not a contested one"
    );
}

#[test]
fn test_contested_targets_from_p2_perspective() {
    let game = game_with_rival_racing_for_food();
    // For P2 (checking_player 2) the player snake is the rival: it is 1 step
    // from (4,10) while P2 is 2 steps away.
    let contested = game.contested_targets(p(6, 10), 2, &[p(4, 10), p(6, 18)]);
    assert!(contested.contains(&p(4, 10)));
    assert!(!contested.contains(&p(6, 18)));
}

#[test]
fn test_verified_search_contested_skips_race_losing_food() {
    let game = game_with_rival_racing_for_food();
    let targets = [p(5, 10), p(3, 18)];
    let (_, path) = game
        .astar_search_verified_contested(p(3, 10), Some(Direction::Right), &targets, 1)
        .expect("bot should find a verified path to the uncontested power-up");
    assert_eq!(
        path.last(),
        Some(&p(3, 18)),
        "the contested food must be skipped in favor of the free power-up"
    );
}

#[test]
fn test_verified_search_without_rival_still_takes_closer_food() {
    let mut game = game_with_rival_racing_for_food();
    game.player2 = None;
    let targets = [p(5, 10), p(3, 18)];
    let (_, path) = game
        .astar_search_verified_contested(p(3, 10), Some(Direction::Right), &targets, 1)
        .expect("bot should path to the closer food when nothing is contested");
    assert_eq!(
        path.last(),
        Some(&p(5, 10)),
        "without rivals the bot keeps taking the shorter route to the food"
    );
}

#[test]
fn test_verified_search_contested_falls_back_when_all_targets_contested() {
    let mut game = game_with_rival_racing_for_food();
    // Move the power-up next to P2 so it is contested too; every target is
    // now lost, so the bot must fall back to racing anyway (or tail-chase).
    game.power_up = Some(PowerUp {
        p_type: PowerUpType::ScoreMultiplier,
        location: p(7, 10),
        activation_time: None,
    });
    let targets = [p(5, 10), p(7, 10)];
    let contested = game.contested_targets(p(3, 10), 1, &targets);
    assert_eq!(contested.len(), 2, "both targets should be contested here");
    let result =
        game.astar_search_verified_contested(p(3, 10), Some(Direction::Right), &targets, 1);
    assert!(
        result.is_some(),
        "when every target is contested the bot still returns the best verified race"
    );
}

#[test]
fn test_autopilot_move_avoids_food_rival_wins() {
    let mut game = game_with_rival_racing_for_food();
    let dir = game
        .calculate_autopilot_move()
        .expect("bot should find a move");
    assert_eq!(
        game.autopilot_path.last(),
        Some(&p(3, 18)),
        "autopilot (dir {dir:?}) must route to the uncontested power-up instead of the lost food"
    );
}

#[test]
fn test_autopilot_move_without_rival_goes_for_food() {
    let mut game = game_with_rival_racing_for_food();
    game.player2 = None;
    game.calculate_autopilot_move().expect("bot should find a move");
    assert_eq!(
        game.autopilot_path.last(),
        Some(&p(5, 10)),
        "without a rival the autopilot keeps heading for the closer food"
    );
}
