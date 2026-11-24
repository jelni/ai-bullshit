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

// Builds a 1-wide dead-end pocket at x=15, y=9..=11, open only at (15, 8).
fn add_pocket(game: &mut Game) {
    for y in 9..=11_u16 {
        game.obstacles.insert(Point { x: 14, y });
        game.obstacles.insert(Point { x: 16, y });
    }
    for x in 14..=16_u16 {
        game.obstacles.insert(Point { x, y: 12 });
    }
}

// Snake with head at the pocket entrance (15, 8) and body trailing to the left.
fn pocket_snake() -> Snake {
    let mut snake = Snake::new(Point { x: 15, y: 8 });
    snake.body.clear();
    snake.body.push_back(Point { x: 15, y: 8 });
    snake.body.push_back(Point { x: 14, y: 8 });
    snake.body.push_back(Point { x: 13, y: 8 });
    snake.rebuild_map();
    snake.direction = Direction::Right;
    snake
}

#[test]
fn test_bot_rejects_trapping_path_to_food() {
    let mut game = base_game();
    add_pocket(&mut game);
    game.snake = pocket_snake();
    // Food at the bottom of the pocket: eating it grows the snake to length 4,
    // which exactly seals the 4-cell pocket (entrance + 3 interior cells).
    game.food = Point { x: 15, y: 11 };

    let head = game.snake.head();
    assert!(
        game.astar_pathfind(head, game.food, 1).is_none(),
        "path into the dead-end pocket would trap the snake and must be rejected"
    );
}

#[test]
fn test_bot_still_paths_to_safe_food() {
    let mut game = base_game();
    add_pocket(&mut game);
    game.snake = pocket_snake();
    game.food = Point { x: 10, y: 8 };

    let head = game.snake.head();
    let dir = game.astar_pathfind(head, game.food, 1);
    assert!(dir.is_some(), "a safe path to food must still be found");
    assert_ne!(
        dir,
        Some(Direction::Down),
        "the bot must not step into the dead-end pocket"
    );
}

#[test]
fn test_autopilot_tail_chases_when_food_path_traps() {
    let mut game = base_game();
    add_pocket(&mut game);
    game.snake = pocket_snake();
    game.food = Point { x: 15, y: 11 };

    let tail = *game.snake.body.back().unwrap();
    let dir = game.calculate_autopilot_move();
    assert!(dir.is_some(), "bot should still find a survival move");
    assert_eq!(
        game.autopilot_path.last(),
        Some(&tail),
        "bot should chase its own tail when the only path to food is a trap"
    );
}

#[test]
fn test_bot_survives_despite_trapping_food() {
    let mut game = base_game();
    add_pocket(&mut game);
    game.snake = pocket_snake();
    game.food = Point { x: 15, y: 11 };

    for _ in 0..20 {
        let head = game.snake.head();
        let dir = game
            .calculate_autopilot_move()
            .expect("bot should always find a survival move");
        let next = Game::calculate_next_head_dir(head, dir);
        assert!(
            !game.obstacles.contains(&next),
            "trap-aware bot must never enter a wall"
        );
        assert_ne!(
            next,
            Point { x: 15, y: 9 },
            "trap-aware bot must not enter the dead-end pocket"
        );
        game.snake.direction = dir;
        game.snake.move_to(next, false);
        let new_head = game.snake.head();
        assert!(
            !game.snake.body.iter().skip(1).any(|&p| p == new_head),
            "trap-aware bot collided with its own body"
        );
    }
}

#[test]
fn test_arena_bot_rejects_trapping_path() {
    let mut game = base_game();
    game.mode = GameMode::MassiveMultiplayer;
    game.weather = Weather::Clear;
    game.auto_pilot = false;
    game.flow_field = None;
    game.flow_field_targets.clear();
    game.snake = Snake::new(Point { x: 2, y: 2 });
    add_pocket(&mut game);
    game.bots.push(pocket_snake());
    game.bots_autopilot_paths.push(Vec::new());
    game.food = Point { x: 15, y: 11 };

    let head = game.bots[0].head();
    assert!(
        game.astar_pathfind(head, game.food, 4).is_none(),
        "arena bot must reject the trapping path into the pocket"
    );

    game.handle_autopilot_moves();
    assert!(
        !game.bots[0].direction_queue.is_empty(),
        "arena bot should queue a survival move"
    );
    let tail = *game.bots[0].body.back().unwrap();
    assert_eq!(
        game.bots_autopilot_paths[0].last(),
        Some(&tail),
        "arena bot should chase its own tail when the food path is a trap"
    );
}

#[test]
fn test_p2_bot_rejects_trapping_path() {
    let mut game = base_game();
    add_pocket(&mut game);
    game.snake = Snake::new(Point { x: 2, y: 2 });
    game.player2 = Some(pocket_snake());
    game.food = Point { x: 15, y: 11 };

    let head = game.player2.as_ref().unwrap().head();
    assert!(
        game.astar_pathfind(head, game.food, 2).is_none(),
        "player 2 bot must reject the trapping path into the pocket"
    );

    let move_dir = game.calculate_p2_autopilot_move();
    assert!(move_dir.is_some(), "p2 bot should still find a survival move");
    let tail = *game.player2.as_ref().unwrap().body.back().unwrap();
    assert_eq!(
        game.p2_autopilot_path.last(),
        Some(&tail),
        "p2 bot should chase its own tail when the food path is a trap"
    );
}
