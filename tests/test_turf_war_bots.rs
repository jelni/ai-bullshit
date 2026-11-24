use rand::SeedableRng;
use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme};
use snake_game::snake::{Direction, Point, Snake};

fn new_turf_war_game() -> Game {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Easy);
    game.mode = GameMode::TurfWar;
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.state = GameState::Playing;
    game.match_time = 1000;
    game.obstacles.clear();
    game.painted_tiles.clear();
    game.bots.clear();
    game.bots_autopilot_paths.clear();
    game
}

#[test]
fn test_turf_war_player_bot_receives_autopilot_move() {
    let mut game = new_turf_war_game();
    game.auto_pilot = true;
    game.snake = Snake::new(Point {
        x: 10,
        y: 10,
    });
    game.food = Point {
        x: 5,
        y: 5,
    };

    game.handle_autopilot_moves();

    assert!(
        !game.snake.direction_queue.is_empty(),
        "player bot should have a queued move in Turf War"
    );
}

#[test]
fn test_turf_war_player_bot_heads_for_own_unpainted_frontier() {
    let mut game = new_turf_war_game();
    game.auto_pilot = true;
    let mut snake = Snake::new(Point {
        x: 10,
        y: 10,
    });
    snake.direction = Direction::Right;
    game.snake = snake;
    game.food = Point {
        x: 2,
        y: 2,
    };

    // Paint everything as the player's own territory except a lane to the
    // right: the player bot's frontier is then exactly that lane.
    let lane = [
        Point {
            x: 11,
            y: 10,
        },
        Point {
            x: 12,
            y: 10,
        },
        Point {
            x: 13,
            y: 10,
        },
    ];
    for x in 1..game.width - 1 {
        for y in 1..game.height - 1 {
            let p = Point {
                x,
                y,
            };
            if !lane.contains(&p) {
                game.painted_tiles.insert(p, 1);
            }
        }
    }

    game.handle_autopilot_moves();

    let dir =
        game.snake.direction_queue.back().copied().expect("player bot should have a queued move");
    assert_eq!(
        dir,
        Direction::Right,
        "with the player's only unpainted frontier to the right, the player bot should head Right"
    );
    let target = game.autopilot_path.last().copied().expect("autopilot path should be non-empty");
    assert!(
        lane.contains(&target) || target == game.food,
        "player bot should path toward its own unpainted frontier, ended on {target:?}"
    );
}

#[test]
fn test_turf_war_player_bot_survives_with_arena_bots() {
    let mut game = new_turf_war_game();
    game.auto_pilot = true;
    game.snake = Snake::new(Point {
        x: 15,
        y: 15,
    });
    game.food = Point {
        x: 10,
        y: 17,
    };
    game.bots.push(Snake::new(Point {
        x: 5,
        y: 5,
    }));
    game.bots_autopilot_paths.push(Vec::new());

    for _ in 0..10 {
        game.update();
    }

    assert!(!game.just_died, "player bot should survive in an open Turf War field");
    assert_eq!(game.bots.len(), 1, "arena bot should survive alongside the player bot");
}
