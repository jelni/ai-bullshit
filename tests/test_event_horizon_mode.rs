use snake_game::game::{Difficulty, Game, GameMode, GameState, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn event_horizon_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::EventHorizon;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

fn keep_playing(game: &mut Game) {
    if game.state != GameState::Playing {
        if game.lives == 0 {
            return;
        }
        game.state = GameState::Playing;
    }
}

fn parked_game() -> Game {
    let mut game = event_horizon_game(30, 30);
    game.obstacles.clear();
    game.bots.clear();
    game.bots_autopilot_paths.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 25, y: 25 };
    game
}

#[test]
fn test_event_horizon_reset_seeds_permanent_hole() {
    let game = event_horizon_game(30, 30);

    let bh = game
        .black_hole
        .expect("Event Horizon mode must seed a black hole on reset");
    assert_eq!(game.event_horizon_drifts, 0);
    assert_eq!(game.event_horizon_surges, 0);
    assert!(!game.obstacles.contains(&bh));
    assert_ne!(game.food, bh);
    assert!(!game.snake.body_map.contains_key(&bh));
    assert_eq!(game.weather, Weather::Clear);
}

#[test]
fn test_event_horizon_hole_never_despawns() {
    let mut game = parked_game();
    game.auto_pilot = true;

    for _ in 0..120 {
        game.update();
        keep_playing(&mut game);
        assert!(
            game.black_hole.is_some(),
            "the event horizon must never despawn in Event Horizon mode"
        );
        assert_eq!(
            game.weather,
            Weather::Clear,
            "Event Horizon mode must keep the skies clear"
        );
        if game.lives == 0 {
            break;
        }
    }
}

#[test]
fn test_event_horizon_scheduled_drift_every_6_ticks() {
    let mut game = parked_game();
    game.black_hole = Some(Point { x: 15, y: 15 });
    // update() increments tick_counter first, so 5 -> tick 6 must trigger a drift
    game.tick_counter = 5;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.event_horizon_drifts, 1, "tick 6 must drift the event horizon");
    assert_ne!(
        game.black_hole,
        Some(Point { x: 15, y: 15 }),
        "on an empty board the event horizon must actually move"
    );

    // tick 12 is also a multiple of 6 and must drift again
    game.tick_counter = 11;
    game.update();
    assert_eq!(game.event_horizon_drifts, 2, "tick 12 must drift the event horizon");
}

#[test]
fn test_event_horizon_drift_never_lands_on_snake_or_obstacle() {
    let mut game = parked_game();
    game.black_hole = Some(Point { x: 10, y: 10 });
    game.obstacles.insert(Point { x: 9, y: 10 });
    game.obstacles.insert(Point { x: 11, y: 10 });
    game.obstacles.insert(Point { x: 10, y: 11 });
    // The snake is about to enter (10, 9), blocking the last free drift cell
    game.snake = Snake::new(Point { x: 10, y: 8 });
    game.snake.direction = Direction::Down;
    game.snake.direction_queue.push_back(Direction::Down);
    game.tick_counter = 5;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.event_horizon_drifts, 0,
        "with every drift cell blocked the hole must stay put"
    );
    assert_eq!(game.black_hole, Some(Point { x: 10, y: 10 }));
}

#[test]
fn test_event_horizon_pulls_food_towards_itself() {
    let mut game = parked_game();
    game.black_hole = Some(Point { x: 15, y: 15 });
    game.food = Point { x: 15, y: 18 };
    game.tick_counter = 1; // tick 2: no scheduled drift
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.food,
        Point { x: 15, y: 17 },
        "food must be dragged one cell towards the event horizon every tick"
    );
}

#[test]
fn test_event_horizon_never_pulls_items_onto_itself() {
    let mut game = parked_game();
    game.black_hole = Some(Point { x: 15, y: 15 });
    game.food = Point { x: 15, y: 16 }; // orthogonally adjacent
    game.tick_counter = 1;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(
        game.food,
        Point { x: 15, y: 16 },
        "food must never be pulled onto the lethal hole cell"
    );
}

#[test]
fn test_event_horizon_surge_shoves_snake_towards_hole() {
    let mut game = parked_game();
    game.black_hole = Some(Point { x: 10, y: 10 });
    // Block all four drift cells so the hole stays put on tick 12
    game.obstacles.insert(Point { x: 9, y: 10 });
    game.obstacles.insert(Point { x: 11, y: 10 });
    game.obstacles.insert(Point { x: 10, y: 9 });
    game.obstacles.insert(Point { x: 10, y: 11 });
    game.snake = Snake::new(Point { x: 5, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.tick_counter = 11; // next update runs tick 12: drift (blocked) + surge
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.event_horizon_surges, 1, "tick 12 must trigger a surge");
    assert_eq!(
        game.snake.head(),
        Point { x: 7, y: 10 },
        "the surge must shove the snake one cell towards the event horizon (normal move to (6,10) + shove to (7,10))"
    );
}

#[test]
fn test_event_horizon_surge_fizzles_into_obstacle() {
    let mut game = parked_game();
    game.black_hole = Some(Point { x: 10, y: 10 });
    game.obstacles.insert(Point { x: 9, y: 10 });
    game.obstacles.insert(Point { x: 11, y: 10 });
    game.obstacles.insert(Point { x: 10, y: 9 });
    game.obstacles.insert(Point { x: 10, y: 11 });
    // Far enough from the hole to survive the consume, but blocking the surge path
    game.obstacles.insert(Point { x: 7, y: 10 });
    game.snake = Snake::new(Point { x: 5, y: 10 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.tick_counter = 11;
    game.state = GameState::Playing;
    game.update();

    assert_eq!(game.event_horizon_surges, 1);
    assert_eq!(
        game.snake.head(),
        Point { x: 6, y: 10 },
        "a surge into a blocked cell must fizzle, never shove into the obstacle"
    );
}

#[test]
fn test_event_horizon_autopilot_avoids_hole() {
    let mut game = parked_game();
    game.auto_pilot = true;
    game.black_hole = Some(Point { x: 6, y: 5 });
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;

    let next_move = game.calculate_autopilot_move();
    assert!(
        next_move != Some(Direction::Right),
        "the bot must never step into the event horizon, instead chose {next_move:?}"
    );
}

#[test]
fn test_event_horizon_verified_safe_rejects_drift_zone() {
    let mut game = parked_game();
    game.black_hole = Some(Point { x: 6, y: 6 });

    // No drift is due within one step: the adjacent cell (6,5) is safe
    game.tick_counter = 0;
    assert!(
        game.is_safe_final_p(Point { x: 6, y: 5 }, 1, 1),
        "with no drift due, a cell next to the hole is safe"
    );

    // A drift is due within one step (tick 5 -> tick 6): the hole can reach
    // every cell within Manhattan distance 1, so (6,5) must be rejected
    game.tick_counter = 5;
    assert!(
        !game.is_safe_final_p(Point { x: 6, y: 5 }, 1, 1),
        "a cell the event horizon can drift into within the arrival step must be unsafe"
    );
    assert!(
        !game.is_safe_final_p(Point { x: 6, y: 6 }, 1, 1),
        "the hole cell itself must always be unsafe"
    );
}

#[test]
fn test_event_horizon_autopilot_survives_the_hole() {
    let mut game = parked_game();
    game.auto_pilot = true;

    for _ in 0..100 {
        game.update();
        keep_playing(&mut game);
        if let Some(bh) = game.black_hole {
            assert_ne!(
                game.snake.head(),
                bh,
                "the bot must never end a tick inside the event horizon"
            );
        }
        if game.lives == 0 {
            break;
        }
    }
    assert!(game.lives > 0, "the autopilot should survive Event Horizon mode");
}
