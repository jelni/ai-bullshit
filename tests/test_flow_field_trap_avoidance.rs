use snake_game::game::{
    Difficulty, Game, GameMode, PowerUp, PowerUpType, Theme, Weather, follow_flow_field,
    generate_flow_field,
};
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
    game.weather = Weather::Clear;
    game.auto_pilot = false;
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

fn set_flow_field(game: &mut Game, targets: Vec<Point>) {
    game.flow_field = Some(generate_flow_field(game, &targets));
    game.flow_field_targets = targets;
}

#[test]
fn test_follow_flow_field_reaches_target() {
    let mut game = base_game();
    for x in 8..=12_u16 {
        game.obstacles.insert(Point { x, y: 10 });
    }
    let target = Point { x: 10, y: 15 };
    let start = Point { x: 10, y: 5 };
    let field = generate_flow_field(&game, &[target]);

    let path = follow_flow_field(&game, &field, start, &[target]);
    assert_eq!(path.last(), Some(&target), "flow field path must reach the target");
    assert!(
        path.iter().all(|p| !game.obstacles.contains(p)),
        "flow field path must not cross obstacles"
    );
}

#[test]
fn test_follow_flow_field_uses_portals() {
    let mut game = base_game();
    for x in 0..20_u16 {
        game.obstacles.insert(Point { x, y: 10 });
    }
    game.portals = Some((Point { x: 5, y: 5 }, Point { x: 5, y: 15 }));
    let target = Point { x: 5, y: 17 };
    let start = Point { x: 5, y: 3 };
    let field = generate_flow_field(&game, &[target]);

    let path = follow_flow_field(&game, &field, start, &[target]);
    assert_eq!(path.last(), Some(&target), "flow field path must reach the target via portals");
    assert!(path.len() < 10, "flow field path should be short due to portal usage");
}

#[test]
fn test_follow_flow_field_empty_when_unreachable() {
    let mut game = base_game();
    let start = Point { x: 5, y: 5 };
    for p in [
        Point { x: 4, y: 5 },
        Point { x: 6, y: 5 },
        Point { x: 5, y: 4 },
        Point { x: 5, y: 6 },
    ] {
        game.obstacles.insert(p);
    }
    let target = Point { x: 10, y: 10 };
    let field = generate_flow_field(&game, &[target]);

    let path = follow_flow_field(&game, &field, start, &[target]);
    assert!(path.is_empty(), "an enclosed start has no flow field route");
}

#[test]
fn test_mmo_flow_field_bot_rejects_trapping_route() {
    let mut game = base_game();
    game.mode = GameMode::MassiveMultiplayer;
    game.snake = Snake::new(Point { x: 2, y: 2 });
    add_pocket(&mut game);
    game.bots.push(pocket_snake());
    game.bots_autopilot_paths.push(Vec::new());
    game.food = Point { x: 15, y: 11 };
    let food = game.food;
    set_flow_field(&mut game, vec![food]);

    game.handle_autopilot_moves();
    assert!(
        !game.bots[0].direction_queue.is_empty(),
        "arena bot should queue a survival move"
    );
    assert_ne!(
        game.bots[0].direction_queue.front(),
        Some(&Direction::Down),
        "flow-field bot must not step into the dead-end pocket"
    );
    let tail = *game.bots[0].body.back().unwrap();
    assert_eq!(
        game.bots_autopilot_paths[0].last(),
        Some(&tail),
        "arena bot should fall back to tail-chasing when the flow field route is a trap"
    );
}

#[test]
fn test_mmo_flow_field_bot_follows_safe_route() {
    let mut game = base_game();
    game.mode = GameMode::MassiveMultiplayer;
    game.snake = Snake::new(Point { x: 2, y: 2 });
    add_pocket(&mut game);
    game.bots.push(pocket_snake());
    game.bots_autopilot_paths.push(Vec::new());
    game.food = Point { x: 17, y: 8 };
    let food = game.food;
    set_flow_field(&mut game, vec![food]);
    let field_dir = *game
        .flow_field
        .as_ref()
        .and_then(|ff| ff.get(&Point { x: 15, y: 8 }))
        .expect("flow field should have an entry at the bot's head");

    game.handle_autopilot_moves();
    assert_eq!(
        game.bots[0].direction_queue.front(),
        Some(&field_dir),
        "flow-field bot should follow the safe flow field route to the food"
    );
    assert_eq!(
        game.bots_autopilot_paths[0].last(),
        Some(&Point { x: 17, y: 8 }),
        "stored flow field path should end at the food"
    );
}

#[test]
fn test_zombie_flow_field_bot_follows_safe_route_while_invisible() {
    let mut game = base_game();
    game.mode = GameMode::Zombie;
    game.snake = Snake::new(Point { x: 2, y: 2 });
    // While the player is invisible, zombies hunt food, so the flow field
    // targets the food and the fast path applies.
    game.power_up = Some(PowerUp {
        p_type: PowerUpType::Invisibility,
        location: Point { x: 1, y: 1 },
        activation_time: Some(u64::MAX),
    });
    let mut bot = Snake::new(Point { x: 2, y: 5 });
    bot.direction = Direction::Right;
    game.bots.push(bot);
    game.bots_autopilot_paths.push(Vec::new());
    game.food = Point { x: 8, y: 5 };
    let food = game.food;
    set_flow_field(&mut game, vec![food]);
    let field_dir = *game
        .flow_field
        .as_ref()
        .and_then(|ff| ff.get(&Point { x: 2, y: 5 }))
        .expect("flow field should have an entry at the bot's head");

    game.handle_autopilot_moves();
    assert_eq!(
        game.bots[0].direction_queue.front(),
        Some(&field_dir),
        "zombie bot should follow the safe flow field route to the food"
    );
    assert_eq!(
        game.bots_autopilot_paths[0].last(),
        Some(&Point { x: 8, y: 5 }),
        "stored flow field path should end at the food"
    );
}
