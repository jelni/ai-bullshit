use snake_game::game::{Difficulty, Game, GameMode, Theme};
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

fn p(x: u16, y: u16) -> Point {
    Point { x, y }
}

fn set_body(snake: &mut Snake, cells: &[Point]) {
    snake.body = cells.iter().copied().collect();
    snake.body_map.clear();
    for pt in &snake.body {
        *snake.body_map.entry(*pt).or_insert(0) += 1;
    }
}

// Builds a 3x4 room at x=14..=16, y=9..=12 with a single 1-wide entrance at
// (15, 8). A snake of length 4 entering it cannot seal itself in, but its body
// blocks the entrance corridor, leaving only the small room as reachable space.
fn add_room(game: &mut Game) {
    for y in 8..=13_u16 {
        game.obstacles.insert(p(13, y));
        game.obstacles.insert(p(17, y));
    }
    for x in 13..=17_u16 {
        game.obstacles.insert(p(x, 13));
    }
    game.obstacles.insert(p(14, 8));
    game.obstacles.insert(p(16, 8));
}

// Snake of length 4 with its head at the room entrance (15, 8), body trailing
// to the left.
fn entrance_snake() -> Snake {
    let mut snake = Snake::new(p(15, 8));
    set_body(&mut snake, &[p(15, 8), p(14, 8), p(13, 8), p(12, 8)]);
    snake.direction = Direction::Right;
    snake
}

#[test]
fn test_reachable_space_after_path_scores_pocket_low() {
    let mut game = base_game();
    add_room(&mut game);
    game.snake = entrance_snake();

    let head = game.snake.head();
    let pocket_path = vec![p(15, 9), p(15, 10), p(15, 11), p(15, 12)];
    let pocket_space = game.reachable_space_after_path(head, &pocket_path, 1);
    assert!(
        pocket_space < 20,
        "arriving inside the small room should leave little reachable space, got {pocket_space}"
    );

    let open_path = vec![p(15, 7), p(14, 7), p(13, 7), p(12, 7)];
    let open_space = game.reachable_space_after_path(head, &open_path, 1);
    assert!(
        open_space > 200,
        "arriving in the open area should leave lots of reachable space, got {open_space}"
    );
}

#[test]
fn test_verified_search_prefers_roomy_target_over_closer_pocket_food() {
    let mut game = base_game();
    add_room(&mut game);
    game.snake = entrance_snake();
    // Food is 4 steps away, inside the cramped room.
    game.food = p(15, 12);
    // Bonus food is far away but in the open.
    let bonus = p(5, 5);

    let head = game.snake.head();
    let (dir, path) = game
        .astar_search_verified(head, Some(Direction::Right), &[game.food, bonus], 1)
        .expect("a verified path to the roomy bonus food should exist");
    assert_eq!(
        path.last(),
        Some(&bonus),
        "the bot should prefer the far roomy target over the closer pocket target (dir {dir:?})"
    );

    // Target order must not matter.
    let (_, path2) = game
        .astar_search_verified(head, Some(Direction::Right), &[bonus, game.food], 1)
        .expect("a verified path should exist regardless of target order");
    assert_eq!(path2.last(), Some(&bonus));
}

#[test]
fn test_verified_search_still_prefers_nearby_food_on_open_board() {
    let mut game = base_game();
    add_room(&mut game);
    game.snake = entrance_snake();
    // Both targets in the open: near food vs far bonus food.
    game.food = p(12, 8);
    let bonus = p(5, 5);

    let head = game.snake.head();
    let (_, path) = game
        .astar_search_verified(head, Some(Direction::Right), &[game.food, bonus], 1)
        .expect("a verified path should exist");
    assert_eq!(
        path.last(),
        Some(&game.food),
        "with similar space after arrival, the shorter path to nearby food should win"
    );
}

#[test]
fn test_autopilot_avoids_cramped_pocket_even_when_food_is_closer() {
    let mut game = base_game();
    game.mode = GameMode::SinglePlayer;
    add_room(&mut game);
    game.snake = entrance_snake();
    game.food = p(15, 12);
    let bonus = p(5, 5);
    game.bonus_food = Some((bonus, web_time::Instant::now()));

    let next_move = game
        .calculate_autopilot_move()
        .expect("bot should find a verified move");
    assert_ne!(
        next_move,
        Direction::Down,
        "the bot must not step into the cramped room when a roomy target exists"
    );
    assert_eq!(
        game.autopilot_path.last(),
        Some(&bonus),
        "the autopilot should route to the roomy bonus food instead of the pocket food"
    );
}

#[test]
fn test_p2_bot_space_aware_selection() {
    let mut game = base_game();
    add_room(&mut game);
    let mut p2 = Snake::new(p(15, 8));
    set_body(&mut p2, &[p(15, 8), p(14, 8), p(13, 8), p(12, 8)]);
    p2.direction = Direction::Right;
    game.player2 = Some(p2);
    game.snake = Snake::new(p(3, 3));
    game.food = p(15, 12);
    let bonus = p(5, 5);

    let head = game.player2.as_ref().unwrap().head();
    let (_, path) = game
        .astar_search_verified(head, Some(Direction::Right), &[game.food, bonus], 2)
        .expect("p2 bot should find a verified path");
    assert_eq!(
        path.last(),
        Some(&bonus),
        "p2 bot should also prefer the roomy target over the pocket food"
    );
}

#[test]
fn test_space_scored_search_still_rejects_unsafe_paths() {
    let mut game = base_game();
    game.snake = entrance_snake();
    game.food = p(15, 7); // open, safe
    let bonus = p(5, 5);
    // Cover the bonus food with a laser so its path can never verify.
    game.lasers.push(snake_game::game::Laser {
        position: bonus,
        direction: Direction::Up,
        player: 1,
    });

    let head = game.snake.head();
    let (_, path) = game
        .astar_search_verified(head, Some(Direction::Right), &[game.food, bonus], 1)
        .expect("the safe food path should still be found");
    assert_eq!(
        path.last(),
        Some(&game.food),
        "unsafe targets must be skipped in favor of the verified safe one"
    );
}
