use snake_game::game::{Difficulty, Game, GameMode, Theme, Weather};
use snake_game::snake::{Direction, Point, Snake};

fn tornado_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::Tornado;
    game.reset();
    game.xp_to_next_level = u32::MAX; // keep the game in Playing state while the bot eats
    game
}

#[test]
fn test_tornado_reset_forces_tornado_weather() {
    let game = tornado_game(30, 30);

    assert_eq!(game.weather, Weather::Tornado);
    assert_eq!(game.tornado_gusts, 0);
    let vortex = game.tornado.expect("Tornado mode must spawn a vortex on reset");
    assert!(
        !game.snake.body_map.contains_key(&vortex),
        "the vortex must never spawn on the snake"
    );
    assert!(
        vortex.x > 0 && vortex.x < game.width - 1 && vortex.y > 0 && vortex.y < game.height - 1,
        "the vortex must spawn inside the board"
    );
}

#[test]
fn test_tornado_weather_never_breaks() {
    let mut game = tornado_game(30, 30);
    game.auto_pilot = true;

    for _ in 0..60 {
        game.update();
        if game.state != snake_game::game::GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = snake_game::game::GameState::Playing;
        }
        assert_eq!(
            game.weather,
            Weather::Tornado,
            "Tornado mode must force tornado weather on every tick"
        );
    }
}

#[test]
fn test_tornado_scheduled_gust_every_6_ticks() {
    let mut game = tornado_game(30, 30);
    game.obstacles.clear();
    game.auto_pilot = true;

    // Drive the scheduled ticks directly: update() increments tick_counter first,
    // so setting it to 6k - 1 makes the next update run tick 6k, which must wander.
    for tick in [5_u32, 11, 17, 23, 29, 35] {
        game.state = snake_game::game::GameState::Playing;
        game.tick_counter = tick;
        let gusts_before = game.tornado_gusts;
        game.update();
        assert_eq!(
            game.tick_counter,
            tick + 1,
            "update must run exactly one tick (lives: {})",
            game.lives
        );
        assert!(
            game.tornado_gusts > gusts_before,
            "a scheduled gust must move the vortex on tick {} (gusts before: {}, after: {})",
            tick + 1,
            gusts_before,
            game.tornado_gusts
        );
    }
    assert!(game.tornado_gusts >= 6);
}

#[test]
fn test_tornado_gust_never_lands_on_snake() {
    let mut game = tornado_game(20, 20);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 15, y: 15 };
    game.auto_pilot = true;

    for _ in 0..60 {
        game.update();
        if game.state != snake_game::game::GameState::Playing {
            if game.lives == 0 {
                break;
            }
            game.state = snake_game::game::GameState::Playing;
        }
        if let Some(t) = game.tornado {
            assert_ne!(
                t,
                game.snake.head(),
                "the vortex must never move onto the cell the snake occupies"
            );
        }
    }

    assert!(
        game.tornado_gusts >= 10,
        "expected at least 10 gusts over 60 ticks, got {}",
        game.tornado_gusts
    );
}

#[test]
fn test_tornado_vortex_is_lethal() {
    let mut game = tornado_game(20, 20);
    game.obstacles.clear();
    game.tornado = Some(Point { x: 5, y: 4 });

    // Moving up from (5, 5) enters the vortex cell
    assert!(
        !game.is_safe_final_p(Point { x: 5, y: 4 }, 1, 1),
        "is_safe_final_p must reject the tornado vortex cell"
    );
    assert!(
        game.is_safe_final_p(Point { x: 6, y: 5 }, 1, 1),
        "a different cell must remain safe"
    );
}

#[test]
fn test_tornado_kills_snake_entering_vortex() {
    let mut game = tornado_game(20, 20);
    game.obstacles.clear();
    game.snake = Snake::new(Point { x: 5, y: 5 });
    game.snake.direction = Direction::Up;
    game.snake.direction_queue.push_back(Direction::Up);
    game.food = Point { x: 15, y: 15 };
    game.tornado = Some(Point { x: 5, y: 4 });
    game.auto_pilot = false;
    game.tick_counter = 1; // not a multiple of 6, so the vortex does not wander this tick
    let lives_before = game.lives;

    game.update();

    assert!(
        game.lives < lives_before,
        "moving into the tornado vortex must cost a life (lives before: {}, after: {})",
        lives_before,
        game.lives
    );
}

#[test]
fn test_tornado_vortex_sucks_in_obstacles() {
    let mut game = tornado_game(20, 20);
    game.obstacles.clear();
    game.obstacles.insert(Point { x: 10, y: 10 });
    game.obstacles.insert(Point { x: 11, y: 10 });
    game.obstacles.insert(Point { x: 15, y: 15 }); // far away, must survive
    game.tornado = Some(Point { x: 10, y: 11 });
    game.snake = Snake::new(Point { x: 3, y: 3 });
    game.snake.direction = Direction::Right;
    game.snake.direction_queue.push_back(Direction::Right);
    game.food = Point { x: 18, y: 18 };
    game.auto_pilot = false;
    game.tick_counter = 5; // next update runs tick 6: scheduled gust

    game.update();

    assert!(
        game.tornado_gusts >= 1,
        "the scheduled gust should have moved the vortex"
    );
    if let Some(t) = game.tornado {
        let near = |p: Point| t.x.abs_diff(p.x).saturating_add(t.y.abs_diff(p.y)) <= 1;
        assert!(
            !game.obstacles.iter().any(|o| near(*o)),
            "obstacles on and around the vortex must be sucked in (vortex at {:?}, obstacles: {:?})",
            t,
            game.obstacles
        );
        assert!(
            game.obstacles.contains(&Point { x: 15, y: 15 }),
            "far-away obstacles must survive"
        );
    }
}

#[test]
fn test_tornado_pulls_food_towards_vortex() {
    let mut game = tornado_game(30, 30);
    game.obstacles.clear();
    game.auto_pilot = false;
    let start_food = Point { x: 20, y: 15 };
    let vortex = Point { x: 10, y: 15 };
    let mut pulled = false;

    for _ in 0..100 {
        game.state = snake_game::game::GameState::Playing;
        game.snake = Snake::new(Point { x: 2, y: 2 });
        game.snake.direction = Direction::Right;
        game.snake.direction_queue.clear();
        game.snake.direction_queue.push_back(Direction::Right);
        game.food = start_food;
        game.tornado = Some(vortex);
        game.weather = Weather::Clear; // isolate the vortex pull from ambient weather effects
        game.tick_counter = 1; // not a multiple of 6: no wander this tick
        game.update();
        if game.state != snake_game::game::GameState::Playing && game.lives == 0 {
            break;
        }
        if game.food != start_food {
            pulled = true;
            break;
        }
    }

    assert!(pulled, "the vortex should drag the food towards itself");
}

#[test]
fn test_tornado_bot_survives_the_twister() {
    let mut game = tornado_game(40, 30);
    game.auto_pilot = true;

    for _ in 0..120 {
        game.update();
        if game.state != snake_game::game::GameState::Playing {
            if game.lives == 0 {
                break;
            }
            // Non-death interruption (e.g. merchant visit): resume playing
            game.state = snake_game::game::GameState::Playing;
        }
    }

    assert!(
        game.state == snake_game::game::GameState::Playing && game.lives > 0,
        "Autopilot should survive the tornado (lives: {}, gusts: {}, score: {})",
        game.lives,
        game.tornado_gusts,
        game.score
    );
    assert!(
        game.tornado_gusts >= 15,
        "expected at least 15 gusts over 120 ticks, got {}",
        game.tornado_gusts
    );
}
