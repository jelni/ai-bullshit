use snake_game::game::{Game, GameMode, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn base_game() -> Game {
    let mut game =
        Game::new(20, 20, false, 'x', Theme::Classic, snake_game::game::Difficulty::Normal);
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

fn add_pen(game: &mut Game) {
    for x in 3..=7_u16 {
        game.obstacles.insert(Point {
            x,
            y: 3,
        });
        game.obstacles.insert(Point {
            x,
            y: 9,
        });
    }
    for y in 3..=9_u16 {
        game.obstacles.insert(Point {
            x: 3,
            y,
        });
        game.obstacles.insert(Point {
            x: 7,
            y,
        });
    }
}

#[test]
fn test_bot_tail_chases_when_food_unreachable() {
    let mut game = base_game();
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;
    add_pen(&mut game);
    game.food = Point {
        x: 15,
        y: 15,
    };

    let head = game.snake.head();
    let tail = *game.snake.body.back().unwrap();
    assert!(
        game.astar_pathfind(head, game.food, 1).is_none(),
        "food outside the pen should be unreachable"
    );

    let next_move = game.calculate_autopilot_move();
    assert!(next_move.is_some(), "bot should still find a survival move");
    assert_eq!(
        game.autopilot_path.last(),
        Some(&tail),
        "bot should chase its own tail when no target is reachable"
    );
}

#[test]
fn test_bot_still_seeks_food_when_reachable() {
    let mut game = base_game();
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;
    game.food = Point {
        x: 8,
        y: 5,
    };

    let next_move = game.calculate_autopilot_move();
    assert_eq!(next_move, Some(Direction::Right));
    assert_eq!(
        game.autopilot_path.last(),
        Some(&game.food),
        "bot should path to food when food is reachable, not chase its tail"
    );
}

#[test]
fn test_p2_bot_tail_chases_when_food_unreachable() {
    let mut game = base_game();
    game.snake = Snake::new(Point {
        x: 15,
        y: 17,
    });
    game.snake.direction = Direction::Up;
    let mut p2 = Snake::new(Point {
        x: 5,
        y: 5,
    });
    p2.direction = Direction::Right;
    game.player2 = Some(p2);
    add_pen(&mut game);
    game.food = Point {
        x: 15,
        y: 15,
    };

    let head = game.player2.as_ref().unwrap().head();
    let tail = *game.player2.as_ref().unwrap().body.back().unwrap();
    assert!(
        game.astar_pathfind(head, game.food, 2).is_none(),
        "food outside the pen should be unreachable for player 2"
    );

    let next_move = game.calculate_p2_autopilot_move();
    assert!(next_move.is_some(), "p2 bot should still find a survival move");
    assert_eq!(
        game.p2_autopilot_path.last(),
        Some(&tail),
        "p2 bot should chase its own tail when no target is reachable"
    );
}

#[test]
fn test_mmo_bot_tail_chases_when_food_unreachable() {
    let mut game = base_game();
    game.mode = GameMode::MassiveMultiplayer;
    game.weather = Weather::Clear;
    game.auto_pilot = false;
    game.flow_field = None;
    game.flow_field_targets.clear();
    game.snake = Snake::new(Point {
        x: 15,
        y: 17,
    });
    game.snake.direction = Direction::Up;

    let mut bot = Snake::new(Point {
        x: 5,
        y: 5,
    });
    bot.direction = Direction::Right;
    game.bots.push(bot);
    game.bots_autopilot_paths.push(Vec::new());
    add_pen(&mut game);
    game.food = Point {
        x: 15,
        y: 15,
    };

    let head = game.bots[0].head();
    let tail = *game.bots[0].body.back().unwrap();
    assert!(
        game.astar_pathfind(head, game.food, 4).is_none(),
        "food outside the pen should be unreachable for the bot"
    );

    game.handle_autopilot_moves();
    assert!(!game.bots[0].direction_queue.is_empty(), "bot should queue a survival move");
    assert_eq!(
        game.bots_autopilot_paths[0].last(),
        Some(&tail),
        "MMO bot should chase its own tail when no target is reachable"
    );
}

#[test]
fn test_tail_chase_survives_in_pen() {
    let mut game = base_game();
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;
    add_pen(&mut game);
    game.food = Point {
        x: 15,
        y: 15,
    };

    for _ in 0..20 {
        let head = game.snake.head();
        let dir = game
            .calculate_autopilot_move()
            .expect("bot should always find a survival move inside the pen");
        game.snake.direction = dir;
        let next = Game::calculate_next_head_dir(head, dir);
        assert!(!game.obstacles.contains(&next), "tail-chase move must not hit a wall");
        game.snake.move_to(next, false);
        let new_head = game.snake.head();
        assert!(
            !game.snake.body.iter().skip(1).any(|&p| p == new_head),
            "tail-chasing bot collided with its own body"
        );
    }
}
