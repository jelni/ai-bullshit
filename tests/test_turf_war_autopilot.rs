use snake_game::{
    game::{Difficulty, Game, GameMode, GameState, Theme, Weather},
    snake::{Direction, Point, Snake},
};

fn turf_war_game() -> Game {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal,);
    game.mode = GameMode::TurfWar;
    game.state = GameState::Playing;
    game.obstacles.clear();
    game.painted_tiles.clear();
    game.player2 = None;
    game.bots.clear();
    game.bots_autopilot_paths.clear();
    game.food = Point {
        x: 3,
        y: 15,
    };
    game.match_time = 10_000;
    game.weather = Weather::Clear;
    game.auto_pilot = true;
    game.xp_to_next_level = u32::MAX;
    game
}

fn add_bot(game: &mut Game, head: Point, direction: Direction,) {
    let mut bot = Snake::new(head,);
    bot.direction = direction;
    game.bots.push(bot,);
    game.bots_autopilot_paths.push(Vec::new(),);
}

#[test]
fn test_turf_war_bots_receive_verified_autopilot_moves() {
    let mut game = turf_war_game();
    add_bot(
        &mut game,
        Point {
            x: 5,
            y: 5,
        },
        Direction::Right,
    );

    game.handle_autopilot_moves();

    assert_eq!(game.bots.len(), 1);
    assert!(
        !game.bots[0].direction_queue.is_empty(),
        "TurfWar arena bot should be queued an autopilot move"
    );
    assert!(
        !game.bots_autopilot_paths[0].is_empty(),
        "TurfWar arena bot should have a verified path to a target"
    );
}

#[test]
fn test_turf_war_bot_avoids_wall_and_survives() {
    let mut game = turf_war_game();
    add_bot(
        &mut game,
        Point {
            x: 14,
            y: 5,
        },
        Direction::Right,
    );

    for _ in 0..10 {
        game.update();
    }

    assert_eq!(game.bots.len(), 1, "TurfWar bot should turn away from the wall instead of dying");
    let head = game.bots[0].head();
    assert!(head.x >= 1 && head.x < game.width - 1);
    assert!(head.y >= 1 && head.y < game.height - 1);
}

#[test]
fn test_turf_war_bot_paints_territory_while_autopiloting() {
    let mut game = turf_war_game();
    add_bot(
        &mut game,
        Point {
            x: 5,
            y: 5,
        },
        Direction::Right,
    );

    for _ in 0..6 {
        game.update();
    }

    assert_eq!(game.bots.len(), 1);
    let painted = game.painted_tiles.values().filter(|&&v| v == 3,).count();
    assert!(
        painted >= 3,
        "TurfWar bot should paint its trail under bot id 3, painted {painted} tiles"
    );
}

#[test]
fn test_turf_war_frontier_targets_skip_own_paint_obstacles_and_bodies() {
    let mut game = turf_war_game();
    let bot_id = 3;
    let start = Point {
        x: 5,
        y: 5,
    };
    for p in [
        Point {
            x: 4,
            y: 5,
        },
        Point {
            x: 6,
            y: 5,
        },
        Point {
            x: 5,
            y: 4,
        },
        Point {
            x: 5,
            y: 6,
        },
    ] {
        game.painted_tiles.insert(p, bot_id,);
    }
    game.obstacles.insert(Point {
        x: 3,
        y: 5,
    },);

    let targets = game.turf_war_frontier_targets(start, bot_id, 3,);

    assert_eq!(targets.len(), 3);
    let mut prev = 0;
    for t in &targets {
        assert_ne!(game.painted_tiles.get(t), Some(&bot_id));
        assert!(!game.obstacles.contains(t));
        assert!(!game.snake.body_map.contains_key(t));
        let dist = u32::from(start.x.abs_diff(t.x,),) + u32::from(start.y.abs_diff(t.y,),);
        assert!(dist >= 2, "own painted neighbours must be skipped");
        assert!(dist >= prev, "targets must be sorted by distance");
        prev = dist;
    }
    assert!(!targets.contains(&Point {
        x: 3,
        y: 5
    }));
}

#[test]
fn test_turf_war_frontier_includes_enemy_tiles_for_reclaiming() {
    let mut game = turf_war_game();
    let start = Point {
        x: 5,
        y: 5,
    };
    game.painted_tiles.insert(
        Point {
            x: 6,
            y: 5,
        },
        1,
    );

    let targets = game.turf_war_frontier_targets(start, 3, 5,);

    assert!(
        targets.contains(&Point {
            x: 6,
            y: 5
        }),
        "enemy-owned tiles are valid frontier targets so bots can reclaim them"
    );
}
