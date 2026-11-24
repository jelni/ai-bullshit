use snake_game::game::{Difficulty, Game, GameMode, Laser, Theme, Weather};
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
    Point {
        x,
        y,
    }
}

#[test]
fn test_path_is_safe_accepts_clear_path() {
    let mut game = base_game();
    game.snake = Snake::new(p(10, 10));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(10, 10), p(9, 10), p(8, 10), p(7, 10)]);

    let path = vec![p(10, 9), p(9, 9), p(8, 9)];
    assert!(
        game.path_is_safe(game.snake.head(), &path, &[], 1),
        "a path through free cells should be considered safe"
    );
}

#[test]
fn test_path_is_safe_rejects_own_body_collision_at_arrival() {
    let mut game = base_game();
    game.snake = Snake::new(p(10, 10));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(10, 10), p(9, 10), p(8, 10), p(7, 10)]);

    // (9,10) is body segment index 1 of a 4-long snake, so it only clears after
    // 4 - 1 = 3 steps; arriving there in 1 step is a collision.
    let short_path = vec![p(9, 10)];
    assert!(
        !game.path_is_safe(game.snake.head(), &short_path, &[], 1),
        "arriving on a body segment before it vacates must be unsafe"
    );

    // The same cell reached after 3 steps is fine.
    let long_path = vec![p(10, 9), p(9, 9), p(9, 10)];
    assert!(
        game.path_is_safe(game.snake.head(), &long_path, &[], 1),
        "reaching the body segment only after it has vacated should be safe"
    );
}

#[test]
fn test_path_is_safe_rejects_laser_cell() {
    let mut game = base_game();
    game.snake = Snake::new(p(5, 5));
    game.snake.direction = Direction::Right;
    game.lasers.push(Laser {
        position: p(12, 5),
        direction: Direction::Right,
        player: 1,
    });

    let into_laser = vec![p(12, 5)];
    assert!(
        !game.path_is_safe(game.snake.head(), &into_laser, &[p(12, 5)], 1),
        "a path ending on a laser-covered non-boss target must be unsafe"
    );

    let beside_laser = vec![p(12, 6)];
    assert!(game.path_is_safe(game.snake.head(), &beside_laser, &[], 1), "a cell off the laser ray should be safe");
}

#[test]
fn test_verified_search_rejects_path_onto_laser_covered_food() {
    let mut game = base_game();
    game.snake = Snake::new(p(5, 5));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(5, 5), p(4, 5), p(3, 5), p(2, 5)]);
    game.food = p(8, 5);
    game.lasers.push(Laser {
        position: p(8, 5),
        direction: Direction::Up,
        player: 1,
    });

    let head = game.snake.head();
    let raw = game.astar_pathfind(head, game.food, 1);
    assert!(raw.is_some(), "raw A* still targets the food because the target cell is exempt");

    let verified = game.astar_search_verified(head, Some(Direction::Right), &[game.food], 1);
    assert!(
        verified.is_none(),
        "verified search must reject the only path to the food because it ends on a laser"
    );
}

#[test]
fn test_autopilot_falls_back_to_tail_chase_when_food_path_unsafe() {
    let mut game = base_game();
    game.snake = Snake::new(p(5, 5));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(5, 5), p(4, 5), p(3, 5), p(2, 5)]);
    game.food = p(8, 5);
    game.lasers.push(Laser {
        position: p(8, 5),
        direction: Direction::Up,
        player: 1,
    });

    let tail = p(2, 5);
    let mv = game.calculate_autopilot_move().expect("bot should still find a survival move");
    let next = Game::calculate_next_head_dir(game.snake.head(), mv);
    assert_ne!(next, game.food, "bot must not step onto the laser-covered food");
    assert_eq!(
        game.autopilot_path.last(),
        Some(&tail),
        "bot should fall back to chasing its tail when the food path is unsafe"
    );
}

#[test]
fn test_autopilot_still_takes_safe_food_path() {
    let mut game = base_game();
    game.snake = Snake::new(p(5, 5));
    game.snake.direction = Direction::Right;
    set_body(&mut game.snake, &[p(5, 5), p(4, 5), p(3, 5), p(2, 5)]);
    game.food = p(8, 5);

    let mv = game.calculate_autopilot_move();
    assert_eq!(mv, Some(Direction::Right));
    assert_eq!(
        game.autopilot_path.last(),
        Some(&game.food),
        "bot should still path straight to food when the path is safe"
    );
}

#[test]
fn test_p2_autopilot_avoids_unsafe_food_path() {
    let mut game = base_game();
    game.snake = Snake::new(p(15, 17));
    game.snake.direction = Direction::Up;
    let mut p2 = Snake::new(p(5, 5));
    p2.direction = Direction::Right;
    set_body(&mut p2, &[p(5, 5), p(4, 5), p(3, 5), p(2, 5)]);
    game.player2 = Some(p2);
    game.food = p(8, 5);
    game.lasers.push(Laser {
        position: p(8, 5),
        direction: Direction::Up,
        player: 1,
    });

    let mv = game.calculate_p2_autopilot_move().expect("p2 bot should still find a survival move");
    let head = game.player2.as_ref().unwrap().head();
    let next = Game::calculate_next_head_dir(head, mv);
    assert_ne!(next, game.food, "p2 bot must not step onto the laser-covered food");
    assert_eq!(
        game.p2_autopilot_path.last(),
        Some(&p(2, 5)),
        "p2 bot should fall back to chasing its tail when the food path is unsafe"
    );
}

#[test]
fn test_mmo_bot_avoids_unsafe_food_path() {
    let mut game = base_game();
    game.mode = GameMode::MassiveMultiplayer;
    game.weather = Weather::Clear;
    game.auto_pilot = false;
    game.flow_field = None;
    game.flow_field_targets.clear();
    game.snake = Snake::new(p(15, 17));
    game.snake.direction = Direction::Up;
    let mut bot = Snake::new(p(5, 5));
    bot.direction = Direction::Right;
    set_body(&mut bot, &[p(5, 5), p(4, 5), p(3, 5), p(2, 5)]);
    game.bots.push(bot);
    game.bots_autopilot_paths.push(Vec::new());
    game.food = p(8, 5);
    game.lasers.push(Laser {
        position: p(8, 5),
        direction: Direction::Up,
        player: 1,
    });

    game.handle_autopilot_moves();
    assert!(!game.bots[0].direction_queue.is_empty(), "arena bot should queue a survival move");
    let head = game.bots[0].head();
    let dir = *game.bots[0].direction_queue.front().unwrap();
    let next = Game::calculate_next_head_dir(head, dir);
    assert_ne!(next, game.food, "arena bot must not step onto the laser-covered food");
    assert_eq!(
        game.bots_autopilot_paths[0].last(),
        Some(&p(2, 5)),
        "arena bot should chase its tail when the food path is unsafe"
    );
}
